pub mod accessor;
pub mod constants;
pub mod git;
pub mod http;
pub mod local;
pub mod types;
pub mod validation;

pub use constants::*;
pub use git::GitRepository;
pub use http::HttpResource;
pub use local::LocalPath;
pub use types::Address;
pub use validation::{Validate, ValidationError, ValidationResult};
pub mod access_ctrl;
pub mod error;

pub use error::{AddrError, AddrReason, AddrResult};
pub mod proxy;
