use derive_more::From;

use crate::vars::ValueDict;

mod upload_options;
pub use upload_options::*;

//use super::predule::*;
/// Defines the duration for which updates are kept or applied.
///
/// Currently, only project-level duration is supported.
#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum KeepDuration {
    /// Keep or apply updates at the project level.
    #[default]
    DurProj,
}

/// Defines the scope levels for updates, determining how broadly changes are applied.
///
/// The scope levels are ordered from most specific (`Element`) to broadest (`Host`),
/// with `Project` as the default.
#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum UpdateScope {
    #[default]
    None,
    RemoteCache,
}

impl From<(usize, ValueDict)> for DownloadOptions {
    fn from(value: (usize, ValueDict)) -> Self {
        match value.0 {
            0 => Self::new(UpdateScope::None, value.1),
            1 => Self::new(UpdateScope::RemoteCache, value.1),
            _ => Self::new(UpdateScope::RemoteCache, value.1),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DownloadOptions {
    scope_level: UpdateScope,
    values: ValueDict,
}
impl DownloadOptions {
    pub fn new(scope_level: UpdateScope, values: ValueDict) -> Self {
        Self {
            scope_level,
            values,
        }
    }

    pub fn for_test() -> Self {
        Self {
            scope_level: UpdateScope::RemoteCache,
            values: ValueDict::default(),
        }
    }

    pub fn clean_cache(&self) -> bool {
        match self.scope_level {
            UpdateScope::None => false,
            UpdateScope::RemoteCache => true,
        }
    }
    pub fn reuse_cache(&self) -> bool {
        match self.scope_level {
            UpdateScope::None => true,
            UpdateScope::RemoteCache => false,
        }
    }

    pub fn clean_git_cache(&self) -> bool {
        self.clean_cache()
    }

    pub fn scope_level(&self) -> &UpdateScope {
        &self.scope_level
    }

    pub fn values(&self) -> &ValueDict {
        &self.values
    }
}
