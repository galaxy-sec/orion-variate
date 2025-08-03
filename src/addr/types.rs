use crate::{predule::*, vars::EnvDict};
use derive_more::{Display, From};

use crate::vars::EnvEvalable;

use super::{GitAddr, HttpAddr, LocalAddr};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Display)]
#[serde(untagged)]
pub enum AddrType {
    #[display("git")]
    #[serde(rename = "git")]
    Git(GitAddr),
    #[display("http")]
    #[serde(rename = "http")]
    Http(HttpAddr),
    #[display("local")]
    #[serde(rename = "local")]
    Local(LocalAddr),
}

impl EnvEvalable<AddrType> for AddrType {
    fn env_eval(self, dict: &EnvDict) -> AddrType {
        match self {
            AddrType::Git(v) => AddrType::Git(v.env_eval(dict)),
            AddrType::Http(v) => AddrType::Http(v.env_eval(dict)),
            AddrType::Local(v) => AddrType::Local(v.env_eval(dict)),
        }
    }
}

impl From<GitAddr> for AddrType {
    fn from(value: GitAddr) -> Self {
        Self::Git(value)
    }
}

impl From<HttpAddr> for AddrType {
    fn from(value: HttpAddr) -> Self {
        Self::Http(value)
    }
}

impl From<LocalAddr> for AddrType {
    fn from(value: LocalAddr) -> Self {
        Self::Local(value)
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, From, Default)]
#[serde(transparent)]
pub struct EnvVarPath {
    origin: String,
}
impl EnvVarPath {
    pub fn path(&self, dict: &EnvDict) -> PathBuf {
        let real = self.origin.clone().env_eval(dict);
        PathBuf::from(real)
    }
}

impl From<&str> for EnvVarPath {
    fn from(value: &str) -> Self {
        Self {
            origin: value.to_string(),
        }
    }
}

impl From<PathBuf> for EnvVarPath {
    fn from(value: PathBuf) -> Self {
        Self {
            origin: format!("{}", value.display()),
        }
    }
}

impl From<&PathBuf> for EnvVarPath {
    fn from(value: &PathBuf) -> Self {
        Self {
            origin: format!("{}", value.display()),
        }
    }
}

impl From<&Path> for EnvVarPath {
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

impl FromStr for AddrType {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.starts_with("git@") || s.starts_with("https://") && s.contains(".git") {
            Ok(AddrType::Git(GitAddr::from(s)))
        } else if s.starts_with("http://") || s.starts_with("https://") {
            Ok(AddrType::Http(HttpAddr::from(s)))
        } else if s.starts_with("./")
            || s.starts_with("/")
            || s.starts_with("~")
            || (!s.contains("://") && std::path::Path::new(s).exists())
        {
            Ok(AddrType::Local(LocalAddr::from(s)))
        } else if s.contains("github.com") || s.contains("gitlab.com") || s.contains("gitea.com") {
            Ok(AddrType::Git(GitAddr::from(s)))
        } else {
            Err(AddrParseError::InvalidFormat(s.to_string()))
        }
    }
}

impl<'a> From<&'a str> for AddrType {
    fn from(s: &'a str) -> Self {
        AddrType::from_str(s).unwrap_or_else(|_| AddrType::Local(LocalAddr::from(s)))
    }
}
