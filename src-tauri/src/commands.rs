//! Tauri commands: the thin, typed surface the UI talks to. All heavy work is done by the engine on
//! native threads; commands that may block are run through `blocking` so they never stall the UI.
use crate::audio::{self, DeviceList};
use crate::db::{Clip, ClipQuery, GameConfig, Marker, Profile, SessionRow};
use crate::editor::ExportOpts;
use crate::engine::{Engine, StatusDto};
use crate::error::{AppError, Result};
use crate::hotkeys::{self, HotkeyStatus};
use crate::quality::{self, Quality};
use crate::settings::{Rect, Settings};
use crate::system::hardware::{self, HardwareInfo};
use crate::system::monitors::{self, MonitorInfo};
use crate::system::winenum::{self, GameInfo, WindowInfo};
use crate::{shell, storage};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt as _;

type Eng<'a> = State<'a, Arc<Engine>>;

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(AppError::internal)?
}

// ---- settings & status --------------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings(e: Eng) -> Settings {
    e.settings.read().clone()
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, e: Eng<'_>, settings: Settings) -> Result<Settings> {
    let engine = e.inner().clone();
    blocking(move || {
        let old = engine.settings.read().clone();
        let new = engine.apply_settings(settings)?;
        if old.general.launch_with_windows != new.general.launch_with_windows {
            let al = app.autolaunch();
            let r = if new.general.launch_with_windows {
                al.enable()
            } else {
                al.disable()
            };
            if let Err(err) = r {
                tracing::warn!("autostart: {err}");
                return Err(AppError::new(
                    "autostart",
                    format!("Windows startup could not be changed: {err}"),
                ));
            }
        }
        if old.hotkeys != new.hotkeys {
            let st = hotkeys::register_all(&app, &engine);
            let _ = tauri::Emitter::emit(&app, "hotkeys-status", &st);
        }
        allow_asset_dirs(&app, &new);
        shell::on_recording_changed(&engine);
        Ok(new)
    })
    .await
}

pub fn allow_asset_dirs(app: &AppHandle, s: &Settings) {
    let scope = app.asset_protocol_scope();
    let thumbs = crate::paths::thumbs_dir();
    let _ = std::fs::create_dir_all(&thumbs);
    let _ = scope.allow_directory(&thumbs, true);
    for d in [
        s.storage.recordings_dir.as_str(),
        s.screenshots.dir.as_str(),
    ] {
        if !d.trim().is_empty() {
            let _ = std::fs::create_dir_all(d);
            let _ = scope.allow_directory(d, true);
        }
    }
}

#[tauri::command]
pub fn get_status(e: Eng) -> StatusDto {
    e.status()
}

#[tauri::command]
pub fn get_hardware(e: Eng) -> Option<HardwareInfo> {
    e.hardware.read().clone()
}

#[tauri::command]
pub async fn reprobe_hardware(e: Eng<'_>) -> Result<HardwareInfo> {
    let engine = e.inner().clone();
    blocking(move || Ok(crate::probe_hardware(&engine))).await
}

#[tauri::command]
pub fn list_monitors() -> Vec<MonitorInfo> {
    monitors::list()
}

