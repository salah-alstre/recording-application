//! Starting, pausing and stopping recordings.
use super::{now_ms, Engine, Recording};
use crate::encoder::args::{self, Output};
use crate::encoder::ALL_ENCODERS;
use crate::error::{AppError, Result};
use crate::naming::{self, NameContext};
use crate::quality::Quality;
use crate::settings::CameraSettings;
use crate::sink::{Sink, SinkKind, SinkSpec, TrackSel};
use crate::state_machine::RecEvent;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;

fn encoder_label(id: &str) -> String {
    ALL_ENCODERS
        .iter()
        .find(|e| e.0 == id)
        .map(|e| e.3.to_string())
        .unwrap_or_else(|| id.to_string())
}

impl Engine {
    fn audio_tracks(&self) -> Vec<TrackSel> {
        let a = self.settings.read().audio.clone();
        let mut t = vec![TrackSel::Mix];
        if a.track_system {
            t.push(TrackSel::System);
        }
        if a.track_mic {
            t.push(TrackSel::Mic);
        }
        t
    }

    /// Starts an encoder sink, trying every candidate encoder in fallback order.
    /// Returns the running sink and the human-readable encoder name.
    pub(crate) fn launch_sink(
        &self,
        kind: SinkKind,
        session: &Arc<crate::capture::Session>,
        quality: &Quality,
        output: impl Fn() -> Output,
        camera: Option<CameraSettings>,
    ) -> Result<(Sink, String)> {
        let vendor = if self.settings.read().general.hardware_acceleration {
            quality.encoder.clone()
        } else {
            "cpu".to_string()
        };
        let candidates = args::resolve_encoders(&vendor, &quality.codec, &self.encoders.read());
        let mut q = quality.clone();
        q.fps = session.fps;
        let mut last_err = String::new();
        for (i, enc) in candidates.iter().enumerate() {
            self.audio.acquire();
            let spec = SinkSpec {
                kind,
                src: session.canvas,
                fps: session.fps,
                quality: q.clone(),
                encoder: enc.clone(),
                camera: camera.clone(),
                output: output(),
                tracks: self.audio_tracks(),
            };
            match Sink::launch(spec, &self.audio, session) {
                Ok(sink) => {
                    let label = encoder_label(enc);
                    if i > 0 {
                        self.notify(
                            "warning",
                            "encoder_changed",
                            json!({ "from": encoder_label(&candidates[0]), "to": label }),
                            None,
                        );
                    }
                    return Ok((sink, label));
                }
                Err(e) => {
                    self.audio.release();
                    tracing::warn!(
                        "encoder {enc} failed: {} | {}",
                        e.message,
                        e.stderr.lines().last().unwrap_or("")
                    );
                    last_err = if e.stderr.is_empty() {
                        e.message
                    } else {
                        format!("{} ({})", e.message, e.stderr.lines().last().unwrap_or(""))
                    };
                }
            }
        }
        Err(AppError::new(
            "encoder_failed",
            format!("No video encoder could be started. Last error: {last_err}"),
        )
        .hint("Update your graphics driver, or choose CPU encoding in Recording settings."))
    }

    pub fn start_recording(self: &Arc<Self>) -> Result<()> {
        let _op = self.op_lock.lock();
        self.require_ffmpeg()?;
        self.transition(RecEvent::Start)?;
        self.inner.lock().error = None;
        self.emit_status();
        match self.start_recording_inner() {
            Ok(()) => {
                let _ = self.transition(RecEvent::Started);
                self.emit_status();
                self.notify("success", "recording_started", json!({}), None);
                if self.settings.read().privacy.hide_notifications {
                    crate::system::notifications::suppress(&self.db);
                }
                crate::shell::on_recording_changed(self);
                self.ensure_supervisor();
                Ok(())
            }
            Err(e) => {
                let _ = self.transition(RecEvent::Fail);
                self.set_error(e.clone());
                self.release_session_if_idle();
                self.emit_status();
                Err(e)
            }
        }
    }

