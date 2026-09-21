//! User-facing errors. `code` is a stable identifier the UI localises (`errors.<code>` in the i18n
//! dictionaries); `message` is an English fallback that already explains what happened and
//! `hint` states what the user can do about it. We never surface "something went wrong".
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: None,
        }
    }
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
    pub fn internal(e: impl std::fmt::Display) -> Self {
        Self::new("internal", format!("Unexpected internal error: {e}"))
            .hint("Open Settings → Advanced → Open logs folder to see details.")
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("{e:#}");
        Self::internal(format!("{e:#}"))
    }
}
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::new("io", format!("File operation failed: {e}"))
    }
}
impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        tracing::error!("database: {e}");
        Self::new(
            "database",
            format!("The local library database reported an error: {e}"),
        )
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::internal(e)
    }
}
