//! Global hotkeys: parsing, conflict detection and registration.
use crate::engine::Engine;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

static LAST: Lazy<Mutex<Vec<HotkeyStatus>>> = Lazy::new(|| Mutex::new(vec![]));

/// Outcome of the most recent registration pass.
pub fn last_status() -> Vec<HotkeyStatus> {
    LAST.lock().clone()
}

pub const ACTIONS: [&str; 9] = [
    "overlay",
    "record",
    "saveReplay",
    "toggleReplay",
    "screenshot",
    "toggleMic",
    "toggleCamera",
    "toggleStats",
    "marker",
];

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub action: String,
    pub accel: String,
    pub ok: bool,
    /// invalid | duplicate | in_use | empty
    pub problem: Option<String>,
    /// For duplicates: the other action that already uses this combination.
    pub conflicts_with: Option<String>,
}

/// Canonical form used to compare combinations: modifiers sorted, everything upper-case.
pub fn normalize(accel: &str) -> String {
    let mut mods: Vec<String> = vec![];
    let mut key = String::new();
    for part in accel.split('+').map(|p| p.trim()).filter(|p| !p.is_empty()) {
        let up = part.to_uppercase();
        match up.as_str() {
            "CTRL" | "CONTROL" => mods.push("CTRL".into()),
            "ALT" | "OPTION" => mods.push("ALT".into()),
            "SHIFT" => mods.push("SHIFT".into()),
            "WIN" | "SUPER" | "META" | "CMD" | "COMMAND" | "WINDOWS" => mods.push("WIN".into()),
            _ => key = up,
        }
    }
    mods.sort();
    mods.dedup();
    mods.push(key);
    mods.join("+")
}

