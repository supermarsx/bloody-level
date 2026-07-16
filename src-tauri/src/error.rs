use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Locked,
    AlreadyInitialized,
    NotFound,
    BadRequest,
    Crypto,
    Database,
    Filesystem,
    Pdf,
    Internal,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ErrorContext {
    pub stage: Option<String>,
    pub path: Option<String>,
    pub patient: Option<String>,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppErrorPayload {
    pub kind: ErrorKind,
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
    pub retryable: bool,
    pub timestamp: i64,
    pub context: Option<ErrorContext>,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("migration: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("crypto: {0}")]
    Crypto(String),

    #[error("base64: {0}")]
    Base64(String),

    #[error("locked")]
    Locked,

    #[error("already initialized")]
    AlreadyInitialized,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("pdf: {0}")]
    Pdf(String),

    #[error("internal: {0}")]
    Internal(String),

    /// Wraps another `AppError` with a structured `ErrorContext`. The wrapped
    /// error keeps its original kind/code for routing; the context is what the
    /// frontend renders to help the user understand and fix the issue.
    #[error("{source}")]
    Context {
        source: Box<AppError>,
        ctx: Box<ErrorContext>,
    },
}

impl AppError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Context { source, .. } => source.kind(),
            Self::Io(_) => ErrorKind::Filesystem,
            Self::Sqlite(_) | Self::Migration(_) => ErrorKind::Database,
            Self::Json(_) => ErrorKind::Internal,
            Self::Crypto(_) | Self::Base64(_) => ErrorKind::Crypto,
            Self::Locked => ErrorKind::Locked,
            Self::AlreadyInitialized => ErrorKind::AlreadyInitialized,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::BadRequest(_) => ErrorKind::BadRequest,
            Self::Pdf(_) => ErrorKind::Pdf,
            Self::Internal(_) => ErrorKind::Internal,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Context { source, .. } => source.code(),
            Self::Io(e) => match e.kind() {
                std::io::ErrorKind::NotFound => "fs.not_found",
                std::io::ErrorKind::PermissionDenied => "fs.permission_denied",
                std::io::ErrorKind::AlreadyExists => "fs.already_exists",
                _ => "fs.io",
            },
            Self::Sqlite(rusqlite::Error::SqliteFailure(ffi, _)) => match ffi.code {
                rusqlite::ErrorCode::ConstraintViolation => "db.constraint",
                rusqlite::ErrorCode::DatabaseBusy => "db.busy",
                rusqlite::ErrorCode::DatabaseLocked => "db.locked",
                rusqlite::ErrorCode::DiskFull => "db.disk_full",
                rusqlite::ErrorCode::DatabaseCorrupt => "db.corrupt",
                _ => "db.sqlite",
            },
            Self::Sqlite(_) => "db.sqlite",
            Self::Migration(_) => "db.migration",
            Self::Json(_) => "json.parse",
            Self::Crypto(_) => "crypto.failed",
            Self::Base64(_) => "crypto.base64",
            Self::Locked => "auth.locked",
            Self::AlreadyInitialized => "auth.already_initialized",
            Self::NotFound(_) => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::Pdf(_) => "pdf.failed",
            Self::Internal(_) => "internal",
        }
    }

    pub fn retryable(&self) -> bool {
        match self {
            Self::Context { source, .. } => source.retryable(),
            Self::Sqlite(rusqlite::Error::SqliteFailure(ffi, _)) => matches!(
                ffi.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            ),
            Self::Io(e) => matches!(
                e.kind(),
                std::io::ErrorKind::Interrupted
                    | std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::WouldBlock
            ),
            _ => false,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::Context { source, .. } => source.user_message(),
            Self::Locked => "Database is locked. Please unlock to continue.".into(),
            Self::AlreadyInitialized => "App is already initialized.".into(),
            Self::NotFound(what) => format!("Not found: {what}"),
            Self::BadRequest(msg) => msg.clone(),
            Self::Crypto(_) => "Decryption failed. Check your password.".into(),
            Self::Base64(_) => "Encoding error in submitted data.".into(),
            Self::Pdf(msg) => format!("PDF error: {msg}"),
            Self::Sqlite(rusqlite::Error::SqliteFailure(ffi, msg)) => match ffi.code {
                rusqlite::ErrorCode::ConstraintViolation => {
                    msg.clone().unwrap_or_else(|| "Constraint violated.".into())
                }
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked => {
                    "Database is busy. Please retry.".into()
                }
                rusqlite::ErrorCode::DiskFull => "Disk is full.".into(),
                rusqlite::ErrorCode::DatabaseCorrupt => "Database file is corrupted.".into(),
                _ => msg.clone().unwrap_or_else(|| self.to_string()),
            },
            Self::Io(e) => match e.kind() {
                std::io::ErrorKind::NotFound => "File not found.".into(),
                std::io::ErrorKind::PermissionDenied => "Permission denied.".into(),
                _ => format!("Filesystem error: {e}"),
            },
            _ => self.to_string(),
        }
    }

    pub fn context(&self) -> Option<&ErrorContext> {
        match self {
            Self::Context { ctx, .. } => Some(ctx),
            _ => None,
        }
    }

    pub fn to_payload(&self) -> AppErrorPayload {
        AppErrorPayload {
            kind: self.kind(),
            code: self.code().to_string(),
            message: self.user_message(),
            detail: Some(self.to_string()),
            retryable: self.retryable(),
            timestamp: now_secs(),
            context: self.context().cloned(),
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        tracing::error!(
            kind = ?self.kind(),
            code = self.code(),
            stage = self.context().and_then(|c| c.stage.as_deref()).unwrap_or(""),
            path = self.context().and_then(|c| c.path.as_deref()).unwrap_or(""),
            error = %self,
            "command failed"
        );
        self.to_payload().serialize(s)
    }
}

