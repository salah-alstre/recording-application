//! Native windows and tray: main window, quick overlay, recording HUD, stats overlay, toast
//! window, region picker – plus positioning, capture exclusion and the system-tray menu.
//!
//! The overlay / HUD / stats / toast windows are created once at start-up (hidden), so showing
//! the overlay is just a `show()` on an already-loaded WebView – effectively instant.
use crate::engine::Engine;
use crate::state_machine::RecState;
use crate::system::monitors::{self, MonitorInfo};
use crate::system::winenum;
use std::sync::Arc;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE,
    HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SW_SHOWNOACTIVATE, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW,
};

const TRAY_IDLE: &[u8] = include_bytes!("../icons/tray-idle.png");
const TRAY_REC: &[u8] = include_bytes!("../icons/tray-rec.png");

fn hwnd_of(w: &WebviewWindow) -> Option<HWND> {
    w.hwnd().ok().map(|h| HWND(h.0 as *mut _))
}

/// Makes a window click-through-safe and non-activating so it never steals focus from a game.
fn make_non_activating(w: &WebviewWindow) {
    if let Some(h) = hwnd_of(w) {
        unsafe {
            let ex = GetWindowLongPtrW(h, GWL_EXSTYLE);
            SetWindowLongPtrW(
                h,
                GWL_EXSTYLE,
                ex | WS_EX_NOACTIVATE.0 as isize | WS_EX_TOOLWINDOW.0 as isize,
            );
        }
    }
}