    fn start_recording_inner(self: &Arc<Self>) -> Result<()> {
        let settings = self.settings.read().clone();
        let profile = self.active_profile();
        let quality = profile.quality.clone();

        let dir = std::path::PathBuf::from(&settings.storage.recordings_dir);
        std::fs::create_dir_all(&dir).map_err(|e| {
            AppError::new(
                "folder_unavailable",
                format!(
                    "The recordings folder \"{}\" is unavailable: {e}",
                    dir.display()
                ),
            )
            .hint("Choose another folder in Settings → Storage.")
        })?;
        if let Some(d) = crate::storage::disk_space(&dir) {
            if d.free_bytes < 500 * 1024 * 1024 {
                return Err(AppError::new(
                    "disk_full",
                    "There is less than 500 MB of free space in the recordings folder.",
                )
                .hint("Free up space or choose another folder in Settings → Storage."));
            }
        }

        let (session, resolved) = self.ensure_session(quality.fps)?;
        let (w, h) = session.canvas;
        let (ow, oh) = quality.output_size(w, h);

        let stem = naming::render(
            &settings.file_name_template,
            &NameContext {
                game: &resolved.game,
                profile: &profile.name,
                resolution: &format!("{oh}p"),
                fps: session.fps,
                now: chrono::Local::now(),
            },
        );
        let ext = if quality.container == "mkv" {
            "mkv"
        } else {
            "mp4"
        };
        let final_path = naming::unique_path(&dir, &stem, ext);
        let tmp_path = if ext == "mp4" {
            let base = final_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or(stem.clone());
            naming::unique_path(&dir, &format!("{base}.recording"), "mkv")
        } else {
            final_path.clone()
        };

        let mut camera = settings.camera.clone();
        if let Some(gc) = self
            .db
            .get_game_config(&resolved.exe)
            .and_then(|g| g.camera)
        {
            camera.corner = gc.corner;
            camera.size_pct = gc.size_pct;
            camera.shape = gc.shape;
            camera.margin_px = gc.margin_px;
            camera.mirror = gc.mirror;
        }
        let cam = if camera.enabled && !camera.device.is_empty() {
            Some(camera)
        } else {
            None
        };

        let out_path = tmp_path.clone();
        let (sink, label) = self.launch_sink(
            SinkKind::Recording,
            &session,
            &quality,
            move || Output::File {
                path: out_path.clone(),
            },
            cam,
        )?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let _ = self.db.begin_session(&crate::db::SessionRow {
            id: session_id.clone(),
            tmp_path: tmp_path.to_string_lossy().into_owned(),
            final_path: final_path.to_string_lossy().into_owned(),
            kind: "recording".into(),
            started_at: now_ms(),
            game: resolved.game.clone(),
        });
        tracing::info!(
            "recording to {} ({}x{} → {}x{} @ {} fps, {})",
            tmp_path.display(),
            w,
            h,
            ow,
            oh,
            session.fps,
            label
        );
        self.inner.lock().rec = Some(Recording {
            sink,
            session_id,
            final_path,
            tmp_path,
            container: ext.into(),
            started_ms: now_ms(),
            elapsed_base: Duration::ZERO,
            running_since: Some(Instant::now()),
            game: resolved.game,
            encoder_label: label,
            bound_pid: if settings.capture.stop_when_app_closes {
                resolved.pid
            } else {
                None
            },
            width: ow,
            height: oh,
            fps: session.fps,
        });
        Ok(())
    }

    pub fn stop_recording(self: &Arc<Self>) -> Result<()> {
        let _op = self.op_lock.lock();
        self.transition(RecEvent::Stop)?;
        let rec = self.inner.lock().rec.take();
        self.emit_status();
        let Some(rec) = rec else {
            return Ok(());
        };
        crate::shell::on_recording_changed(self);
        let me = self.clone();
        std::thread::Builder::new()
            .name("rimlight-finalize".into())
            .spawn(move || {
                me.finalize_recording(rec);
            })
            .map_err(AppError::internal)?;
        Ok(())
    }

    pub fn toggle_recording(self: &Arc<Self>) -> Result<()> {
        match self.state() {
            crate::state_machine::RecState::Recording | crate::state_machine::RecState::Paused => {
                self.stop_recording()
            }
            crate::state_machine::RecState::Idle | crate::state_machine::RecState::Error => {
                self.start_recording()
            }
            _ => Ok(()), // Preparing / Stopping: ignore rapid repeated key presses
        }
    }

    pub fn pause_recording(self: &Arc<Self>) -> Result<()> {
        let _op = self.op_lock.lock();
        self.transition(RecEvent::Pause)?;
        if let Some(r) = self.inner.lock().rec.as_mut() {
            r.sink
                .paused
                .store(true, std::sync::atomic::Ordering::SeqCst);
            r.elapsed_base = r.elapsed();
            r.running_since = None;
        }
        self.emit_status();
        Ok(())
    }

    pub fn resume_recording(self: &Arc<Self>) -> Result<()> {
        let _op = self.op_lock.lock();
        self.transition(RecEvent::Resume)?;
        if let Some(r) = self.inner.lock().rec.as_mut() {
            r.sink
                .paused
                .store(false, std::sync::atomic::Ordering::SeqCst);
            r.running_since = Some(Instant::now());
        }
        self.emit_status();
        Ok(())
    }

    /// Drops a marker at the current recording time (shown later in the player timeline).
    pub fn add_marker(&self) -> Result<()> {
        let (sid, t) = {
            let g = self.inner.lock();
            let Some(r) = g.rec.as_ref() else {
                return Err(AppError::new(
                    "not_recording",
                    "Markers can only be added while recording.",
                ));
            };
            (r.session_id.clone(), r.elapsed().as_millis() as i64)
        };
        self.db.add_session_marker(&sid, t, "")?;
        let _ = self.app.emit("marker-added", t);
        self.notify("info", "marker_added", json!({ "time": t }), None);
        Ok(())
    }
}
