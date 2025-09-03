use derive_more::From;
use orion_conf::error::SerdeReason;
use orion_error::{ErrorCode, StructError, UvsConfFrom, UvsReason};
use orion_sec::{OrionSecReason, SecReason};
use serde_derive::Serialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, Error, From)]
pub enum OrionAiReason {
    #[error("{0}")]
    Ai(AiErrReason),
    #[error("{0}")]
    Sec(SecReason),
    #[error("{0}")]
    Uvs(UvsReason),
}

#[derive(Debug, PartialEq, Serialize, Error)]
pub enum AiErrReason {
    #[error("API rate limit exceeded for provider: {0}")]
    RateLimitError(String),

    #[error("Token limit exceeded: {0} tokens requested, max {1}")]
    TokenLimitError(usize, usize),
    #[error("Context collection failed: {0}")]
    ContextError(String),

    #[error("No suitable provider found for request")]
    NoProviderAvailable,

    #[error("Invalid model specified: {0}")]
    InvalidModel(String),

    #[error("Sensitive content filtered")]
    SensitiveContentFiltered,
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Tool call error: {0}")]
    ToolCallError(String),
    #[error("Tool get error: {0}")]
    ToolGetError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<SerdeReason> for OrionAiReason {
    fn from(value: SerdeReason) -> Self {
        match value {
            SerdeReason::Brief(msg) => Self::Uvs(UvsReason::from_conf(msg)),
            SerdeReason::Uvs(uvs) => Self::Uvs(uvs),
        }
    }
}

impl From<OrionSecReason> for OrionAiReason {
    fn from(value: OrionSecReason) -> Self {
        match value {
            OrionSecReason::Sec(sec) => Self::Sec(sec),
            OrionSecReason::Uvs(uvs) => Self::Uvs(uvs),
        }
    }
}
impl ErrorCode for OrionAiReason {
    fn error_code(&self) -> i32 {
        800
    }
}

pub type AiError = StructError<OrionAiReason>;
pub type AiResult<T> = Result<T, AiError>;
