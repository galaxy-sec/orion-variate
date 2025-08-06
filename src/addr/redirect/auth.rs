use getset::Getters;
use serde_derive::{Deserialize, Serialize};
use crate::vars::{EnvDict, EnvEvalable};
#[derive(Debug, Clone, Serialize, Deserialize, Getters, PartialEq)]
#[getset(get = "pub")]
pub struct AuthConfig {
    username: String,
    password: String,
}

impl AuthConfig {
    pub fn new<S: Into<String>>(username: S, password: S) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
    pub fn make_example() -> Self {
        Self {
            username: "galaxy".into(),
            password: "this-is-password".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vars::EnvDict;

    #[test]
    fn test_auth_config_env_eval() {
        use crate::vars::{EnvDict, ValueType};

        let mut env_dict = EnvDict::new();
        env_dict.insert("USERNAME".to_string(), ValueType::String("test_user".to_string()));
        env_dict.insert("PASSWORD".to_string(), ValueType::String("test_pass".to_string()));

        let auth = AuthConfig::new("${USERNAME}", "${PASSWORD}");
        let evaluated = auth.env_eval(&env_dict);

        assert_eq!(evaluated.username(), "test_user");
        assert_eq!(evaluated.password(), "test_pass");
    }

    #[test]
    fn test_auth_config_env_eval_with_defaults() {
        use crate::vars::{EnvDict, ValueType};

        let env_dict = EnvDict::new();

        let auth = AuthConfig::new("${MISSING_USER:default_user}", "${MISSING_PASS:default_pass}");
        let evaluated = auth.env_eval(&env_dict);

        assert_eq!(evaluated.username(), "default_user");
        assert_eq!(evaluated.password(), "default_pass");
    }
}

impl EnvEvalable<AuthConfig> for AuthConfig {
    fn env_eval(self, dict: &EnvDict) -> AuthConfig {
        AuthConfig {
            username: self.username.env_eval(dict),
            password: self.password.env_eval(dict),
        }
    }
}
