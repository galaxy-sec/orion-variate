use crate::{
    addr::{
        GitRepository, HttpResource,
        redirect::{auth::AuthConfig, rule::Rule},
    },
    opt::OptionFrom,
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

impl EnvEvalable<Unit> for Unit {
    fn env_eval(self, dict: &EnvDict) -> Unit {
        Unit {
            rules: self
                .rules
                .into_iter()
                .map(|rule| rule.env_eval(dict))
                .collect(),
            auth: self.auth.map(|auth| auth.env_eval(dict)),
        }
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

    #[test]
    fn test_unit() {
        let unit = Unit::new(
            vec![Rule::new(
                "https://github.com/galaxy-sec/galaxy-flow*",
                "https://gflow.com",
            )],
            None,
        );
        let git = GitRepository::from("https://github.com/galaxy-sec/galaxy-flow");
        let result = unit.direct_git_addr(&git);
        assert!(result.is_some());
        assert_eq!(result.unwrap().repo(), "https://gflow.com");
    }

    #[test]
    fn test_unit_env_eval() {
        use crate::vars::{EnvDict, ValueType};

        let mut env_dict = EnvDict::new();
        env_dict.insert(
            "DOMAIN".to_string(),
            ValueType::String("example.com".to_string()),
        );
        env_dict.insert(
            "TARGET".to_string(),
            ValueType::String("redirect.com".to_string()),
        );
        env_dict.insert(
            "USERNAME".to_string(),
            ValueType::String("test_user".to_string()),
        );
        env_dict.insert(
            "PASSWORD".to_string(),
            ValueType::String("test_pass".to_string()),
        );

        let mut unit = Unit::new(
            vec![Rule::new("https://${DOMAIN}/*", "https://${TARGET}")],
            None,
        );
        unit.set_auth(AuthConfig::new(
            "${USERNAME}".to_string(),
            "${PASSWORD}".to_string(),
        ));

        let evaluated = unit.env_eval(&env_dict);

        assert_eq!(evaluated.rules().len(), 1);
        assert_eq!(evaluated.rules()[0].pattern(), "https://example.com/*");
        assert_eq!(evaluated.rules()[0].target(), "https://redirect.com");
        assert!(evaluated.auth().is_some());
        assert_eq!(evaluated.auth().as_ref().unwrap().username(), "test_user");
        assert_eq!(evaluated.auth().as_ref().unwrap().password(), "test_pass");
    }

    #[test]
    fn test_unit_env_eval_without_auth() {
        use crate::vars::{EnvDict, ValueType};

        let mut env_dict = EnvDict::new();
        env_dict.insert(
            "DOMAIN".to_string(),
            ValueType::String("example.com".to_string()),
        );

        let unit = Unit::new(
            vec![Rule::new("https://${DOMAIN}/*", "https://redirect.com")],
            None,
        );

        let evaluated = unit.env_eval(&env_dict);

        assert_eq!(evaluated.rules().len(), 1);
        assert_eq!(evaluated.rules()[0].pattern(), "https://example.com/*");
        assert!(evaluated.auth().is_none());
    }
}
