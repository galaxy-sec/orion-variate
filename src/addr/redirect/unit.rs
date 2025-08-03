use crate::{
    addr::{
        GitRepository, HttpResource,
        redirect::{auth::AuthConfig, rule::Rule},
    },
    opt::OptionFrom,
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
    pub fn new(rules: Vec<Rule>, auth: Option<AuthConfig>) -> Self {
        Self { rules, auth }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn set_auth(&mut self, auth: AuthConfig) {
        self.auth = Some(auth);
    }

    pub fn proxy(&self, input: &str) -> RedirectResult {
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
    //GitAddr
    pub fn direct_git_addr(&self, input: &GitRepository) -> Option<GitRepository> {
        for rule in &self.rules {
            let result = rule.replace(input.repo());
            if let Some(result) = result {
                info!(target:"git", "redirect to {result}, origin: {}", input.repo());
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
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_new() {
        let rules = vec![Rule::new("https://github.com/*", "https://mirror.com/")];
        let auth = Some(AuthConfig::new("user".to_string(), "pass".to_string()));
        let unit = Unit::new(rules.clone(), auth.clone());

        assert_eq!(unit.rules().len(), 1);
        assert!(unit.auth().is_some());
    }

    #[test]
    fn test_unit_without_auth() {
        let rules = vec![Rule::new("https://github.com/*", "https://mirror.com/")];
        let unit = Unit::new(rules, None);

        assert_eq!(unit.rules().len(), 1);
        assert!(unit.auth().is_none());
    }

    #[test]
    fn test_unit_add_rule() {
        let mut unit = Unit::new(vec![], None);
        unit.add_rule(Rule::new("https://github.com/*", "https://mirror.com/"));

        assert_eq!(unit.rules().len(), 1);
    }

    #[test]
    fn test_unit_set_auth() {
        let mut unit = Unit::new(vec![], None);
        unit.set_auth(AuthConfig::new("user".to_string(), "pass".to_string()));

        assert!(unit.auth().is_some());
    }

    #[test]
    fn test_unit_serialize_deserialize() {
        let rules = vec![Rule::new("https://github.com/*", "https://mirror.com/")];
        let auth = Some(AuthConfig::new("user".to_string(), "pass".to_string()));
        let unit = Unit::new(rules, auth);

        let serialized = serde_json::to_string(&unit).unwrap();
        let deserialized: Unit = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.rules().len(), 1);
        assert!(deserialized.auth().is_some());
    }
}
