use crate::{predule::*, vars::EnvDict};

use crate::vars::EnvEvalable;

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "local")]
pub struct LocalPath {
    path: String,
}

impl EnvEvalable<LocalPath> for LocalPath {
    fn env_eval(self, dict: &EnvDict) -> LocalPath {
        Self {
            path: self.path.env_eval(dict),
        }
    }
}
impl From<&str> for LocalPath {
    fn from(value: &str) -> Self {
        Self {
            path: value.to_string(),
        }
    }
}
