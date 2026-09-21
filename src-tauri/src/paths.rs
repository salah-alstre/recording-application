//! Well-known filesystem locations. Everything the app writes lives under `%APPDATA%\Rimlight`
//! (settings, database, logs, thumbnails) or `%LOCALAPPDATA%\Rimlight` (replay cache).
use std::path::PathBuf;

fn env_dir(var: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

pub fn data_dir() -> PathBuf {
    env_dir("APPDATA").join("Rimlight")
}
pub fn local_dir() -> PathBuf {
    env_dir("LOCALAPPDATA").join("Rimlight")
}
pub fn logs_dir() -> PathBuf {
    data_dir().join("logs")
}
pub fn thumbs_dir() -> PathBuf {
    local_dir().join("thumbnails")
}
pub fn db_path() -> PathBuf {
    data_dir().join("rimlight.db")
}
pub fn replay_cache_dir() -> PathBuf {
    local_dir().join("replay-cache")
}
pub fn default_recordings_dir() -> PathBuf {
    let base = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("Videos").join("Rimlight")
}

pub fn ensure_dirs() {
    for d in [data_dir(), logs_dir(), thumbs_dir(), replay_cache_dir()] {
        let _ = std::fs::create_dir_all(d);
    }
}
