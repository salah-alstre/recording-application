//! Turning encoder output into validated library entries.
//! "Recording saved" is only ever reported after the file was re-opened with ffprobe and found
//! to have a non-zero duration and size.
use super::{now_ms, Engine, Recording};
use crate::db::Clip;
use crate::encoder;
use crate::error::{AppError, Result};
use crate::state_machine::RecEvent;
use serde_json::json;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use tauri::Emitter;

/// Lossless MKV → MP4 remux with the moov atom up front (fast start).
pub fn remux(src: &Path, dst: &Path) -> bool {
    encoder::command(&encoder::ffmpeg())
        .args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(src)
        .args(["-map", "0", "-c", "copy", "-movflags", "+faststart"])
        .arg(dst)
        .stdin(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        && dst.exists()
}

impl Engine {
    /// Probes a finished video file, creates its thumbnail and stores it in the library.
    pub fn register_clip(
        &self,
        path: &Path,
        kind: &str,
        game: &str,
        encoder_label: &str,
        created_ms: i64,
    ) -> Result<Clip> {
        let info = encoder::probe_media(path)
            .filter(|i| i.duration_ms > 0 && i.size_bytes > 0)
            .ok_or_else(|| {
                AppError::new(
                    "file_invalid",
                    format!(
                        "\"{}\" was written but could not be read back as a valid video.",
                        path.display()
                    ),
                )
                .hint("The file was kept so you can try to open or recover it manually.")
            })?;
        let id = uuid::Uuid::new_v4().to_string();
        let thumb_path =
            crate::thumbs::generate(path, kind, &id, info.duration_ms).unwrap_or_default();
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let clip = Clip {
            id,
            path: path.to_string_lossy().into_owned(),
            kind: kind.into(),
            title,
            game: game.into(),
            created_at: created_ms,
            duration_ms: info.duration_ms,
            width: info.width,
            height: info.height,
            fps: info.fps,
            codec: info.codec,
            encoder: encoder_label.into(),
            size_bytes: info.size_bytes,
            favorite: false,
            notes: String::new(),
            thumb_path,
            audio_tracks: info.audio_tracks,
        };
        self.db.upsert_clip(&clip)?;
        let _ = self.app.emit("library-changed", ());
        Ok(clip)
    }

    pub(crate) fn finalize_recording(self: &Arc<Self>, rec: Recording) {
        let Recording {
            sink,
            session_id,
            final_path,
            tmp_path,
            container,
            game,
            encoder_label,
            started_ms,
            ..
        } = rec;
        let (exit_ok, stderr) = sink.finish();
        self.audio.release();
        if !exit_ok {
            tracing::warn!(
                "encoder exited abnormally: {}",
                stderr.lines().last().unwrap_or("")
            );
        }

        let mut result: Result<Clip> = Err(AppError::new(
            "file_missing",
            "The recording file was not created.",
        ));
        if tmp_path.exists()
            && std::fs::metadata(&tmp_path)
                .map(|m| m.len() > 0)
                .unwrap_or(false)
        {
            let ready = if container == "mp4" {
                if remux(&tmp_path, &final_path) {
                    let _ = std::fs::remove_file(&tmp_path);
                    true
                } else {
                    tracing::error!("remux to MP4 failed; keeping the MKV");
                    false
                }
            } else {
                true
            };
            let file = if ready { &final_path } else { &tmp_path };
            result = self.register_clip(file, "recording", &game, &encoder_label, started_ms);
        }

        match result {
            Ok(clip) => {
                let _ = self.db.promote_session_markers(&session_id, &clip.id);
                let _ = self.db.end_session(&session_id, "done");
                let _ = self.transition(RecEvent::Stopped);
                self.inner.lock().error = None;
                self.notify(
                    "success",
                    "recording_saved",
                    json!({ "name": clip.title }),
                    Some(clip.path.clone()),
                );
                self.after_save();
            }
            Err(e) => {
                let _ = self.db.end_session(&session_id, "failed");
                let _ = self.transition(RecEvent::Fail);
                let e = if !stderr.is_empty() && e.code == "file_missing" {
                    AppError::new(
                        "encoder_died",
                        format!(
                            "The encoder stopped unexpectedly: {}",
                            stderr.lines().last().unwrap_or("")
                        ),
                    )
                    .hint("Check the logs in Settings → Advanced.")
                } else {
                    e
                };
                self.set_error(e);
            }
        }
        self.release_session_if_idle();
        crate::system::notifications::restore(&self.db);
        crate::shell::on_recording_changed(self);
        self.emit_status();
    }

    /// Post-save housekeeping: library size limit and low-disk warning.
    pub fn after_save(&self) {
        let s = self.settings.read().clone();
        if s.storage.auto_delete && s.storage.max_library_gb > 0 {
            self.enforce_library_limit(s.storage.max_library_gb as u64 * 1024 * 1024 * 1024);
        }
        if let Some(d) = crate::storage::disk_space(Path::new(&s.storage.recordings_dir)) {
            if d.free_bytes < s.storage.low_space_warn_gb as u64 * 1024 * 1024 * 1024 {
                self.notify(
                    "warning",
                    "storage_low",
                    json!({ "freeGb": (d.free_bytes as f64 / 1e9 * 10.0).round() / 10.0 }),
                    None,
                );
            }
        }
    }

    /// Deletes the oldest non-favorite clips until the library fits into `limit_bytes`.
    pub fn enforce_library_limit(&self, limit_bytes: u64) {
        let Ok(clips) = self.db.all_clips() else {
            return;
        };
        let entries: Vec<crate::storage::Entry> = clips
            .iter()
            .map(|c| crate::storage::Entry {
                id: c.id.clone(),
                size: c.size_bytes.max(0) as u64,
                created_at: c.created_at,
                favorite: c.favorite,
            })
            .collect();
        let doomed = crate::storage::plan_cleanup(&entries, limit_bytes);
        let mut n = 0;
        for id in &doomed {
            if let Some(c) = clips.iter().find(|c| &c.id == id) {
                let _ = std::fs::remove_file(&c.path);
                if !c.thumb_path.is_empty() && c.kind != "screenshot" {
                    let _ = std::fs::remove_file(&c.thumb_path);
                }
                let _ = self.db.delete_clip(id);
                n += 1;
            }
        }
        if n > 0 {
            tracing::info!("library limit reached: removed {n} oldest clips");
            self.notify("info", "storage_cleaned", json!({ "count": n }), None);
            let _ = self.app.emit("library-changed", ());
        }
    }

    // ---- crash recovery ---------------------------------------------------------------------------------
    /// Recordings that were still open when the app last exited abnormally.
    pub fn interrupted(&self) -> Vec<crate::db::SessionRow> {
        self.db
            .interrupted_sessions()
            .unwrap_or_default()
            .into_iter()
            .filter(|s| Path::new(&s.tmp_path).exists() || Path::new(&s.final_path).exists())
            .collect()
    }

    /// Remuxes an interrupted recording into a playable file and adds it to the library.
    pub fn recover_session(&self, id: &str) -> Result<Clip> {
        let row = self
            .db
            .interrupted_sessions()?
            .into_iter()
            .find(|s| s.id == id)
            .ok_or_else(|| {
                AppError::new("not_found", "That recording session no longer exists.")
            })?;
        let tmp = Path::new(&row.tmp_path);
        let fin = Path::new(&row.final_path);
        let src = if tmp.exists() { tmp } else { fin };
        if !src.exists() {
            return Err(AppError::new(
                "file_missing",
                "The interrupted recording file is gone.",
            ));
        }
        let target = if fin.extension().map(|e| e == "mp4").unwrap_or(false) && src != fin {
            if remux(src, fin) {
                let _ = std::fs::remove_file(src);
                fin.to_path_buf()
            } else {
                src.to_path_buf()
            }
        } else {
            src.to_path_buf()
        };
        let clip = self.register_clip(&target, "recording", &row.game, "", row.started_at)?;
        let _ = self.db.end_session(id, "recovered");
        Ok(clip)
    }

    pub fn discard_session(&self, id: &str) -> Result<()> {
        if let Some(row) = self
            .db
            .interrupted_sessions()?
            .into_iter()
            .find(|s| s.id == id)
        {
            let _ = std::fs::remove_file(&row.tmp_path);
        }
        self.db.end_session(id, "discarded")
    }

    /// Indexes video / image files in the recordings folder that are not yet in the library.
    pub fn scan_library(&self) -> usize {
        let dir = std::path::PathBuf::from(self.settings.read().storage.recordings_dir.clone());
        let mut added = 0;
        let Ok(rd) = std::fs::read_dir(&dir) else {
            return 0;
        };
        for e in rd.flatten() {
            let p = e.path();
            let ext = p
                .extension()
                .map(|x| x.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if !matches!(ext.as_str(), "mp4" | "mkv") || p.to_string_lossy().contains(".recording.")
            {
                continue;
            }
            if self.db.clip_by_path(&p.to_string_lossy()).is_some() {
                continue;
            }
            let created = e
                .metadata()
                .and_then(|m| m.created())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or_else(now_ms);
            if self.register_clip(&p, "recording", "", "", created).is_ok() {
                added += 1;
            }
        }
        // drop entries whose files were deleted outside the app
        for c in self.db.all_clips().unwrap_or_default() {
            if !Path::new(&c.path).exists() {
                let _ = self.db.delete_clip(&c.id);
                added += 1;
            }
        }
        if added > 0 {
            let _ = self.app.emit("library-changed", ());
        }
        added
    }
}
