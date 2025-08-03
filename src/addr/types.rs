use crate::{predule::*, vars::EnvDict};
use derive_more::{Display, From};

use crate::vars::EnvEvalable;

use super::{GitRepository, HttpResource, LocalPath};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Display)]
#[serde(untagged)]
pub enum Address {
    #[display("git")]
    #[serde(rename = "git")]
    Git(GitRepository),
    #[display("http")]
    #[serde(rename = "http")]
    Http(HttpResource),
    #[display("local")]
    #[serde(rename = "local")]
    Local(LocalPath),
}

impl EnvEvalable<Address> for Address {
    fn env_eval(self, dict: &EnvDict) -> Address {
        match self {
            Address::Git(v) => Address::Git(v.env_eval(dict)),
            Address::Http(v) => Address::Http(v.env_eval(dict)),
            Address::Local(v) => Address::Local(v.env_eval(dict)),
        }
    }
}

impl From<GitRepository> for Address {
    fn from(value: GitRepository) -> Self {
        Self::Git(value)
    }
}

impl From<HttpResource> for Address {
    fn from(value: HttpResource) -> Self {
        Self::Http(value)
    }
}

impl From<LocalPath> for Address {
    fn from(value: LocalPath) -> Self {
        Self::Local(value)
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, From, Default)]
#[serde(transparent)]
pub struct PathTemplate {
    origin: String,
}
impl PathTemplate {
    pub fn path(&self, dict: &EnvDict) -> PathBuf {
        let real = self.origin.clone().env_eval(dict);
        PathBuf::from(real)
    }
}

impl From<&str> for PathTemplate {
    fn from(value: &str) -> Self {
        Self {
            origin: value.to_string(),
        }
    }
}

impl From<PathBuf> for PathTemplate {
    fn from(value: PathBuf) -> Self {
        Self {
            origin: format!("{}", value.display()),
        }
    }
}

impl From<&PathBuf> for PathTemplate {
    fn from(value: &PathBuf) -> Self {
        Self {
            origin: format!("{}", value.display()),
        }
    }
}

impl From<&Path> for PathTemplate {
    fn from(value: &Path) -> Self {
        Self {
            origin: format!("{}", value.display()),
        }
    }
}

/// 地址类型解析错误
#[derive(Debug, Error)]
pub enum AddrParseError {
    #[error("invalid address format: {0}")]
    InvalidFormat(String),
}

impl FromStr for Address {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.starts_with("git@") || s.starts_with("https://") && s.contains(".git") {
            Ok(Address::Git(GitRepository::from(s)))
        } else if s.starts_with("http://") || s.starts_with("https://") {
            Ok(Address::Http(HttpResource::from(s)))
        } else if s.starts_with("./")
            || s.starts_with("/")
            || s.starts_with("~")
            || (!s.contains("://") && std::path::Path::new(s).exists())
        {
            Ok(Address::Local(LocalPath::from(s)))
        } else if s.contains("github.com") || s.contains("gitlab.com") || s.contains("gitea.com") {
            Ok(Address::Git(GitRepository::from(s)))
        } else {
            Err(AddrParseError::InvalidFormat(s.to_string()))
        }
    }
}

impl<'a> From<&'a str> for Address {
    fn from(s: &'a str) -> Self {
        Address::from_str(s).unwrap_or_else(|_| Address::Local(LocalPath::from(s)))
    }
}
