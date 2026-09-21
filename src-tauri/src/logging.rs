//! Local-only logging to `%APPDATA%\Rimlight\logs`. Nothing is ever sent anywhere, and we log
//! device / encoder / hotkey diagnostics only – never captured content or file contents.
use once_cell::sync::OnceCell;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static GUARD: OnceCell<WorkerGuard> = OnceCell::new();

pub fn init(level: &str) {
    let dir = crate::paths::logs_dir();
    let _ = std::fs::create_dir_all(&dir);
    prune_old(&dir, 14);
    let appender = tracing_appender::rolling::daily(&dir, "rimlight.log");
    let (nb, guard) = tracing_appender::non_blocking(appender);
    let _ = GUARD.set(guard);
    let filter = EnvFilter::try_new(format!("rimlight_lib={level},warn"))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(nb)
                .with_ansi(false)
                .with_target(false),
        )
        .try_init();
    tracing::info!("Rimlight {} starting", env!("CARGO_PKG_VERSION"));
}

fn prune_old(dir: &std::path::Path, keep: usize) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<_> = rd.flatten().filter(|e| e.path().is_file()).collect();
    files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    if files.len() > keep {
        for f in &files[..files.len() - keep] {
            let _ = std::fs::remove_file(f.path());
        }
    }
}
