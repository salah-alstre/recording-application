//! Pure construction of FFmpeg command lines. Kept free of I/O so it can be unit-tested.
//!
//! Input layout of every encode job:
//!   input 0      raw BGRA frames on stdin (constant frame-rate, produced by the capture regulator)
//!   inputs 1..N  raw f32 stereo PCM over loopback TCP, one per audio track
//!   input N+1    optional DirectShow webcam
use super::EncoderInfo;
use crate::quality::Quality;
use crate::settings::CameraSettings;
use std::path::PathBuf;

pub struct AudioTrack {
    pub port: u16,
    pub title: String,
}

pub enum Output {
    /// A single file. Recordings are always written as Matroska first (crash-safe); MP4 is produced
    /// by a lossless remux when the recording is finalised.
    File { path: PathBuf },
    /// Rolling ~2 s MPEG-TS segments for Instant Replay.
    Segments { dir: PathBuf, seg_secs: u32 },
}

pub struct Job<'a> {
    pub src_w: u32,
    pub src_h: u32,
    pub fps: u32,
    pub quality: &'a Quality,
    pub encoder: &'a str,
    pub tracks: Vec<AudioTrack>,
    pub output: Output,
    pub camera: Option<&'a CameraSettings>,
}

fn s(v: &str) -> String {
    v.to_string()
}

/// Ordered fallback list of ffmpeg encoders for the user's request.
/// `vendor`: auto | nvenc | amf | qsv | cpu. Hardware is preferred; the CPU encoder of the same
/// codec is always the final fallback so recording never fails outright.
pub fn resolve_encoders(vendor: &str, codec: &str, available: &[EncoderInfo]) -> Vec<String> {
    let ok = |v: &str| {
        available
            .iter()
            .find(|e| e.available && e.vendor == v && e.codec == codec)
            .map(|e| e.id.clone())
    };
    let order: Vec<&str> = match vendor {
        "nvenc" => vec!["nvenc", "amf", "qsv", "cpu"],
        "amf" => vec!["amf", "nvenc", "qsv", "cpu"],
        "qsv" => vec!["qsv", "nvenc", "amf", "cpu"],
        "cpu" => vec!["cpu"],
        _ => vec!["nvenc", "amf", "qsv", "cpu"],
    };
    let mut out: Vec<String> = order.iter().filter_map(|v| ok(v)).collect();
    if out.is_empty() {
        out.push(
            match codec {
                "hevc" => "libx265",
                "av1" => "libsvtav1",
                _ => "libx264",
            }
            .to_string(),
        );
    }
    out
}

