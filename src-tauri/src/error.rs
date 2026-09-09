use serde::Serialize;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone, Serialize)]
pub struct AppError {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl AppError {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            params: None,
        }
    }

    pub fn with_count(code: impl Into<String>, count: i64) -> Self {
        Self {
            code: code.into(),
            params: Some(serde_json::json!({ "count": count })),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        eprintln!("sqlite error: {err}");
        Self::new("error.db")
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        eprintln!("json error: {err}");
        Self::new("error.payloadInvalid")
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        eprintln!("io error: {err}");
        Self::new("error.io")
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, rusqlite::Connection>>> for AppError {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, rusqlite::Connection>>) -> Self {
        Self::new("error.dbLock")
    }
}
