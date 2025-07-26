use derive_more::From;
use orion_error::{ErrorCode, StructError, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;
#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum VarsReason {
    #[error("unknow")]
    UnKnow,
    #[error("format")]
    Format,
    #[error("{0}")]
    Uvs(UvsReason),
}

impl ErrorCode for VarsReason {
    fn error_code(&self) -> i32 {
        match self {
            VarsReason::Format => 501,
            VarsReason::UnKnow => 502,
            VarsReason::Uvs(r) => r.error_code(),
        }
    }
}

pub type VarsResult<T> = Result<T, StructError<VarsReason>>;
pub type VarsError = StructError<VarsReason>;
