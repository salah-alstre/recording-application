//! FFmpeg integration: locating the bundled binaries, probing which encoders genuinely work on this
//! machine, inspecting media files and generating thumbnails.
pub mod args;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const TRIPLE: &str = "x86_64-pc-windows-msvc";

/// Locates a bundled tool (`ffmpeg` / `ffprobe`): next to the executable when installed,
/// or in `src-tauri/binaries` during development.
pub fn tool_path(name: &str) -> PathBuf {
    let mut candidates: Vec<PathBuf> = vec![];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(format!("{name}.exe")));
            candidates.push(dir.join(format!("{name}-{TRIPLE}.exe")));
            candidates.push(dir.join("binaries").join(format!("{name}-{TRIPLE}.exe")));
        }
    }
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(format!("{name}-{TRIPLE}.exe")),
    );
    candidates
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from(format!("{name}.exe")))
}

pub fn ffmpeg() -> PathBuf {
    tool_path("ffmpeg")
}
pub fn ffprobe() -> PathBuf {
    tool_path("ffprobe")
}

/// A `Command` that never flashes a console window.
pub fn command(path: &Path) -> Command {
    use std::os::windows::process::CommandExt;
    let mut c = Command::new(path);
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

pub fn ffmpeg_available() -> bool {
    command(&ffmpeg())
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// ---- encoder probing --------------------------------------------------------------------------

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncoderInfo {
    /// ffmpeg encoder name, e.g. `h264_nvenc`.
    pub id: String,
    /// nvenc | amf | qsv | cpu
    pub vendor: String,
    /// h264 | hevc | av1
    pub codec: String,
    pub label: String,
    pub available: bool,
    pub error: Option<String>,
}

pub const ALL_ENCODERS: &[(&str, &str, &str, &str)] = &[
    ("h264_nvenc", "nvenc", "h264", "NVIDIA NVENC H.264"),
    ("hevc_nvenc", "nvenc", "hevc", "NVIDIA NVENC H.265"),
    ("av1_nvenc", "nvenc", "av1", "NVIDIA NVENC AV1"),
    ("h264_amf", "amf", "h264", "AMD AMF H.264"),
    ("hevc_amf", "amf", "hevc", "AMD AMF H.265"),
    ("av1_amf", "amf", "av1", "AMD AMF AV1"),
    ("h264_qsv", "qsv", "h264", "Intel Quick Sync H.264"),
    ("hevc_qsv", "qsv", "hevc", "Intel Quick Sync H.265"),
    ("av1_qsv", "qsv", "av1", "Intel Quick Sync AV1"),
    ("libx264", "cpu", "h264", "CPU x264 H.264"),
    ("libx265", "cpu", "hevc", "CPU x265 H.265"),
    ("libsvtav1", "cpu", "av1", "CPU SVT-AV1"),
];

/// Actually initialises the encoder on a tiny synthetic clip. This is the only reliable way to know
/// that a GPU encoder works (driver present, session available, codec supported by this GPU).
pub fn probe_encoder(id: &str) -> Result<(), String> {
    let out = command(&ffmpeg())
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=1280x720:r=30",
            "-frames:v",
            "3",
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            id,
            "-f",
            "null",
            "-",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("cannot run ffmpeg: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        let line = err
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("encoder initialisation failed");
        Err(line.trim().to_string())
    }
}

/// Probes every encoder relevant to the detected GPU vendors (plus CPU encoders) in parallel.
pub fn probe_all(vendors: &[String]) -> Vec<EncoderInfo> {
    let handles: Vec<_> = ALL_ENCODERS
        .iter()
        .map(|&(id, vendor, codec, label)| {
            let relevant = vendor == "cpu" || vendors.iter().any(|v| v == vendor);
            std::thread::spawn(move || {
                let (available, error) = if relevant {
                    match probe_encoder(id) {
                        Ok(()) => (true, None),
                        Err(e) => (false, Some(e)),
                    }
                } else {
                    (false, Some("no matching GPU detected".into()))
                };
                EncoderInfo {
                    id: id.into(),
                    vendor: vendor.into(),
                    codec: codec.into(),
                    label: label.into(),
                    available,
                    error,
                }
            })
        })
        .collect();
    handles.into_iter().filter_map(|h| h.join().ok()).collect()
}

// ---- media inspection -------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub duration_ms: i64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: String,
    pub audio_tracks: u32,
    pub size_bytes: i64,
}

pub fn parse_probe_json(text: &str) -> Option<MediaInfo> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    let mut info = MediaInfo::default();
    if let Some(d) = v["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
    {
        info.duration_ms = (d * 1000.0).round() as i64;
    }
    info.size_bytes = v["format"]["size"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    for s in v["streams"].as_array()? {
        match s["codec_type"].as_str() {
            Some("video") if info.width == 0 => {
                info.width = s["width"].as_u64().unwrap_or(0) as u32;
                info.height = s["height"].as_u64().unwrap_or(0) as u32;
                info.codec = s["codec_name"].as_str().unwrap_or("").to_string();
                let rate = s["avg_frame_rate"]
                    .as_str()
                    .filter(|r| *r != "0/0")
                    .or(s["r_frame_rate"].as_str())
                    .unwrap_or("0/1");
                let mut it = rate.split('/');
                let n: f64 = it.next().and_then(|x| x.parse().ok()).unwrap_or(0.0);
                let d: f64 = it.next().and_then(|x| x.parse().ok()).unwrap_or(1.0);
                info.fps = if d > 0.0 {
                    (n / d * 100.0).round() / 100.0
                } else {
                    0.0
                };
                if info.duration_ms == 0 {
                    if let Some(sd) = s["duration"].as_str().and_then(|x| x.parse::<f64>().ok()) {
                        info.duration_ms = (sd * 1000.0).round() as i64;
                    }
                }
            }
            Some("audio") => info.audio_tracks += 1,
            _ => {}
        }
    }
    Some(info)
}

pub fn probe_media(path: &Path) -> Option<MediaInfo> {
    let out = command(&ffprobe())
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut info = parse_probe_json(&String::from_utf8_lossy(&out.stdout))?;
    if info.size_bytes == 0 {
        info.size_bytes = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);
    }
    Some(info)
}

/// Writes a JPEG thumbnail (max 480px wide) taken `at_secs` into the clip.
pub fn make_thumbnail(src: &Path, dst: &Path, at_secs: f64) -> bool {
    if let Some(p) = dst.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let run = |t: f64| {
        command(&ffmpeg())
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-ss",
                &format!("{t:.2}"),
                "-i",
            ])
            .arg(src)
            .args(["-frames:v", "1", "-vf", "scale=480:-2", "-q:v", "5"])
            .arg(dst)
            .stdin(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            && dst.exists()
    };
    run(at_secs) || run(0.0)
}

