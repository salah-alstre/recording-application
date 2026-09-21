//! File naming templates, e.g. `{game}_{date}_{time}` → `GTA-V_2026-09-20_18-42-31`.
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};

pub struct NameContext<'a> {
    pub game: &'a str,
    pub profile: &'a str,
    pub resolution: &'a str,
    pub fps: u32,
    pub now: DateTime<Local>,
}

const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Makes a single path component safe on Windows: strips illegal characters, collapses whitespace
/// into `-`, avoids reserved device names and trailing dots/spaces.
pub fn sanitize_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut last_dash = false;
    for c in input.chars() {
        let bad = c.is_control() || "<>:\"/\\|?*".contains(c);
        let c = if bad || c.is_whitespace() { '-' } else { c };
        if c == '-' {
            if last_dash {
                continue;
            }
            last_dash = true;
        } else {
            last_dash = false;
        }
        out.push(c);
    }
    let mut out = out
        .trim_matches(|c| c == '-' || c == '.' || c == ' ')
        .to_string();
    if out.is_empty() {
        out = "Capture".into();
    }
    if RESERVED.contains(&out.to_uppercase().as_str()) {
        out.push('_');
    }
    if out.chars().count() > 120 {
        out = out.chars().take(120).collect();
    }
    out
}

pub fn render(template: &str, ctx: &NameContext) -> String {
    let game = if ctx.game.trim().is_empty() {
        "Desktop"
    } else {
        ctx.game
    };
    let s = template
        .replace("{game}", game)
        .replace("{date}", &ctx.now.format("%Y-%m-%d").to_string())
        .replace("{time}", &ctx.now.format("%H-%M-%S").to_string())
        .replace("{profile}", ctx.profile)
        .replace("{res}", ctx.resolution)
        .replace("{fps}", &ctx.fps.to_string());
    sanitize_component(&s)
}

/// Returns `dir/name.ext`, appending ` (2)`, ` (3)`… if a file with that name already exists.
pub fn unique_path(dir: &Path, name: &str, ext: &str) -> PathBuf {
    let first = dir.join(format!("{name}.{ext}"));
    if !first.exists() {
        return first;
    }
    for n in 2..10_000 {
        let p = dir.join(format!("{name} ({n}).{ext}"));
        if !p.exists() {
            return p;
        }
    }
    dir.join(format!("{name}-{}.{ext}", uuid::Uuid::new_v4().simple()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ctx(game: &str) -> NameContext<'_> {
        NameContext {
            game,
            profile: "Competitive",
            resolution: "1440p",
            fps: 120,
            now: Local.with_ymd_and_hms(2026, 9, 20, 18, 42, 31).unwrap(),
        }
    }

    #[test]
    fn default_template_matches_spec_example() {
        assert_eq!(
            render("{game}_{date}_{time}", &ctx("GTA-V")),
            "GTA-V_2026-09-20_18-42-31"
        );
    }

    #[test]
    fn spaces_and_illegal_chars_are_cleaned() {
        assert_eq!(
            sanitize_component("Half-Life: Alyx / VR?"),
            "Half-Life-Alyx-VR"
        );
        assert_eq!(render("{game}", &ctx("A  B")), "A-B");
    }

    #[test]
    fn empty_game_falls_back_to_desktop() {
        assert!(render("{game}_{fps}", &ctx("")).starts_with("Desktop_120"));
    }

    #[test]
    fn reserved_names_are_escaped() {
        assert_eq!(sanitize_component("con"), "con_");
        assert_eq!(sanitize_component("..."), "Capture");
    }

    #[test]
    fn unique_path_increments() {
        let dir = std::env::temp_dir().join(format!("rimlight-name-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = unique_path(&dir, "clip", "mp4");
        std::fs::write(&a, b"x").unwrap();
        let b = unique_path(&dir, "clip", "mp4");
        assert_ne!(a, b);
        assert!(b.to_string_lossy().contains("(2)"));
        let _ = std::fs::remove_dir_all(dir);
    }
}
