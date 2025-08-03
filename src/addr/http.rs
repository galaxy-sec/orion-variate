use crate::{
    addr::{
        proxy::create_http_client,
        redirect::{DirectPath, serv::DirectServ},
    },
    predule::*,
    types::RemoteUpdate,
    update::UpdateOptions,
    vars::EnvDict,
};

use getset::{Getters, WithSetters};
use orion_error::UvsResFrom;
use tokio::io::AsyncWriteExt;
use tracing::info;
use url::Url;

use crate::{types::LocalUpdate, vars::EnvEvalable};

use super::AddrResult;

#[derive(Getters, Clone, Debug, Serialize, Deserialize, WithSetters)]
#[getset(get = "pub")]
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