/// DirectShow video devices, parsed from `ffmpeg -list_devices`.
pub fn list_cameras() -> Vec<String> {
    let Ok(out) = command(&ffmpeg())
        .args([
            "-hide_banner",
            "-list_devices",
            "true",
            "-f",
            "dshow",
            "-i",
            "dummy",
        ])
        .stdin(Stdio::null())
        .output()
    else {
        return vec![];
    };
    parse_dshow_video_devices(&String::from_utf8_lossy(&out.stderr))
}

pub fn parse_dshow_video_devices(stderr: &str) -> Vec<String> {
    let mut names = vec![];
    for line in stderr.lines() {
        if line.contains("(video)") {
            if let (Some(a), Some(b)) = (line.find('"'), line.rfind('"')) {
                if b > a {
                    names.push(line[a + 1..b].to_string());
                }
            }
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ffprobe_json() {
        let json = r#"{"streams":[{"codec_type":"video","codec_name":"h264","width":2560,"height":1440,"avg_frame_rate":"120/1"},
            {"codec_type":"audio"},{"codec_type":"audio"}],"format":{"duration":"222.5","size":"123456"}}"#;
        let i = parse_probe_json(json).unwrap();
        assert_eq!((i.width, i.height, i.audio_tracks), (2560, 1440, 2));
        assert_eq!(i.duration_ms, 222_500);
        assert_eq!(i.fps, 120.0);
        assert_eq!(i.codec, "h264");
    }

    #[test]
    fn parses_dshow_listing() {
        let s = "[dshow @ 000] DirectShow video devices (some may be both video and audio devices)\n[dshow @ 000]  \"Logitech BRIO\" (video)\n[dshow @ 000]     Alternative name \"@device_pnp\"\n[dshow @ 000] DirectShow audio devices\n[dshow @ 000]  \"Mic\" (audio)\n";
        assert_eq!(parse_dshow_video_devices(s), vec!["Logitech BRIO"]);
    }

    #[test]
    fn bundled_ffmpeg_can_initialise_a_cpu_encoder() {
        if !ffmpeg_available() {
            eprintln!("ffmpeg not bundled; skipping");
            return;
        }
        probe_encoder("libx264").expect("libx264 should work");
    }
}
