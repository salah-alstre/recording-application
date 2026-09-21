//! A *sink* owns one FFmpeg encoder process and feeds it: video frames over stdin, one PCM audio
//! stream per track over loopback TCP. Recording and Instant Replay are each a sink attached to the
//! same capture session, so they encode independently and can start/stop independently.
use crate::audio::{AudioChunk, AudioEngine};
use crate::capture::frame::FrameBuf;
use crate::capture::{Session, SinkPort};
use crate::encoder::args::{self, AudioTrack, Job, Output};
use crate::encoder::{self};
use crate::quality::Quality;
use crate::settings::CameraSettings;
use crossbeam_channel::{bounded, Receiver};
use parking_lot::Mutex;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SinkKind {
    Recording,
    Replay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackSel {
    Mix,
    System,
    Mic,
}

impl TrackSel {
    pub fn title(self) -> &'static str {
        match self {
            TrackSel::Mix => "Full mix",
            TrackSel::System => "System audio",
            TrackSel::Mic => "Microphone",
        }
    }
    fn pick(self, c: &AudioChunk) -> &[f32] {
        match self {
            TrackSel::Mix => &c.mix,
            TrackSel::System => &c.system,
            TrackSel::Mic => &c.mic,
        }
    }
}

pub struct SinkSpec {
    pub kind: SinkKind,
    pub src: (u32, u32),
    pub fps: u32,
    pub quality: Quality,
    pub encoder: String,
    pub camera: Option<CameraSettings>,
    pub output: Output,
    pub tracks: Vec<TrackSel>,
}

/// Live encoder statistics parsed from FFmpeg's `-progress` stream.
#[derive(Default)]
pub struct Progress {
    pub frames: AtomicU64,
    pub fps_x100: AtomicU32,
    pub bitrate_kbps_x10: AtomicU64,
    pub total_size: AtomicU64,
    pub out_time_us: AtomicU64,
    pub speed_x100: AtomicU32,
    pub dropped_by_encoder: AtomicU64,
}

pub struct Sink {
    pub id: u64,
    pub kind: SinkKind,
    pub encoder: String,
    pub progress: Arc<Progress>,
    pub paused: Arc<AtomicBool>,
    pub dropped: Arc<AtomicU64>,
    pub started: Arc<AtomicBool>,
    pub started_at: Instant,
    child: Child,
    audio_sub: u64,
    audio: Arc<AudioEngine>,
    session: Arc<Session>,
    threads: Vec<JoinHandle<()>>,
    stderr_tail: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug)]
pub struct SinkError {
    pub message: String,
    pub stderr: String,
}

impl Sink {
    pub fn launch(
        spec: SinkSpec,
        audio: &Arc<AudioEngine>,
        session: &Arc<Session>,
    ) -> Result<Sink, SinkError> {
        let fail = |m: String, s: String| SinkError {
            message: m,
            stderr: s,
        };
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| {
            fail(
                format!("cannot open a local audio port: {e}"),
                String::new(),
            )
        })?;
        let port = listener.local_addr().map(|a| a.port()).unwrap_or(0);
        listener.set_nonblocking(true).ok();

        let tracks: Vec<AudioTrack> = spec
            .tracks
            .iter()
            .map(|t| AudioTrack {
                port,
                title: t.title().to_string(),
            })
            .collect();
        let cmd_args = args::build(&Job {
            src_w: spec.src.0,
            src_h: spec.src.1,
            fps: spec.fps,
            quality: &spec.quality,
            encoder: &spec.encoder,
            tracks,
            output: match &spec.output {
                Output::File { path } => Output::File { path: path.clone() },
                Output::Segments { dir, seg_secs } => Output::Segments {
                    dir: dir.clone(),
                    seg_secs: *seg_secs,
                },
            },
            camera: spec.camera.as_ref(),
        });
        tracing::info!(
            "starting {:?} encoder {} ({}x{} @ {} fps)",
            spec.kind,
            spec.encoder,
            spec.src.0,
            spec.src.1,
            spec.fps
        );
        tracing::debug!("ffmpeg {}", cmd_args.join(" "));

