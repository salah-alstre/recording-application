//! Screen / window / region capture on top of the Windows Graphics Capture API, plus the
//! *regulator*: a dedicated thread that turns the irregular arrival of captured frames into a
//! constant-frame-rate stream and fans it out to any number of sinks (recorder, replay buffer).
pub mod frame;

use crate::error::{AppError, Result};
use crate::settings::Rect;
use crate::system::monitors;
use crossbeam_channel::{Sender, TrySendError};
use frame::{draw_disc, fill_rect, fit_into, FrameBuf, FramePool};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::monitor::Monitor;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    GraphicsCaptureItemType, MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};
use windows_capture::window::Window;

type BoxErr = Box<dyn std::error::Error + Send + Sync>;
type Ctl = CaptureControl<Worker, BoxErr>;

#[derive(Clone, Debug, PartialEq)]
pub enum TargetSpec {
    Display { index: u32 },
    Window { hwnd: isize },
    Region { index: u32, rect: Rect },
}

impl TargetSpec {
    /// Top-left of the captured area in virtual-screen coordinates.
    pub fn origin(&self) -> (i32, i32) {
        match self {
            TargetSpec::Display { index } => {
                monitors::get(*index).map(|m| (m.x, m.y)).unwrap_or((0, 0))
            }
            TargetSpec::Region { index, rect } => monitors::get(*index)
                .map(|m| (m.x + rect.x, m.y + rect.y))
                .unwrap_or((rect.x, rect.y)),
            TargetSpec::Window { hwnd } => crate::system::winenum::window_by_hwnd(*hwnd)
                .map(|w| (w.x, w.y))
                .unwrap_or((0, 0)),
        }
    }
    pub fn monitor_index(&self) -> u32 {
        match self {
            TargetSpec::Display { index } | TargetSpec::Region { index, .. } => *index,
            TargetSpec::Window { hwnd } => crate::system::winenum::window_by_hwnd(*hwnd)
                .and_then(|w| {
                    monitors::at_point(w.x + w.width as i32 / 2, w.y + w.height as i32 / 2)
                })
                .map(|m| m.index)
                .unwrap_or(1),
        }
    }
}

// ---- frame arrival statistics -------------------------------------------------------------------

#[derive(Default)]
pub struct FpsTracker {
    intervals_us: VecDeque<u32>,
    last: Option<Instant>,
}
impl FpsTracker {
    pub fn note(&mut self) {
        let now = Instant::now();
        if let Some(l) = self.last {
            self.intervals_us
                .push_back(now.duration_since(l).as_micros().min(2_000_000) as u32);
            if self.intervals_us.len() > 600 {
                self.intervals_us.pop_front();
            }
        }
        self.last = Some(now);
    }
    /// (average fps, 1 % low fps) over the recent window; `None` until enough frames arrived.
    pub fn stats(&self) -> Option<(f32, f32)> {
        if self.intervals_us.len() < 10 {
            return None;
        }
        // stale: no frame for > 1.5 s means the source is idle rather than slow
        if self
            .last
            .map(|l| l.elapsed() > Duration::from_millis(1500))
            .unwrap_or(true)
        {
            return Some((0.0, 0.0));
        }
        let mut v: Vec<u32> = self.intervals_us.iter().copied().collect();
        let mean = v.iter().map(|&x| x as f64).sum::<f64>() / v.len() as f64;
        v.sort_unstable();
        let p99 = v[((v.len() as f64 * 0.99) as usize).min(v.len() - 1)] as f64;
        Some(((1e6 / mean) as f32, (1e6 / p99.max(1.0)) as f32))
    }
}

// ---- capture worker ---------------------------------------------------------------------------

pub struct CaptureShared {
    pub latest: Mutex<Option<Arc<FrameBuf>>>,
    pool: Mutex<FramePool>,
    /// Pixels are only copied off the GPU while someone consumes them.
    pub want_pixels: AtomicBool,
    pub width: AtomicU32,
    pub height: AtomicU32,
    pub arrived: AtomicU64,
    pub closed: AtomicBool,
    pub fps: Mutex<FpsTracker>,
    pub last_error: Mutex<Option<String>>,
}

impl CaptureShared {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            latest: Mutex::new(None),
            pool: Mutex::new(FramePool::new()),
            want_pixels: AtomicBool::new(true),
            width: AtomicU32::new(0),
            height: AtomicU32::new(0),
            arrived: AtomicU64::new(0),
            closed: AtomicBool::new(false),
            fps: Mutex::new(FpsTracker::default()),
            last_error: Mutex::new(None),
        })
    }
}

