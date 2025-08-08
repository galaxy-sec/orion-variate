use crate::{
    addr::{
        GitRepository, HttpResource,
        access_ctrl::{auth::AuthConfig, rule::Rule},
        proxy::ProxyConfig,
    },
    opt::OptionFrom,
    timeout::TimeoutConfig,
    vars::{EnvDict, EnvEvalable},
};
use derive_more::From;
use getset::Getters;
use log::info;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Unit {
    rules: Vec<Rule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth: Option<AuthConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<TimeoutConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxy: Option<ProxyConfig>,
}
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct UnitCtrl {
    auth: Option<AuthConfig>,
    timeout: Option<TimeoutConfig>,
    proxy: Option<ProxyConfig>,
}
impl UnitCtrl {
    pub fn new(
        auth: Option<AuthConfig>,
        timeout: Option<TimeoutConfig>,
        proxy: Option<ProxyConfig>,
    ) -> Self {
        Self {
            auth,
            timeout,
            proxy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub enum RedirectResult {
    Origin(String),
    Direct(String, Option<AuthConfig>),
}

impl RedirectResult {
    pub fn path(&self) -> &str {
        match self {
            RedirectResult::Origin(path) => path,
            RedirectResult::Direct(path, _) => path,
        }
    }

    pub fn is_proxy(&self) -> bool {
        match self {
            RedirectResult::Origin(_) => false,
            RedirectResult::Direct(_, _) => true,
        }
    }
}

impl Unit {
    pub fn new(rules: Vec<Rule>, auth: Option<AuthConfig>, proxy: Option<ProxyConfig>) -> Self {
        Self {
            rules,
            auth,
            timeout: Some(TimeoutConfig::http_simple()),
            proxy,
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn set_auth(&mut self, auth: AuthConfig) {
        self.auth = Some(auth);
    }

    pub fn set_timeout_config(&mut self, config: TimeoutConfig) {
        self.timeout = Some(config);
    }

    pub fn timeout_config_mut(&mut self) -> &mut Option<TimeoutConfig> {
        &mut self.timeout
    }

    pub fn redirect(&self, input: &str) -> RedirectResult {
        for rule in &self.rules {
            let result = rule.replace(input);
            if let Some(result) = result {
                return RedirectResult::Direct(result, self.auth.clone());
            }
        }
        RedirectResult::Origin(input.to_string())
    }

    pub fn direct_http_addr(&self, input: &HttpResource) -> Option<HttpResource> {
        for rule in &self.rules {
            let result = rule.replace(input.url());
            if let Some(result) = result {
                let mut direct = input.clone();
                direct.set_url(result);
                if let Some(auth) = self.auth() {
                    direct.set_username(auth.username().clone().to_opt());
                    direct.set_password(auth.password().clone().to_opt());
                }
                return Some(direct);
            }
        }
        None
    }

    pub fn direct_git_addr(&self, input: &GitRepository) -> Option<GitRepository> {
        for rule in &self.rules {
            let result = rule.replace(input.repo());
            if let Some(result) = result {
                info!(
                    target: "git",
                    "redirect to {result}, origin: {}",
                    input.repo()
                );
                let mut direct = input.clone();
                direct.set_repo(result);
                if let Some(auth) = self.auth() {
                    direct.set_username(auth.username().clone().to_opt());
                    direct.set_token(auth.password().clone().to_opt());
                }
                return Some(direct);
            }
        }
        None
    }

    pub fn make_example() -> Self {
        Self {
            rules: vec![Rule::new(
                "https://github.com/example/*",
                "https://mirror.example.com/*",
            )],
            auth: Some(AuthConfig::new(
                "username".to_string(),
                "password".to_string(),
            )),
            timeout: Some(TimeoutConfig::http_large_file()),
            proxy: None,
        }
    }
}

impl EnvEvalable<Unit> for Unit {
    fn env_eval(self, dict: &EnvDict) -> Unit {
        Unit {
            rules: self
                .rules
                .into_iter()
                .map(|rule| rule.env_eval(dict))
                .collect(),
            auth: self.auth.map(|auth| auth.env_eval(dict)),
            timeout: self.timeout,
            proxy: self.proxy.map(|x| x.env_eval(dict)),
        }
    }
}
