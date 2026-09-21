//! Audio engine: WASAPI capture (microphone + system loopback) through `cpal`, a wall-clock driven
//! mixer producing exact 10 ms chunks, per-source DSP, real-time level meters and device hot-swap.
//!
//! The mixer emits chunks on a fixed schedule (`frames_due = elapsed × 48 000`), inserting silence
//! when a source delivers nothing. WASAPI loopback stops delivering packets while nothing plays, so
//! this schedule – not the device clock – is what keeps audio and video from drifting apart.
pub mod dsp;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SizedSample};
use crossbeam_channel::{Receiver, Sender};
use dsp::{db_to_lin, peak, Compressor, Gate, Limiter, Resampler};
use parking_lot::Mutex;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub const CHUNK_FRAMES: usize = 480; // 10 ms @ 48 kHz
const CHUNK_SAMPLES: usize = CHUNK_FRAMES * 2;
/// Maximum backlog kept per source before old audio is discarded (keeps latency bounded).
const MAX_BACKLOG_SAMPLES: usize = 48_000 / 4 * 2;

pub struct AudioChunk {
    pub mix: Vec<f32>,
    pub system: Vec<f32>,
    pub mic: Vec<f32>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeviceEntry {
    pub name: String,
    pub is_default: bool,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeviceList {
    pub inputs: Vec<DeviceEntry>,
    pub outputs: Vec<DeviceEntry>,
}

pub fn list_devices() -> DeviceList {
    let host = cpal::default_host();
    let dflt_in = host.default_input_device().and_then(|d| d.name().ok());
    let dflt_out = host.default_output_device().and_then(|d| d.name().ok());
    let map = |it: Option<Vec<cpal::Device>>, dflt: &Option<String>| -> Vec<DeviceEntry> {
        it.unwrap_or_default()
            .iter()
            .filter_map(|d| d.name().ok())
            .map(|name| DeviceEntry {
                is_default: Some(&name) == dflt.as_ref(),
                name,
            })
            .collect()
    };
    DeviceList {
        inputs: map(host.input_devices().ok().map(|i| i.collect()), &dflt_in),
        outputs: map(host.output_devices().ok().map(|i| i.collect()), &dflt_out),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MixParams {
    pub system_enabled: bool,
    pub system_volume: f32,
    pub system_device: String,
    pub mic_enabled: bool,
    pub mic_volume: f32,
    pub mic_device: String,
    pub mic_gain_db: f32,
    pub gate_enabled: bool,
    pub gate_db: f32,
    pub compressor: bool,
    pub limiter: bool,
    pub fallback_default: bool,
    /// off | push_to_talk | push_to_mute
    pub ptt_mode: String,
    pub ptt_vk: u32,
}

impl Default for MixParams {
    fn default() -> Self {
        Self {
            system_enabled: true,
            system_volume: 1.0,
            system_device: String::new(),
            mic_enabled: false,
            mic_volume: 1.0,
            mic_device: String::new(),
            mic_gain_db: 0.0,
            gate_enabled: false,
            gate_db: -50.0,
            compressor: false,
            limiter: true,
            fallback_default: true,
            ptt_mode: "off".into(),
            ptt_vk: 0,
        }
    }
}

impl MixParams {
    pub fn from_settings(a: &crate::settings::AudioSettings) -> Self {
        Self {
            system_enabled: a.system_enabled,
            system_volume: a.system_volume as f32 / 100.0,
            system_device: a.system_device.clone(),
            mic_enabled: a.mic_enabled,
            mic_volume: a.mic_volume as f32 / 100.0,
            mic_device: a.mic_device.clone(),
            mic_gain_db: a.mic_gain_db,
            gate_enabled: a.gate_enabled,
            gate_db: a.gate_threshold_db,
            compressor: a.compressor,
            limiter: a.limiter,
            fallback_default: a.fallback_to_default,
            ptt_mode: a.ptt_mode.clone(),
            ptt_vk: key_to_vk(&a.ptt_key).unwrap_or(0),
        }
    }
}

/// Virtual-key code for a single key name used by push-to-talk.
pub fn key_to_vk(name: &str) -> Option<u32> {
    let n = name.trim().to_uppercase();
    if n.len() == 1 {
        let c = n.chars().next()?;
        if c.is_ascii_alphanumeric() {
            return Some(c as u32);
        }
    }
    if let Some(num) = n.strip_prefix('F').and_then(|x| x.parse::<u32>().ok()) {
        if (1..=24).contains(&num) {
            return Some(0x6F + num);
        }
    }
    Some(match n.as_str() {
        "SPACE" => 0x20,
        "TAB" => 0x09,
        "CAPSLOCK" => 0x14,
        "MIDDLEMOUSE" => 0x04,
        "MOUSE4" => 0x05,
        "MOUSE5" => 0x06,
        "LCTRL" | "CTRL" => 0xA2,
        "LALT" | "ALT" => 0xA4,
        "LSHIFT" | "SHIFT" => 0xA0,
        _ => return None,
    })
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AudioEvent {
    /// A device disappeared (`kind`: mic | system).
    Disconnected { kind: String, device: String },
    /// The engine switched to another device (`device` = new device name).
    Switched { kind: String, device: String },
    /// A source could not be opened at all.
    Failed { kind: String, reason: String },
}

#[derive(Default)]
pub struct AudioStats {
    pub latency_ms: AtomicU32,
    pub chunks: AtomicU64,
    pub late_ticks: AtomicU64,
}

type EventFn = Arc<dyn Fn(AudioEvent) + Send + Sync>;

struct Shared {
    params: Mutex<MixParams>,
    params_version: AtomicU64,
    subscribers: Mutex<Vec<(u64, Sender<Arc<AudioChunk>>)>>,
    running: AtomicBool,
    mic_level: AtomicU32,
    sys_level: AtomicU32,
    on_event: Mutex<Option<EventFn>>,
    next_id: AtomicU64,
    stats: AudioStats,
}

pub struct AudioEngine {
    shared: Arc<Shared>,
    thread: Mutex<Option<JoinHandle<()>>>,
    users: AtomicUsize,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                params: Mutex::new(MixParams::default()),
                params_version: AtomicU64::new(1),
                subscribers: Mutex::new(vec![]),
                running: AtomicBool::new(false),
                mic_level: AtomicU32::new(0),
                sys_level: AtomicU32::new(0),
                on_event: Mutex::new(None),
                next_id: AtomicU64::new(1),
                stats: AudioStats::default(),
            }),
            thread: Mutex::new(None),
            users: AtomicUsize::new(0),
        }
    }

    pub fn set_event_handler(&self, f: EventFn) {
        *self.shared.on_event.lock() = Some(f);
    }

    pub fn set_params(&self, p: MixParams) {
        let mut g = self.shared.params.lock();
        if *g != p {
            *g = p;
            self.shared.params_version.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Starts the mixer thread if this is the first user (recording, replay or a level meter).
    pub fn acquire(&self) {
        if self.users.fetch_add(1, Ordering::SeqCst) == 0 {
            self.shared.running.store(true, Ordering::SeqCst);
            let sh = self.shared.clone();
            *self.thread.lock() = std::thread::Builder::new()
                .name("rimlight-audio".into())
                .spawn(move || run(sh))
                .ok();
        }
    }

    pub fn release(&self) {
        let prev = self
            .users
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |u| u.checked_sub(1));
        if prev == Ok(1) {
            self.shared.running.store(false, Ordering::SeqCst);
            if let Some(h) = self.thread.lock().take() {
                let _ = h.join();
            }
            self.shared.mic_level.store(0, Ordering::Relaxed);
            self.shared.sys_level.store(0, Ordering::Relaxed);
        }
    }

    pub fn subscribe(&self) -> (u64, Receiver<Arc<AudioChunk>>) {
        let (tx, rx) = crossbeam_channel::bounded(2000);
        let id = self.shared.next_id.fetch_add(1, Ordering::SeqCst);
        self.shared.subscribers.lock().push((id, tx));
        (id, rx)
    }

    pub fn unsubscribe(&self, id: u64) {
        self.shared.subscribers.lock().retain(|(i, _)| *i != id);
    }

    /// (mic, system) peak levels in 0..1.
    pub fn levels(&self) -> (f32, f32) {
        (
            f32::from_bits(self.shared.mic_level.load(Ordering::Relaxed)),
            f32::from_bits(self.shared.sys_level.load(Ordering::Relaxed)),
        )
    }

    pub fn latency_ms(&self) -> u32 {
        self.shared.stats.latency_ms.load(Ordering::Relaxed)
    }
    pub fn late_ticks(&self) -> u64 {
        self.shared.stats.late_ticks.load(Ordering::Relaxed)
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---- capture sources ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Mic,
    System,
}
impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Mic => "mic",
            Kind::System => "system",
        }
    }
}

