pub mod accessor;
pub mod git;
pub mod http;
pub mod local;
pub mod types;

pub use git::GitAddr;
pub use http::HttpAddr;
pub use local::LocalAddr;
pub use types::AddrType;
pub mod error;
pub mod redirect;

pub use error::{AddrError, AddrReason, AddrResult};
pub mod proxy;
