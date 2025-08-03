use crate::{predule::*, vars::EnvDict};

use crate::vars::EnvEvalable;

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "local")]
pub struct LocalAddr {
    path: String,
}

impl EnvEvalable<LocalAddr> for LocalAddr {
    fn env_eval(self, dict: &EnvDict) -> LocalAddr {
        Self {
            path: self.path.env_eval(dict),
        }
    }
}
impl From<&str> for LocalAddr {
    fn from(value: &str) -> Self {
        Self {
            path: value.to_string(),
        }
    }
}