        let mut child = encoder::command(&encoder::ffmpeg())
            .args(&cmd_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| fail(format!("could not start ffmpeg: {e}"), String::new()))?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let progress = Arc::new(Progress::default());
        let stderr_tail = Arc::new(Mutex::new(Vec::<String>::new()));
        let started = Arc::new(AtomicBool::new(false));
        let paused = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicU64::new(0));
        let mut threads = vec![];

        if let Some(out) = stdout {
            let p = progress.clone();
            threads.push(std::thread::spawn(move || parse_progress(out, p)));
        }
        if let Some(err) = stderr {
            let tail = stderr_tail.clone();
            threads.push(std::thread::spawn(move || {
                for line in BufReader::new(err)
                    .lines()
                    .map_while(std::result::Result::ok)
                {
                    tracing::debug!("ffmpeg: {line}");
                    let mut t = tail.lock();
                    t.push(line);
                    if t.len() > 40 {
                        t.remove(0);
                    }
                }
            }));
        }

        // video writer
        let (vtx, vrx) = bounded::<Arc<FrameBuf>>(3);
        if let Some(mut stdin) = stdin {
            let started = started.clone();
            threads.push(std::thread::spawn(move || {
                while let Ok(f) = vrx.recv() {
                    if stdin.write_all(&f.data).is_err() {
                        break;
                    }
                    started.store(true, Ordering::SeqCst);
                }
            }));
        }

        // audio writer
        let (audio_sub, arx): (u64, Receiver<Arc<AudioChunk>>) = audio.subscribe();
        {
            let sels = spec.tracks.clone();
            let started = started.clone();
            let paused = paused.clone();
            threads.push(std::thread::spawn(move || {
                audio_writer(listener, sels, arx, started, paused)
            }));
        }

        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        session.add_sink(SinkPort {
            id,
            tx: vtx,
            paused: paused.clone(),
            dropped: dropped.clone(),
        });

        let mut sink = Sink {
            id,
            kind: spec.kind,
            encoder: spec.encoder.clone(),
            progress,
            paused,
            dropped,
            started,
            started_at: Instant::now(),
            child,
            audio_sub,
            audio: audio.clone(),
            session: session.clone(),
            threads,
            stderr_tail,
        };

        // wait until the encoder consumed its first frame, or died
        let t0 = Instant::now();
        loop {
            if sink.started.load(Ordering::SeqCst) {
                sink.started_at = Instant::now();
                sink.dropped.store(0, Ordering::SeqCst);
                return Ok(sink);
            }
            if let Ok(Some(status)) = sink.child.try_wait() {
                let err = sink.stderr_text();
                sink.teardown();
                return Err(fail(
                    format!("the encoder exited during start-up ({status})"),
                    err,
                ));
            }
            if t0.elapsed() > Duration::from_secs(10) {
                let err = sink.stderr_text();
                sink.teardown();
                return Err(fail(
                    "the encoder did not become ready within 10 seconds".into(),
                    err,
                ));
            }
            std::thread::sleep(Duration::from_millis(15));
        }
    }

    pub fn stderr_text(&self) -> String {
        self.stderr_tail.lock().join("\n")
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// True while the FFmpeg process is alive.
    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn detach(&self) {
        self.session.remove_sink(self.id);
        self.audio.unsubscribe(self.audio_sub);
    }

    fn teardown(&mut self) {
        self.detach();
        let _ = self.child.kill();
        let _ = self.child.wait();
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }

    /// Flushes and finalises the output. Returns the FFmpeg exit success and its stderr tail.
    pub fn finish(mut self) -> (bool, String) {
        self.detach();
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
        let t0 = Instant::now();
        let ok = loop {
            match self.child.try_wait() {
                Ok(Some(status)) => break status.success(),
                Ok(None) if t0.elapsed() < Duration::from_secs(60) => {
                    std::thread::sleep(Duration::from_millis(20))
                }
                _ => {
                    tracing::warn!("ffmpeg did not exit in time; killing it");
                    let _ = self.child.kill();
                    let _ = self.child.wait();
                    break false;
                }
            }
        };
        (ok, self.stderr_text())
    }
}