struct Source {
    _stream: cpal::Stream,
    ring: Arc<Mutex<VecDeque<f32>>>,
    alive: Arc<AtomicBool>,
    name: String,
    /// True when opened on the default device because the requested one was unavailable.
    on_fallback: bool,
}

fn build_typed<T>(
    device: &cpal::Device,
    cfg: &cpal::StreamConfig,
    ring: Arc<Mutex<VecDeque<f32>>>,
    alive: Arc<AtomicBool>,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let ch = cfg.channels.max(1) as usize;
    let mut rs = Resampler::new(cfg.sample_rate.0);
    let mut stereo: Vec<f32> = Vec::with_capacity(4096);
    let mut out: Vec<f32> = Vec::with_capacity(4096);
    let alive_err = alive.clone();
    device.build_input_stream(
        cfg,
        move |data: &[T], _| {
            stereo.clear();
            for fr in data.chunks_exact(ch) {
                let l = f32::from_sample(fr[0]);
                let r = if ch > 1 { f32::from_sample(fr[1]) } else { l };
                stereo.push(l);
                stereo.push(r);
            }
            out.clear();
            rs.process(&stereo, &mut out);
            let mut g = ring.lock();
            g.extend(out.iter().copied());
            if g.len() > MAX_BACKLOG_SAMPLES * 2 {
                let excess = g.len() - MAX_BACKLOG_SAMPLES;
                g.drain(..excess);
            }
        },
        move |e| {
            tracing::warn!("audio stream error: {e}");
            alive_err.store(false, Ordering::SeqCst);
        },
        None,
    )
}

