//! Background watchers. Each one only runs while it has work to do:
//! * supervisor – while a sink exists: encoder liveness, capture-source recovery, privacy,
//!   active-window following, app-exit auto-stop, dropped-frame and disk health.
//! * stats / levels hubs – only while a window subscribed to them.
//! * game watcher – a 2 s foreground-window check (a few microseconds each).
use super::Engine;
use crate::capture::TargetSpec;
use crate::error::AppError;
use crate::state_machine::RecState;
use crate::system::stats::{Sampler, SystemStats};
use crate::system::winenum;
use parking_lot::{Condvar, Mutex};
use serde::Serialize;
use serde_json::json;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;

#[derive(Default)]
pub struct StatsHub {
    stats_subs: Mutex<usize>,
    levels_subs: Mutex<usize>,
    cv: Condvar,
    levels_cv: Condvar,
}

impl StatsHub {
    fn adjust(counter: &Mutex<usize>, delta: i32) -> (usize, usize) {
        let mut g = counter.lock();
        let before = *g;
        *g = (*g as i64 + delta as i64).max(0) as usize;
        (before, *g)
    }
    pub fn stats_subscribe(&self, delta: i32) {
        Self::adjust(&self.stats_subs, delta);
        self.cv.notify_all();
    }
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CaptureStats {
    pub fps: f32,
    pub low1: f32,
    pub width: u32,
    pub height: u32,
    pub regulator_skipped: u64,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SinkStats {
    pub bitrate_kbps: f32,
    pub encoder_fps: f32,
    pub speed: f32,
    pub dropped_frames: u64,
    pub dropped_pct: f32,
    pub size_bytes: u64,
    pub write_kbps: f32,
    pub encoder: String,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatsPayload {
    pub system: SystemStats,
    pub capture: Option<CaptureStats>,
    pub recording: Option<SinkStats>,
    pub replay: Option<SinkStats>,
    pub audio_latency_ms: u32,
    pub audio_late_ticks: u64,
}

impl Engine {
    fn sink_stats(
        sink: &crate::sink::Sink,
        label: &str,
        ticks: u64,
        prev_size: &mut u64,
        dt: f32,
    ) -> SinkStats {
        let p = &sink.progress;
        let size = p.total_size.load(Ordering::Relaxed);
        let write_kbps = if dt > 0.0 {
            size.saturating_sub(*prev_size) as f32 / 1024.0 / dt
        } else {
            0.0
        };
        *prev_size = size;
        let dropped = sink.dropped.load(Ordering::Relaxed);
        SinkStats {
            bitrate_kbps: p.bitrate_kbps(),
            encoder_fps: p.fps(),
            speed: p.speed(),
            dropped_frames: dropped,
            dropped_pct: if ticks > 0 {
                dropped as f32 * 100.0 / ticks as f32
            } else {
                0.0
            },
            size_bytes: size,
            write_kbps,
            encoder: label.to_string(),
        }
    }

    pub fn snapshot_stats(
        &self,
        sampler: &mut Sampler,
        prev: &mut (u64, u64),
        dt: f32,
    ) -> StatsPayload {
        let (session, pids) = {
            let g = self.inner.lock();
            let mut pids = vec![];
            if let Some(r) = g.rec.as_ref() {
                pids.push(r.sink.pid());
            }
            if let Some(r) = g.replay.as_ref() {
                pids.push(r.sink.pid());
            }
            (g.session.clone(), pids)
        };
        let system = sampler.sample(&pids);
        let capture = session.as_ref().and_then(|s| {
            s.shared.fps.lock().stats().map(|(fps, low1)| CaptureStats {
                fps,
                low1,
                width: s.canvas.0,
                height: s.canvas.1,
                regulator_skipped: s.stats.skipped.load(Ordering::Relaxed),
            })
        });
        let ticks = session
            .as_ref()
            .map(|s| s.stats.ticks.load(Ordering::Relaxed))
            .unwrap_or(0);
        let g = self.inner.lock();
        let recording = g
            .rec
            .as_ref()
            .map(|r| Self::sink_stats(&r.sink, &r.encoder_label, ticks, &mut prev.0, dt));
        let replay = g
            .replay
            .as_ref()
            .map(|r| Self::sink_stats(&r.sink, &r.encoder_label, ticks, &mut prev.1, dt));
        drop(g);
        StatsPayload {
            system,
            capture,
            recording,
            replay,
            audio_latency_ms: self.audio.latency_ms(),
            audio_late_ticks: self.audio.late_ticks(),
        }
    }

    // ---- hubs -------------------------------------------------------------------------------------
    pub fn spawn_hubs(self: &Arc<Self>) {
        let me = self.clone();
        let _ = std::thread::Builder::new()
            .name("rimlight-stats".into())
            .spawn(move || {
                let mut sampler = Sampler::new();
                let mut prev = (0u64, 0u64);
                let mut last = Instant::now();
                loop {
                    {
                        let mut g = me.hub.stats_subs.lock();
                        while *g == 0 {
                            me.hub.cv.wait(&mut g);
                        }
                    }
                    let dt = last.elapsed().as_secs_f32();
                    last = Instant::now();
                    let payload = me.snapshot_stats(&mut sampler, &mut prev, dt);
                    let _ = me.app.emit("stats", &payload);
                    let mut g = me.hub.stats_subs.lock();
                    if *g > 0 {
                        me.hub.cv.wait_for(&mut g, Duration::from_millis(1000));
                    }
                }
            });
        let me = self.clone();
        let _ = std::thread::Builder::new()
            .name("rimlight-levels".into())
            .spawn(move || loop {
                {
                    let mut g = me.hub.levels_subs.lock();
                    while *g == 0 {
                        me.hub.levels_cv.wait(&mut g);
                    }
                }
                let (mic, sys) = me.audio.levels();
                let _ = me
                    .app
                    .emit("audio-levels", json!({ "mic": mic, "system": sys }));
                let mut g = me.hub.levels_subs.lock();
                if *g > 0 {
                    me.hub.levels_cv.wait_for(&mut g, Duration::from_millis(66));
                }
            });
    }

    /// Meters need the mixer running even when nothing records.
    pub fn levels_subscribe(&self, on: bool) {
        let (before, after) = StatsHub::adjust(&self.hub.levels_subs, if on { 1 } else { -1 });
        if before == 0 && after == 1 {
            self.audio.acquire();
        } else if before == 1 && after == 0 {
            self.audio.release();
        }
        self.hub.levels_cv.notify_all();
    }

    // ---- game watcher ---------------------------------------------------------------------------------
    pub fn spawn_game_watcher(self: &Arc<Self>) {
        let me = self.clone();
        let _ = std::thread::Builder::new()
            .name("rimlight-games".into())
            .spawn(move || loop {
                std::thread::sleep(Duration::from_secs(2));
                let detected = winenum::detect_game();
                let changed = {
                    let mut g = me.inner.lock();
                    let changed = match (&detected, &g.game) {
                        (Some(d), Some(o)) => d.exe != o.exe || d.hwnd != o.hwnd,
                        (Some(_), None) => true,
                        (None, Some(o)) => !winenum::process_alive(o.pid),
                        (None, None) => false,
                    };
                    // keep the last game while it is alive but not in the foreground (alt-tab)
                    match (&detected, &g.game) {
                        (Some(_), _) => g.game = detected.clone(),
                        (None, Some(old)) if !winenum::process_alive(old.pid) => g.game = None,
                        _ => {}
                    }
                    changed
                };
                if changed {
                    if let Some(game) = detected {
                        me.on_game_detected(game);
                    }
                    me.emit_status();
                }
            });
    }

    fn on_game_detected(self: &Arc<Self>, game: winenum::GameInfo) {
        let _ = self.app.emit("game-detected", &game);
        if self.state() != RecState::Idle && self.state() != RecState::Error {
            return;
        }
        // automatic per-game / per-app profile
        if let Some(p) = self.db.profile_for_exe(&game.exe) {
            let mut s = self.settings.read().clone();
            if s.active_profile_id != p.id {
                s.active_profile_id = p.id.clone();
                s.audio.mic_enabled = p.mic_enabled;
                s.audio.system_enabled = p.system_enabled;
                s.camera.enabled = p.camera_enabled;
                if self.apply_settings(s).is_ok() {
                    self.notify(
                        "info",
                        "profile_switched",
                        json!({ "profile": p.name, "game": game.name }),
                        None,
                    );
                }
            }
        }
    }

    // ---- supervisor -----------------------------------------------------------------------------------
    pub fn ensure_supervisor(self: &Arc<Self>) {
        {
            let mut g = self.inner.lock();
            if g.supervisor {
                return;
            }
            g.supervisor = true;
        }
        let me = self.clone();
        let _ = std::thread::Builder::new()
            .name("rimlight-supervisor".into())
            .spawn(move || me.supervise());
    }

    fn supervise(self: Arc<Self>) {
        let mut tick = 0u64;
        let mut lost_source_ticks = 0u32;
        let mut privacy_paused = false;
        let mut last_health = (0u64, 0u64, Instant::now());
        loop {
            std::thread::sleep(Duration::from_millis(500));
            tick += 1;
            let (has_rec, has_replay, session) = {
                let g = self.inner.lock();
                (g.rec.is_some(), g.replay.is_some(), g.session.clone())
            };
            if !has_rec && !has_replay {
                self.inner.lock().supervisor = false;
                return;
            }
            let Some(session) = session else { continue };
            let state = self.state();

            // encoder liveness
            let (rec_dead, replay_dead) = {
                let mut g = self.inner.lock();
                (
                    g.rec.as_mut().map(|r| !r.sink.alive()).unwrap_or(false),
                    g.replay.as_mut().map(|r| !r.sink.alive()).unwrap_or(false),
                )
            };
            if rec_dead && matches!(state, RecState::Recording | RecState::Paused) {
                self.notify("error", "encoder_died", json!({}), None);
                let _ = self.stop_recording();
                continue;
            }
            if replay_dead {
                self.notify("error", "replay_died", json!({}), None);
                let me = self.clone();
                std::thread::spawn(move || {
                    let _ = me.stop_replay();
                });
                continue;
            }

            // capture source: window closed, display removed, GPU reset, resume from sleep …
            if session.source_gone() {
                lost_source_ticks += 1;
                if let Ok(r) = self.resolve_target() {
                    if session.retarget(r.spec, true).is_ok() {
                        tracing::info!("capture source recovered");
                        lost_source_ticks = 0;
                    }
                }
                if lost_source_ticks > 10 && has_rec {
                    self.notify("warning", "source_lost", json!({}), None);
                    let _ = self.stop_recording();
                    continue;
                }
            } else {
                lost_source_ticks = 0;
            }

            // stop when the recorded app exits
            let bound = self.inner.lock().rec.as_ref().and_then(|r| r.bound_pid);
            if let Some(pid) = bound {
                if !winenum::process_alive(pid)
                    && matches!(state, RecState::Recording | RecState::Paused)
                {
                    self.notify("info", "app_closed", json!({}), None);
                    let _ = self.stop_recording();
                    continue;
                }
            }

            let s = self.settings.read().clone();
            // follow the active window
            if s.capture.mode == "active" {
                if let Some(w) = winenum::foreground() {
                    let cur = session.spec.lock().clone();
                    let protected = s
                        .privacy
                        .protected_apps
                        .iter()
                        .any(|a| a.eq_ignore_ascii_case(&w.exe));
                    if !protected
                        && !w.title.is_empty()
                        && cur != (TargetSpec::Window { hwnd: w.hwnd })
                    {
                        let _ = session.retarget(TargetSpec::Window { hwnd: w.hwnd }, false);
                    }
                }
            }
            // privacy
            privacy_paused = self.apply_privacy(&s, &session, privacy_paused);

            // health once per second
            if tick % 2 == 0 && self.check_health(&session, &mut last_health) {
                let _ = self.stop_recording();
            }
        }
    }

    fn apply_privacy(
        &self,
        s: &crate::settings::Settings,
        session: &Arc<crate::capture::Session>,
        was_paused: bool,
    ) -> bool {
        let protected: Vec<String> = s
            .privacy
            .protected_apps
            .iter()
            .map(|a| a.to_lowercase())
            .filter(|a| !a.is_empty())
            .collect();
        if protected.is_empty() {
            if was_paused {
                self.set_sink_pause(false);
            }
            session.fx.privacy_rects.lock().clear();
            return false;
        }
        let hits: Vec<_> = winenum::list_windows()
            .into_iter()
            .filter(|w| protected.contains(&w.exe.to_lowercase()))
            .collect();
        if s.privacy.action == "pause" {
            let want = !hits.is_empty();
            if want != was_paused {
                self.set_sink_pause(want);
                if want {
                    self.notify("info", "privacy_paused", json!({}), None);
                }
            }
            session.fx.privacy_rects.lock().clear();
            want
        } else {
            let (ox, oy) = (
                session.fx.origin_x.load(Ordering::Relaxed),
                session.fx.origin_y.load(Ordering::Relaxed),
            );
            *session.fx.privacy_rects.lock() = hits
                .iter()
                .map(|w| (w.x - ox, w.y - oy, w.width as i32, w.height as i32))
                .collect();
            false
        }
    }

    /// Pauses/resumes the encoders for privacy without touching the user-visible recording state.
    fn set_sink_pause(&self, pause: bool) {
        let g = self.inner.lock();
        let user_paused = g.state == Some(RecState::Paused);
        for p in [
            g.rec.as_ref().map(|r| &r.sink),
            g.replay.as_ref().map(|r| &r.sink),
        ]
        .into_iter()
        .flatten()
        {
            if !pause && user_paused && p.kind == crate::sink::SinkKind::Recording {
                continue;
            }
            p.paused.store(pause, Ordering::SeqCst);
        }
    }

    /// Returns true when the recording must be stopped (disk almost full).
    fn check_health(
        &self,
        session: &Arc<crate::capture::Session>,
        last: &mut (u64, u64, Instant),
    ) -> bool {
        let ticks = session.stats.ticks.load(Ordering::Relaxed);
        let (dropped, path) = {
            let g = self.inner.lock();
            (
                g.rec
                    .as_ref()
                    .map(|r| r.sink.dropped.load(Ordering::Relaxed))
                    .unwrap_or(0)
                    + session.stats.skipped.load(Ordering::Relaxed),
                g.rec.as_ref().map(|r| r.tmp_path.clone()),
            )
        };
        if last.2.elapsed() >= Duration::from_secs(6) {
            let (dt, dd) = (ticks.saturating_sub(last.0), dropped.saturating_sub(last.1));
            *last = (ticks, dropped, Instant::now());
            if dt > 60 && dd * 100 / dt.max(1) >= 3 {
                let pct = (dd as f64 * 1000.0 / dt as f64).round() / 10.0;
                let mut g = self.inner.lock();
                if g.last_warn
                    .map(|t| t.elapsed() > Duration::from_secs(30))
                    .unwrap_or(true)
                {
                    g.last_warn = Some(Instant::now());
                    drop(g);
                    self.notify("warning", "perf_degraded", json!({ "pct": pct }), None);
                }
            }
        }
        if let Some(p) = path {
            if let Some(d) = crate::storage::disk_space(&p) {
                if d.free_bytes < 100 * 1024 * 1024 {
                    self.notify("error", "disk_full_stopped", json!({}), None);
                    return true;
                }
            }
        }
        false
    }
}

/// Convenience for commands: wrap a `Result<T, AppError>` failure into a toast as well.
pub fn toast_error(engine: &Engine, e: &AppError) {
    engine.set_error(e.clone());
}
