//! Instant Replay: a rolling buffer of ~2 s encoded segments on disk (see `replay::buffer`).
use super::{now_ms, Engine, Replay};
use crate::encoder::args::Output;
use crate::error::{AppError, Result};
use crate::naming::{self, NameContext};
use crate::replay::buffer;
use crate::sink::SinkKind;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;

pub const SEGMENT_SECS: u32 = 2;

fn janitor(dir: PathBuf, keep_secs: f64, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::SeqCst) {
        for _ in 0..30 {
            if stop.load(Ordering::SeqCst) {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let Ok(text) = std::fs::read_to_string(dir.join("list.csv")) else {
            continue;
        };
        let segs = buffer::parse_list(&text);
        for s in buffer::expired(&segs, keep_secs) {
            let _ = std::fs::remove_file(dir.join(&s.name));
        }
    }
}

/// Removes replay caches left behind by a previous run (crash or forced exit).
pub fn purge_stale_cache() {
    if let Ok(rd) = std::fs::read_dir(crate::paths::replay_cache_dir()) {
        for e in rd.flatten() {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

impl Engine {
    pub fn start_replay(self: &Arc<Self>) -> Result<()> {
        let _op = self.op_lock.lock();
        if self.inner.lock().replay.is_some() {
            return Ok(());
        }
        self.require_ffmpeg()?;
        let settings = self.settings.read().clone();
        let profile = self.active_profile();
        let (session, resolved) = self.ensure_session(profile.quality.fps)?;
        let dir = crate::paths::replay_cache_dir().join(uuid::Uuid::new_v4().simple().to_string());
        std::fs::create_dir_all(&dir).map_err(|e| {
            AppError::new("io", format!("Cannot create the replay cache folder: {e}"))
        })?;
        let cam = if settings.camera.enabled && !settings.camera.device.is_empty() {
            Some(settings.camera.clone())
        } else {
            None
        };
        let d2 = dir.clone();
        let launched = self.launch_sink(
            SinkKind::Replay,
            &session,
            &profile.quality,
            move || Output::Segments {
                dir: d2.clone(),
                seg_secs: SEGMENT_SECS,
            },
            cam,
        );
        let (sink, label) = match launched {
            Ok(v) => v,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&dir);
                self.release_session_if_idle();
                return Err(e);
            }
        };
        let stop_flag = Arc::new(AtomicBool::new(false));
        let keep = settings.replay.duration_secs as f64 + 4.0 * SEGMENT_SECS as f64;
        let jd = dir.clone();
        let jf = stop_flag.clone();
        let janitor = std::thread::Builder::new()
            .name("rimlight-replay-janitor".into())
            .spawn(move || janitor(jd, keep, jf))
            .ok();
        let (ow, oh) = profile
            .quality
            .output_size(session.canvas.0, session.canvas.1);
        self.inner.lock().replay = Some(Replay {
            sink,
            dir,
            duration_secs: settings.replay.duration_secs,
            stop_flag,
            janitor,
            since_ms: now_ms(),
            encoder_label: label,
            game: resolved.game,
            width: ow,
            height: oh,
            fps: session.fps,
        });
        self.notify(
            "success",
            "replay_on",
            json!({ "seconds": settings.replay.duration_secs }),
            None,
        );
        self.ensure_supervisor();
        crate::shell::on_recording_changed(self);
        self.emit_status();
        Ok(())
    }

    pub fn stop_replay(self: &Arc<Self>) -> Result<()> {
        let _op = self.op_lock.lock();
        let Some(r) = self.inner.lock().replay.take() else {
            return Ok(());
        };
        r.stop_flag.store(true, Ordering::SeqCst);
        if let Some(j) = r.janitor {
            let _ = j.join();
        }
        let _ = r.sink.finish();
        self.audio.release();
        let _ = std::fs::remove_dir_all(&r.dir);
        self.release_session_if_idle();
        self.notify("info", "replay_off", json!({}), None);
        crate::shell::on_recording_changed(self);
        self.emit_status();
        Ok(())
    }

    pub fn toggle_replay(self: &Arc<Self>) -> Result<()> {
        if self.inner.lock().replay.is_some() {
            self.stop_replay()
        } else {
            self.start_replay()
                .inspect_err(|e| self.set_error(e.clone()))
        }
    }

    /// Saves the last `replay.duration_secs` seconds. Runs in the background; returns immediately.
    pub fn save_replay(self: &Arc<Self>) -> Result<()> {
        let (dir, secs, game, label) = {
            let mut g = self.inner.lock();
            let Some(r) = g.replay.as_ref() else {
                return Err(
                    AppError::new("replay_off", "Instant Replay is not running.")
                        .hint("Turn Instant Replay on first, then press the save shortcut."),
                );
            };
            if g.replay_saving {
                return Err(AppError::new(
                    "replay_busy",
                    "A replay is already being saved.",
                ));
            }
            let v = (
                r.dir.clone(),
                r.duration_secs,
                r.game.clone(),
                r.encoder_label.clone(),
            );
            g.replay_saving = true;
            v
        };
        let _ = self.app.emit("replay-save", json!({ "stage": "saving" }));
        self.emit_status();
        let me = self.clone();
        std::thread::Builder::new()
            .name("rimlight-replay-save".into())
            .spawn(move || {
                let result = me.save_replay_blocking(&dir, secs, &game, &label);
                me.inner.lock().replay_saving = false;
                match result {
                    Ok(clip) => {
                        let _ = me.app.emit("replay-save", json!({ "stage": "done" }));
                        me.notify(
                            "success",
                            "replay_saved",
                            json!({ "seconds": clip.duration_ms / 1000 }),
                            Some(clip.path),
                        );
                        me.after_save();
                    }
                    Err(e) => {
                        let _ = me.app.emit("replay-save", json!({ "stage": "error" }));
                        me.set_error(e);
                    }
                }
                me.emit_status();
            })
            .map_err(AppError::internal)?;
        Ok(())
    }

    fn save_replay_blocking(
        &self,
        dir: &Path,
        secs: u32,
        game: &str,
        label: &str,
    ) -> Result<crate::db::Clip> {
        let text = std::fs::read_to_string(dir.join("list.csv")).unwrap_or_default();
        let list = buffer::parse_list(&text);
        let mut names: Vec<String> = buffer::select_tail(&list, secs as f64)
            .into_iter()
            .map(|s| s.name)
            .collect();

        // the segment that is still being written is a valid (truncated) MPEG-TS file – include it
        let last_listed = names.last().cloned();
        let mut partial: Vec<String> = std::fs::read_dir(dir)
            .map(|rd| {
                rd.flatten()
                    .filter_map(|e| e.file_name().into_string().ok())
                    .filter(|n| n.starts_with("seg_") && n.ends_with(".ts"))
                    .collect()
            })
            .unwrap_or_default();
        partial.sort();
        let newest = list.last().map(|s| s.name.clone());
        if let Some(n) = partial
            .into_iter()
            .filter(|n| newest.as_ref().map(|nw| n > nw).unwrap_or(true))
            .last()
        {
            if std::fs::metadata(dir.join(&n))
                .map(|m| m.len() > 4096)
                .unwrap_or(false)
            {
                names.push(n);
            }
        }
        if names.is_empty() {
            return Err(
                AppError::new("replay_empty", "Nothing has been buffered yet.")
                    .hint("Wait a few seconds after turning Instant Replay on."),
            );
        }

        let settings = self.settings.read().clone();
        let out_dir = PathBuf::from(&settings.storage.recordings_dir);
        std::fs::create_dir_all(&out_dir).map_err(|e| {
            AppError::new(
                "folder_unavailable",
                format!("The recordings folder is unavailable: {e}"),
            )
        })?;
        let stem = format!(
            "{}_Replay",
            naming::render(
                &settings.file_name_template,
                &NameContext {
                    game,
                    profile: "",
                    resolution: "",
                    fps: 0,
                    now: chrono::Local::now()
                }
            )
        );
        let out = naming::unique_path(&out_dir, &stem, "mp4");

        let run = |names: &[String]| -> bool {
            let manifest = dir.join("concat.txt");
            if std::fs::write(&manifest, buffer::concat_manifest(dir, names)).is_err() {
                return false;
            }
            crate::encoder::command(&crate::encoder::ffmpeg())
                .args([
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-y",
                    "-f",
                    "concat",
                    "-safe",
                    "0",
                    "-i",
                ])
                .arg(&manifest)
                .args(["-map", "0", "-c", "copy", "-movflags", "+faststart"])
                .arg(&out)
                .stdin(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        };
        let mut ok = run(&names);
        if !ok && names.last() != last_listed.as_ref() {
            // retry without the in-progress segment
            let _ = std::fs::remove_file(&out);
            names.pop();
            ok = !names.is_empty() && run(&names);
        }
        if !ok {
            return Err(AppError::new(
                "replay_save_failed",
                "The replay could not be assembled from the buffer.",
            )
            .hint("Check the logs in Settings → Advanced."));
        }
        self.register_clip(&out, "replay", game, label, now_ms())
    }
}