fn find_device(host: &cpal::Host, kind: Kind, name: &str) -> Option<cpal::Device> {
    if name.is_empty() {
        return match kind {
            Kind::Mic => host.default_input_device(),
            Kind::System => host.default_output_device(),
        };
    }
    let devs = match kind {
        Kind::Mic => host.input_devices().ok()?.collect::<Vec<_>>(),
        Kind::System => host.output_devices().ok()?.collect::<Vec<_>>(),
    };
    let found = devs
        .into_iter()
        .find(|d| d.name().map(|n| n == name).unwrap_or(false));
    found
}

fn open_source(
    host: &cpal::Host,
    kind: Kind,
    wanted: &str,
    allow_fallback: bool,
) -> Result<Source, String> {
    let (device, on_fallback) = match find_device(host, kind, wanted) {
        Some(d) => (d, false),
        None if allow_fallback && !wanted.is_empty() => (
            find_device(host, kind, "").ok_or("no default audio device is available")?,
            true,
        ),
        None => return Err(format!("audio device '{wanted}' was not found")),
    };
    let name = device.name().unwrap_or_else(|_| "unknown".into());
    // System audio = loopback: an *input* stream opened on the render endpoint.
    let supported = match kind {
        Kind::Mic => device.default_input_config(),
        Kind::System => device.default_output_config(),
    }
    .map_err(|e| format!("cannot read the format of '{name}': {e}"))?;
    let fmt = supported.sample_format();
    let cfg: cpal::StreamConfig = supported.into();
    let ring = Arc::new(Mutex::new(VecDeque::with_capacity(48_000)));
    let alive = Arc::new(AtomicBool::new(true));
    let stream = match fmt {
        cpal::SampleFormat::F32 => build_typed::<f32>(&device, &cfg, ring.clone(), alive.clone()),
        cpal::SampleFormat::I16 => build_typed::<i16>(&device, &cfg, ring.clone(), alive.clone()),
        cpal::SampleFormat::I32 => build_typed::<i32>(&device, &cfg, ring.clone(), alive.clone()),
        cpal::SampleFormat::U16 => build_typed::<u16>(&device, &cfg, ring.clone(), alive.clone()),
        other => return Err(format!("unsupported sample format {other:?}")),
    }
    .map_err(|e| format!("cannot open '{name}': {e}"))?;
    stream
        .play()
        .map_err(|e| format!("cannot start '{name}': {e}"))?;
    tracing::info!(
        "audio source {} opened on '{}' ({} ch @ {} Hz, {:?})",
        kind.label(),
        name,
        cfg.channels,
        cfg.sample_rate.0,
        fmt
    );
    Ok(Source {
        _stream: stream,
        ring,
        alive,
        name,
        on_fallback,
    })
}

fn pop_chunk(src: &Option<Source>, out: &mut [f32]) {
    out.fill(0.0);
    if let Some(s) = src {
        let mut g = s.ring.lock();
        // Keep latency small: if we have more than ~60 ms extra, drop the excess (clock drift).
        let target = CHUNK_SAMPLES * 6;
        if g.len() > target * 3 {
            let excess = g.len() - target;
            g.drain(..excess);
        }
        let n = g.len().min(out.len());
        for (o, v) in out.iter_mut().zip(g.drain(..n)) {
            *o = v;
        }
    }
}