fn show_no_activate(w: &WebviewWindow) {
    if let Some(h) = hwnd_of(w) {
        unsafe {
            let _ = ShowWindow(h, SW_SHOWNOACTIVATE);
            let _ = SetWindowPos(
                h,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

/// Monitor of the foreground window, else the one under the cursor, else the first.
pub fn active_monitor() -> MonitorInfo {
    if let Some(w) = winenum::foreground() {
        if let Some(m) = monitors::at_point(w.x + w.width as i32 / 2, w.y + w.height as i32 / 2) {
            return m;
        }
    }
    let mut p = windows::Win32::Foundation::POINT::default();
    if unsafe { GetCursorPos(&mut p) }.is_ok() {
        if let Some(m) = monitors::at_point(p.x, p.y) {
            return m;
        }
    }
    monitors::list().into_iter().next().unwrap_or(MonitorInfo {
        index: 1,
        name: String::new(),
        width: 1920,
        height: 1080,
        refresh_hz: 60,
        x: 0,
        y: 0,
        primary: true,
    })
}

fn place(w: &WebviewWindow, x: i32, y: i32, width: u32, height: u32) {
    let _ = w.set_position(PhysicalPosition::new(x, y));
    let _ = w.set_size(PhysicalSize::new(width, height));
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

fn corner_xy(m: &MonitorInfo, corner: &str, w: u32, h: u32, margin: i32) -> (i32, i32) {
    let right = m.x + m.width as i32 - w as i32 - margin;
    let bottom = m.y + m.height as i32 - h as i32 - margin;
    match corner {
        "top-right" => (right, m.y + margin),
        "bottom-left" => (m.x + margin, bottom),
        "bottom-right" => (right, bottom),
        _ => (m.x + margin, m.y + margin),
    }
}

// ---- window creation ------------------------------------------------------------------------------

fn overlay_builder<'a>(
    app: &'a AppHandle,
    label: &str,
    url: &str,
    w: f64,
    h: f64,
) -> WebviewWindowBuilder<'a, tauri::Wry, AppHandle> {
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .title("Rimlight")
        .inner_size(w, h)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .focused(false)
}

pub fn create_windows(app: &AppHandle, show_main: bool) -> tauri::Result<()> {
    if app.get_webview_window("main").is_none() {
        WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
            .title("Rimlight")
            .inner_size(1240.0, 800.0)
            .min_inner_size(980.0, 640.0)
            .decorations(false)
            .shadow(true)
            .visible(show_main)
            .center()
            .build()?;
    }
    let overlay = overlay_builder(app, "overlay", "overlay.html", 800.0, 600.0)
        .focused(true)
        .build()?;
    let _ = overlay.set_content_protected(true);

    let hud = overlay_builder(app, "hud", "hud.html?kind=hud", 300.0, 64.0).build()?;
    let _ = hud.set_ignore_cursor_events(true);
    make_non_activating(&hud);

    let stats = overlay_builder(app, "stats", "hud.html?kind=stats", 280.0, 320.0).build()?;
    let _ = stats.set_ignore_cursor_events(true);
    make_non_activating(&stats);

    let toast = overlay_builder(app, "toast", "hud.html?kind=toast", 420.0, 420.0).build()?;
    make_non_activating(&toast);

    let region = overlay_builder(app, "region", "region.html", 800.0, 600.0)
        .focused(true)
        .build()?;
    let _ = region.set_content_protected(true);
    Ok(())
}

// ---- overlay ---------------------------------------------------------------------------------------

pub fn overlay_visible(app: &AppHandle) -> bool {
    app.get_webview_window("overlay")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

pub fn show_overlay(app: &AppHandle) {
    let Some(w) = app.get_webview_window("overlay") else {
        return;
    };
    let m = active_monitor();
    place(&w, m.x, m.y, m.width, m.height);
    let _ = w.show();
    let _ = w.set_always_on_top(true);
    let _ = w.set_focus();
    let _ = w.emit("overlay-shown", ());
}

pub fn hide_overlay(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.emit("overlay-hidden", ());
        let _ = w.hide();
    }
}

pub fn toggle_overlay(app: &AppHandle) {
    if overlay_visible(app) {
        hide_overlay(app);
    } else {
        show_overlay(app);
    }
}

// ---- HUD / stats / toast ------------------------------------------------------------------------------

pub fn show_toast_window(app: &AppHandle) {
    let Some(w) = app.get_webview_window("toast") else {
        return;
    };
    let m = active_monitor();
    let (ww, hh) = (420u32, 420u32);
    let (x, y) = corner_xy(&m, "bottom-right", ww, hh, 24);
    place(&w, x, y, ww, hh);
    show_no_activate(&w);
}

pub fn hide_toast_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("toast") {
        let _ = w.hide();
    }
}

/// Re-evaluates which HUD windows should be visible and where.
pub fn update_hud(engine: &Arc<Engine>) {
    let app = &engine.app;
    let s = engine.settings.read().clone();
    let active = {
        let g = engine.inner.lock();
        g.rec.is_some() || g.replay.is_some()
    };
    let m = crate::shell::capture_monitor(engine);
    let protect = !s.performance.hud_include_in_capture;
    if let Some(w) = app.get_webview_window("hud") {
        let _ = w.set_content_protected(protect);
        if active && s.performance.hud_enabled {
            let (ww, hh) = (300u32, 64u32);
            let (x, y) = corner_xy(&m, &s.performance.hud_position, ww, hh, 20);
            place(&w, x, y, ww, hh);
            show_no_activate(&w);
        } else {
            let _ = w.hide();
        }
    }
    if let Some(w) = app.get_webview_window("stats") {
        let _ = w.set_content_protected(protect);
        if s.performance.stats_enabled {
            let (ww, hh) = (280u32, 320u32);
            // keep the stats block clear of the HUD when both sit in the same corner
            let margin = if s.performance.stats_position == s.performance.hud_position
                && active
                && s.performance.hud_enabled
            {
                88
            } else {
                20
            };
            let (x, mut y) = corner_xy(&m, &s.performance.stats_position, ww, hh, 20);
            if margin > 20 {
                y += if s.performance.stats_position.starts_with("top") {
                    68
                } else {
                    -68
                };
            }
            place(&w, x, y, ww, hh);
            show_no_activate(&w);
        } else {
            let _ = w.hide();
        }
    }
    if let Some(w) = app.get_webview_window("toast") {
        let _ = w.set_content_protected(protect);
    }
}

/// Monitor being captured (or the active one when idle).
pub fn capture_monitor(engine: &Engine) -> MonitorInfo {
    let idx = engine
        .inner
        .lock()
        .session
        .as_ref()
        .map(|s| s.spec.lock().monitor_index());
    match idx.and_then(monitors::get) {
        Some(m) => m,
        None => active_monitor(),
    }
}

pub fn on_recording_changed(engine: &Arc<Engine>) {
    update_hud(engine);
    update_tray(engine);
}

// ---- region picker ----------------------------------------------------------------------------------

pub fn show_region_picker(app: &AppHandle) {
    let Some(w) = app.get_webview_window("region") else {
        return;
    };
    let mut p = windows::Win32::Foundation::POINT::default();
    let m = if unsafe { GetCursorPos(&mut p) }.is_ok() {
        monitors::at_point(p.x, p.y)
    } else {
        None
    }
    .unwrap_or_else(active_monitor);
    place(&w, m.x, m.y, m.width, m.height);
    let _ = w.emit(
        "region-open",
        serde_json::json!({ "displayIndex": m.index }),
    );
    let _ = w.show();
    let _ = w.set_focus();
}

pub fn hide_region_picker(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("region") {
        let _ = w.hide();
    }
}

// ---- main window ---------------------------------------------------------------------------------------

pub fn show_main(app: &AppHandle, page: Option<&str>) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        if let Some(p) = page {
            let _ = w.emit("navigate", p);
        }
    }
}