struct Worker {
    shared: Arc<CaptureShared>,
    crop: Option<(u32, u32, u32, u32)>,
    scratch: Vec<u8>,
}

impl GraphicsCaptureApiHandler for Worker {
    type Flags = (Arc<CaptureShared>, Option<(u32, u32, u32, u32)>);
    type Error = BoxErr;

    fn new(ctx: Context<Self::Flags>) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            shared: ctx.flags.0,
            crop: ctx.flags.1,
            scratch: Vec::new(),
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _control: InternalCaptureControl,
    ) -> std::result::Result<(), Self::Error> {
        let sh = &self.shared;
        sh.arrived.fetch_add(1, Ordering::Relaxed);
        sh.fps.lock().note();
        let (fw, fh) = (frame.width(), frame.height());
        if !sh.want_pixels.load(Ordering::Relaxed) && sh.width.load(Ordering::Relaxed) != 0 {
            return Ok(());
        }
        let buf = match self.crop {
            Some((x0, y0, x1, y1)) => {
                let (x1, y1) = (x1.min(fw), y1.min(fh));
                if x0 >= x1 || y0 >= y1 {
                    return Ok(());
                }
                frame.buffer_crop(x0, y0, x1, y1)?
            }
            None => frame.buffer()?,
        };
        let (w, h) = (buf.width(), buf.height());
        let data = buf.as_nopadding_buffer(&mut self.scratch);
        let published = sh.pool.lock().produce(w, h, |v| v.extend_from_slice(data));
        sh.width.store(w, Ordering::Relaxed);
        sh.height.store(h, Ordering::Relaxed);
        *sh.latest.lock() = Some(published);
        Ok(())
    }

    fn on_closed(&mut self) -> std::result::Result<(), Self::Error> {
        self.shared.closed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

type Flags = (Arc<CaptureShared>, Option<(u32, u32, u32, u32)>);

fn capture_settings<T: TryInto<GraphicsCaptureItemType>>(
    item: T,
    cursor: CursorCaptureSettings,
    shared: Arc<CaptureShared>,
    crop: Option<(u32, u32, u32, u32)>,
) -> Settings<Flags, T> {
    Settings::new(
        item,
        cursor,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8,
        (shared, crop),
    )
}

fn start_worker(spec: &TargetSpec, cursor: bool, shared: Arc<CaptureShared>) -> Result<Ctl> {
    let cursor_set = if cursor {
        CursorCaptureSettings::WithCursor
    } else {
        CursorCaptureSettings::WithoutCursor
    };
    let err = |e: &dyn std::fmt::Display| {
        AppError::new(
            "capture_start",
            format!("Screen capture could not be started: {e}"),
        )
        .hint("Make sure the selected display or window still exists, then try again.")
    };
    match spec {
        TargetSpec::Display { index } => {
            let m = Monitor::from_index(*index as usize).map_err(|e| err(&e))?;
            Worker::start_free_threaded(capture_settings(m, cursor_set, shared.clone(), None))
                .map_err(|e| err(&e))
        }
        TargetSpec::Region { index, rect } => {
            let mi = monitors::get(*index).ok_or_else(|| {
                AppError::new("display_missing", "The selected display is not connected.")
            })?;
            let m = Monitor::from_index(mi.index as usize).map_err(|e| err(&e))?;
            let x0 = rect.x.max(0) as u32;
            let y0 = rect.y.max(0) as u32;
            let x1 = (x0 + rect.w).min(mi.width);
            let y1 = (y0 + rect.h).min(mi.height);
            if x1 <= x0 + 8 || y1 <= y0 + 8 {
                return Err(AppError::new(
                    "region_invalid",
                    "The capture region is empty or outside the display.",
                )
                .hint("Select the region again."));
            }
            Worker::start_free_threaded(capture_settings(
                m,
                cursor_set,
                shared.clone(),
                Some((x0, y0, x1, y1)),
            ))
            .map_err(|e| err(&e))
        }
        TargetSpec::Window { hwnd } => {
            let w = Window::from_raw_hwnd(*hwnd as *mut _);
            if !w.is_valid() {
                return Err(AppError::new(
                    "window_missing",
                    "The selected window is no longer available.",
                )
                .hint("Pick the window again or switch the capture mode."));
            }
            Worker::start_free_threaded(capture_settings(w, cursor_set, shared.clone(), None))
                .map_err(|e| err(&e))
        }
    }
}

/// Captures a single frame (used for screenshots when no session is running).
pub fn grab_once(spec: &TargetSpec, cursor: bool) -> Result<Arc<FrameBuf>> {
    let shared = CaptureShared::new();
    let ctl = start_worker(spec, cursor, shared.clone())?;
    let t0 = Instant::now();
    let mut out = None;
    while t0.elapsed() < Duration::from_secs(3) {
        if let Some(f) = shared.latest.lock().clone() {
            out = Some(f);
            break;
        }
        if ctl.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    let _ = ctl.stop();
    out.ok_or_else(|| {
        AppError::new(
            "capture_timeout",
            "No frame arrived from the capture source.",
        )
        .hint("The window may be minimized or protected from capture.")
    })
}

pub fn latest_from(shared: &CaptureShared) -> Option<Arc<FrameBuf>> {
    shared.latest.lock().clone()
}

// ---- session + regulator ------------------------------------------------------------------------

pub struct SinkPort {
    pub id: u64,
    pub tx: Sender<Arc<FrameBuf>>,
    pub paused: Arc<AtomicBool>,
    pub dropped: Arc<AtomicU64>,
}

#[derive(Default)]
pub struct Effects {
    /// Rectangles (x, y, w, h) in *capture* coordinates that must be painted over.
    pub privacy_rects: Mutex<Vec<(i32, i32, i32, i32)>>,
    pub highlight_cursor: AtomicBool,
    pub show_clicks: AtomicBool,
    pub origin_x: AtomicI32,
    pub origin_y: AtomicI32,
}

#[derive(Default)]
pub struct RegulatorStats {
    pub ticks: AtomicU64,
    /// Ticks that were skipped because the regulator itself fell behind.
    pub skipped: AtomicU64,
}

pub struct Session {
    pub fps: u32,
    pub canvas: (u32, u32),
    pub spec: Mutex<TargetSpec>,
    pub shared: Arc<CaptureShared>,
    pub fx: Arc<Effects>,
    pub stats: Arc<RegulatorStats>,
    cursor: bool,
    control: Mutex<Option<Ctl>>,
    sinks: Arc<Mutex<Vec<SinkPort>>>,
    pixel_holds: AtomicUsize,
    stop: Arc<AtomicBool>,
    regulator: Mutex<Option<JoinHandle<()>>>,
}

fn even(v: u32) -> u32 {
    (v & !1).max(2)
}

impl Session {
    pub fn start(
        spec: TargetSpec,
        fps: u32,
        cursor: bool,
        highlight: bool,
        clicks: bool,
    ) -> Result<Arc<Session>> {
        let shared = CaptureShared::new();
        let ctl = start_worker(&spec, cursor, shared.clone())?;
        // wait for the first frame so the canvas size is known
        let t0 = Instant::now();
        while shared.width.load(Ordering::Relaxed) == 0 {
            if t0.elapsed() > Duration::from_secs(4) || ctl.is_finished() {
                let _ = ctl.stop();
                return Err(AppError::new(
                    "capture_timeout",
                    "The capture source did not deliver any frames.",
                )
                .hint(
                    "Restore the window if it is minimized, or choose Display capture instead.",
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let canvas = (
            even(shared.width.load(Ordering::Relaxed)),
            even(shared.height.load(Ordering::Relaxed)),
        );
        let fx = Arc::new(Effects::default());
        fx.highlight_cursor.store(highlight, Ordering::Relaxed);
        fx.show_clicks.store(clicks, Ordering::Relaxed);
        let (ox, oy) = spec.origin();
        fx.origin_x.store(ox, Ordering::Relaxed);
        fx.origin_y.store(oy, Ordering::Relaxed);
        let s = Arc::new(Session {
            fps: fps.clamp(1, 240),
            canvas,
            spec: Mutex::new(spec),
            shared: shared.clone(),
            fx: fx.clone(),
            stats: Arc::new(RegulatorStats::default()),
            cursor,
            control: Mutex::new(Some(ctl)),
            sinks: Arc::new(Mutex::new(vec![])),
            pixel_holds: AtomicUsize::new(0),
            stop: Arc::new(AtomicBool::new(false)),
            regulator: Mutex::new(None),
        });
        s.refresh_want_pixels();
        let (sinks, stop, stats) = (s.sinks.clone(), s.stop.clone(), s.stats.clone());
        let fps = s.fps;
        let h = std::thread::Builder::new()
            .name("rimlight-regulator".into())
            .spawn(move || regulate(shared, sinks, fps, canvas, fx, stop, stats))
            .map_err(|e| AppError::internal(e))?;
        *s.regulator.lock() = Some(h);
        Ok(s)
    }

    fn refresh_want_pixels(&self) {
        let want = !self.sinks.lock().is_empty() || self.pixel_holds.load(Ordering::SeqCst) > 0;
        self.shared.want_pixels.store(want, Ordering::SeqCst);
    }

    pub fn add_sink(&self, port: SinkPort) {
        self.sinks.lock().push(port);
        self.refresh_want_pixels();
    }
    pub fn remove_sink(&self, id: u64) {
        self.sinks.lock().retain(|p| p.id != id);
        self.refresh_want_pixels();
    }
    pub fn sink_count(&self) -> usize {
        self.sinks.lock().len()
    }

    /// Latest raw frame; keeps pixel readback enabled while the returned guard lives (used by screenshots).
    pub fn latest_frame(&self) -> Option<Arc<FrameBuf>> {
        latest_from(&self.shared)
    }

    /// Switches the capture source without interrupting attached sinks (frames are fitted to the canvas).
    pub fn retarget(&self, spec: TargetSpec, force: bool) -> Result<()> {
        if !force && *self.spec.lock() == spec {
            return Ok(());
        }
        let new = start_worker(&spec, self.cursor, self.shared.clone())?;
        if let Some(old) = self.control.lock().replace(new) {
            let _ = old.stop();
        }
        self.shared.closed.store(false, Ordering::SeqCst);
        let (ox, oy) = spec.origin();
        self.fx.origin_x.store(ox, Ordering::Relaxed);
        self.fx.origin_y.store(oy, Ordering::Relaxed);
        *self.spec.lock() = spec;
        Ok(())
    }

    /// True when the captured window was closed or the capture thread died.
    pub fn source_gone(&self) -> bool {
        self.shared.closed.load(Ordering::SeqCst)
            || self
                .control
                .lock()
                .as_ref()
                .map(|c| c.is_finished())
                .unwrap_or(true)
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(h) = self.regulator.lock().take() {
            let _ = h.join();
        }
        if let Some(c) = self.control.lock().take() {
            let _ = c.stop();
        }
        self.sinks.lock().clear();
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.stop();
    }
}

fn sleep_until(t: Instant) {
    let now = Instant::now();
    if t <= now {
        return;
    }
    let d = t - now;
    if d > Duration::from_micros(2000) {
        std::thread::sleep(d - Duration::from_micros(1200));
    }
    while Instant::now() < t {
        std::hint::spin_loop();
    }
}

struct ClickRing {
    x: i32,
    y: i32,
    start: Instant,
    right: bool,
}

fn regulate(
    shared: Arc<CaptureShared>,
    sinks: Arc<Mutex<Vec<SinkPort>>>,
    fps: u32,
    canvas: (u32, u32),
    fx: Arc<Effects>,
    stop: Arc<AtomicBool>,
    stats: Arc<RegulatorStats>,
) {
    use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};
    unsafe {
        timeBeginPeriod(1);
    }
    let mut pool = FramePool::new();
    let start = Instant::now();
    let mut sent: u64 = 0;
    let mut last_src: Option<Arc<FrameBuf>> = None;
    let mut last_out: Option<Arc<FrameBuf>> = None;
    let mut clicks: Vec<ClickRing> = vec![];
    let mut prev_down = [false, false];

    while !stop.load(Ordering::Relaxed) {
        let expected = (start.elapsed().as_secs_f64() * fps as f64) as u64;
        if expected > sent + (fps as u64 / 2).max(2) {
            stats
                .skipped
                .fetch_add(expected - sent - 1, Ordering::Relaxed);
            sent = expected - 1;
        }
        if expected > sent {
            let n = expected - sent;
            sent = expected;
            stats.ticks.fetch_add(n, Ordering::Relaxed);
            if !sinks.lock().is_empty() {
                if let Some(src) = shared.latest.lock().clone() {
                    let rects = fx.privacy_rects.lock().clone();
                    let dynamic = fx.highlight_cursor.load(Ordering::Relaxed)
                        || fx.show_clicks.load(Ordering::Relaxed);
                    let same = last_src
                        .as_ref()
                        .map(|l| Arc::ptr_eq(l, &src))
                        .unwrap_or(false);
                    let needs_copy =
                        src.w != canvas.0 || src.h != canvas.1 || !rects.is_empty() || dynamic;
                    if !(same && !dynamic && last_out.is_some()) {
                        let out = if !needs_copy {
                            src.clone()
                        } else {
                            let (ox, oy) = (
                                fx.origin_x.load(Ordering::Relaxed),
                                fx.origin_y.load(Ordering::Relaxed),
                            );
                            let (cw, ch) = canvas;
                            pool.produce(cw, ch, |buf| {
                                if src.w != cw || src.h != ch {
                                    fit_into(&src, cw, ch, buf);
                                } else {
                                    buf.extend_from_slice(&src.data);
                                }
                                for &(x, y, w, h) in &rects {
                                    fill_rect(buf, cw, ch, x, y, w, h, [16, 16, 20, 255]);
                                }
                                draw_cursor_effects(
                                    buf,
                                    cw,
                                    ch,
                                    ox,
                                    oy,
                                    &fx,
                                    &mut clicks,
                                    &mut prev_down,
                                );
                            })
                        };
                        last_src = Some(src);
                        last_out = Some(out);
                    }
                    if let Some(frame) = &last_out {
                        let mut g = sinks.lock();
                        let mut dead: Vec<u64> = vec![];
                        for _ in 0..n.min(4) {
                            for p in g.iter() {
                                if p.paused.load(Ordering::Relaxed) {
                                    continue;
                                }
                                match p.tx.try_send(frame.clone()) {
                                    Ok(()) => {}
                                    Err(TrySendError::Full(_)) => {
                                        p.dropped.fetch_add(1, Ordering::Relaxed);
                                    }
                                    Err(TrySendError::Disconnected(_)) => dead.push(p.id),
                                }
                            }
                        }
                        if !dead.is_empty() {
                            g.retain(|p| !dead.contains(&p.id));
                        }
                    }
                }
            }
        }
        sleep_until(start + Duration::from_secs_f64((sent + 1) as f64 / fps as f64));
    }
    unsafe {
        timeEndPeriod(1);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_cursor_effects(
    buf: &mut [u8],
    w: u32,
    h: u32,
    ox: i32,
    oy: i32,
    fx: &Effects,
    clicks: &mut Vec<ClickRing>,
    prev_down: &mut [bool; 2],
) {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON};
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
    let mut p = POINT::default();
    if unsafe { GetCursorPos(&mut p) }.is_err() {
        return;
    }
    let (cx, cy) = (p.x - ox, p.y - oy);
    if fx.highlight_cursor.load(Ordering::Relaxed) {
        draw_disc(buf, w, h, cx, cy, 34, 0, [70, 210, 255], 0.28);
    }
    if fx.show_clicks.load(Ordering::Relaxed) {
        let down = unsafe {
            [
                GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000 != 0,
                GetAsyncKeyState(VK_RBUTTON.0 as i32) as u16 & 0x8000 != 0,
            ]
        };
        for i in 0..2 {
            if down[i] && !prev_down[i] {
                clicks.push(ClickRing {
                    x: cx,
                    y: cy,
                    start: Instant::now(),
                    right: i == 1,
                });
            }
            prev_down[i] = down[i];
        }
        clicks.retain(|c| c.start.elapsed() < Duration::from_millis(450));
        for c in clicks.iter() {
            let t = c.start.elapsed().as_secs_f32() / 0.45;
            let radius = (14.0 + t * 30.0) as i32;
            let colour = if c.right {
                [80, 140, 255]
            } else {
                [90, 220, 140]
            };
            draw_disc(buf, w, h, c.x, c.y, radius, 5, colour, (1.0 - t) * 0.9);
        }
    } else {
        prev_down.fill(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_tracker_needs_samples_and_reports_average() {
        let mut t = FpsTracker::default();
        assert!(t.stats().is_none());
        for _ in 0..40 {
            t.note();
            std::thread::sleep(Duration::from_millis(10));
        }
        let (avg, low) = t.stats().expect("stats");
        assert!(avg > 40.0 && avg < 130.0, "avg {avg}");
        assert!(low <= avg + 0.5);
    }

    #[test]
    fn even_rounds_down() {
        assert_eq!(even(1921), 1920);
        assert_eq!(even(1), 2);
    }
}
