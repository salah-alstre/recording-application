//! Quick clip editor: non-destructive export (the original is never modified) with trim, crop,
//! rotate, mute / volume and an optional text overlay.
use crate::encoder;
use crate::engine::Engine;
use crate::error::{AppError, Result};
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tauri::Emitter;

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TextOverlay {
    pub text: String,
    /// 0..1 relative position of the text's top-left corner
    pub x: f32,
    pub y: f32,
    pub size: u32,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ExportOpts {
    pub id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub crop: Option<CropRect>,
    /// 0 | 90 | 180 | 270 (clockwise)
    pub rotate: u32,
    pub mute: bool,
    /// 100 = unchanged
    pub volume_pct: u32,
    pub text: Option<TextOverlay>,
}

pub fn escape_drawtext(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(':', "\\:")
        .replace('\'', "\u{2019}")
        .replace('%', "\\%")
}

/// Video filter chain for the requested edits (`None` when there are none).
pub fn video_filters(o: &ExportOpts) -> Option<String> {
    let mut f: Vec<String> = vec![];
    if let Some(c) = &o.crop {
        if c.w >= 16 && c.h >= 16 {
            f.push(format!("crop={}:{}:{}:{}", c.w & !1, c.h & !1, c.x, c.y));
        }
    }
    match o.rotate % 360 {
        90 => f.push("transpose=1".into()),
        180 => f.push("transpose=1,transpose=1".into()),
        270 => f.push("transpose=2".into()),
        _ => {}
    }
    if let Some(t) = &o.text {
        if !t.text.trim().is_empty() {
            f.push(format!(
                "drawtext=fontfile='C\\:/Windows/Fonts/segoeui.ttf':text='{}':x=w*{:.3}:y=h*{:.3}:fontsize={}:fontcolor=white:box=1:boxcolor=black@0.55:boxborderw=10",
                escape_drawtext(&t.text),
                t.x.clamp(0.0, 1.0),
                t.y.clamp(0.0, 1.0),
                t.size.clamp(12, 200)
            ));
        }
    }
    (!f.is_empty()).then(|| f.join(","))
}

pub fn build_args(
    input: &str,
    output: &str,
    o: &ExportOpts,
    encoder_id: &str,
    has_audio: bool,
) -> Vec<String> {
    let mut a: Vec<String> = [
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-progress",
        "pipe:1",
        "-nostats",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    a.extend([
        "-ss".into(),
        format!("{:.3}", o.start_ms.max(0) as f64 / 1000.0),
    ]);
    a.extend([
        "-to".into(),
        format!("{:.3}", o.end_ms.max(0) as f64 / 1000.0),
    ]);
    a.extend(["-i".into(), input.into(), "-map".into(), "0:v:0".into()]);
    if has_audio && !o.mute {
        a.extend(["-map".into(), "0:a?".into()]);
    }
    if let Some(vf) = video_filters(o) {
        a.extend(["-vf".into(), vf]);
    }
    a.extend(["-c:v".into(), encoder_id.into()]);
    if encoder_id.ends_with("_nvenc") {
        a.extend([
            "-preset".into(),
            "p5".into(),
            "-cq".into(),
            "21".into(),
            "-rc".into(),
            "vbr".into(),
            "-b:v".into(),
            "0".into(),
        ]);
    } else if encoder_id == "libx264" {
        a.extend([
            "-preset".into(),
            "veryfast".into(),
            "-crf".into(),
            "20".into(),
        ]);
    } else {
        a.extend(["-b:v".into(), "12M".into()]);
    }
    a.extend(["-pix_fmt".into(), "yuv420p".into()]);
    if has_audio && !o.mute {
        a.extend(["-c:a".into(), "aac".into(), "-b:a".into(), "192k".into()]);
        if o.volume_pct != 100 && o.volume_pct > 0 {
            a.extend([
                "-af".into(),
                format!("volume={:.2}", o.volume_pct as f32 / 100.0),
            ]);
        }
    }
    a.extend(["-movflags".into(), "+faststart".into(), output.into()]);
    a
}

impl Engine {
    pub fn export_clip(self: &Arc<Self>, o: ExportOpts) -> Result<String> {
        let clip = self
            .db
            .get_clip(&o.id)
            .ok_or_else(|| AppError::new("not_found", "That clip is no longer in the library."))?;
        if o.end_ms <= o.start_ms + 100 {
            return Err(AppError::new("edit_range", "The selected range is empty.")
                .hint("Move the trim handles apart."));
        }
        let src = PathBuf::from(&clip.path);
        if !src.exists() {
            return Err(AppError::new("file_missing", "The source file is missing."));
        }
        let encoders = self.encoders.read().clone();
        let enc = if encoders.iter().any(|e| e.id == "h264_nvenc" && e.available) {
            "h264_nvenc"
        } else if encoders.iter().any(|e| e.id == "h264_amf" && e.available) {
            "h264_amf"
        } else {
            "libx264"
        };
        let stem = src
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let out = crate::naming::unique_path(
            src.parent().unwrap_or(std::path::Path::new(".")),
            &format!("{stem}_edit"),
            "mp4",
        );
        let args = build_args(
            &src.to_string_lossy(),
            &out.to_string_lossy(),
            &o,
            enc,
            clip.audio_tracks > 0,
        );
        tracing::info!("exporting {} → {}", src.display(), out.display());
        let mut child = encoder::command(&encoder::ffmpeg())
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                AppError::new("ffmpeg_missing", format!("Cannot start the encoder: {e}"))
            })?;
        let total_us = ((o.end_ms - o.start_ms) * 1000).max(1) as f64;
        let stderr = child.stderr.take();
        let err_thread = std::thread::spawn(move || {
            let mut s = String::new();
            if let Some(mut e) = stderr {
                let _ = std::io::Read::read_to_string(&mut e, &mut s);
            }
            s
        });
        if let Some(out_pipe) = child.stdout.take() {
            for line in BufReader::new(out_pipe)
                .lines()
                .map_while(std::result::Result::ok)
            {
                if let Some(v) = line.strip_prefix("out_time_us=") {
                    if let Ok(us) = v.trim().parse::<f64>() {
                        let _ = self.app.emit("export-progress", serde_json::json!({ "id": o.id, "pct": (us / total_us * 100.0).clamp(0.0, 100.0) }));
                    }
                }
            }
        }
        let ok = child.wait().map(|s| s.success()).unwrap_or(false);
        let err = err_thread.join().unwrap_or_default();
        if !ok || !out.exists() {
            let _ = std::fs::remove_file(&out);
            return Err(AppError::new(
                "export_failed",
                format!(
                    "Export failed: {}",
                    err.lines().last().unwrap_or("unknown encoder error")
                ),
            ));
        }
        let new = self.register_clip(
            &out,
            &clip.kind,
            &clip.game,
            &clip.encoder,
            crate::engine::now_ms(),
        )?;
        Ok(new.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_edits_means_no_filter() {
        assert_eq!(video_filters(&ExportOpts::default()), None);
    }

    #[test]
    fn crop_rotate_and_text_compose_in_order() {
        let o = ExportOpts {
            crop: Some(CropRect {
                x: 10,
                y: 20,
                w: 641,
                h: 361,
            }),
            rotate: 90,
            text: Some(TextOverlay {
                text: "GG: 100%".into(),
                x: 0.1,
                y: 0.9,
                size: 32,
            }),
            ..ExportOpts::default()
        };
        let f = video_filters(&o).unwrap();
        assert!(f.starts_with("crop=640:360:10:20,transpose=1,drawtext="));
        assert!(f.contains("GG\\: 100\\%"));
    }

    #[test]
    fn trimming_and_mute_shape_the_command() {
        let o = ExportOpts {
            start_ms: 1500,
            end_ms: 9000,
            mute: true,
            ..ExportOpts::default()
        };
        let a = build_args("in.mp4", "out.mp4", &o, "libx264", true).join(" ");
        assert!(a.contains("-ss 1.500") && a.contains("-to 9.000"));
        assert!(!a.contains("-map 0:a"));
        assert!(a.ends_with("out.mp4"));
        let v = ExportOpts {
            start_ms: 0,
            end_ms: 1000,
            volume_pct: 50,
            ..ExportOpts::default()
        };
        assert!(build_args("in.mp4", "out.mp4", &v, "libx264", true)
            .join(" ")
            .contains("volume=0.50"));
    }

    #[test]
    fn drawtext_escaping() {
        assert_eq!(escape_drawtext("a:b'c"), "a\\:b\u{2019}c");
    }
}
