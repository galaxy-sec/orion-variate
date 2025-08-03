pub mod accessor;
pub mod git;
pub mod http;
pub mod local;
pub mod types;

pub use git::GitRepository;
pub use http::HttpResource;
pub use local::LocalPath;
pub use types::Address;
pub mod error;
pub mod redirect;

pub use error::{AddrError, AddrReason, AddrResult};
pub mod proxy;
