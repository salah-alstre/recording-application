//! Application settings. Persisted as a single JSON document in the SQLite `kv` table.
//! Every struct uses `#[serde(default)]` so that settings written by older versions keep loading.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct General {
    pub language: String,
    pub launch_with_windows: bool,
    /// normal | minimized | tray
    pub start_mode: String,
    pub close_to_tray: bool,
    pub auto_start_replay: bool,
    pub check_updates: bool,
    pub update_feed_url: String,
    pub hardware_acceleration: bool,
    pub onboarded: bool,
}
impl Default for General {
    fn default() -> Self {
        Self {
            language: "en".into(),
            launch_with_windows: false,
            start_mode: "normal".into(),
            close_to_tray: true,
            auto_start_replay: false,
            check_updates: false,
            update_feed_url: String::new(),
            hardware_acceleration: true,
            onboarded: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CaptureSetup {
    /// game | window | display | region | active
    pub mode: String,
    /// 1-based monitor index (display + region modes)
    pub display_index: u32,
    pub window_title: String,
    pub window_exe: String,
    pub region: Rect,
    pub capture_cursor: bool,
    pub highlight_cursor: bool,
    pub show_clicks: bool,
    /// Stop recording automatically when the captured application exits.
    pub stop_when_app_closes: bool,
}
impl Default for CaptureSetup {
    fn default() -> Self {
        Self {
            mode: "display".into(),
            display_index: 1,
            window_title: String::new(),
            window_exe: String::new(),
            region: Rect {
                x: 0,
                y: 0,
                w: 1280,
                h: 720,
            },
            capture_cursor: true,
            highlight_cursor: false,
            show_clicks: false,
            stop_when_app_closes: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ReplaySettings {
    pub duration_secs: u32,
}
impl Default for ReplaySettings {
    fn default() -> Self {
        Self { duration_secs: 60 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct AudioSettings {
    pub system_enabled: bool,
    pub system_volume: u32,
    pub system_device: String,
    pub mic_enabled: bool,
    pub mic_volume: u32,
    pub mic_device: String,
    pub mic_gain_db: f32,
    pub gate_enabled: bool,
    pub gate_threshold_db: f32,
    pub compressor: bool,
    pub limiter: bool,
    /// Falls back to the default device when the selected one disappears.
    pub fallback_to_default: bool,
    /// off | push_to_talk | push_to_mute
    pub ptt_mode: String,
    pub ptt_key: String,
    /// Extra tracks besides the full mix (track 1).
    pub track_system: bool,
    pub track_mic: bool,
}
impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            system_enabled: true,
            system_volume: 100,
            system_device: String::new(),
            mic_enabled: false,
            mic_volume: 100,
            mic_device: String::new(),
            mic_gain_db: 0.0,
            gate_enabled: false,
            gate_threshold_db: -50.0,
            compressor: false,
            limiter: true,
            fallback_to_default: true,
            ptt_mode: "off".into(),
            ptt_key: "V".into(),
            track_system: true,
            track_mic: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct CameraSettings {
    pub enabled: bool,
    pub device: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// circle | rounded | square
    pub shape: String,
    /// Size as a percentage of the frame height.
    pub size_pct: u32,
    /// top-left | top-right | bottom-left | bottom-right
    pub corner: String,
    pub margin_px: u32,
    pub mirror: bool,
}
impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            device: String::new(),
            width: 1280,
            height: 720,
            fps: 30,
            shape: "circle".into(),
            size_pct: 24,
            corner: "bottom-right".into(),
            margin_px: 32,
            mirror: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct ScreenshotSettings {
    /// png | jpeg | webp
    pub format: String,
    pub quality: u32,
    pub dir: String,
    pub include_cursor: bool,
    pub delay_secs: u32,
    pub copy_to_clipboard: bool,
}
impl Default for ScreenshotSettings {
    fn default() -> Self {
        Self {
            format: "png".into(),
            quality: 90,
            dir: String::new(),
            include_cursor: false,
            delay_secs: 0,
            copy_to_clipboard: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlaySettings {
    /// compact | full
    pub mode: String,
    /// small | medium | large
    pub scale: String,
    pub modules: Vec<String>,
}
impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            mode: "full".into(),
            scale: "medium".into(),
            modules: [
                "record",
                "replay",
                "saveReplay",
                "screenshot",
                "mic",
                "system",
                "camera",
                "mixer",
                "source",
                "mode",
                "quality",
                "stats",
                "clips",
                "settings",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct StorageSettings {
    pub recordings_dir: String,
    /// 0 disables the limit.
    pub max_library_gb: u32,
    pub auto_delete: bool,
    pub low_space_warn_gb: u32,
}
impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            recordings_dir: crate::paths::default_recordings_dir()
                .to_string_lossy()
                .into_owned(),
            max_library_gb: 0,
            auto_delete: false,
            low_space_warn_gb: 5,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PerformanceSettings {
    pub stats_enabled: bool,
    pub stats_metrics: Vec<String>,
    /// top-left | top-right | bottom-left | bottom-right
    pub stats_position: String,
    pub hud_enabled: bool,
    pub hud_position: String,
    pub hud_include_in_capture: bool,
    pub notifications: bool,
}
impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            stats_enabled: false,
            stats_metrics: ["fps", "cpu", "gpu", "ram", "bitrate"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            stats_position: "top-right".into(),
            hud_enabled: true,
            hud_position: "top-left".into(),
            hud_include_in_capture: false,
            notifications: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct PrivacySettings {
    /// Executable names (e.g. `keepass.exe`) that must never be recorded.
    pub protected_apps: Vec<String>,
    /// pause | blackout
    pub action: String,
    /// Turn Windows pop-up notifications off while recording.
    pub hide_notifications: bool,
}
impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            protected_apps: vec![],
            action: "blackout".into(),
            hide_notifications: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct AppearanceSettings {
    /// midnight | graphite | oled | glass
    pub theme: String,
    pub accent: String,
    pub ui_scale: u32,
}
impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "midnight".into(),
            accent: "#7aa2ff".into(),
            ui_scale: 100,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub general: General,
    pub capture: CaptureSetup,
    pub active_profile_id: String,
    pub replay: ReplaySettings,
    pub audio: AudioSettings,
    pub camera: CameraSettings,
    pub screenshots: ScreenshotSettings,
    pub overlay: OverlaySettings,
    pub hotkeys: BTreeMap<String, String>,
    pub storage: StorageSettings,
    pub performance: PerformanceSettings,
    pub privacy: PrivacySettings,
    pub appearance: AppearanceSettings,
    pub file_name_template: String,
    pub log_level: String,
}

pub fn default_hotkeys() -> BTreeMap<String, String> {
    [
        ("overlay", "Alt+Z"),
        ("record", "Alt+F9"),
        ("saveReplay", "Alt+F10"),
        ("toggleReplay", "Alt+Shift+F10"),
        ("screenshot", "Alt+F1"),
        ("toggleMic", "Ctrl+Alt+M"),
        ("toggleCamera", "Ctrl+Alt+C"),
        ("toggleStats", "Alt+R"),
        ("marker", "Alt+F8"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: General::default(),
            capture: CaptureSetup::default(),
            active_profile_id: "profile-default".into(),
            replay: ReplaySettings::default(),
            audio: AudioSettings::default(),
            camera: CameraSettings::default(),
            screenshots: ScreenshotSettings::default(),
            overlay: OverlaySettings::default(),
            hotkeys: default_hotkeys(),
            storage: StorageSettings::default(),
            performance: PerformanceSettings::default(),
            privacy: PrivacySettings::default(),
            appearance: AppearanceSettings::default(),
            file_name_template: "{game}_{date}_{time}".into(),
            log_level: "info".into(),
        }
    }
}

impl Settings {
    /// Repairs values that would break the engine (out-of-range numbers, unknown enums, missing hotkeys).
    pub fn sanitize(&mut self) {
        for (k, v) in default_hotkeys() {
            self.hotkeys.entry(k).or_insert(v);
        }
        self.replay.duration_secs = self.replay.duration_secs.clamp(10, 3600);
        self.audio.system_volume = self.audio.system_volume.min(200);
        self.audio.mic_volume = self.audio.mic_volume.min(200);
        self.audio.mic_gain_db = self.audio.mic_gain_db.clamp(-20.0, 30.0);
        self.screenshots.quality = self.screenshots.quality.clamp(1, 100);
        self.screenshots.delay_secs = self.screenshots.delay_secs.min(30);
        self.appearance.ui_scale = self.appearance.ui_scale.clamp(80, 140);
        self.capture.display_index = self.capture.display_index.max(1);
        self.camera.size_pct = self.camera.size_pct.clamp(8, 60);
        if !matches!(self.general.language.as_str(), "en" | "ar") {
            self.general.language = "en".into();
        }
        if self.storage.recordings_dir.trim().is_empty() {
            self.storage.recordings_dir = crate::paths::default_recordings_dir()
                .to_string_lossy()
                .into_owned();
        }
        if self.file_name_template.trim().is_empty() {
            self.file_name_template = "{game}_{date}_{time}".into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn old_files_missing_fields_still_load() {
        let s: Settings = serde_json::from_str(r#"{"general":{"language":"ar"}}"#).unwrap();
        assert_eq!(s.general.language, "ar");
        assert_eq!(s.hotkeys.get("overlay").map(String::as_str), Some("Alt+Z"));
        assert_eq!(s.replay.duration_secs, 60);
    }

    #[test]
    fn sanitize_clamps_bad_values() {
        let mut s = Settings::default();
        s.replay.duration_secs = 1;
        s.audio.mic_volume = 9999;
        s.general.language = "xx".into();
        s.hotkeys.remove("record");
        s.sanitize();
        assert_eq!(s.replay.duration_secs, 10);
        assert_eq!(s.audio.mic_volume, 200);
        assert_eq!(s.general.language, "en");
        assert!(s.hotkeys.contains_key("record"));
    }
}
