//! Update mechanism: check a JSON feed, show release notes, download and launch the installer.
//! Off by default and never contacts anything unless the user configures a feed URL and asks.
//! Feed format: `{ "version": "1.2.0", "notes": "…", "url": "https://…/Rimlight_1.2.0_x64-setup.exe" }`
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;

#[derive(Deserialize, Debug)]
struct Feed {
    version: String,
    #[serde(default)]
    notes: String,
    url: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub available: bool,
    pub notes: String,
    pub url: String,
}

/// Compares dotted numeric versions ("1.10.0" > "1.9.3").
pub fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split(['.', '-'])
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    let (a, b) = (parse(candidate), parse(current));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (
            a.get(i).copied().unwrap_or(0),
            b.get(i).copied().unwrap_or(0),
        );
        if x != y {
            return x > y;
        }
    }
    false
}

fn curl(args: &[&str]) -> Result<Vec<u8>> {
    let out = std::process::Command::new("curl.exe")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| AppError::new("network", format!("Cannot run curl: {e}")))?;
    if !out.status.success() {
        return Err(
            AppError::new("network", "The update server could not be reached.")
                .hint("Check your internet connection and the feed URL in Settings → General."),
        );
    }
    Ok(out.stdout)
}

pub fn check(feed_url: &str) -> Result<UpdateInfo> {
    if !feed_url.starts_with("https://") {
        return Err(AppError::new(
            "update_feed",
            "No secure (https) update feed is configured.",
        )
        .hint("Enter the feed URL in Settings → General."));
    }
    let body = curl(&["-sSL", "--max-time", "15", feed_url])?;
    let feed: Feed = serde_json::from_slice(&body)
        .map_err(|_| AppError::new("update_feed", "The update feed has an unexpected format."))?;
    let current = env!("CARGO_PKG_VERSION").to_string();
    Ok(UpdateInfo {
        available: is_newer(&feed.version, &current),
        current,
        latest: feed.version,
        notes: feed.notes,
        url: feed.url,
    })
}

/// Downloads the installer and starts it. The caller must have verified that nothing is recording.
pub fn download_and_launch(url: &str) -> Result<()> {
    if !url.starts_with("https://") {
        return Err(AppError::new(
            "update_feed",
            "Refusing to download an update over an insecure connection.",
        ));
    }
    let dest = std::env::temp_dir().join("Rimlight-update-setup.exe");
    curl(&[
        "-sSL",
        "--max-time",
        "600",
        "-o",
        &dest.to_string_lossy(),
        url,
    ])?;
    std::process::Command::new(&dest).spawn().map_err(|e| {
        AppError::new(
            "update_launch",
            format!("The installer could not be started: {e}"),
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_comparison() {
        assert!(is_newer("1.10.0", "1.9.3"));
        assert!(is_newer("v0.2.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.1"));
        assert!(is_newer("1.0.1", "1.0"));
    }

    #[test]
    fn insecure_feeds_are_rejected() {
        assert_eq!(
            check("http://example.com/feed.json").unwrap_err().code,
            "update_feed"
        );
    }
}