// ---------------------------------------------------------------------------
// Context-builder extensions on `Result<T, AppError>`.
// ---------------------------------------------------------------------------

pub trait ContextExt {
    fn stage(self, s: &str) -> Self;
    fn path<P: AsRef<str>>(self, s: P) -> Self;
    fn patient<S: AsRef<str>>(self, s: S) -> Self;
    fn hint<S: AsRef<str>>(self, s: S) -> Self;
}

fn ensure_ctx(e: AppError) -> (Box<AppError>, Box<ErrorContext>) {
    match e {
        AppError::Context { source, ctx } => (source, ctx),
        other => (Box::new(other), Box::new(ErrorContext::default())),
    }
}

impl<T> ContextExt for Result<T, AppError> {
    fn stage(self, s: &str) -> Self {
        self.map_err(|e| {
            let (src, mut ctx) = ensure_ctx(e);
            if ctx.stage.is_none() {
                ctx.stage = Some(s.into());
            }
            AppError::Context { source: src, ctx }
        })
    }
    fn path<P: AsRef<str>>(self, s: P) -> Self {
        self.map_err(|e| {
            let (src, mut ctx) = ensure_ctx(e);
            if ctx.path.is_none() {
                ctx.path = Some(s.as_ref().to_string());
            }
            AppError::Context { source: src, ctx }
        })
    }
    fn patient<S: AsRef<str>>(self, s: S) -> Self {
        self.map_err(|e| {
            let (src, mut ctx) = ensure_ctx(e);
            if ctx.patient.is_none() {
                ctx.patient = Some(s.as_ref().to_string());
            }
            AppError::Context { source: src, ctx }
        })
    }
    fn hint<S: AsRef<str>>(self, s: S) -> Self {
        self.map_err(|e| {
            let (src, mut ctx) = ensure_ctx(e);
            ctx.hints.push(s.as_ref().to_string());
            AppError::Context { source: src, ctx }
        })
    }
}

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub type AppResult<T> = Result<T, AppError>;