#[tauri::command]
pub async fn list_windows() -> Vec<WindowInfo> {
    tauri::async_runtime::spawn_blocking(winenum::list_windows)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub fn detect_game() -> Option<GameInfo> {
    winenum::detect_game()
}

#[tauri::command]
pub async fn list_audio_devices() -> DeviceList {
    tauri::async_runtime::spawn_blocking(audio::list_devices)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn list_cameras() -> Vec<String> {
    tauri::async_runtime::spawn_blocking(crate::encoder::list_cameras)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub fn recommend_quality(e: Eng) -> Quality {
    let m = monitors::list()
        .into_iter()
        .find(|m| m.primary)
        .or_else(|| monitors::list().into_iter().next());
    let (h, hz) = m.map(|m| (m.height, m.refresh_hz)).unwrap_or((1080, 60));
    hardware::recommend(h, hz, &e.encoders.read())
}

#[tauri::command]
pub fn estimate_size(quality: Quality, tracks: u32, seconds: u64) -> u64 {
    quality::estimate_bytes(&quality, tracks, seconds)
}

// ---- recording control -------------------------------------------------------------------------------

#[tauri::command]
pub async fn start_recording(e: Eng<'_>) -> Result<()> {
    let e = e.inner().clone();
    blocking(move || e.start_recording()).await
}
#[tauri::command]
pub async fn stop_recording(e: Eng<'_>) -> Result<()> {
    let e = e.inner().clone();
    blocking(move || e.stop_recording()).await
}
#[tauri::command]
pub async fn toggle_recording(e: Eng<'_>) -> Result<()> {
    let e = e.inner().clone();
    blocking(move || e.toggle_recording()).await
}
#[tauri::command]
pub async fn pause_recording(e: Eng<'_>) -> Result<()> {
    let e = e.inner().clone();
    blocking(move || e.pause_recording()).await
}
#[tauri::command]
pub async fn resume_recording(e: Eng<'_>) -> Result<()> {
    let e = e.inner().clone();
    blocking(move || e.resume_recording()).await
}
#[tauri::command]
pub async fn toggle_replay(e: Eng<'_>) -> Result<()> {
    let e = e.inner().clone();
    blocking(move || e.toggle_replay()).await
}
#[tauri::command]
pub async fn save_replay(e: Eng<'_>) -> Result<()> {
    let e = e.inner().clone();
    blocking(move || e.save_replay()).await
}
#[tauri::command]
pub async fn add_marker(e: Eng<'_>) -> Result<()> {
    let e = e.inner().clone();
    blocking(move || e.add_marker()).await
}
#[tauri::command]
pub async fn toggle_mic(e: Eng<'_>) -> Result<()> {
    e.toggle_mic();
    Ok(())
}
#[tauri::command]
pub async fn toggle_camera(e: Eng<'_>) -> Result<()> {
    e.toggle_camera();
    Ok(())
}

#[tauri::command]
pub async fn take_screenshot(e: Eng<'_>, mode: Option<String>) -> Result<String> {
    let e = e.inner().clone();
    blocking(move || crate::screenshot::take(&e, mode.as_deref())).await
}

#[tauri::command]
pub fn region_pick(app: AppHandle, e: Eng) {
    e.inner.lock().pending_region = Some("select".into());
    shell::show_region_picker(&app);
}

#[tauri::command]
pub async fn region_confirm(
    app: AppHandle,
    e: Eng<'_>,
    display_index: u32,
    rect: Rect,
) -> Result<()> {
    shell::hide_region_picker(&app);
    let action = e.inner.lock().pending_region.take();
    let engine = e.inner().clone();
    blocking(move || {
        if action.as_deref() == Some("screenshot") {
            std::thread::sleep(std::time::Duration::from_millis(120)); // let the picker fade out
            crate::screenshot::take_region(&engine, display_index, rect).map(|_| ())
        } else {
            let mut s = engine.settings.read().clone();
            s.capture.mode = "region".into();
            s.capture.display_index = display_index;
            s.capture.region = rect;
            engine.apply_settings(s).map(|_| ())
        }
    })
    .await
}

#[tauri::command]
pub fn region_cancel(app: AppHandle, e: Eng) {
    e.inner.lock().pending_region = None;
    shell::hide_region_picker(&app);
}

// ---- overlay / windows -------------------------------------------------------------------------------------

#[tauri::command]
pub fn overlay_hide(app: AppHandle) {
    shell::hide_overlay(&app);
}
#[tauri::command]
pub fn overlay_toggle(app: AppHandle) {
    shell::toggle_overlay(&app);
}
#[tauri::command]
pub fn open_main(app: AppHandle, page: Option<String>) {
    shell::hide_overlay(&app);
    shell::show_main(&app, page.as_deref());
}
#[tauri::command]
pub fn hide_toast_window(app: AppHandle) {
    shell::hide_toast_window(&app);
}
#[tauri::command]
pub fn quit_app(app: AppHandle, e: Eng) {
    shell::quit(&app, e.inner());
}

// ---- library --------------------------------------------------------------------------------------------------

#[tauri::command]
pub fn list_clips(e: Eng, query: ClipQuery) -> Result<Vec<Clip>> {
    e.db.query_clips(&query)
}

#[tauri::command]
pub async fn scan_library(e: Eng<'_>) -> Result<usize> {
    let e = e.inner().clone();
    blocking(move || {
        let n = e.scan_library();
        e.repair_thumbnails();
        Ok(n)
    })
    .await
}

#[tauri::command]
pub fn thumbs_busy() -> bool {
    crate::thumbs::is_repairing()
}

#[tauri::command]
pub fn get_clip(e: Eng, id: String) -> Option<Clip> {
    e.db.get_clip(&id)
}

#[tauri::command]
pub fn set_favorite(e: Eng, id: String, favorite: bool) -> Result<()> {
    e.db.set_favorite(&id, favorite)
}

#[tauri::command]
pub fn set_notes(e: Eng, id: String, notes: String) -> Result<()> {
    e.db.set_notes(&id, &notes)
}

#[tauri::command]
pub fn rename_clip(e: Eng, id: String, name: String) -> Result<Clip> {
    let clip =
        e.db.get_clip(&id)
            .ok_or_else(|| AppError::new("not_found", "That clip is no longer in the library."))?;
    let old = PathBuf::from(&clip.path);
    let stem = crate::naming::sanitize_component(&name);
    let ext = old
        .extension()
        .map(|x| x.to_string_lossy().into_owned())
        .unwrap_or_default();
    let new = old.with_file_name(format!("{stem}.{ext}"));
    if new != old {
        if new.exists() {
            return Err(
                AppError::new("name_taken", "A file with that name already exists.")
                    .hint("Choose a different name."),
            );
        }
        std::fs::rename(&old, &new)
            .map_err(|err| AppError::new("io", format!("The file could not be renamed: {err}")))?;
    }
    let is_shot = clip.kind == "screenshot";
    e.db.set_clip_path(&id, &new.to_string_lossy(), &stem)?;
    if is_shot {
        e.db.upsert_clip(&Clip {
            path: new.to_string_lossy().into_owned(),
            title: stem.clone(),
            thumb_path: new.to_string_lossy().into_owned(),
            ..clip.clone()
        })?;
    }
    e.db.get_clip(&id)
        .ok_or_else(|| AppError::new("not_found", "That clip is no longer in the library."))
}

/// Moves a file to the Recycle Bin (recoverable). Falls back to plain deletion only if that fails.
fn recycle(path: &Path) -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{
        SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FO_DELETE,
        SHFILEOPSTRUCTW,
    };
    let mut wide: Vec<u16> = path.as_os_str().to_string_lossy().encode_utf16().collect();
    wide.extend([0, 0]);
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: PCWSTR(wide.as_ptr()),
        fFlags: (FOF_ALLOWUNDO.0 | FOF_NOCONFIRMATION.0 | FOF_SILENT.0 | FOF_NOERRORUI.0) as u16,
        ..Default::default()
    };
    unsafe { SHFileOperationW(&mut op) == 0 && !op.fAnyOperationsAborted.as_bool() }
}

#[tauri::command]
pub fn delete_clip(e: Eng, id: String) -> Result<()> {
    if let Some(c) = e.db.get_clip(&id) {
        let p = PathBuf::from(&c.path);
        if p.exists() && !recycle(&p) {
            std::fs::remove_file(&p)?;
        }
        if c.kind != "screenshot" && !c.thumb_path.is_empty() {
            let _ = std::fs::remove_file(&c.thumb_path);
        }
    }
    e.db.delete_clip(&id)
}

#[tauri::command]
pub fn reveal_clip(e: Eng, id: String) -> Result<()> {
    let c =
        e.db.get_clip(&id)
            .ok_or_else(|| AppError::new("not_found", "That clip is no longer in the library."))?;
    reveal_path(&c.path)
}

pub fn reveal_path(path: &str) -> Result<()> {
    std::process::Command::new("explorer.exe")
        .arg(format!("/select,{path}"))
        .spawn()
        .map_err(|err| AppError::new("io", format!("Explorer could not be opened: {err}")))?;
    Ok(())
}

#[tauri::command]
pub fn reveal_file(path: String) -> Result<()> {
    reveal_path(&path)
}

#[tauri::command]
pub fn list_markers(e: Eng, clip_id: String) -> Result<Vec<Marker>> {
    e.db.list_markers(&clip_id)
}
#[tauri::command]
pub fn add_clip_marker(e: Eng, clip_id: String, t_ms: i64, label: String) -> Result<i64> {
    e.db.add_marker(&clip_id, t_ms, &label)
}
#[tauri::command]
pub fn delete_marker(e: Eng, id: i64) -> Result<()> {
    e.db.delete_marker(id)
}

#[tauri::command]
pub async fn export_clip(e: Eng<'_>, opts: ExportOpts) -> Result<String> {
    let e = e.inner().clone();
    blocking(move || e.export_clip(opts)).await
}

// ---- profiles & games ------------------------------------------------------------------------------------------

#[tauri::command]
pub fn list_profiles(e: Eng) -> Result<Vec<Profile>> {
    e.db.list_profiles()
}
#[tauri::command]
pub fn save_profile(e: Eng, profile: Profile) -> Result<Profile> {
    let engine = e.inner().clone();
    let mut p = profile;
    if p.id.is_empty() {
        p.id = format!("profile-{}", uuid::Uuid::new_v4().simple());
    }
    p.builtin = e.db.get_profile(&p.id).map(|x| x.builtin).unwrap_or(false);
    e.db.save_profile(&p)?;
    engine.profile_changed(&p.id);
    Ok(p)
}
#[tauri::command]
pub fn delete_profile(e: Eng, id: String) -> Result<bool> {
    let deleted = e.db.delete_profile(&id)?;
    if deleted && e.settings.read().active_profile_id == id {
        if let Some(first) = e.db.list_profiles()?.first() {
            let mut s = e.settings.read().clone();
            s.active_profile_id = first.id.clone();
            e.apply_settings(s)?;
        }
    }
    Ok(deleted)
}
#[tauri::command]
pub fn set_active_profile(e: Eng, id: String) -> Result<Settings> {
    let p =
        e.db.get_profile(&id)
            .ok_or_else(|| AppError::new("not_found", "That profile no longer exists."))?;
    let mut s = e.settings.read().clone();
    s.active_profile_id = p.id;
    s.audio.mic_enabled = p.mic_enabled;
    s.audio.system_enabled = p.system_enabled;
    s.camera.enabled = p.camera_enabled;
    e.apply_settings(s)
}
#[tauri::command]
pub fn list_game_configs(e: Eng) -> Result<Vec<GameConfig>> {
    e.db.list_game_configs()
}
#[tauri::command]
pub fn save_game_config(e: Eng, config: GameConfig) -> Result<()> {
    e.db.save_game_config(&config)
}
#[tauri::command]
pub fn delete_game_config(e: Eng, exe: String) -> Result<()> {
    e.db.delete_game_config(&exe)
}

// ---- storage ------------------------------------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub dir: String,
    pub exists: bool,
    pub free_bytes: u64,
    pub total_bytes: u64,
    pub library_bytes: i64,
    pub clip_count: usize,
}

