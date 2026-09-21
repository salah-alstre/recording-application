//! Thumbnails: generation, validation and background repair.
//!
//! Every clip gets a 480 px wide JPEG in `%LOCALAPPDATA%\Rimlight\thumbnails\<clip id>.jpg`. That folder
//! is added to the asset-protocol scope at start-up, which is what allows the WebView to load them.
use crate::engine::Engine;
use crate::paths;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Emitter;

static REPAIRING: AtomicBool = AtomicBool::new(false);

pub fn thumb_path_for(clip_id: &str) -> PathBuf {
    paths::thumbs_dir().join(format!("{clip_id}.jpg"))
}

/// A frame around 25 % into short clips (never frame 0, which is often black); 2 s into longer ones.
pub fn thumb_time_secs(duration_ms: i64) -> f64 {
    let secs = duration_ms.max(0) as f64 / 1000.0;
    if secs < 10.0 {
        (secs * 0.25).max(0.0)
    } else {
        2.0
    }
}

/// True when `path` is a non-empty, decodable image inside the managed thumbnail folder.
pub fn thumb_valid(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let p = Path::new(path);
    if !p.starts_with(paths::thumbs_dir()) {
        return false; // legacy location (or a full-size screenshot) – regenerate into the managed folder
    }
    std::fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false) && image::image_dimensions(p).is_ok()
}

fn image_thumb(src: &Path, dst: &Path) -> bool {
    let Ok(img) = image::open(src) else {
        return false;
    };
    let small = img.thumbnail(480, 270).to_rgb8();
    if let Some(d) = dst.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    small
        .save_with_format(dst, image::ImageFormat::Jpeg)
        .is_ok()
}

/// Creates the thumbnail for a clip file and returns its path when it was written and verified.
pub fn generate(path: &Path, kind: &str, clip_id: &str, duration_ms: i64) -> Option<String> {
    let dst = thumb_path_for(clip_id);
    let ok = if kind == "screenshot" {
        image_thumb(path, &dst)
    } else {
        let t = thumb_time_secs(duration_ms);
        crate::encoder::make_thumbnail(path, &dst, t)
    };
    let s = dst.to_string_lossy().into_owned();
    if ok && thumb_valid(&s) {
        tracing::debug!(
            "thumbnail {} ({} bytes)",
            s,
            std::fs::metadata(&dst).map(|m| m.len()).unwrap_or(0)
        );
        Some(s)
    } else {
        let _ = std::fs::remove_file(&dst);
        tracing::warn!("thumbnail generation failed for {}", path.display());
        None
    }
}

pub fn is_repairing() -> bool {
    REPAIRING.load(Ordering::SeqCst)
}

impl Engine {
    /// Regenerates missing / broken / legacy-location thumbnails on a background thread.
    /// Clips whose thumbnail is invalid are first reset to "pending" (empty path) so the UI shows a
    /// loading placeholder instead of a broken image, then fixed one by one.
    pub fn repair_thumbnails(self: &Arc<Self>) {
        if REPAIRING.swap(true, Ordering::SeqCst) {
            return;
        }
        let me = self.clone();
        let spawned = std::thread::Builder::new()
            .name("rimlight-thumbs".into())
            .spawn(move || {
                let todo: Vec<_> = me
                    .db
                    .all_clips()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|c| Path::new(&c.path).exists() && !thumb_valid(&c.thumb_path))
                    .collect();
                if !todo.is_empty() {
                    tracing::info!("repairing {} thumbnails", todo.len());
                    let _ = me.app.emit("thumbs-busy", true);
                    for c in &todo {
                        let _ = me.db.set_thumb(&c.id, "");
                    }
                    let _ = me.app.emit("library-changed", ());
                    for (i, c) in todo.iter().enumerate() {
                        if let Some(p) = generate(Path::new(&c.path), &c.kind, &c.id, c.duration_ms)
                        {
                            let _ = me.db.set_thumb(&c.id, &p);
                        }
                        if i % 3 == 2 || i + 1 == todo.len() {
                            let _ = me.app.emit("library-changed", ());
                        }
                    }
                    let _ = me.app.emit("thumbs-busy", false);
                }
                // the old, un-scoped folder is no longer referenced by any clip
                let legacy = paths::data_dir().join("thumbs");
                if legacy.exists() {
                    let _ = std::fs::remove_dir_all(legacy);
                }
                REPAIRING.store(false, Ordering::SeqCst);
            });
        if spawned.is_err() {
            REPAIRING.store(false, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_choice_avoids_the_first_frame() {
        assert!((thumb_time_secs(4_000) - 1.0).abs() < 1e-9); // 25 % of a 4 s clip
        assert!(thumb_time_secs(8_000) > 1.5);
        assert_eq!(thumb_time_secs(60_000), 2.0);
        assert_eq!(thumb_time_secs(0), 0.0);
    }

    #[test]
    fn validity_requires_managed_folder_size_and_decodable_image() {
        assert!(!thumb_valid(""));
        assert!(!thumb_valid(r"C:\does\not\exist.jpg"));
        let dir = paths::thumbs_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("unit-test-bad.jpg");
        std::fs::write(&bad, b"not an image").unwrap();
        assert!(!thumb_valid(&bad.to_string_lossy()));
        let empty = dir.join("unit-test-empty.jpg");
        std::fs::write(&empty, b"").unwrap();
        assert!(!thumb_valid(&empty.to_string_lossy()));
        let good = dir.join("unit-test-good.jpg");
        image::RgbImage::from_pixel(8, 8, image::Rgb([200, 40, 40]))
            .save_with_format(&good, image::ImageFormat::Jpeg)
            .unwrap();
        assert!(thumb_valid(&good.to_string_lossy()));
        // a valid image outside the managed folder still counts as needing migration
        let outside = std::env::temp_dir().join("unit-test-outside.jpg");
        std::fs::copy(&good, &outside).unwrap();
        assert!(!thumb_valid(&outside.to_string_lossy()));
        for f in [bad, empty, good, outside] {
            let _ = std::fs::remove_file(f);
        }
    }

    #[test]
    fn screenshots_get_a_small_jpeg_thumbnail() {
        let dir = std::env::temp_dir().join(format!("rimlight-thumb-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let shot = dir.join("shot.png");
        image::RgbImage::from_pixel(1920, 1080, image::Rgb([10, 120, 200]))
            .save(&shot)
            .unwrap();
        let id = format!("unit-{}", uuid::Uuid::new_v4().simple());
        let p = generate(&shot, "screenshot", &id, 0).expect("thumbnail");
        let (w, h) = image::image_dimensions(&p).unwrap();
        assert_eq!((w, h), (480, 270));
        let _ = std::fs::remove_file(p);
        let _ = std::fs::remove_dir_all(dir);
    }
}
