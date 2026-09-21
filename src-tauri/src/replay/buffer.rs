//! Pure logic of the Instant Replay rolling buffer.
//!
//! The replay encoder writes ~2 s MPEG-TS segments to a cache folder and appends one CSV line
//! (`name,start,end`) per *finished* segment to a list file. Only encoded data ever touches the
//! disk, so a 3-minute buffer costs `bitrate × 180 s`, not raw-frame gigabytes. Saving a replay
//! concatenates the newest segments with `-c copy` (no re-encode), so it is near-instant.
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub name: String,
    pub start: f64,
    pub end: f64,
}

impl Segment {
    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }
}

/// Parses ffmpeg's `-segment_list_type csv` output.
pub fn parse_list(text: &str) -> Vec<Segment> {
    text.lines()
        .filter_map(|l| {
            let mut it = l.trim().split(',');
            let name = it.next()?.trim().to_string();
            let start = it.next()?.trim().parse().ok()?;
            let end = it.next()?.trim().parse().ok()?;
            if name.is_empty() {
                return None;
            }
            Some(Segment { name, start, end })
        })
        .collect()
}

/// Newest segments covering at least `want_secs` (or everything if the buffer is shorter),
/// returned in chronological order.
pub fn select_tail(segs: &[Segment], want_secs: f64) -> Vec<Segment> {
    let mut acc = 0.0;
    let mut picked: Vec<Segment> = vec![];
    for s in segs.iter().rev() {
        picked.push(s.clone());
        acc += s.duration();
        if acc >= want_secs {
            break;
        }
    }
    picked.reverse();
    picked
}

/// Segments that are older than the retention window and can be deleted from disk.
/// `keep_secs` should be the configured replay length plus a safety margin.
pub fn expired(segs: &[Segment], keep_secs: f64) -> Vec<&Segment> {
    let Some(newest_end) = segs.last().map(|s| s.end) else {
        return vec![];
    };
    segs.iter()
        .filter(|s| newest_end - s.end > keep_secs)
        .collect()
}

/// Content for ffmpeg's concat demuxer (`-f concat -safe 0`).
pub fn concat_manifest(dir: &Path, names: &[String]) -> String {
    let mut out = String::from("ffconcat version 1.0\n");
    for n in names {
        let full = dir
            .join(n)
            .to_string_lossy()
            .replace('\\', "/")
            .replace('\'', "'\\''");
        out.push_str(&format!("file '{full}'\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segs(n: usize) -> Vec<Segment> {
        (0..n)
            .map(|i| Segment {
                name: format!("seg_{i:06}.ts"),
                start: i as f64 * 2.0,
                end: (i + 1) as f64 * 2.0,
            })
            .collect()
    }

    #[test]
    fn parses_ffmpeg_csv() {
        let s = parse_list(
            "seg_000000.ts,0.000000,2.004000\nseg_000001.ts,2.004000,4.008000\nbroken\n",
        );
        assert_eq!(s.len(), 2);
        assert_eq!(s[1].name, "seg_000001.ts");
        assert!((s[1].duration() - 2.004).abs() < 1e-9);
    }

    #[test]
    fn tail_covers_requested_duration() {
        let all = segs(30); // 60 s of footage
        let t = select_tail(&all, 15.0);
        let total: f64 = t.iter().map(Segment::duration).sum();
        assert!(total >= 15.0 && total < 17.0);
        assert_eq!(t.last().unwrap().name, "seg_000029.ts");
        assert!(t.windows(2).all(|w| w[0].start < w[1].start));
    }

    #[test]
    fn tail_of_short_buffer_returns_everything() {
        assert_eq!(select_tail(&segs(3), 300.0).len(), 3);
        assert!(select_tail(&[], 30.0).is_empty());
    }

    #[test]
    fn expiry_keeps_the_retention_window() {
        let all = segs(30);
        let old = expired(&all, 20.0);
        // newest end = 60; keep segments whose end >= 40
        assert!(old.iter().all(|s| s.end < 40.0));
        assert_eq!(old.len(), 19);
        assert!(expired(&[], 10.0).is_empty());
    }

    #[test]
    fn manifest_escapes_paths() {
        let m = concat_manifest(Path::new(r"C:\a b\it's"), &["x.ts".to_string()]);
        assert!(m.contains("file 'C:/a b/it'\\''s/x.ts'"));
    }
}
