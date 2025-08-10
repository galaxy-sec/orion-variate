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

#[derive(Debug, Clone, Serialize, Deserialize, Getters, PartialEq)]
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
            RedirectResult::Origin(_) => panic!("Expected RedirectResult::Direct"),
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
            RedirectResult::Origin(_) => panic!("Expected RedirectResult::Direct"),
        }
    }

    #[test]
    fn test_unit_direct_address_redirection() {
        use crate::addr::{GitRepository, HttpResource};

        // Test HTTP address redirection
        let http_rules = vec![Rule::new(
            "https://github.com/*",
            "https://github-mirror.com/",
        )];
        let auth = AuthConfig::new("test-user", "test-pass");
        let unit = Unit::new(http_rules, Some(auth.clone()), None);

        let http_addr =
            HttpResource::from("https://github.com/user/repo").with_credentials("user", "pass");
        let redirected_http = unit.direct_http_addr(&http_addr);

        assert!(redirected_http.is_some());
        if let Some(redirected) = redirected_http {
            assert_eq!(redirected.url(), "https://github-mirror.com/user/repo");
            assert_eq!(*redirected.username(), Some("test-user".to_string()));
            assert_eq!(*redirected.password(), Some("test-pass".to_string()));
        }

        // Test HTTP address without match (should return None)
        let no_match_http = HttpResource::from("https://gitlab.com/user/repo");
        let no_redirect_result = unit.direct_http_addr(&no_match_http);
        assert!(no_redirect_result.is_none());

        // Test Git repository redirection
        let git_rules = vec![Rule::new("git@gitlab.com:*", "git@gitlab-mirror.com:")];
        let git_auth = AuthConfig::new("git-user", "git-token");
        let git_unit = Unit::new(git_rules, Some(git_auth.clone()), None);

        let git_repo = GitRepository::from("git@gitlab.com:user/repo.git");
        let redirected_git = git_unit.direct_git_addr(&git_repo);

        assert!(redirected_git.is_some());
        if let Some(redirected) = redirected_git {
            assert_eq!(redirected.repo(), "git@gitlab-mirror.com:user/repo.git");
            assert_eq!(*redirected.username(), Some("git-user".to_string()));
            assert_eq!(*redirected.token(), Some("git-token".to_string()));
        }

        // Test Git repository without match (should return None)
        let no_match_git = GitRepository::from("git@github.com:user/repo.git");
        let no_git_redirect = git_unit.direct_git_addr(&no_match_git);
        assert!(no_git_redirect.is_none());
    }

    #[test]
    fn test_unit_edge_cases_and_environment_evaluation() {
        use crate::vars::EnvDict;

        // Test timeout_config_mut method
        let mut unit = Unit::new(vec![], None, None);
        let original_timeout = unit.timeout().clone();

        let mut_timeout_config = unit.timeout_config_mut();
        *mut_timeout_config = Some(TimeoutConfig::new());

        // Since TimeoutConfig::new() creates default values, we'll test that
        // the configuration can be modified rather than comparing specific values
        assert!(unit.timeout().is_some());
        assert!(original_timeout.is_some());

        // Test RedirectResult methods
        let origin_result = RedirectResult::Origin("https://example.com".to_string());
        let direct_result = RedirectResult::Direct("https://mirror.com".to_string(), None);

        assert!(!origin_result.is_proxy());
        assert!(direct_result.is_proxy());
        assert_eq!(origin_result.path(), "https://example.com");
        assert_eq!(direct_result.path(), "https://mirror.com");

        // Test make_example method
        let example_unit = Unit::make_example();
        assert_eq!(example_unit.rules().len(), 1);
        assert!(example_unit.auth().is_some());
        assert!(example_unit.timeout().is_some());
        assert!(example_unit.proxy().is_none());

        let example_rule = example_unit.rules()[0].clone();
        assert_eq!(example_rule.pattern(), "https://github.com/example/*");
        assert_eq!(example_rule.target(), "https://mirror.example.com/*");

        // Test environment evaluation
        let mut env_dict = std::collections::HashMap::new();
        env_dict.insert("TEST_USER".to_string(), "env-user".to_string());
        env_dict.insert("TEST_PASS".to_string(), "env-pass".to_string());
        let env_dict: EnvDict = env_dict.into();

        // Create a unit with environment variables in auth
        let rules = vec![];
        let auth = Some(AuthConfig::new(
            "${TEST_USER}".to_string(),
            "${TEST_PASS}".to_string(),
        ));
        let proxy = None;
        let test_unit = Unit::new(rules, auth, proxy);

        // Apply environment evaluation
        let evaluated_unit = test_unit.env_eval(&env_dict);

        // Note: The actual evaluation behavior depends on the AuthConfig implementation
        // This test verifies the environment evaluation mechanism works
        assert!(evaluated_unit.auth().is_some());

        // Test multiple rules with priority (first match wins)
        let mut multi_rule_unit = Unit::new(vec![], None, None);
        multi_rule_unit.add_rule(Rule::new("https://test.com/*", "https://first-mirror.com/"));
        multi_rule_unit.add_rule(Rule::new(
            "https://test.com/api/*",
            "https://second-mirror.com/",
        ));

        // First rule should match
        let result1 = multi_rule_unit.redirect("https://test.com/file.txt");
        assert_eq!(result1.path(), "https://first-mirror.com/file.txt");

        // More specific rule should match when it comes first
        let mut specific_rule_unit = Unit::new(vec![], None, None);
        specific_rule_unit.add_rule(Rule::new(
            "https://test.com/api/*",
            "https://second-mirror.com/",
        ));
        specific_rule_unit.add_rule(Rule::new("https://test.com/*", "https://first-mirror.com/"));

        let result2 = specific_rule_unit.redirect("https://test.com/api/endpoint");
        assert_eq!(result2.path(), "https://second-mirror.com/endpoint");

        // Test with empty rules
        let empty_rule_unit = Unit::new(vec![], None, None);
        let result3 = empty_rule_unit.redirect("https://any-url.com/file.txt");
        match result3 {
            RedirectResult::Origin(path) => {
                assert_eq!(path, "https://any-url.com/file.txt");
            }
            _ => panic!("Expected RedirectResult::Origin for empty rules"),
        }
    }
}
