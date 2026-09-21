//! Screenshots: full display, the current capture target, a window or an interactively chosen region.
use crate::capture::{self, frame::FrameBuf, TargetSpec};
use crate::db::Clip;
use crate::engine::{now_ms, Engine};
use crate::error::{AppError, Result};
use crate::naming::{self, NameContext};
use crate::settings::Rect;
use crate::system::winenum;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;

fn to_rgba(f: &FrameBuf) -> image::RgbaImage {
    let mut px = f.data.clone();
    for p in px.chunks_exact_mut(4) {
        p.swap(0, 2); // BGRA → RGBA
        p[3] = 255;
    }
    image::RgbaImage::from_raw(f.w, f.h, px).expect("frame buffer size matches dimensions")
}

pub fn encode_to_file(
    img: &image::RgbaImage,
    path: &std::path::Path,
    format: &str,
    quality: u32,
) -> Result<()> {
    let mut file = std::fs::File::create(path)?;
    match format {
        "jpeg" | "jpg" => {
            let rgb = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
            let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut file,
                quality.clamp(1, 100) as u8,
            );
            rgb.write_with_encoder(enc).map_err(AppError::internal)?;
        }
        "webp" => {
            // The bundled encoder is lossless; `quality` is accepted for API symmetry.
            let enc = image::codecs::webp::WebPEncoder::new_lossless(&mut file);
            img.write_with_encoder(enc).map_err(AppError::internal)?;
        }
        _ => {
            let enc = image::codecs::png::PngEncoder::new(&mut file);
            img.write_with_encoder(enc).map_err(AppError::internal)?;
        }
    }
    Ok(())
}

fn ext_for(format: &str) -> &'static str {
    match format {
        "jpeg" | "jpg" => "jpg",
        "webp" => "webp",
        _ => "png",
    }
}

/// `mode`: None = the configured screenshot mode; "current" | "display" | "window" | "region".
pub fn take(engine: &Arc<Engine>, mode: Option<&str>) -> Result<String> {
    let s = engine.settings.read().clone();
    let mode = mode.unwrap_or("current");

    if mode == "region" {
        engine.inner.lock().pending_region = Some("screenshot".into());
        crate::shell::show_region_picker(&engine.app);
        return Ok(String::new());
    }

    if s.screenshots.delay_secs > 0 {
        engine.notify(
            "info",
            "screenshot_countdown",
            json!({ "seconds": s.screenshots.delay_secs }),
            None,
        );
        std::thread::sleep(Duration::from_secs(s.screenshots.delay_secs as u64));
    }

    let (spec, game) = match mode {
        "window" => match winenum::foreground() {
            Some(w) => (TargetSpec::Window { hwnd: w.hwnd }, winenum::game_name(&w)),
            None => {
                return Err(AppError::new(
                    "window_missing",
                    "There is no active window to capture.",
                ))
            }
        },
        "display" => (
            TargetSpec::Display {
                index: crate::shell::active_monitor().index,
            },
            String::new(),
        ),
        _ => {
            let r = engine.resolve_target()?;
            (r.spec, r.game)
        }
    };
    save_from_spec(engine, spec, &game)
}

/// Screenshot of a screen region chosen in the picker (`rect` relative to the display).
pub fn take_region(engine: &Arc<Engine>, display_index: u32, rect: Rect) -> Result<String> {
    save_from_spec(
        engine,
        TargetSpec::Region {
            index: display_index,
            rect,
        },
        "",
    )
}

fn save_from_spec(engine: &Arc<Engine>, spec: TargetSpec, game: &str) -> Result<String> {
    let s = engine.settings.read().clone();
    // Reuse the live frame when that exact source is already being captured.
    let live = {
        let g = engine.inner.lock();
        g.session
            .as_ref()
            .filter(|x| *x.spec.lock() == spec)
            .and_then(|x| x.latest_frame())
    };
    let frame = match live {
        Some(f) => f,
        None => capture::grab_once(&spec, s.screenshots.include_cursor)?,
    };
    let img = to_rgba(&frame);

    let dir = if s.screenshots.dir.trim().is_empty() {
        PathBuf::from(&s.storage.recordings_dir).join("Screenshots")
    } else {
        PathBuf::from(&s.screenshots.dir)
    };
    std::fs::create_dir_all(&dir).map_err(|e| {
        AppError::new(
            "folder_unavailable",
            format!(
                "The screenshot folder \"{}\" is unavailable: {e}",
                dir.display()
            ),
        )
        .hint("Choose another folder in Settings → Screenshots.")
    })?;
    let stem = format!(
        "{}_Screenshot",
        naming::render(
            &s.file_name_template,
            &NameContext {
                game,
                profile: "",
                resolution: "",
                fps: 0,
                now: chrono::Local::now()
            }
        )
    );
    let path = naming::unique_path(&dir, &stem, ext_for(&s.screenshots.format));
    encode_to_file(&img, &path, &s.screenshots.format, s.screenshots.quality)?;

    if s.screenshots.copy_to_clipboard {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_image(arboard::ImageData {
                width: img.width() as usize,
                height: img.height() as usize,
                bytes: std::borrow::Cow::Borrowed(img.as_raw()),
            });
        }
    }

    let size = std::fs::metadata(&path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);
    let p = path.to_string_lossy().into_owned();
    let id = uuid::Uuid::new_v4().to_string();
    let thumb_path = crate::thumbs::generate(&path, "screenshot", &id, 0).unwrap_or_default();
    let clip = Clip {
        id,
        path: p.clone(),
        kind: "screenshot".into(),
        title: path
            .file_stem()
            .map(|x| x.to_string_lossy().into_owned())
            .unwrap_or_default(),
        game: game.into(),
        created_at: now_ms(),
        width: img.width(),
        height: img.height(),
        size_bytes: size,
        thumb_path,
        ..Clip::default()
    };
    engine.db.upsert_clip(&clip)?;
    let _ = engine.app.emit("library-changed", ());
    let _ = engine.app.emit("screenshot-flash", ());
    engine.notify(
        "success",
        "screenshot_saved",
        json!({ "name": clip.title }),
        Some(p.clone()),
    );
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> image::RgbaImage {
        image::RgbaImage::from_fn(8, 4, |x, y| {
            image::Rgba([x as u8 * 30, y as u8 * 60, 128, 255])
        })
    }

    #[test]
    fn writes_all_three_formats_that_decode_back() {
        let dir = std::env::temp_dir().join(format!("rimlight-shot-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        for fmt in ["png", "jpeg", "webp"] {
            let p = dir.join(format!("a.{}", ext_for(fmt)));
            encode_to_file(&sample(), &p, fmt, 90).unwrap();
            let back = image::open(&p).unwrap_or_else(|e| panic!("{fmt}: {e}"));
            assert_eq!((back.width(), back.height()), (8, 4), "{fmt}");
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn bgra_frames_are_converted_to_rgba() {
        let f = FrameBuf {
            data: vec![10, 20, 30, 0, 1, 2, 3, 0],
            w: 2,
            h: 1,
        };
        let img = to_rgba(&f);
        assert_eq!(img.get_pixel(0, 0).0, [30, 20, 10, 255]);
    }
}