struct SourceState {
    src: Option<Source>,
    /// Requested device this state was built for.
    built_for: String,
    next_retry: Instant,
    reported_lost: bool,
}
impl SourceState {
    fn new() -> Self {
        Self {
            src: None,
            built_for: String::new(),
            next_retry: Instant::now(),
            reported_lost: false,
        }
    }
}

fn emit(shared: &Shared, e: AudioEvent) {
    if let Some(f) = shared.on_event.lock().as_ref() {
        f(e);
    }
}

fn maintain(
    shared: &Shared,
    host: &cpal::Host,
    kind: Kind,
    enabled: bool,
    wanted: &str,
    fallback: bool,
    st: &mut SourceState,
    deep_check: bool,
) {
    if !enabled {
        st.src = None;
        st.built_for.clear();
        st.reported_lost = false;
        return;
    }
    let mut need_rebuild = st.src.is_none() && Instant::now() >= st.next_retry;
    if let Some(s) = &st.src {
        if !s.alive.load(Ordering::SeqCst) {
            emit(
                shared,
                AudioEvent::Disconnected {
                    kind: kind.label().into(),
                    device: s.name.clone(),
                },
            );
            st.src = None;
            need_rebuild = true;
        } else if st.built_for != wanted {
            need_rebuild = true;
        } else if deep_check {
            // default device changed, or the preferred device came back after a fallback
            let current_default = find_device(host, kind, "")
                .and_then(|d| d.name().ok())
                .unwrap_or_default();
            if wanted.is_empty() && s.name != current_default {
                need_rebuild = true;
            } else if !wanted.is_empty()
                && s.on_fallback
                && find_device(host, kind, wanted).is_some()
            {
                need_rebuild = true;
            } else if !wanted.is_empty()
                && !s.on_fallback
                && find_device(host, kind, wanted).is_none()
            {
                emit(
                    shared,
                    AudioEvent::Disconnected {
                        kind: kind.label().into(),
                        device: s.name.clone(),
                    },
                );
                st.src = None;
                need_rebuild = true;
            }
        }
    }
    if need_rebuild {
        st.src = None;
        match open_source(host, kind, wanted, fallback) {
            Ok(s) => {
                if s.on_fallback || st.reported_lost {
                    emit(
                        shared,
                        AudioEvent::Switched {
                            kind: kind.label().into(),
                            device: s.name.clone(),
                        },
                    );
                }
                st.reported_lost = s.on_fallback;
                st.built_for = wanted.to_string();
                st.src = Some(s);
            }
            Err(reason) => {
                if !st.reported_lost {
                    emit(
                        shared,
                        AudioEvent::Failed {
                            kind: kind.label().into(),
                            reason,
                        },
                    );
                }
                st.reported_lost = true;
                st.built_for = wanted.to_string();
                st.next_retry = Instant::now() + Duration::from_secs(2);
            }
        }
    }
}

#[cfg(windows)]
fn key_down(vk: u32) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    vk != 0 && unsafe { GetAsyncKeyState(vk as i32) } as u16 & 0x8000 != 0
}

