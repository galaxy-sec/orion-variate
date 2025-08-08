use derive_more::From;
use orion_error::{ErrorCode, StructError, UvsReason};
use serde_derive::Serialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum AddrReason {
    #[error("unknown")]
    Brief(String),
    #[error("{0}")]
    Uvs(UvsReason),
    #[error("Operation timed out after {timeout:?} and {attempts} attempts")]
    OperationTimeoutExceeded { timeout: Duration, attempts: u32 },
    #[error("Total timeout {total_timeout:?} exceeded after {elapsed:?}")]
    TotalTimeoutExceeded {
        total_timeout: Duration,
        elapsed: Duration,
    },
    #[error("Retry exhausted after {attempts} attempts, last error: {last_error}")]
    RetryExhausted { attempts: u32, last_error: String },
}

impl ErrorCode for AddrReason {
    fn error_code(&self) -> i32 {
        match self {
            AddrReason::Brief(_) => 500,
            AddrReason::Uvs(r) => r.error_code(),
            AddrReason::OperationTimeoutExceeded { .. } => 408,
            AddrReason::TotalTimeoutExceeded { .. } => 408,
            AddrReason::RetryExhausted { .. } => 504,
        }
    }
}

pub type AddrResult<T> = Result<T, StructError<AddrReason>>;
pub type AddrError = StructError<AddrReason>;