// ---- tray ------------------------------------------------------------------------------------------------

struct TrayText {
    open: &'static str,
    start: &'static str,
    stop: &'static str,
    save_replay: &'static str,
    shot: &'static str,
    mic: &'static str,
    settings: &'static str,
    exit: &'static str,
    idle: &'static str,
    rec: &'static str,
}

fn tray_text(lang: &str) -> TrayText {
    if lang == "ar" {
        TrayText {
            open: "فتح Rimlight",
            start: "بدء التسجيل",
            stop: "إيقاف التسجيل",
            save_replay: "حفظ الإعادة الفورية",
            shot: "لقطة شاشة",
            mic: "تبديل الميكروفون",
            settings: "الإعدادات",
            exit: "خروج",
            idle: "Rimlight — جاهز",
            rec: "Rimlight — جارٍ التسجيل",
        }
    } else {
        TrayText {
            open: "Open Rimlight",
            start: "Start recording",
            stop: "Stop recording",
            save_replay: "Save replay",
            shot: "Take screenshot",
            mic: "Toggle microphone",
            settings: "Settings",
            exit: "Exit",
            idle: "Rimlight — ready",
            rec: "Rimlight — recording",
        }
    }
}

fn build_menu(app: &AppHandle, engine: &Engine) -> tauri::Result<Menu<tauri::Wry>> {
    let lang = engine.settings.read().general.language.clone();
    let t = tray_text(&lang);
    let recording = matches!(engine.state(), RecState::Recording | RecState::Paused);
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(app, "open", t.open, true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "start",
        t.start,
        !recording,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "stop",
        t.stop,
        recording,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        "replay",
        t.save_replay,
        engine.inner.lock().replay.is_some(),
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, "shot", t.shot, true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "mic", t.mic, true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "settings",
        t.settings,
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, "exit", t.exit, true, None::<&str>)?)?;
    Ok(menu)
}

pub fn create_tray(app: &AppHandle, engine: &Arc<Engine>) -> tauri::Result<()> {
    let menu = build_menu(app, engine)?;
    let e = engine.clone();
    TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(TRAY_IDLE)?)
        .tooltip(tray_text(&engine.settings.read().general.language).idle)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, ev| handle_tray_action(app, &e, ev.id.as_ref()))
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = ev
            {
                show_main(tray.app_handle(), None);
            }
        })
        .build(app)?;
    Ok(())
}

fn handle_tray_action(app: &AppHandle, engine: &Arc<Engine>, id: &str) {
    let e = engine.clone();
    match id {
        "open" => show_main(app, None),
        "settings" => show_main(app, Some("settings")),
        "start" => {
            std::thread::spawn(move || {
                let _ = e.start_recording();
            });
        }
        "stop" => {
            std::thread::spawn(move || {
                let _ = e.stop_recording();
            });
        }
        "replay" => {
            std::thread::spawn(move || {
                if let Err(err) = e.save_replay() {
                    e.set_error(err);
                }
            });
        }
        "shot" => {
            std::thread::spawn(move || {
                let _ = crate::screenshot::take(&e, None);
            });
        }
        "mic" => e.toggle_mic(),
        "exit" => quit(app, engine),
        _ => {}
    }
}

pub fn update_tray(engine: &Arc<Engine>) {
    let app = &engine.app;
    let Some(tray) = app.tray_by_id("main") else {
        return;
    };
    let lang = engine.settings.read().general.language.clone();
    let t = tray_text(&lang);
    let recording = matches!(engine.state(), RecState::Recording | RecState::Paused);
    let _ = tray.set_icon(Image::from_bytes(if recording { TRAY_REC } else { TRAY_IDLE }).ok());
    let _ = tray.set_tooltip(Some(if recording { t.rec } else { t.idle }));
    if let Ok(menu) = build_menu(app, engine) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// Stops everything cleanly (finalising an active recording) and exits.
pub fn quit(app: &AppHandle, engine: &Arc<Engine>) {
    let app = app.clone();
    let e = engine.clone();
    std::thread::spawn(move || {
        let _ = e.stop_replay();
        if matches!(e.state(), RecState::Recording | RecState::Paused) {
            let _ = e.stop_recording();
        }
        let t0 = std::time::Instant::now();
        while e.state() == RecState::Stopping && t0.elapsed() < std::time::Duration::from_secs(90) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        tracing::info!("clean exit");
        app.exit(0);
    });
}
