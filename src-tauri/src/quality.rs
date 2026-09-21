//! Recording quality model: presets, resolution/fps handling and the storage estimate shown in the UI.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Quality {
    /// low | medium | high | ultra | custom
    pub preset: String,
    /// Output height in pixels; 0 = keep the native size of the source.
    pub height: u32,
    pub fps: u32,
    pub bitrate_kbps: u32,
    /// h264 | hevc | av1
    pub codec: String,
    /// auto | nvenc | amf | qsv | cpu
    pub encoder: String,
    /// cbr | vbr | cqp
    pub rate_control: String,
    /// Constant quantiser used when `rate_control == "cqp"` (lower = better).
    pub cq: u32,
    /// fast | balanced | quality
    pub speed: String,
    /// mp4 | mkv
    pub container: String,
    pub audio_bitrate_kbps: u32,
}

impl Default for Quality {
    fn default() -> Self {
        Self::from_preset("high")
    }
}

impl Quality {
    pub fn from_preset(name: &str) -> Self {
        let (height, fps, bitrate_kbps, cq) = match name {
            "low" => (720, 30, 5_000, 30),
            "medium" => (1080, 60, 12_000, 26),
            "ultra" => (1440, 60, 45_000, 19),
            _ => (1080, 60, 20_000, 22),
        };
        Self {
            preset: if matches!(name, "low" | "medium" | "high" | "ultra") {
                name.into()
            } else {
                "high".into()
            },
            height,
            fps,
            bitrate_kbps,
            codec: "h264".into(),
            encoder: "auto".into(),
            rate_control: "cbr".into(),
            cq,
            speed: "balanced".into(),
            container: "mp4".into(),
            audio_bitrate_kbps: 160,
        }
    }

    /// Bitrate that a preset would recommend for an arbitrary height/fps/codec.
    /// Scales linearly with pixel count and (sub-linearly) with frame-rate; newer codecs need less.
    pub fn recommended_bitrate(height: u32, fps: u32, codec: &str) -> u32 {
        let h = if height == 0 { 1080 } else { height } as f64;
        let pixels = (h * h * 16.0 / 9.0) / (1920.0 * 1080.0);
        let fps_f = (fps as f64 / 60.0).powf(0.75);
        let codec_f = match codec {
            "hevc" => 0.72,
            "av1" => 0.6,
            _ => 1.0,
        };
        let kbps = 18_000.0 * pixels * fps_f * codec_f;
        (((kbps / 500.0).round() * 500.0) as u32).clamp(1_500, 150_000)
    }

    /// Output frame size for a source of `sw`x`sh`, honouring `height` and keeping the aspect ratio.
    /// Both dimensions are even (required by 4:2:0 encoders). Never upscales.
    pub fn output_size(&self, sw: u32, sh: u32) -> (u32, u32) {
        let (mut w, mut h) = (sw.max(2), sh.max(2));
        if self.height != 0 && self.height < sh {
            w = ((sw as f64) * (self.height as f64) / (sh as f64)).round() as u32;
            h = self.height;
        }
        (w & !1, h & !1)
    }
}

/// Estimated bytes for `seconds` of recording.
pub fn estimate_bytes(q: &Quality, audio_tracks: u32, seconds: u64) -> u64 {
    let video = if q.rate_control == "cqp" {
        // CQP has no fixed rate; approximate from the recommended bitrate, adjusted by qp distance.
        let base = Quality::recommended_bitrate(q.height, q.fps, &q.codec) as f64;
        (base * (0.92f64).powi(q.cq as i32 - 22)) as u64
    } else {
        q.bitrate_kbps as u64
    };
    let audio = q.audio_bitrate_kbps as u64 * audio_tracks as u64;
    (video + audio) * 1000 / 8 * seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_scale_up() {
        let l = Quality::from_preset("low");
        let u = Quality::from_preset("ultra");
        assert!(l.bitrate_kbps < u.bitrate_kbps);
        assert_eq!(l.preset, "low");
        assert_eq!(Quality::from_preset("bogus").preset, "high");
    }

    #[test]
    fn newer_codecs_need_less_bitrate() {
        let h264 = Quality::recommended_bitrate(1440, 60, "h264");
        let hevc = Quality::recommended_bitrate(1440, 60, "hevc");
        let av1 = Quality::recommended_bitrate(1440, 60, "av1");
        assert!(av1 < hevc && hevc < h264);
    }

    #[test]
    fn output_size_is_even_and_never_upscales() {
        let q = Quality {
            height: 1080,
            ..Quality::default()
        };
        assert_eq!(q.output_size(2560, 1440), (1920, 1080));
        assert_eq!(q.output_size(1280, 720), (1280, 720));
        assert_eq!(q.output_size(1281, 721), (1280, 720));
        let native = Quality {
            height: 0,
            ..Quality::default()
        };
        assert_eq!(native.output_size(2560, 1440), (2560, 1440));
    }

    #[test]
    fn estimate_matches_bitrate_math() {
        let q = Quality {
            bitrate_kbps: 8_000,
            audio_bitrate_kbps: 0,
            rate_control: "cbr".into(),
            ..Quality::default()
        };
        // 8 Mbit/s = 1 MB/s
        assert_eq!(estimate_bytes(&q, 0, 60), 60_000_000);
        assert_eq!(estimate_bytes(&q, 2, 60), 60_000_000);
        let q2 = Quality {
            audio_bitrate_kbps: 160,
            ..q
        };
        assert_eq!(estimate_bytes(&q2, 2, 60), (8_000 + 320) * 1000 / 8 * 60);
    }
}