#[tauri::command]
pub fn storage_info(e: Eng) -> Result<StorageInfo> {
    let dir = e.settings.read().storage.recordings_dir.clone();
    let clips = e.db.all_clips()?;
    let d = storage::disk_space(Path::new(&dir));
    Ok(StorageInfo {
        exists: Path::new(&dir).exists(),
        dir,
        free_bytes: d.as_ref().map(|d| d.free_bytes).unwrap_or(0),
        total_bytes: d.as_ref().map(|d| d.total_bytes).unwrap_or(0),
        library_bytes: clips.iter().map(|c| c.size_bytes).sum(),
        clip_count: clips.len(),
    })
}

// ---- hotkeys -----------------------------------------------------------------------------------------------------------

#[tauri::command]
pub fn hotkey_status() -> Vec<HotkeyStatus> {
    hotkeys::last_status()
}

#[tauri::command]
pub fn check_hotkey(app: AppHandle, e: Eng, action: String, accel: String) -> HotkeyStatus {
    hotkeys::check(&app, &e, &action, &accel)
}

// ---- stats / diagnostics -------------------------------------------------------------------------------------------------

#[tauri::command]
pub fn stats_subscribe(e: Eng, on: bool) {
    e.hub.stats_subscribe(if on { 1 } else { -1 });
}
#[tauri::command]
pub fn levels_subscribe(e: Eng, on: bool) {
    e.levels_subscribe(on);
}

