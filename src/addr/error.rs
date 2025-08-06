use derive_more::From;
use orion_error::{ErrorCode, StructError, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;
#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum AddrReason {
    #[error("unknow")]
    Brief(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

impl ErrorCode for AddrReason {
    fn error_code(&self) -> i32 {
        match self {
            AddrReason::Brief(_) => 500,
            AddrReason::Uvs(r) => r.error_code(),
        }
    }
}

pub type AddrResult<T> = Result<T, StructError<AddrReason>>;
pub type AddrError = StructError<AddrReason>;
