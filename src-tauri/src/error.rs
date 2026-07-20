use serde::ser::{Serialize, SerializeStruct, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("git config error: {0}")]
    GitConfig(String),
    #[error("store error: {0}")]
    Store(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[allow(dead_code)]
    #[error("credential error: {0}")]
    Credential(String),
    #[allow(dead_code)]
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[allow(dead_code)]
    #[error("undo unavailable")]
    UndoUnavailable,
    #[allow(dead_code)]
    #[error("not implemented")]
    NotImplemented,
    #[error("internal: {0}")]
    Internal(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Validation(_) => "VALIDATION",
            Self::GitConfig(_) => "GIT_CONFIG",
            Self::Store(_) => "STORE",
            Self::Io(_) => "IO",
            Self::Conflict(_) => "CONFLICT",
            Self::Crypto(_) => "CRYPTO",
            Self::Credential(_) => "CREDENTIAL",
            Self::Unsupported(_) => "UNSUPPORTED",
            Self::UndoUnavailable => "UNDO_UNAVAILABLE",
            Self::NotImplemented => "NOT_IMPLEMENTED",
            Self::Internal(_) => "INTERNAL",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}