fn encoder_args(enc: &str, q: &Quality, fps: u32, forced_keyint: u32) -> Vec<String> {
    let mut a: Vec<String> = vec![s("-c:v"), s(enc)];
    let kbps = q.bitrate_kbps.max(500);
    let br = |a: &mut Vec<String>, vbr: bool| {
        a.extend([s("-b:v"), format!("{kbps}k")]);
        let max = if vbr { kbps * 3 / 2 } else { kbps };
        a.extend([
            s("-maxrate"),
            format!("{max}k"),
            s("-bufsize"),
            format!("{}k", kbps * 2),
        ]);
    };
    let speed = q.speed.as_str();
    let cqp = q.rate_control == "cqp";
    let vbr = q.rate_control == "vbr";
    if enc.ends_with("_nvenc") {
        let p = match speed {
            "fast" => "p2",
            "quality" => "p6",
            _ => "p4",
        };
        a.extend([s("-preset"), s(p), s("-tune"), s("hq")]);
        if cqp {
            a.extend([s("-rc"), s("constqp"), s("-qp"), q.cq.to_string()]);
        } else {
            a.extend([s("-rc"), s(if vbr { "vbr" } else { "cbr" })]);
            br(&mut a, vbr);
        }
        a.extend([
            s("-g"),
            forced_keyint.to_string(),
            s("-bf"),
            s("2"),
            s("-forced-idr"),
            s("1"),
        ]);
    } else if enc.ends_with("_amf") {
        let qual = match speed {
            "fast" => "speed",
            "quality" => "quality",
            _ => "balanced",
        };
        a.extend([s("-quality"), s(qual)]);
        if cqp {
            a.extend([
                s("-rc"),
                s("cqp"),
                s("-qp_i"),
                q.cq.to_string(),
                s("-qp_p"),
                q.cq.to_string(),
            ]);
        } else {
            a.extend([s("-rc"), s(if vbr { "vbr_peak" } else { "cbr" })]);
            br(&mut a, vbr);
        }
        a.extend([s("-g"), forced_keyint.to_string()]);
    } else if enc.ends_with("_qsv") {
        let p = match speed {
            "fast" => "veryfast",
            "quality" => "slow",
            _ => "medium",
        };
        a.extend([s("-preset"), s(p)]);
        if cqp {
            a.extend([s("-global_quality"), q.cq.to_string()]);
        } else {
            br(&mut a, vbr);
        }
        a.extend([s("-g"), forced_keyint.to_string()]);
    } else if enc == "libx264" {
        let p = match speed {
            "fast" => "superfast",
            "quality" => "fast",
            _ => "veryfast",
        };
        a.extend([s("-preset"), s(p)]);
        if cqp {
            a.extend([s("-crf"), q.cq.to_string()]);
        } else {
            br(&mut a, vbr);
        }
        a.extend([s("-g"), forced_keyint.to_string()]);
    } else if enc == "libx265" {
        let p = match speed {
            "quality" => "veryfast",
            "balanced" => "superfast",
            _ => "ultrafast",
        };
        a.extend([s("-preset"), s(p), s("-x265-params"), s("log-level=error")]);
        if cqp {
            a.extend([s("-crf"), q.cq.to_string()]);
        } else {
            br(&mut a, vbr);
        }
        a.extend([s("-g"), forced_keyint.to_string()]);
    } else {
        // libsvtav1
        let p = match speed {
            "quality" => "8",
            "balanced" => "10",
            _ => "12",
        };
        a.extend([s("-preset"), s(p)]);
        if cqp {
            a.extend([s("-crf"), (q.cq + 8).min(63).to_string()]);
        } else {
            br(&mut a, vbr);
        }
        a.extend([s("-g"), forced_keyint.to_string()]);
    }
    if q.codec == "hevc" {
        a.extend([s("-tag:v"), s("hvc1")]);
    }
    let _ = fps;
    a
}

fn camera_mask(shape: &str, w: u32, h: u32) -> String {
    match shape {
        "circle" => format!("a='255*lte(hypot(X-{0}/2,Y-{1}/2),{0}/2)'", w, h),
        "rounded" => {
            let r = (w.min(h) as f64 * 0.18).round() as u32;
            format!(
                "a='255*gt(lte(abs(X-{w}/2),{w}/2-{r})+lte(abs(Y-{h}/2),{h}/2-{r})+lte(hypot(abs(X-{w}/2)-({w}/2-{r}),abs(Y-{h}/2)-({h}/2-{r})),{r}),0)'"
            )
        }
        _ => "a='255'".to_string(),
    }
}

/// Filter graph producing the `[v]` label. Scales to the output size, optionally composites the webcam.
fn filter_graph(job: &Job, ow: u32, oh: u32, cam_index: Option<usize>) -> String {
    let base = format!("[0:v]scale={ow}:{oh}:flags=fast_bilinear:out_color_matrix=bt709:out_range=tv,format=yuv420p[base]");
    let (Some(cam), Some(ci)) = (job.camera, cam_index) else {
        return format!("{base};[base]format=nv12[v]");
    };
    let side = ((oh as f64 * cam.size_pct as f64 / 100.0).round() as u32).max(32) & !1;
    let m = cam.margin_px;
    let (x, y) = match cam.corner.as_str() {
        "top-left" => (m, m),
        "top-right" => (ow.saturating_sub(side + m), m),
        "bottom-left" => (m, oh.saturating_sub(side + m)),
        _ => (ow.saturating_sub(side + m), oh.saturating_sub(side + m)),
    };
    let flip = if cam.mirror { "hflip," } else { "" };
    let cam_chain = format!(
        "[{ci}:v]setpts=PTS-STARTPTS,scale=-2:{side},crop={side}:{side},{flip}format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':{mask}[cam]",
        mask = camera_mask(&cam.shape, side, side)
    );
    format!("{base};{cam_chain};[base][cam]overlay={x}:{y}:eof_action=pass:repeatlast=1:format=auto,format=nv12[v]")
}