fn run(shared: Arc<Shared>) {
    let host = cpal::default_host();
    let mut sys = SourceState::new();
    let mut mic = SourceState::new();
    let mut params = shared.params.lock().clone();
    let mut version = shared.params_version.load(Ordering::SeqCst);
    let (mut gate, mut comp, mut lim_mic, mut lim_mix) = (
        Gate::new(),
        Compressor::new(),
        Limiter::new(),
        Limiter::new(),
    );
    let start = Instant::now();
    let mut produced: u64 = 0;
    let mut last_deep = Instant::now();
    let mut mute_gain = 1.0f32;
    let (mut mic_lv, mut sys_lv) = (0.0f32, 0.0f32);
    let mut sys_buf = vec![0.0f32; CHUNK_SAMPLES];
    let mut mic_buf = vec![0.0f32; CHUNK_SAMPLES];
    let mut level_tick = 0u32;

    while shared.running.load(Ordering::SeqCst) {
        let v = shared.params_version.load(Ordering::SeqCst);
        if v != version {
            version = v;
            params = shared.params.lock().clone();
        }
        let deep = last_deep.elapsed() >= Duration::from_secs(2);
        if deep {
            last_deep = Instant::now();
        }
        maintain(
            &shared,
            &host,
            Kind::System,
            params.system_enabled,
            &params.system_device,
            params.fallback_default,
            &mut sys,
            deep,
        );
        maintain(
            &shared,
            &host,
            Kind::Mic,
            params.mic_enabled,
            &params.mic_device,
            params.fallback_default,
            &mut mic,
            deep,
        );

        let due = (start.elapsed().as_secs_f64() * 48_000.0) as u64;
        if due > produced + 48_000 {
            produced = due - CHUNK_FRAMES as u64; // resumed from sleep / long stall
            shared.stats.late_ticks.fetch_add(1, Ordering::Relaxed);
        }
        while produced + CHUNK_FRAMES as u64 <= due {
            produced += CHUNK_FRAMES as u64;
            pop_chunk(&sys.src, &mut sys_buf);
            pop_chunk(&mic.src, &mut mic_buf);
            if let Some(s) = &mic.src {
                let backlog_ms = (s.ring.lock().len() / 2) as u32 / 48;
                shared
                    .stats
                    .latency_ms
                    .store(10 + backlog_ms, Ordering::Relaxed);
            }

            // system: volume only
            let sv = params.system_volume;
            if (sv - 1.0).abs() > 1e-3 {
                sys_buf.iter_mut().for_each(|x| *x *= sv);
            }
            // microphone chain
            let pressed = key_down(params.ptt_vk);
            let muted = match params.ptt_mode.as_str() {
                "push_to_talk" => !pressed,
                "push_to_mute" => pressed,
                _ => false,
            };
            let target = if muted { 0.0 } else { 1.0 };
            let step = (target - mute_gain) / CHUNK_FRAMES as f32;
            let g = db_to_lin(params.mic_gain_db) * params.mic_volume;
            for f in mic_buf.chunks_exact_mut(2) {
                mute_gain += step;
                f[0] *= g * mute_gain;
                f[1] *= g * mute_gain;
            }
            mute_gain = target;
            if params.gate_enabled {
                gate.process(&mut mic_buf, params.gate_db);
            }
            if params.compressor {
                comp.process(&mut mic_buf);
            }
            if params.limiter {
                lim_mic.process(&mut mic_buf, -1.0);
            }

            let mut mix: Vec<f32> = sys_buf
                .iter()
                .zip(mic_buf.iter())
                .map(|(a, b)| a + b)
                .collect();
            lim_mix.process(&mut mix, -0.5);

            mic_lv = peak(&mic_buf).max(mic_lv * 0.94);
            sys_lv = peak(&sys_buf).max(sys_lv * 0.94);
            level_tick += 1;
            if level_tick % 5 == 0 {
                shared
                    .mic_level
                    .store(mic_lv.min(1.0).to_bits(), Ordering::Relaxed);
                shared
                    .sys_level
                    .store(sys_lv.min(1.0).to_bits(), Ordering::Relaxed);
            }
            shared.stats.chunks.fetch_add(1, Ordering::Relaxed);

            let subs = shared.subscribers.lock();
            if !subs.is_empty() {
                let chunk = Arc::new(AudioChunk {
                    mix,
                    system: sys_buf.clone(),
                    mic: mic_buf.clone(),
                });
                for (_, tx) in subs.iter() {
                    let _ = tx.try_send(chunk.clone());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(4));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_names_map_to_virtual_keys() {
        assert_eq!(key_to_vk("V"), Some(0x56));
        assert_eq!(key_to_vk("f9"), Some(0x78));
        assert_eq!(key_to_vk("Mouse4"), Some(5));
        assert_eq!(key_to_vk("nonsense"), None);
    }

    #[test]
    fn params_from_settings_convert_percent() {
        let mut a = crate::settings::AudioSettings::default();
        a.system_volume = 80;
        a.ptt_key = "F5".into();
        let p = MixParams::from_settings(&a);
        assert!((p.system_volume - 0.8).abs() < 1e-6);
        assert_eq!(p.ptt_vk, 0x74);
    }

    #[test]
    fn mixer_emits_chunks_on_schedule_even_without_devices() {
        let eng = AudioEngine::new();
        eng.set_params(MixParams {
            system_enabled: false,
            mic_enabled: false,
            ..MixParams::default()
        });
        let (id, rx) = eng.subscribe();
        eng.acquire();
        std::thread::sleep(Duration::from_millis(1000));
        eng.release();
        eng.unsubscribe(id);
        let n = rx.try_iter().count();
        // 1 s ≈ 100 chunks of 10 ms; allow scheduling slack
        assert!((85..=115).contains(&n), "got {n} chunks");
    }
}
