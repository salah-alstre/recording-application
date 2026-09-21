//! Window enumeration and game detection.
use serde::Serialize;
use windows::Win32::Foundation::{CloseHandle, HWND, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, IsIconic};
use windows_capture::window::Window;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub exe: String,
    pub exe_path: String,
    pub pid: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    pub name: String,
    pub exe: String,
    pub pid: u32,
    pub hwnd: isize,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub display_index: u32,
}

/// Shell / utility processes that are never treated as capturable "apps" in pickers.
const SHELL_EXES: [&str; 6] = [
    "explorer.exe",
    "searchhost.exe",
    "startmenuexperiencehost.exe",
    "textinputhost.exe",
    "shellexperiencehost.exe",
    "applicationframehost.exe",
];
/// Full-screen candidates that are not games.
const NOT_GAMES: [&str; 14] = [
    "explorer.exe",
    "chrome.exe",
    "msedge.exe",
    "firefox.exe",
    "brave.exe",
    "opera.exe",
    "vlc.exe",
    "mpc-hc64.exe",
    "mpv.exe",
    "powerpnt.exe",
    "code.exe",
    "wmplayer.exe",
    "vivaldi.exe",
    "rimlight.exe",
];
const GAME_PATH_HINTS: [&str; 9] = [
    "\\steamapps\\common\\",
    "\\epic games\\",
    "\\gog galaxy\\games\\",
    "\\riot games\\",
    "\\battle.net\\",
    "\\ubisoft game launcher\\games\\",
    "\\xboxgames\\",
    "\\ea games\\",
    "\\origin games\\",
];

fn process_path(pid: u32) -> String {
    unsafe {
        let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return String::new();
        };
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            h,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(h);
        if ok {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

fn is_cloaked(hwnd: HWND) -> bool {
    let mut cloaked = 0u32;
    unsafe {
        DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, 4).is_ok()
            && cloaked != 0
    }
}

fn describe(w: &Window, own_pid: u32) -> Option<WindowInfo> {
    let hwnd = HWND(w.as_raw_hwnd());
    if unsafe { IsIconic(hwnd).as_bool() } || is_cloaked(hwnd) {
        return None;
    }
    let pid = w.process_id().ok()?;
    if pid == own_pid {
        return None;
    }
    let title = w.title().unwrap_or_default();
    let exe_path = process_path(pid);
    let exe = std::path::Path::new(&exe_path)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .or_else(|| w.process_name().ok())
        .unwrap_or_default();
    let mut r = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut r).ok()? };
    let (width, height) = (
        (r.right - r.left).max(0) as u32,
        (r.bottom - r.top).max(0) as u32,
    );
    if width < 64 || height < 64 {
        return None;
    }
    let fullscreen = unsafe {
        let mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(mon, &mut mi).as_bool()
            && r.left <= mi.rcMonitor.left
            && r.top <= mi.rcMonitor.top
            && r.right >= mi.rcMonitor.right
            && r.bottom >= mi.rcMonitor.bottom
    };
    Some(WindowInfo {
        hwnd: hwnd.0 as isize,
        title,
        exe,
        exe_path,
        pid,
        x: r.left,
        y: r.top,
        width,
        height,
        fullscreen,
    })
}

/// Visible, capturable top-level windows (excluding this app).
pub fn list_windows() -> Vec<WindowInfo> {
    let own = std::process::id();
    let Ok(all) = Window::enumerate() else {
        return vec![];
    };
    all.iter()
        .filter_map(|w| describe(w, own))
        .filter(|w| {
            !w.title.trim().is_empty() && !SHELL_EXES.contains(&w.exe.to_lowercase().as_str())
        })
        .collect()
}

pub fn foreground() -> Option<WindowInfo> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return None;
    }
    describe(&Window::from_raw_hwnd(hwnd.0), std::process::id())
}

pub fn window_by_hwnd(hwnd: isize) -> Option<WindowInfo> {
    let w = Window::from_raw_hwnd(hwnd as *mut _);
    if !w.is_valid() {
        return None;
    }
    describe(&w, std::process::id())
}

pub fn process_alive(pid: u32) -> bool {
    unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => {
                let mut code = 0u32;
                let alive = windows::Win32::System::Threading::GetExitCodeProcess(h, &mut code)
                    .is_ok()
                    && code == 259;
                let _ = CloseHandle(h);
                alive
            }
            Err(_) => false,
        }
    }
}

/// Human-friendly game name: the install folder for launcher libraries, else the window title.
pub fn game_name(w: &WindowInfo) -> String {
    let lower = w.exe_path.to_lowercase();
    for hint in GAME_PATH_HINTS {
        if let Some(pos) = lower.find(hint) {
            let rest = &w.exe_path[pos + hint.len()..];
            if let Some(folder) = rest.split('\\').next().filter(|f| !f.is_empty()) {
                return folder.to_string();
            }
        }
    }
    if !w.title.trim().is_empty() {
        return w.title.trim().to_string();
    }
    w.exe.trim_end_matches(".exe").to_string()
}

pub fn looks_like_game(w: &WindowInfo) -> bool {
    let exe = w.exe.to_lowercase();
    if exe.is_empty() || NOT_GAMES.contains(&exe.as_str()) {
        return false;
    }
    let lower = w.exe_path.to_lowercase();
    GAME_PATH_HINTS.iter().any(|h| lower.contains(h)) || w.fullscreen
}

/// The foreground application if it looks like a game (launcher library path or fullscreen).
pub fn detect_game() -> Option<GameInfo> {
    let w = foreground()?;
    if !looks_like_game(&w) {
        return None;
    }
    let display_index =
        super::monitors::at_point(w.x + w.width as i32 / 2, w.y + w.height as i32 / 2)
            .map(|m| m.index)
            .unwrap_or(1);
    Some(GameInfo {
        name: game_name(&w),
        exe: w.exe.clone(),
        pid: w.pid,
        hwnd: w.hwnd,
        width: w.width,
        height: w.height,
        fullscreen: w.fullscreen,
        display_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(exe: &str, path: &str, title: &str, fullscreen: bool) -> WindowInfo {
        WindowInfo {
            hwnd: 1,
            title: title.into(),
            exe: exe.into(),
            exe_path: path.into(),
            pid: 1,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            fullscreen,
        }
    }

    #[test]
    fn steam_games_are_named_after_their_folder() {
        let w = win(
            "game.exe",
            r"C:\Program Files (x86)\Steam\steamapps\common\Half-Life 2\hl2.exe",
            "HL2",
            false,
        );
        assert_eq!(game_name(&w), "Half-Life 2");
        assert!(looks_like_game(&w));
    }

    #[test]
    fn browsers_and_shell_are_not_games_even_fullscreen() {
        assert!(!looks_like_game(&win(
            "chrome.exe",
            r"C:\x\chrome.exe",
            "YouTube",
            true
        )));
        assert!(!looks_like_game(&win("explorer.exe", "", "", true)));
    }

    #[test]
    fn unknown_fullscreen_apps_count_as_games() {
        assert!(looks_like_game(&win(
            "indie.exe",
            r"D:\Stuff\indie.exe",
            "Indie Game",
            true
        )));
        assert!(!looks_like_game(&win(
            "notepad.exe",
            r"C:\Windows\notepad.exe",
            "Untitled",
            false
        )));
    }

    #[test]
    fn falls_back_to_title_then_exe() {
        assert_eq!(
            game_name(&win("a.exe", r"D:\a.exe", "My Title", false)),
            "My Title"
        );
        assert_eq!(game_name(&win("a.exe", r"D:\a.exe", "", false)), "a");
    }
}
