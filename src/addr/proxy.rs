use getset::Getters;

use crate::vars::EnvEvalable;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Socks5,
}
use serde_derive::{Deserialize, Serialize};
#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct ProxyConfig {
    url: String,
}
impl ProxyConfig {
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self { url: url.into() }
    }
}

impl EnvEvalable<ProxyConfig> for ProxyConfig {
    fn env_eval(self, dict: &crate::vars::EnvDict) -> ProxyConfig {
        Self {
            url: self.url.env_eval(dict),
        }
    }
}
