use crate::{predule::*, vars::EnvDict};

use getset::{Getters, Setters, WithSetters};
use url::Url;

use crate::vars::EnvEvalable;

#[derive(Getters, Clone, Debug, Serialize, Deserialize, WithSetters, Setters)]
#[getset(get = "pub", set = "pub")]
#[serde(rename = "http")]
pub struct HttpAddr {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
}

impl PartialEq for HttpAddr {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url && self.username == other.username && self.password == other.password
    }
}

impl Eq for HttpAddr {}

impl EnvEvalable<HttpAddr> for HttpAddr {
    fn env_eval(self, dict: &EnvDict) -> HttpAddr {
        Self {
            url: self.url.env_eval(dict),
            username: self.username.env_eval(dict),
            password: self.password.env_eval(dict),
        }
    }
}

impl HttpAddr {
    pub fn from<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            username: None,
            password: None,
        }
    }

    pub fn with_credentials<S: Into<String>>(mut self, username: S, password: S) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }
}

pub fn filename_of_url(url: &str) -> Option<String> {
    let parsed_url = Url::parse(url).ok()?;
    parsed_url.path_segments()?.next_back().and_then(|s| {
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    })
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_get_filename_with_regular_url() {
        assert_eq!(
            filename_of_url("http://example.com/file.txt"),
            Some("file.txt".to_string())
        );
    }

    #[test]
    fn test_get_filename_with_query_params() {
        assert_eq!(
            filename_of_url("http://example.com/file.txt?version=1.0"),
            Some("file.txt".to_string())
        );
    }

    #[test]
    fn test_get_filename_with_fragment() {
        assert_eq!(
            filename_of_url("http://example.com/file.txt#section1"),
            Some("file.txt".to_string())
        );
    }

    #[test]
    fn test_get_filename_with_multiple_path_segments() {
        assert_eq!(
            filename_of_url("http://example.com/path/to/file.txt"),
            Some("file.txt".to_string())
        );
    }

    #[test]
    fn test_get_filename_with_trailing_slash() {
        assert_eq!(filename_of_url("http://example.com/path/"), None);
    }

    #[test]
    fn test_get_filename_with_empty_path() {
        assert_eq!(filename_of_url("http://example.com"), None);
    }

    #[test]
    fn test_get_filename_with_invalid_url() {
        assert_eq!(filename_of_url("not a valid url"), None);
    }

    #[test]
    fn test_get_filename_with_encoded_characters() {
        assert_eq!(
            filename_of_url("http://example.com/file%20name.txt"),
            Some("file%20name.txt".to_string())
        );
    }
}
