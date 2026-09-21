//! "Hide Windows notifications while recording": flips the per-user *Get notifications from apps*
//! switch (`ToastEnabled`) for the duration of a recording and restores the user's own value.
//! The previous value is written to the database first, so a crash can never leave notifications
//! disabled: the next start restores it.
use crate::db::Db;
use std::process::Stdio;

const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\PushNotifications";
const VALUE: &str = "ToastEnabled";
const RESTORE_KEY: &str = "toast_restore";

fn reg(args: &[&str]) -> Option<String> {
    let out = crate::encoder::command(std::path::Path::new("reg.exe"))
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Parses `reg query` output: Some(0|1) when the value exists, None when absent.
pub fn parse_query(out: &str) -> Option<u32> {
    let line = out.lines().find(|l| l.contains(VALUE))?;
    let hex = line.split_whitespace().last()?;
    u32::from_str_radix(hex.trim_start_matches("0x"), 16).ok()
}

fn current() -> Option<u32> {
    reg(&["query", KEY, "/v", VALUE]).and_then(|o| parse_query(&o))
}

fn set(v: Option<u32>) {
    match v {
        Some(n) => {
            let _ = reg(&[
                "add",
                KEY,
                "/v",
                VALUE,
                "/t",
                "REG_DWORD",
                "/d",
                &n.to_string(),
                "/f",
            ]);
        }
        None => {
            let _ = reg(&["delete", KEY, "/v", VALUE, "/f"]);
        }
    }
}

/// Turns pop-up notifications off (idempotent) after remembering what to restore.
pub fn suppress(db: &Db) {
    if db.kv_get(RESTORE_KEY).is_some() {
        return; // already suppressed by us
    }
    let cur = current();
    if cur == Some(0) {
        return; // the user already has them off – nothing to change or restore
    }
    let _ = db.kv_set(
        RESTORE_KEY,
        &cur.map(|v| v.to_string())
            .unwrap_or_else(|| "absent".into()),
    );
    set(Some(0));
}

pub fn restore(db: &Db) {
    let Some(prev) = db.kv_get(RESTORE_KEY) else {
        return;
    };
    set(if prev == "absent" {
        None
    } else {
        prev.parse().ok()
    });
    let _ = db.kv_set(RESTORE_KEY, "");
    // an empty marker means "nothing pending"
}

/// Called at start-up: undoes a suppression left behind by a crash.
pub fn restore_if_pending(db: &Db) {
    if db
        .kv_get(RESTORE_KEY)
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        restore(db);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reg_query_output() {
        let out = "\r\nHKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\PushNotifications\r\n    ToastEnabled    REG_DWORD    0x1\r\n\r\n";
        assert_eq!(parse_query(out), Some(1));
        assert_eq!(parse_query("    ToastEnabled    REG_DWORD    0x0"), Some(0));
        assert_eq!(parse_query("nothing here"), None);
    }
}
