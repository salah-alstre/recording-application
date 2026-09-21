//! Rimlight – capture platform for Windows.
pub mod audio;
pub mod capture;
pub mod commands;
pub mod db;
pub mod editor;
pub mod encoder;
pub mod engine;
pub mod error;
pub mod hotkeys;
pub mod logging;
pub mod naming;
pub mod paths;
pub mod quality;
pub mod replay;
pub mod screenshot;
pub mod selftest;
pub mod settings;
pub mod shell;
pub mod sink;
pub mod state_machine;
pub mod storage;
pub mod system;
pub mod thumbs;
pub mod update;

use db::Db;
use engine::Engine;
use serde_json::json;
use std::sync::Arc;
use system::hardware::{self, HardwareInfo};
use tauri::{Emitter, Manager, WindowEvent};

/// Detects GPUs and probes which encoders actually initialise on this machine.
pub fn probe_hardware(engine: &Arc<Engine>) -> HardwareInfo {
    let gpus = hardware::detect_gpus();
    let vendors: Vec<String> = gpus
        .iter()
        .filter_map(|g| hardware::encoder_family(&g.vendor))
        .map(String::from)
        .collect();
    let enc = encoder::probe_all(&vendors);
    for e in enc.iter().filter(|e| e.available) {
        tracing::info!("encoder available: {}", e.label);
    }
    *engine.encoders.write() = enc.clone();
    let hw = hardware::gather(enc);
    *engine.hardware.write() = Some(hw.clone());
    let _ = engine.app.emit("hardware-ready", &hw);
    hw
}

pub fn run() {
    paths::ensure_dirs();
    let db = match Db::open(&paths::db_path()) {
        Ok(d) => Arc::new(d),
        Err(e) => {
            eprintln!(
                "database unavailable ({}), using a temporary in-memory database",
                e.message
            );
            Arc::new(Db::open_memory().expect("in-memory database"))
        }
    };
    let settings = db.load_settings();
    logging::init(&settings.log_level);

    let selftest = std::env::args().any(|a| a == "--selftest");
    let autostart = std::env::args().any(|a| a == "--autostart");

    let result = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            shell::show_main(app, None);
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::get_status,
            commands::get_hardware,
            commands::reprobe_hardware,
            commands::list_monitors,
            commands::list_windows,
            commands::detect_game,
            commands::list_audio_devices,
            commands::list_cameras,
            commands::recommend_quality,
            commands::estimate_size,
            commands::start_recording,
            commands::stop_recording,
            commands::toggle_recording,
            commands::pause_recording,
            commands::resume_recording,
            commands::toggle_replay,
            commands::save_replay,
            commands::add_marker,
            commands::toggle_mic,
            commands::toggle_camera,
            commands::take_screenshot,
            commands::region_pick,
            commands::region_confirm,
            commands::region_cancel,
            commands::overlay_hide,
            commands::overlay_toggle,
            commands::open_main,
            commands::hide_toast_window,
            commands::quit_app,
            commands::list_clips,
            commands::scan_library,
            commands::get_clip,
            commands::set_favorite,
            commands::set_notes,
            commands::rename_clip,
            commands::delete_clip,
            commands::reveal_clip,
            commands::reveal_file,
            commands::list_markers,
            commands::add_clip_marker,
            commands::delete_marker,
            commands::export_clip,
            commands::list_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::set_active_profile,
            commands::list_game_configs,
            commands::save_game_config,
            commands::delete_game_config,
            commands::storage_info,
            commands::hotkey_status,
            commands::check_hotkey,
            commands::stats_subscribe,
            commands::levels_subscribe,
            commands::list_interrupted,
            commands::recover_session,
            commands::discard_session,
            commands::session_history,
            commands::app_info,
            commands::open_logs_dir,
            commands::open_folder,
            commands::check_update,
            commands::install_update,
            commands::dismiss_error,
            commands::open_path,
            commands::thumbs_busy,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let engine = Engine::new(handle.clone(), db.clone(), settings.clone());
            app.manage(engine.clone());

            // A manual start always shows the window; a Windows-startup launch honours the start mode.
            let mode = settings.general.start_mode.clone();
            let show_main = !selftest && !(autostart && mode == "tray");
            shell::create_windows(&handle, show_main)?;
            if autostart && mode == "minimized" {
                if let Some(w) = handle.get_webview_window("main") {
                    let _ = w.minimize();
                }
            }
            shell::create_tray(&handle, &engine)?;
            commands::allow_asset_dirs(&handle, &settings);

            engine::purge_stale_replay_cache();
            system::notifications::restore_if_pending(&db);
            engine.spawn_hubs();
            engine.spawn_game_watcher();
            engine.repair_thumbnails();

            let status = hotkeys::register_all(&handle, &engine);
            for s in status.iter().filter(|s| !s.ok) {
                engine.notify(
                    "warning",
                    "hotkey_failed",
                    json!({ "accel": s.accel, "action": s.action, "problem": s.problem }),
                    None,
                );
            }

            let e = engine.clone();
            std::thread::Builder::new()
                .name("rimlight-startup".into())
                .spawn(move || {
                    probe_hardware(&e);
                    if selftest {
                        selftest::run(&e);
                        return;
                    }
                    if e.settings.read().general.auto_start_replay {
                        if let Err(err) = e.start_replay() {
                            e.set_error(err);
                        }
                    }
                    let (check, url) = {
                        let s = e.settings.read();
                        (s.general.check_updates, s.general.update_feed_url.clone())
                    };
                    if check && !url.is_empty() {
                        if let Ok(info) = update::check(&url) {
                            if info.available {
                                e.notify(
                                    "info",
                                    "update_available",
                                    json!({ "version": info.latest }),
                                    None,
                                );
                            }
                        }
                    }
                })
                .ok();
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() != "main" {
                    return;
                }
                let engine = window.app_handle().state::<Arc<Engine>>().inner().clone();
                api.prevent_close();
                if engine.settings.read().general.close_to_tray {
                    let _ = window.hide();
                } else {
                    shell::quit(window.app_handle(), &engine);
                }
            }
        })
        .run(tauri::generate_context!());
    if let Err(e) = result {
        tracing::error!("fatal: {e}");
        eprintln!("Rimlight failed to start: {e}");
    }
}