/// Rewrites user-facing names into the tokens the shortcut parser understands (Win → Super).
pub fn for_plugin(accel: &str) -> String {
    accel
        .split('+')
        .map(|p| match p.trim().to_uppercase().as_str() {
            "WIN" | "WINDOWS" | "META" => "Super".to_string(),
            "CONTROL" => "Ctrl".to_string(),
            other => {
                let _ = other;
                p.trim().to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("+")
}

/// Pairs of actions configured with the same combination.
pub fn find_duplicates(map: &BTreeMap<String, String>) -> Vec<(String, String)> {
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut out = vec![];
    for (action, accel) in map {
        if accel.trim().is_empty() {
            continue;
        }
        let n = normalize(accel);
        if let Some(first) = seen.get(&n) {
            out.push((first.clone(), action.clone()));
        } else {
            seen.insert(n, action.clone());
        }
    }
    out
}

fn dispatch(engine: &Arc<Engine>, action: &str) {
    let e = engine.clone();
    let action = action.to_string();
    // never run engine work on the hotkey thread
    std::thread::spawn(move || {
        let r = match action.as_str() {
            "overlay" => {
                crate::shell::toggle_overlay(&e.app);
                Ok(())
            }
            "record" => e.toggle_recording(),
            "saveReplay" => e.save_replay(),
            "toggleReplay" => e.toggle_replay(),
            "screenshot" => crate::screenshot::take(&e, None).map(|_| ()),
            "toggleMic" => {
                e.toggle_mic();
                Ok(())
            }
            "toggleCamera" => {
                e.toggle_camera();
                Ok(())
            }
            "toggleStats" => {
                let mut s = e.settings.read().clone();
                s.performance.stats_enabled = !s.performance.stats_enabled;
                e.apply_settings(s).map(|_| crate::shell::update_hud(&e))
            }
            "marker" => e.add_marker(),
            _ => Ok(()),
        };
        if let Err(err) = r {
            e.set_error(err);
        }
    });
}

/// (Re)registers every configured hotkey and reports the outcome for each.
pub fn register_all(app: &AppHandle, engine: &Arc<Engine>) -> Vec<HotkeyStatus> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let map = engine.settings.read().hotkeys.clone();
    let dups = find_duplicates(&map);
    let mut out = vec![];
    for action in ACTIONS {
        let accel = map.get(action).cloned().unwrap_or_default();
        let mut st = HotkeyStatus {
            action: action.into(),
            accel: accel.clone(),
            ok: false,
            problem: None,
            conflicts_with: None,
        };
        if accel.trim().is_empty() {
            st.problem = Some("empty".into());
            out.push(st);
            continue;
        }
        if let Some((first, _)) = dups.iter().find(|(_, second)| second == action) {
            st.problem = Some("duplicate".into());
            st.conflicts_with = Some(first.clone());
            out.push(st);
            continue;
        }
        match Shortcut::from_str(&for_plugin(&accel)) {
            Err(_) => st.problem = Some("invalid".into()),
            Ok(sc) => {
                let e = engine.clone();
                let act = action.to_string();
                let res = gs.on_shortcut(sc, move |_app, _sc, ev| {
                    if ev.state == ShortcutState::Pressed {
                        dispatch(&e, &act);
                    }
                });
                match res {
                    Ok(()) => st.ok = true,
                    Err(err) => {
                        tracing::warn!(
                            "hotkey {accel} for {action} could not be registered: {err}"
                        );
                        st.problem = Some("in_use".into());
                    }
                }
            }
        }
        out.push(st);
    }
    *LAST.lock() = out.clone();
    out
}

/// Checks whether `accel` could be used for `action` right now (before the user commits to it).
pub fn check(app: &AppHandle, engine: &Engine, action: &str, accel: &str) -> HotkeyStatus {
    let mut st = HotkeyStatus {
        action: action.into(),
        accel: accel.into(),
        ok: false,
        problem: None,
        conflicts_with: None,
    };
    if accel.trim().is_empty() {
        st.problem = Some("empty".into());
        return st;
    }
    let Ok(sc) = Shortcut::from_str(&for_plugin(accel)) else {
        st.problem = Some("invalid".into());
        return st;
    };
    let norm = normalize(accel);
    let map = engine.settings.read().hotkeys.clone();
    if let Some((other, _)) = map
        .iter()
        .find(|(a, v)| a.as_str() != action && normalize(v) == norm)
    {
        st.problem = Some("duplicate".into());
        st.conflicts_with = Some(other.clone());
        return st;
    }
    let gs = app.global_shortcut();
    if gs.is_registered(sc) {
        st.ok = true; // held by this very action
        return st;
    }
    match gs.register(sc) {
        Ok(()) => {
            let _ = gs.unregister(sc);
            st.ok = true;
        }
        Err(_) => st.problem = Some("in_use".into()),
    }
    st
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalisation_ignores_case_and_modifier_order() {
        assert_eq!(normalize("alt+CTRL+m"), normalize("Ctrl+Alt+M"));
        assert_eq!(normalize("Win+Shift+S"), "SHIFT+WIN+S");
        assert_ne!(normalize("Alt+Z"), normalize("Alt+Shift+Z"));
    }

    #[test]
    fn detects_duplicate_combinations() {
        let mut m = crate::settings::default_hotkeys();
        assert!(find_duplicates(&m).is_empty(), "defaults must not collide");
        m.insert("screenshot".into(), "alt+z".into());
        let d = find_duplicates(&m);
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn every_action_has_a_default() {
        let m = crate::settings::default_hotkeys();
        for a in ACTIONS {
            assert!(m.contains_key(a), "missing default for {a}");
        }
    }

    #[test]
    fn default_combinations_parse() {
        for (a, v) in crate::settings::default_hotkeys() {
            assert!(Shortcut::from_str(&v).is_ok(), "{a}: {v}");
        }
        assert!(Shortcut::from_str(&for_plugin("Ctrl+Alt+Shift+Win+F12")).is_ok());
        assert_eq!(for_plugin("Win+Shift+S"), "Super+Shift+S");
        assert!(Shortcut::from_str("garbage++").is_err());
    }
}