impl Drop for Sink {
    fn drop(&mut self) {
        // Reached only if a Sink is dropped without finish(): make sure nothing leaks.
        self.detach();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn audio_writer(
    listener: TcpListener,
    sels: Vec<TrackSel>,
    rx: Receiver<Arc<AudioChunk>>,
    started: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
) {
    // ffmpeg opens its audio inputs in order, so the n-th connection is the n-th track.
    let mut conns: Vec<TcpStream> = vec![];
    let t0 = Instant::now();
    while conns.len() < sels.len() {
        match listener.accept() {
            Ok((s, _)) => {
                s.set_nonblocking(false).ok();
                s.set_nodelay(true).ok();
                conns.push(s);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if t0.elapsed() > Duration::from_secs(20) {
                    tracing::warn!("ffmpeg never connected its audio inputs");
                    return;
                }
                std::thread::sleep(Duration::from_millis(5));
                // keep the subscription queue from growing while we wait
                while rx.len() > 100 {
                    let _ = rx.try_recv();
                }
            }
            Err(_) => return,
        }
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(4096);
    while let Ok(chunk) = rx.recv() {
        // audio only starts once video is flowing, so both streams share the same start instant
        if !started.load(Ordering::SeqCst) || paused.load(Ordering::Relaxed) {
            continue;
        }
        for (sel, conn) in sels.iter().zip(conns.iter_mut()) {
            bytes.clear();
            for f in sel.pick(&chunk) {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            if conn.write_all(&bytes).is_err() {
                return;
            }
        }
    }
    // dropping the connections signals EOF to ffmpeg
}

fn parse_progress(out: std::process::ChildStdout, p: Arc<Progress>) {
    for line in BufReader::new(out)
        .lines()
        .map_while(std::result::Result::ok)
    {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let v = v.trim();
        match k {
            "frame" => p.frames.store(v.parse().unwrap_or(0), Ordering::Relaxed),
            "fps" => p.fps_x100.store(
                (v.parse::<f32>().unwrap_or(0.0) * 100.0) as u32,
                Ordering::Relaxed,
            ),
            "bitrate" => {
                let n: f64 = v.trim_end_matches("kbits/s").trim().parse().unwrap_or(0.0);
                p.bitrate_kbps_x10
                    .store((n * 10.0) as u64, Ordering::Relaxed);
            }
            "total_size" => p
                .total_size
                .store(v.parse().unwrap_or(0), Ordering::Relaxed),
            "out_time_us" => p
                .out_time_us
                .store(v.parse().unwrap_or(0), Ordering::Relaxed),
            "speed" => p.speed_x100.store(
                (v.trim_end_matches('x').trim().parse::<f32>().unwrap_or(0.0) * 100.0) as u32,
                Ordering::Relaxed,
            ),
            "drop_frames" => p
                .dropped_by_encoder
                .store(v.parse().unwrap_or(0), Ordering::Relaxed),
            _ => {}
        }
    }
}

impl Progress {
    pub fn fps(&self) -> f32 {
        self.fps_x100.load(Ordering::Relaxed) as f32 / 100.0
    }
    pub fn bitrate_kbps(&self) -> f32 {
        self.bitrate_kbps_x10.load(Ordering::Relaxed) as f32 / 10.0
    }
    pub fn speed(&self) -> f32 {
        self.speed_x100.load(Ordering::Relaxed) as f32 / 100.0
    }
}