pub fn build(job: &Job) -> Vec<String> {
    let q = job.quality;
    let (ow, oh) = q.output_size(job.src_w, job.src_h);
    let fps = job.fps.max(1);
    let keyint = match job.output {
        Output::Segments { seg_secs, .. } => fps * seg_secs,
        Output::File { .. } => fps * 2,
    };
    let mut a: Vec<String> = [
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-y",
        "-progress",
        "pipe:1",
        "-nostats",
    ]
    .iter()
    .map(|x| x.to_string())
    .collect();

    // input 0: raw video on stdin
    a.extend(
        ["-f", "rawvideo", "-pixel_format", "bgra", "-video_size"]
            .iter()
            .map(|x| x.to_string()),
    );
    a.push(format!("{}x{}", job.src_w, job.src_h));
    a.extend([
        s("-framerate"),
        fps.to_string(),
        s("-thread_queue_size"),
        s("8"),
        s("-i"),
        s("pipe:0"),
    ]);

    // audio inputs
    for t in &job.tracks {
        a.extend(
            [
                "-f",
                "f32le",
                "-ar",
                "48000",
                "-ac",
                "2",
                "-thread_queue_size",
                "1024",
                "-probesize",
                "32",
                "-analyzeduration",
                "0",
                "-i",
            ]
            .iter()
            .map(|x| x.to_string()),
        );
        a.push(format!("tcp://127.0.0.1:{}", t.port));
    }

    // optional webcam
    let cam_index = job
        .camera
        .filter(|c| c.enabled && !c.device.is_empty())
        .map(|_| 1 + job.tracks.len());
    if let (Some(cam), Some(_)) = (job.camera, cam_index) {
        a.extend([
            s("-f"),
            s("dshow"),
            s("-use_wallclock_as_timestamps"),
            s("1"),
            s("-rtbufsize"),
            s("64M"),
            s("-video_size"),
            format!("{}x{}", cam.width, cam.height),
            s("-framerate"),
            cam.fps.to_string(),
            s("-i"),
            format!("video={}", cam.device),
        ]);
    }

    let graph = filter_graph(job, ow, oh, cam_index);
    a.extend([s("-filter_complex"), graph, s("-map"), s("[v]")]);
    for i in 0..job.tracks.len() {
        a.extend([s("-map"), format!("{}:a", i + 1)]);
    }

    a.extend(encoder_args(job.encoder, q, fps, keyint));
    // hardware encoders take NV12 (already produced by the filter graph); software ones planar 4:2:0
    let hw = job.encoder.ends_with("_nvenc")
        || job.encoder.ends_with("_amf")
        || job.encoder.ends_with("_qsv");
    a.extend([
        s("-r"),
        fps.to_string(),
        s("-pix_fmt"),
        s(if hw { "nv12" } else { "yuv420p" }),
    ]);
    a.extend(
        [
            "-colorspace",
            "bt709",
            "-color_primaries",
            "bt709",
            "-color_trc",
            "bt709",
            "-color_range",
            "tv",
        ]
        .iter()
        .map(|x| x.to_string()),
    );

    if !job.tracks.is_empty() {
        a.extend([
            s("-c:a"),
            s("aac"),
            s("-b:a"),
            format!("{}k", q.audio_bitrate_kbps.max(64)),
            s("-ar"),
            s("48000"),
        ]);
        for (i, t) in job.tracks.iter().enumerate() {
            a.extend([format!("-metadata:s:a:{i}"), format!("title={}", t.title)]);
        }
    }

    match &job.output {
        Output::File { path } => {
            a.extend([s("-f"), s("matroska"), s("-flush_packets"), s("1")]);
            a.push(path.to_string_lossy().into_owned());
        }
        Output::Segments { dir, seg_secs } => {
            a.extend([
                s("-force_key_frames"),
                format!("expr:gte(t,n_forced*{seg_secs})"),
            ]);
            a.extend([
                s("-f"),
                s("segment"),
                s("-segment_time"),
                seg_secs.to_string(),
                s("-segment_format"),
                s("mpegts"),
            ]);
            a.extend([
                s("-segment_list"),
                dir.join("list.csv").to_string_lossy().into_owned(),
                s("-segment_list_type"),
                s("csv"),
                s("-segment_list_flags"),
                s("+live"),
            ]);
            a.extend([s("-reset_timestamps"), s("0")]);
            a.push(dir.join("seg_%06d.ts").to_string_lossy().into_owned());
        }
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(id: &str, vendor: &str, codec: &str, ok: bool) -> EncoderInfo {
        EncoderInfo {
            id: id.into(),
            vendor: vendor.into(),
            codec: codec.into(),
            label: id.into(),
            available: ok,
            error: None,
        }
    }

    fn all() -> Vec<EncoderInfo> {
        vec![
            info("h264_nvenc", "nvenc", "h264", true),
            info("av1_nvenc", "nvenc", "av1", true),
            info("h264_amf", "amf", "h264", false),
            info("libx264", "cpu", "h264", true),
            info("libsvtav1", "cpu", "av1", true),
        ]
    }

    #[test]
    fn auto_prefers_hardware_and_keeps_cpu_as_fallback() {
        let r = resolve_encoders("auto", "h264", &all());
        assert_eq!(r, vec!["h264_nvenc", "libx264"]);
    }

    #[test]
    fn unavailable_hardware_falls_back_to_cpu_encoder_of_same_codec() {
        assert_eq!(
            resolve_encoders("amf", "h264", &all()),
            vec!["h264_nvenc", "libx264"]
        );
        assert_eq!(resolve_encoders("auto", "hevc", &all()), vec!["libx265"]);
        assert_eq!(resolve_encoders("cpu", "av1", &all()), vec!["libsvtav1"]);
    }

    fn job<'a>(
        q: &'a Quality,
        enc: &'a str,
        out: Output,
        cam: Option<&'a CameraSettings>,
    ) -> Job<'a> {
        Job {
            src_w: 2560,
            src_h: 1440,
            fps: 60,
            quality: q,
            encoder: enc,
            tracks: vec![
                AudioTrack {
                    port: 5001,
                    title: "Full mix".into(),
                },
                AudioTrack {
                    port: 5002,
                    title: "System".into(),
                },
            ],
            output: out,
            camera: cam,
        }
    }

    #[test]
    fn recording_command_has_expected_shape() {
        let q = Quality::from_preset("high");
        let a = build(&job(
            &q,
            "h264_nvenc",
            Output::File {
                path: "out.mkv".into(),
            },
            None,
        ));
        let joined = a.join(" ");
        assert!(joined.contains("-video_size 2560x1440"));
        assert!(joined.contains("scale=1920:1080"));
        assert!(joined.contains("-c:v h264_nvenc"));
        assert!(joined.contains("-b:v 20000k"));
        assert!(joined.contains("tcp://127.0.0.1:5001") && joined.contains("tcp://127.0.0.1:5002"));
        assert!(joined.contains("-map 1:a -map 2:a"));
        assert!(joined.contains("title=Full mix"));
        assert_eq!(a.last().unwrap(), "out.mkv");
        assert!(joined.contains("-pix_fmt nv12"));
    }

    #[test]
    fn replay_command_writes_segments() {
        let q = Quality::from_preset("medium");
        let a = build(&job(
            &q,
            "libx264",
            Output::Segments {
                dir: "C:/cache".into(),
                seg_secs: 2,
            },
            None,
        ));
        let joined = a.join(" ");
        assert!(joined.contains("-f segment") && joined.contains("-segment_time 2"));
        assert!(joined.contains("-g 120"));
        assert!(joined.contains("seg_%06d.ts"));
        assert!(joined.contains("-pix_fmt yuv420p"));
    }

    #[test]
    fn cqp_uses_quantiser_instead_of_bitrate() {
        let q = Quality {
            rate_control: "cqp".into(),
            cq: 20,
            ..Quality::default()
        };
        let joined = build(&job(
            &q,
            "h264_nvenc",
            Output::File {
                path: "o.mkv".into(),
            },
            None,
        ))
        .join(" ");
        assert!(joined.contains("-rc constqp -qp 20"));
        assert!(!joined.contains("-b:v"));
    }

    #[test]
    fn hevc_in_containers_gets_hvc1_tag() {
        let q = Quality {
            codec: "hevc".into(),
            ..Quality::default()
        };
        assert!(build(&job(
            &q,
            "hevc_nvenc",
            Output::File {
                path: "o.mkv".into()
            },
            None
        ))
        .join(" ")
        .contains("-tag:v hvc1"));
    }

    #[test]
    fn webcam_overlay_is_positioned_in_the_corner() {
        let cam = CameraSettings {
            enabled: true,
            device: "Cam".into(),
            ..CameraSettings::default()
        };
        let q = Quality::from_preset("high");
        let joined = build(&job(
            &q,
            "libx264",
            Output::File {
                path: "o.mkv".into(),
            },
            Some(&cam),
        ))
        .join(" ");
        assert!(joined.contains("-f dshow"));
        assert!(joined.contains("video=Cam"));
        assert!(joined.contains("overlay="));
        assert!(joined.contains("hypot")); // circle mask
                                           // 24% of 1080 = 259 -> 258 (even); bottom-right with 32px margin
        assert!(joined.contains("overlay=1630:790"), "{joined}");
    }
}
