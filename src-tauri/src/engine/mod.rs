//! The capture engine: owns the capture session, the recorder and Instant Replay sinks, and the
//! recording state machine. Everything long-running executes on native threads – nothing here
//! depends on the WebView, so a UI crash or reload never interrupts an active recording.
mod finalize;
mod recording;
mod replay;
pub mod supervisor;

use crate::audio::{AudioEngine, MixParams};
use crate::capture::{Session, TargetSpec};
use crate::db::{Db, Profile};
use crate::encoder::EncoderInfo;
use crate::error::{AppError, Result};
use crate::settings::Settings;
use crate::sink::Sink;
use crate::state_machine::{self, RecEvent, RecState};
use crate::system::hardware::HardwareInfo;
use crate::system::winenum::{self, GameInfo};
use parking_lot::{Mutex, RwLock};
use serde::Serialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

pub use replay::purge_stale_cache as purge_stale_replay_cache;

pub struct Recording {
    pub sink: Sink,
    pub session_id: String,
    pub final_path: PathBuf,
    pub tmp_path: PathBuf,
    pub container: String,
    pub started_ms: i64,
    /// Time spent recording before the current running stretch began (i.e. excluding pauses).
    pub elapsed_base: Duration,
    pub running_since: Option<Instant>,
    pub game: String,
    pub encoder_label: String,
    pub bound_pid: Option<u32>,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl Recording {
    pub fn elapsed(&self) -> Duration {
        self.elapsed_base + self.running_since.map(|t| t.elapsed()).unwrap_or_default()
    }
}

pub struct Replay {
    pub sink: Sink,
    pub dir: PathBuf,
    pub duration_secs: u32,
    pub stop_flag: Arc<AtomicBool>,
    pub janitor: Option<JoinHandle<()>>,
    pub since_ms: i64,
    pub encoder_label: String,
    pub game: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

#[derive(Default)]
pub struct Inner {
    pub session: Option<Arc<Session>>,
    pub rec: Option<Recording>,
    pub replay: Option<Replay>,
    pub state: Option<RecState>,
    pub error: Option<AppError>,
    pub game: Option<GameInfo>,
    pub replay_saving: bool,
    pub supervisor: bool,
    /// What to do with the region chosen in the picker: "screenshot" or "select".
    pub pending_region: Option<String>,
    pub last_warn: Option<Instant>,
}

pub struct Engine {
    pub app: AppHandle,
    pub db: Arc<Db>,
    pub settings: RwLock<Settings>,
    pub audio: Arc<AudioEngine>,
    pub encoders: RwLock<Vec<EncoderInfo>>,
    pub hardware: RwLock<Option<HardwareInfo>>,
    pub hub: Arc<supervisor::StatsHub>,
    pub(crate) op_lock: Mutex<()>,
    pub(crate) inner: Mutex<Inner>,
    pub started_at: Instant,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecInfo {
    pub path: String,
    pub started_ms: i64,
    /// Recorded time before the current running stretch (excludes pauses).
    pub elapsed_base_ms: u64,
    /// Epoch ms when the current running stretch began; `None` while paused.
    pub running_since_ms: Option<i64>,
    pub encoder: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub game: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ReplayInfo {
    pub since_ms: i64,
    pub duration_secs: u32,
    pub encoder: String,
    pub saving: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TargetInfo {
    pub mode: String,
    pub label: String,
    pub display_index: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusDto {
    pub state: RecState,
    pub recording: Option<RecInfo>,
    pub replay: Option<ReplayInfo>,
    pub target: TargetInfo,
    pub game: Option<GameInfo>,
    pub error: Option<AppError>,
    pub profile_id: String,
}

/// A concrete capture target resolved from the user's capture mode.
#[derive(Clone, Debug)]
pub struct Resolved {
    pub spec: TargetSpec,
    /// Game / application name used for clip naming ("" = desktop).
    pub game: String,
    pub pid: Option<u32>,
    pub exe: String,
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

impl Engine {
    pub fn new(app: AppHandle, db: Arc<Db>, settings: Settings) -> Arc<Engine> {
        let audio = Arc::new(AudioEngine::new());
        audio.set_params(MixParams::from_settings(&settings.audio));
        let e = Arc::new(Engine {
            app,
            db,
            settings: RwLock::new(settings),
            audio,
            encoders: RwLock::new(vec![]),
            hardware: RwLock::new(None),
            hub: Arc::new(supervisor::StatsHub::default()),
            op_lock: Mutex::new(()),
            inner: Mutex::new(Inner {
                state: Some(RecState::Idle),
                ..Default::default()
            }),
            started_at: Instant::now(),
        });
        let weak = Arc::downgrade(&e);
        e.audio.set_event_handler(Arc::new(move |ev| {
            if let Some(e) = weak.upgrade() {
                e.on_audio_event(ev);
            }
        }));
        e
    }

    // ---- state machine -----------------------------------------------------------------------------
    pub fn state(&self) -> RecState {
        self.inner.lock().state.unwrap_or(RecState::Idle)
    }

    pub(crate) fn transition(&self, ev: RecEvent) -> Result<RecState> {
        let mut g = self.inner.lock();
        let cur = g.state.unwrap_or(RecState::Idle);
        match state_machine::next(cur, ev) {
            Ok(next) => {
                g.state = Some(next);
                Ok(next)
            }
            Err(e) => Err(AppError::new(
                "invalid_state",
                format!(
                    "That action is not available while the recorder is {:?}.",
                    e.0
                )
                .to_lowercase(),
            )),
        }
    }

    // ---- status --------------------------------------------------------------------------------------
    pub fn status(&self) -> StatusDto {
        let s = self.settings.read().clone();
        let g = self.inner.lock();
        let recording = g.rec.as_ref().map(|r| RecInfo {
            path: r.final_path.to_string_lossy().into_owned(),
            started_ms: r.started_ms,
            elapsed_base_ms: r.elapsed_base.as_millis() as u64,
            running_since_ms: r
                .running_since
                .map(|t| now_ms() - t.elapsed().as_millis() as i64),
            encoder: r.encoder_label.clone(),
            width: r.width,
            height: r.height,
            fps: r.fps,
            game: r.game.clone(),
        });
        let replay = g.replay.as_ref().map(|r| ReplayInfo {
            since_ms: r.since_ms,
            duration_secs: r.duration_secs,
            encoder: r.encoder_label.clone(),
            saving: g.replay_saving,
        });
        let (width, height) = g.session.as_ref().map(|x| x.canvas).unwrap_or((0, 0));
        let target = TargetInfo {
            mode: s.capture.mode.clone(),
            label: self.target_label(&s, g.game.as_ref()),
            display_index: s.capture.display_index,
            width,
            height,
        };
        StatusDto {
            state: g.state.unwrap_or(RecState::Idle),
            recording,
            replay,
            target,
            game: g.game.clone(),
            error: g.error.clone(),
            profile_id: s.active_profile_id.clone(),
        }
    }

    fn target_label(&self, s: &Settings, game: Option<&GameInfo>) -> String {
        match s.capture.mode.as_str() {
            "game" => game.map(|g| g.name.clone()).unwrap_or_default(),
            "window" => s.capture.window_title.clone(),
            _ => String::new(),
        }
    }

    pub fn emit_status(&self) {
        let _ = self.app.emit("engine-status", self.status());
    }

    /// Non-intrusive toast shown by the notification window (skipped when notifications are off,
    /// except for problems the user must know about).
    pub fn notify(&self, kind: &str, code: &str, params: serde_json::Value, path: Option<String>) {
        let important = matches!(kind, "error" | "warning");
        if !self.settings.read().performance.notifications && !important {
            return;
        }
        let payload = json!({ "kind": kind, "code": code, "params": params, "path": path, "id": uuid::Uuid::new_v4().to_string() });
        crate::shell::show_toast_window(&self.app);
        let _ = self.app.emit("toast", payload);
    }

    /// Clears a shown error and returns the recorder to Idle.
    pub fn dismiss_error(&self) {
        {
            let mut g = self.inner.lock();
            g.error = None;
            if g.state == Some(RecState::Error) {
                g.state = Some(RecState::Idle);
            }
        }
        self.emit_status();
    }

    pub fn set_error(&self, err: AppError) {
        tracing::error!("{}: {}", err.code, err.message);
        {
            let mut g = self.inner.lock();
            g.error = Some(err.clone());
        }
        self.notify(
            "error",
            &err.code,
            json!({ "message": err.message, "hint": err.hint }),
            None,
        );
    }

    fn on_audio_event(&self, ev: crate::audio::AudioEvent) {
        use crate::audio::AudioEvent as A;
        tracing::info!("audio event: {ev:?}");
        match ev {
            A::Disconnected { kind, device } => self.notify(
                "warning",
                &format!("{kind}_disconnected"),
                json!({ "device": device }),
                None,
            ),
            A::Switched { kind, device } => self.notify(
                "info",
                &format!("{kind}_switched"),
                json!({ "device": device }),
                None,
            ),
            A::Failed { kind, reason } => self.notify(
                "warning",
                &format!("{kind}_failed"),
                json!({ "reason": reason }),
                None,
            ),
        }
        let _ = self.app.emit("audio-devices-changed", ());
    }

    // ---- settings ------------------------------------------------------------------------------------
    /// Applies new settings: persists, pushes audio params, restarts Instant Replay if the capture
    /// pipeline changed and nothing is being recorded, and informs every window.
    pub fn apply_settings(self: &Arc<Self>, mut new: Settings) -> Result<Settings> {
        new.sanitize();
        let old = self.settings.read().clone();
        self.db.save_settings(&new)?;
        *self.settings.write() = new.clone();
        self.audio.set_params(MixParams::from_settings(&new.audio));
        let pipeline_changed = old.capture != new.capture
            || old.active_profile_id != new.active_profile_id
            || old.camera != new.camera;
        let replay_changed = old.replay != new.replay;
        if (pipeline_changed || replay_changed)
            && self.inner.lock().replay.is_some()
            && self.inner.lock().rec.is_none()
        {
            let me = self.clone();
            std::thread::spawn(move || {
                let _ = me.stop_replay();
                if let Err(e) = me.start_replay() {
                    me.set_error(e);
                }
            });
        }
        let _ = self.app.emit("settings-changed", &new);
        Ok(new)
    }

    /// Called after a profile was edited: if it is the active one, an idle Instant Replay picks up the change.
    pub fn profile_changed(self: &Arc<Self>, id: &str) {
        if self.settings.read().active_profile_id != id {
            return;
        }
        let idle = {
            let g = self.inner.lock();
            g.replay.is_some() && g.rec.is_none()
        };
        if idle {
            let me = self.clone();
            std::thread::spawn(move || {
                let _ = me.stop_replay();
                if let Err(e) = me.start_replay() {
                    me.set_error(e);
                }
            });
        }
    }

    pub fn active_profile(&self) -> Profile {
        let id = self.settings.read().active_profile_id.clone();
        self.db
            .get_profile(&id)
            .or_else(|| {
                self.db
                    .list_profiles()
                    .ok()
                    .and_then(|p| p.into_iter().next())
            })
            .unwrap_or_default()
    }

    pub fn toggle_mic(self: &Arc<Self>) {
        let mut s = self.settings.read().clone();
        s.audio.mic_enabled = !s.audio.mic_enabled;
        let on = s.audio.mic_enabled;
        if self.apply_settings(s).is_ok() {
            self.notify(
                "info",
                if on { "mic_on" } else { "mic_off" },
                json!({}),
                None,
            );
        }
    }

    pub fn toggle_camera(self: &Arc<Self>) {
        let mut s = self.settings.read().clone();
        s.camera.enabled = !s.camera.enabled;
        let on = s.camera.enabled;
        if self.apply_settings(s).is_ok() {
            self.notify(
                "info",
                if on { "camera_on" } else { "camera_off" },
                json!({}),
                None,
            );
        }
    }

    // ---- target resolution -----------------------------------------------------------------------------
    /// Turns the configured capture mode into a concrete capture target plus display name / pid.
    pub fn resolve_target(&self) -> Result<Resolved> {
        let s = self.settings.read().clone();
        let c = &s.capture;
        let display = || TargetSpec::Display {
            index: c.display_index.max(1),
        };
        let (spec, game, pid, exe) = match c.mode.as_str() {
            "region" => (
                TargetSpec::Region {
                    index: c.display_index.max(1),
                    rect: c.region.clone(),
                },
                String::new(),
                None,
                String::new(),
            ),
            "window" => {
                let wins = winenum::list_windows();
                let w = wins
                    .iter()
                    .find(|w| !c.window_exe.is_empty() && w.exe.eq_ignore_ascii_case(&c.window_exe) && (c.window_title.is_empty() || w.title == c.window_title))
                    .or_else(|| wins.iter().find(|w| !c.window_exe.is_empty() && w.exe.eq_ignore_ascii_case(&c.window_exe)))
                    .ok_or_else(|| {
                        AppError::new("window_missing", format!("The window \"{}\" is not open.", c.window_title))
                            .hint("Open the application or pick another window in the capture source settings.")
                    })?;
                (
                    TargetSpec::Window { hwnd: w.hwnd },
                    winenum::game_name(w),
                    Some(w.pid),
                    w.exe.clone(),
                )
            }
            "game" => match winenum::detect_game() {
                Some(g) => (
                    TargetSpec::Window { hwnd: g.hwnd },
                    g.name,
                    Some(g.pid),
                    g.exe,
                ),
                None => match self.inner.lock().game.clone() {
                    Some(g)
                        if winenum::process_alive(g.pid)
                            && winenum::window_by_hwnd(g.hwnd).is_some() =>
                    {
                        (
                            TargetSpec::Window { hwnd: g.hwnd },
                            g.name,
                            Some(g.pid),
                            g.exe,
                        )
                    }
                    _ => (display(), String::new(), None, String::new()),
                },
            },
            "active" => match winenum::foreground() {
                Some(w) if !w.title.is_empty() => (
                    TargetSpec::Window { hwnd: w.hwnd },
                    winenum::game_name(&w),
                    Some(w.pid),
                    w.exe.clone(),
                ),
                _ => (display(), String::new(), None, String::new()),
            },
            _ => (display(), String::new(), None, String::new()),
        };
        Ok(Resolved {
            spec,
            game,
            pid,
            exe,
        })
    }

    /// Returns the running capture session or creates one for the current settings.
    pub(crate) fn ensure_session(&self, fps: u32) -> Result<(Arc<Session>, Resolved)> {
        let resolved = self.resolve_target()?;
        let spec = resolved.spec.clone();
        {
            let mut g = self.inner.lock();
            if let Some(s) = g.session.clone() {
                if !s.source_gone() && (s.sink_count() > 0 || s.fps == fps) {
                    return Ok((s, resolved));
                }
                g.session = None;
                drop(g);
                s.stop();
            }
        }
        let c = self.settings.read().capture.clone();
        let session = Session::start(
            spec,
            fps,
            c.capture_cursor,
            c.highlight_cursor,
            c.show_clicks,
        )?;
        self.inner.lock().session = Some(session.clone());
        Ok((session, resolved))
    }

    /// Stops the capture session when nothing consumes it any more.
    pub(crate) fn release_session_if_idle(&self) {
        let mut g = self.inner.lock();
        if g.rec.is_none() && g.replay.is_none() {
            if let Some(s) = g.session.take() {
                drop(g);
                s.stop();
            }
        }
    }

    pub(crate) fn require_ffmpeg(&self) -> Result<()> {
        if crate::encoder::ffmpeg_available() {
            Ok(())
        } else {
            Err(AppError::new(
                "ffmpeg_missing",
                "The video encoder component (ffmpeg) is missing or damaged.",
            )
            .hint("Reinstall Rimlight to restore it."))
        }
    }
}
