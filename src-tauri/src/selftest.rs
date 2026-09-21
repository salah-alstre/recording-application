//! Headless end-to-end check (`rimlight.exe --selftest`): records, screenshots and saves a replay,
//! then validates the produced files with ffprobe. The report is written to
//! `%APPDATA%\Rimlight\selftest.json` and the process exits with code 0 (pass) or 1 (fail).
use crate::encoder;
use crate::engine::Engine;
use crate::state_machine::RecState;
use crate::system::stats::Sampler;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn wait_state(e: &Engine, want: RecState, secs: u64) -> bool {
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(secs) {
        if e.state() == want {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

fn media(path: &str) -> Value {
    match encoder::probe_media(std::path::Path::new(path)) {
        Some(i) => {
            json!({ "path": path, "durationMs": i.duration_ms, "width": i.width, "height": i.height, "fps": i.fps, "codec": i.codec, "audioTracks": i.audio_tracks, "sizeBytes": i.size_bytes })
        }
        None => json!({ "path": path, "error": "unreadable" }),
    }
}

pub fn run(e: &Arc<Engine>) {
    let out_dir = std::env::temp_dir().join(format!(
        "rimlight-selftest-{}",
        uuid::Uuid::new_v4().simple()
    ));
    let mut checks: Vec<(String, bool, String)> = vec![];
    let mut report = json!({});
    let mut check = |name: &str, ok: bool, detail: String| {
        eprintln!(
            "[selftest] {} {name} {detail}",
            if ok { "PASS" } else { "FAIL" }
        );
        checks.push((name.into(), ok, detail));
    };

    let hw = e.hardware.read().clone();
    check("ffmpeg present", encoder::ffmpeg_available(), String::new());
    check(
        "encoders probed",
        hw.as_ref()
            .map(|h| h.encoders.iter().any(|x| x.available))
            .unwrap_or(false),
        format!(
            "{:?}",
            hw.as_ref().map(|h| h
                .encoders
                .iter()
                .filter(|x| x.available)
                .map(|x| x.id.clone())
                .collect::<Vec<_>>())
        ),
    );

    let original_settings = e.settings.read().clone();
    let original_profile = e.db.get_profile(&original_settings.active_profile_id);
    let mut s = original_settings.clone();
    s.storage.recordings_dir = out_dir.to_string_lossy().into_owned();
    s.capture.mode = "display".into();
    s.audio.system_enabled = true;
    s.audio.mic_enabled = false;
    s.replay.duration_secs = 10;
    if let Ok(mut p) = e.db.get_profile(&s.active_profile_id).ok_or(()) {
        p.quality.height = 1080;
        p.quality.fps = 60;
        p.quality.bitrate_kbps = 12_000;
        let _ = e.db.save_profile(&p);
    }
    let _ = e.apply_settings(s);

    let mut sampler = Sampler::new();
    let _ = sampler.sample(&[]);

    // Instant Replay first, so the buffer has content when we save it.
    match e.start_replay() {
        Ok(()) => check("replay starts", true, String::new()),
        Err(err) => check("replay starts", false, err.message),
    }
    std::thread::sleep(Duration::from_secs(4));

    match e.start_recording() {
        Ok(()) => check(
            "recording starts",
            e.state() == RecState::Recording,
            format!("{:?}", e.state()),
        ),
        Err(err) => check("recording starts", false, err.message),
    }
    std::thread::sleep(Duration::from_secs(3));
    let mid = e.snapshot_stats(&mut sampler, &mut (0, 0), 3.0);
    report["statsDuringRecording"] = serde_json::to_value(&mid).unwrap_or_default();
    check("marker added", e.add_marker().is_ok(), String::new());
    match crate::screenshot::take(e, Some("display")) {
        Ok(p) => {
            let ok = image::open(&p).map(|i| i.width() > 100).unwrap_or(false);
            check("screenshot decodes", ok, p);
        }
        Err(err) => check("screenshot decodes", false, err.message),
    }
    std::thread::sleep(Duration::from_secs(2));
    check(
        "replay save accepted",
        e.save_replay().is_ok(),
        String::new(),
    );
    std::thread::sleep(Duration::from_secs(3));
    check(
        "pause works",
        e.pause_recording().is_ok() && e.state() == RecState::Paused,
        String::new(),
    );
    std::thread::sleep(Duration::from_secs(1));
    check(
        "resume works",
        e.resume_recording().is_ok() && e.state() == RecState::Recording,
        String::new(),
    );
    std::thread::sleep(Duration::from_secs(1));
    check("recording stops", e.stop_recording().is_ok(), String::new());
    check(
        "returns to idle",
        wait_state(e, RecState::Idle, 60),
        format!("{:?}", e.state()),
    );
    let _ = e.stop_replay();

    let clips = e.db.all_clips().unwrap_or_default();
    let rec = clips.iter().find(|c| c.kind == "recording");
    let rep = clips.iter().find(|c| c.kind == "replay");
    let shot = clips.iter().find(|c| c.kind == "screenshot");
    if let Some(r) = rec {
        let m = media(&r.path);
        report["recording"] = m.clone();
        let dur = m["durationMs"].as_i64().unwrap_or(0);
        // 3 + 1(shot) + 2 + 3 recorded, 1 s paused, 1 more → ~10 s of footage, pause excluded
        check(
            "recording duration plausible",
            (7_000..=13_000).contains(&dur),
            format!("{dur} ms"),
        );
        check(
            "recording has 3 audio tracks",
            m["audioTracks"].as_u64() == Some(3),
            format!("{}", m["audioTracks"]),
        );
        check(
            "recording is 60 fps",
            (m["fps"].as_f64().unwrap_or(0.0) - 60.0).abs() < 2.0,
            format!("{}", m["fps"]),
        );
        check(
            "recording has a thumbnail",
            std::path::Path::new(&r.thumb_path).exists(),
            r.thumb_path.clone(),
        );
        check(
            "markers promoted to clip",
            e.db.list_markers(&r.id)
                .map(|m| m.len() == 1)
                .unwrap_or(false),
            String::new(),
        );
    } else {
        check("recording in library", false, "missing".into());
    }
    if let Some(r) = rep {
        let m = media(&r.path);
        report["replay"] = m.clone();
        let dur = m["durationMs"].as_i64().unwrap_or(0);
        check(
            "replay duration plausible",
            (5_000..=14_000).contains(&dur),
            format!("{dur} ms"),
        );
    } else {
        check("replay in library", false, "missing".into());
    }
    check("screenshot in library", shot.is_some(), String::new());

    // leave no trace: the test records the real screen, so remove every artefact and restore settings
    for c in &clips {
        let _ = std::fs::remove_file(&c.path);
        if c.kind != "screenshot" && !c.thumb_path.is_empty() {
            let _ = std::fs::remove_file(&c.thumb_path);
        }
        let _ = e.db.delete_clip(&c.id);
    }
    let _ = std::fs::remove_dir_all(&out_dir);
    if let Some(p) = original_profile {
        let _ = e.db.save_profile(&p);
    }
    let _ = e.apply_settings(original_settings);

    let failed = checks.iter().filter(|c| !c.1).count();
    report["checks"] = json!(checks
        .iter()
        .map(|(n, ok, d)| json!({ "name": n, "ok": ok, "detail": d }))
        .collect::<Vec<_>>());
    report["outputDir"] = json!(out_dir.to_string_lossy());
    report["failed"] = json!(failed);
    let _ = std::fs::write(
        crate::paths::data_dir().join("selftest.json"),
        serde_json::to_string_pretty(&report).unwrap_or_default(),
    );
    eprintln!("[selftest] done: {failed} failed");
    std::thread::sleep(Duration::from_millis(300));
    e.app.exit(if failed == 0 { 0 } else { 1 });
}
