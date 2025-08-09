use crate::{predule::*, vars::EnvDict};
use derive_more::{Display, From};

use crate::vars::EnvEvalable;

use super::{GitRepository, HttpResource, LocalPath};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Display, From)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vars::EnvDict;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    // Address 枚举测试
    #[test]
    fn test_address_git_variant() {
        let git_repo = GitRepository::from("git@github.com:user/repo.git");
        let address = Address::Git(git_repo);
        assert!(matches!(address, Address::Git(_)));
    }

    #[test]
    fn test_address_http_variant() {
        let http_resource = HttpResource::from("https://example.com/file.txt");
        let address = Address::Http(http_resource);
        assert!(matches!(address, Address::Http(_)));
    }

    #[test]
    fn test_address_local_variant() {
        let local_path = LocalPath::from("/local/path");
        let address = Address::Local(local_path);
        assert!(matches!(address, Address::Local(_)));
    }

    #[test]
    fn test_address_env_eval() {
        let git_repo = GitRepository::from("git@github.com:user/${REPO_NAME}.git");
        let address = Address::Git(git_repo);

        let mut dict = HashMap::new();
        dict.insert("REPO_NAME".to_string(), "my-repo".to_string());
        let env_dict = EnvDict::from(dict);

        let result = address.env_eval(&env_dict);
        assert!(matches!(result, Address::Git(_)));
    }

    #[test]
    fn test_address_display() {
        let git_address = Address::Git(GitRepository::from("git@github.com:user/repo.git"));
        let http_address = Address::Http(HttpResource::from("https://example.com/file.txt"));
        let local_address = Address::Local(LocalPath::from("/local/path"));

        assert_eq!(format!("{git_address}"), "git");
        assert_eq!(format!("{http_address}"), "http");
        assert_eq!(format!("{local_address}"), "local");
    }

    #[test]
    fn test_address_clone() {
        let original = Address::Local(LocalPath::from("/test/path"));
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // PathTemplate 测试
    #[test]
    fn test_path_template_from_str() {
        let template = PathTemplate::from("/path/to/file");
        assert_eq!(template.origin, "/path/to/file");
    }

    #[test]
    fn test_path_template_from_pathbuf() {
        let path = PathBuf::from("/path/to/file");
        let template = PathTemplate::from(path);
        assert_eq!(template.origin, "/path/to/file");
    }

    #[test]
    fn test_path_template_from_pathbuf_ref() {
        let path = PathBuf::from("/path/to/file");
        let template = PathTemplate::from(&path);
        assert_eq!(template.origin, "/path/to/file");
    }

    #[test]
    fn test_path_template_from_path() {
        let path = Path::new("/path/to/file");
        let template = PathTemplate::from(path);
        assert_eq!(template.origin, "/path/to/file");
    }

    #[test]
    fn test_path_template_path_no_vars() {
        let template = PathTemplate::from("/static/path");
        let dict = EnvDict::new();
        let result = template.path(&dict);
        assert_eq!(result, PathBuf::from("/static/path"));
    }

    #[test]
    fn test_path_template_path_with_vars() {
        let template = PathTemplate::from("${HOME}/project");
        let mut dict = HashMap::new();
        dict.insert("HOME".to_string(), "/Users/test".to_string());
        let env_dict = EnvDict::from(dict);
        let result = template.path(&env_dict);
        assert_eq!(result, PathBuf::from("/Users/test/project"));
    }

    #[test]
    fn test_path_template_default() {
        let template: PathTemplate = Default::default();
        assert_eq!(template.origin, "");
    }

    #[test]
    fn test_path_template_clone() {
        let original = PathTemplate::from("/test/path");
        let cloned = original.clone();
        assert_eq!(original.origin, cloned.origin);
    }

    // AddrParseError 测试
    #[test]
    fn test_addr_parse_error_display() {
        let error = AddrParseError::InvalidFormat("invalid format".to_string());
        let error_str = format!("{error}");
        assert!(error_str.contains("invalid address format"));
        assert!(error_str.contains("invalid format"));
    }

    // Address FromStr 测试
    #[test]
    fn test_address_from_str_git_ssh() {
        let address = Address::from_str("git@github.com:user/repo.git").unwrap();
        assert!(matches!(address, Address::Git(_)));
    }

    #[test]
    fn test_address_from_str_http() {
        let address = Address::from_str("https://example.com/file.txt").unwrap();
        assert!(matches!(address, Address::Http(_)));
    }

    #[test]
    fn test_address_from_str_local_relative() {
        let address = Address::from_str("./relative/path").unwrap();
        assert!(matches!(address, Address::Local(_)));
    }

    #[test]
    fn test_address_from_str_local_absolute() {
        let address = Address::from_str("/absolute/path").unwrap();
        assert!(matches!(address, Address::Local(_)));
    }

    #[test]
    fn test_address_from_str_local_home() {
        let address = Address::from_str("~/project").unwrap();
        assert!(matches!(address, Address::Local(_)));
    }

    #[test]
    fn test_address_from_str_gitlab_url() {
        let address = Address::from_str("https://gitlab.com/user/repo.git").unwrap();
        assert!(matches!(address, Address::Git(_)));
    }

    #[test]
    fn test_address_from_str_invalid_format() {
        let result = Address::from_str("invalid://format");
        assert!(result.is_err());
        match result.unwrap_err() {
            AddrParseError::InvalidFormat(msg) => {
                assert_eq!(msg, "invalid://format");
            }
        }
    }

    #[test]
    fn test_address_from_str_trim_whitespace() {
        let address = Address::from_str("  https://example.com/file.txt  ").unwrap();
        assert!(matches!(address, Address::Http(_)));
    }

    // Address From<&str> 测试
    #[test]
    fn test_address_from_str_ref() {
        let address: Address = "https://example.com/file.txt".into();
        assert!(matches!(address, Address::Http(_)));
    }

    #[test]
    fn test_address_from_str_ref_fallback_to_local() {
        let address: Address = "invalid://format".into();
        assert!(matches!(address, Address::Local(_)));
    }
}
