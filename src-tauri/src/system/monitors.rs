//! Display enumeration (physical pixels; the process is per-monitor DPI aware).
use serde::Serialize;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};
use windows_capture::monitor::Monitor;

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    /// 1-based index, matching `windows_capture::monitor::Monitor::from_index`.
    pub index: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub x: i32,
    pub y: i32,
    pub primary: bool,
}

pub fn rect_of(m: &Monitor) -> Option<(RECT, bool)> {
    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let ok = unsafe { GetMonitorInfoW(HMONITOR(m.as_raw_hmonitor()), &mut mi).as_bool() };
    ok.then_some((mi.rcMonitor, mi.dwFlags & 1 != 0))
}

pub fn list() -> Vec<MonitorInfo> {
    let Ok(all) = Monitor::enumerate() else {
        return vec![];
    };
    all.iter()
        .enumerate()
        .filter_map(|(i, m)| {
            let (r, primary) = rect_of(m)?;
            Some(MonitorInfo {
                index: i as u32 + 1,
                name: m.name().unwrap_or_else(|_| format!("Display {}", i + 1)),
                width: (r.right - r.left).max(0) as u32,
                height: (r.bottom - r.top).max(0) as u32,
                refresh_hz: m.refresh_rate().unwrap_or(60),
                x: r.left,
                y: r.top,
                primary,
            })
        })
        .collect()
}

pub fn get(index: u32) -> Option<MonitorInfo> {
    list()
        .into_iter()
        .find(|m| m.index == index.max(1))
        .or_else(|| list().into_iter().next())
}

/// Monitor containing the given screen point.
pub fn at_point(x: i32, y: i32) -> Option<MonitorInfo> {
    list()
        .into_iter()
        .find(|m| x >= m.x && y >= m.y && x < m.x + m.width as i32 && y < m.y + m.height as i32)
}