// ---- recovery -------------------------------------------------------------------------------------------------------------

#[tauri::command]
pub fn list_interrupted(e: Eng) -> Vec<SessionRow> {
    e.interrupted()
}
#[tauri::command]
pub async fn recover_session(e: Eng<'_>, id: String) -> Result<Clip> {
    let e = e.inner().clone();
    blocking(move || e.recover_session(&id)).await
}
#[tauri::command]
pub fn discard_session(e: Eng, id: String) -> Result<()> {
    e.discard_session(&id)
}
#[tauri::command]
pub fn session_history(e: Eng, limit: u32) -> Result<Vec<SessionRow>> {
    e.db.session_history(limit)
}

// ---- misc ----------------------------------------------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub data_dir: String,
    pub logs_dir: String,
    pub ffmpeg_ok: bool,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").into(),
        data_dir: crate::paths::data_dir().to_string_lossy().into_owned(),
        logs_dir: crate::paths::logs_dir().to_string_lossy().into_owned(),
        ffmpeg_ok: crate::encoder::ffmpeg_available(),
    }
}

#[tauri::command]
pub fn open_logs_dir() -> Result<()> {
    std::process::Command::new("explorer.exe")
        .arg(crate::paths::logs_dir())
        .spawn()?;
    Ok(())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<()> {
    if !Path::new(&path).is_dir() {
        return Err(AppError::new(
            "folder_unavailable",
            format!("The folder \"{path}\" does not exist."),
        ));
    }
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()?;
    Ok(())
}

#[tauri::command]
pub async fn check_update(e: Eng<'_>) -> Result<crate::update::UpdateInfo> {
    let url = e.settings.read().general.update_feed_url.clone();
    blocking(move || crate::update::check(&url)).await
}

#[tauri::command]
pub async fn install_update(e: Eng<'_>, url: String) -> Result<()> {
    if matches!(
        e.state(),
        crate::state_machine::RecState::Recording
            | crate::state_machine::RecState::Paused
            | crate::state_machine::RecState::Stopping
    ) {
        return Err(AppError::new(
            "update_busy",
            "Updates are not installed while a recording is in progress.",
        )
        .hint("Stop the recording first."));
    }
    blocking(move || crate::update::download_and_launch(&url)).await
}

#[tauri::command]
pub fn dismiss_error(e: Eng) {
    e.dismiss_error();
}

/// Opens a file with its default application (e.g. a screenshot in the image viewer).
#[tauri::command]
pub fn open_path(path: String) -> Result<()> {
    if !Path::new(&path).exists() {
        return Err(AppError::new("file_missing", "That file no longer exists."));
    }
    std::process::Command::new("explorer.exe")
        .arg(path)
        .spawn()?;
    Ok(())
}
