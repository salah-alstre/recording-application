//! GPU / CPU / memory detection and the "recommended settings" logic used by onboarding.
use crate::encoder::EncoderInfo;
use crate::quality::Quality;
use serde::Serialize;
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
};

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    /// nvidia | amd | intel | other
    pub vendor: String,
    pub vram_bytes: u64,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub gpus: Vec<GpuInfo>,
    pub cpu: String,
    pub cpu_threads: usize,
    pub ram_bytes: u64,
    pub os: String,
    pub encoders: Vec<EncoderInfo>,
    pub ffmpeg_ok: bool,
}

pub fn vendor_name(id: u32) -> &'static str {
    match id {
        0x10DE => "nvidia",
        0x1002 | 0x1022 => "amd",
        0x8086 => "intel",
        _ => "other",
    }
}

/// Maps a GPU vendor to the hardware-encoder family that ffmpeg exposes for it.
pub fn encoder_family(vendor: &str) -> Option<&'static str> {
    match vendor {
        "nvidia" => Some("nvenc"),
        "amd" => Some("amf"),
        "intel" => Some("qsv"),
        _ => None,
    }
}

pub fn detect_gpus() -> Vec<GpuInfo> {
    let mut out = vec![];
    unsafe {
        let Ok(f) = CreateDXGIFactory1::<IDXGIFactory1>() else {
            return out;
        };
        let mut i = 0;
        while let Ok(a) = f.EnumAdapters1(i) {
            i += 1;
            let Ok(d) = a.GetDesc1() else { continue };
            if d.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 || d.VendorId == 0x1414 {
                continue; // Microsoft Basic Render Driver
            }
            let end = d
                .Description
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(d.Description.len());
            out.push(GpuInfo {
                name: String::from_utf16_lossy(&d.Description[..end]),
                vendor: vendor_name(d.VendorId).into(),
                vram_bytes: d.DedicatedVideoMemory as u64,
            });
        }
    }
    out
}

pub fn gather(encoders: Vec<EncoderInfo>) -> HardwareInfo {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();
    let cpu = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_default();
    HardwareInfo {
        gpus: detect_gpus(),
        cpu,
        cpu_threads: sys.cpus().len(),
        ram_bytes: sys.total_memory(),
        os: sysinfo::System::long_os_version().unwrap_or_else(|| "Windows".into()),
        encoders,
        ffmpeg_ok: crate::encoder::ffmpeg_available(),
    }
}

/// Recommended quality for a given display and set of working encoders.
/// Prefers hardware AV1, then hardware HEVC, then hardware H.264; falls back to CPU H.264 at a
/// modest resolution so weak machines still record smoothly.
pub fn recommend(display_h: u32, refresh_hz: u32, encoders: &[EncoderInfo]) -> Quality {
    let hw = |codec: &str| {
        encoders
            .iter()
            .any(|e| e.available && e.vendor != "cpu" && e.codec == codec)
    };
    let (codec, has_hw) = if hw("av1") {
        ("av1", true)
    } else if hw("hevc") {
        ("hevc", true)
    } else if hw("h264") {
        ("h264", true)
    } else {
        ("h264", false)
    };
    let height = if has_hw {
        display_h.clamp(720, 1440)
    } else {
        display_h.clamp(720, 1080).min(1080)
    };
    let height = [720u32, 1080, 1440, 2160]
        .iter()
        .copied()
        .rev()
        .find(|&h| h <= height)
        .unwrap_or(720);
    let fps = if has_hw && refresh_hz >= 100 && height <= 1080 {
        120
    } else {
        60
    };
    let mut q = Quality::from_preset("high");
    q.preset = "custom".into();
    q.height = height;
    q.fps = fps;
    q.codec = codec.into();
    q.bitrate_kbps = Quality::recommended_bitrate(height, fps, codec);
    q.encoder = if has_hw { "auto".into() } else { "cpu".into() };
    q
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc(id: &str, vendor: &str, codec: &str, ok: bool) -> EncoderInfo {
        EncoderInfo {
            id: id.into(),
            vendor: vendor.into(),
            codec: codec.into(),
            label: id.into(),
            available: ok,
            error: None,
        }
    }

    #[test]
    fn recommends_hardware_av1_at_1440p() {
        let e = [
            enc("av1_nvenc", "nvenc", "av1", true),
            enc("libx264", "cpu", "h264", true),
        ];
        let q = recommend(1440, 165, &e);
        assert_eq!((q.codec.as_str(), q.height, q.fps), ("av1", 1440, 60));
        assert_eq!(q.encoder, "auto");
    }

    #[test]
    fn cpu_only_machines_get_conservative_settings() {
        let e = [enc("libx264", "cpu", "h264", true)];
        let q = recommend(2160, 144, &e);
        assert_eq!((q.codec.as_str(), q.height, q.fps), ("h264", 1080, 60));
        assert_eq!(q.encoder, "cpu");
    }

    #[test]
    fn high_refresh_1080p_gets_120fps_with_hardware() {
        let e = [enc("h264_nvenc", "nvenc", "h264", true)];
        assert_eq!(recommend(1080, 144, &e).fps, 120);
    }

    #[test]
    fn vendor_ids_map() {
        assert_eq!(vendor_name(0x10DE), "nvidia");
        assert_eq!(encoder_family("amd"), Some("amf"));
        assert_eq!(encoder_family("other"), None);
    }

    #[test]
    fn detects_at_least_one_gpu_or_none_without_crashing() {
        let _ = detect_gpus();
    }
}
