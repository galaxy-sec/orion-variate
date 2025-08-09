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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::access_ctrl::auth::AuthConfig;
    use crate::addr::access_ctrl::rule::Rule;
    use crate::addr::proxy::ProxyConfig;

    #[test]
    fn test_unit_new() {
        let rules = vec![Rule::new("https://example.com/*", "https://mirror.com/*")];
        let auth = AuthConfig::new("user", "pass");
        let proxy = ProxyConfig::new("http://proxy.example.com:8080");

        let unit = Unit::new(rules.clone(), Some(auth.clone()), Some(proxy.clone()));

        assert_eq!(unit.rules(), &rules);
        assert_eq!(unit.auth(), &Some(auth));
        assert!(unit.timeout().is_some());
        assert_eq!(unit.proxy(), &Some(proxy));
    }

    #[test]
    fn test_unit_add_rule() {
        let mut unit = Unit::new(vec![], None, None);
        let rule = Rule::new("https://example.com/*", "https://mirror.com/*");

        unit.add_rule(rule.clone());

        assert_eq!(unit.rules().len(), 1);
        assert_eq!(unit.rules()[0], rule);
    }

    #[test]
    fn test_unit_set_auth() {
        let mut unit = Unit::new(vec![], None, None);
        let auth = AuthConfig::new("user", "pass");

        unit.set_auth(auth.clone());

        assert_eq!(unit.auth(), &Some(auth));
    }

    #[test]
    fn test_unit_set_timeout_config() {
        let mut unit = Unit::new(vec![], None, None);
        let timeout_config = TimeoutConfig::http_large_file();

        unit.set_timeout_config(timeout_config.clone());

        assert_eq!(unit.timeout(), &Some(timeout_config));
    }

    #[test]
    fn test_unit_redirect_with_match() {
        let rules = vec![Rule::new("https://example.com/*", "https://mirror.com/")];
        let unit = Unit::new(rules, None, None);

        let result = unit.redirect("https://example.com/file.txt");

        match result {
            RedirectResult::Direct(path, auth) => {
                assert_eq!(path, "https://mirror.com/file.txt");
                assert_eq!(auth, None);
            }
            _ => panic!("Expected RedirectResult::Direct"),
        }
    }

    #[test]
    fn test_unit_redirect_without_match() {
        let rules = vec![Rule::new("https://example.com/*", "https://mirror.com/")];
        let unit = Unit::new(rules, None, None);

        let result = unit.redirect("https://other.com/file.txt");

        match result {
            RedirectResult::Origin(path) => {
                assert_eq!(path, "https://other.com/file.txt");
            }
            _ => panic!("Expected RedirectResult::Origin"),
        }
    }

    #[test]
    fn test_unit_redirect_with_auth() {
        let rules = vec![Rule::new("https://example.com/*", "https://mirror.com/")];
        let auth = AuthConfig::new("user", "pass");
        let unit = Unit::new(rules, Some(auth.clone()), None);

        let result = unit.redirect("https://example.com/file.txt");

        match result {
            RedirectResult::Direct(path, result_auth) => {
                assert_eq!(path, "https://mirror.com/file.txt");
                assert_eq!(result_auth, Some(auth));
            }
            _ => panic!("Expected RedirectResult::Direct"),
        }
    }
}
