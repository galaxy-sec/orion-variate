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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vars::EnvDict;
    use std::collections::HashMap;

    #[test]
    fn test_local_path_from_str() {
        let local_path = LocalPath::from("/path/to/file");
        assert_eq!(local_path.path, "/path/to/file");
    }

    #[test]
    fn test_local_path_env_eval_no_vars() {
        let local_path = LocalPath::from("/static/path");
        let dict = EnvDict::new();
        let result = local_path.env_eval(&dict);
        assert_eq!(result.path, "/static/path");
    }

    #[test]
    fn test_local_path_env_eval_with_vars() {
        let local_path = LocalPath::from("${HOME}/project");
        let mut dict = HashMap::new();
        dict.insert("HOME".to_string(), "/Users/test".to_string());
        let env_dict = EnvDict::from(dict);
        let result = local_path.env_eval(&env_dict);
        assert_eq!(result.path, "/Users/test/project");
    }

    #[test]
    fn test_local_path_env_eval_multiple_vars() {
        let local_path = LocalPath::from("${HOME}/${PROJECT_NAME}/src");
        let mut dict = HashMap::new();
        dict.insert("HOME".to_string(), "/Users/test".to_string());
        dict.insert("PROJECT_NAME".to_string(), "my-project".to_string());
        let env_dict = EnvDict::from(dict);
        let result = local_path.env_eval(&env_dict);
        assert_eq!(result.path, "/Users/test/my-project/src");
    }

    #[test]
    fn test_local_path_env_eval_undefined_var() {
        let local_path = LocalPath::from("${UNDEFINED_VAR}/path");
        let dict = EnvDict::new();
        let result = local_path.env_eval(&dict);
        // 假设未定义的变量会被保留原样
        assert_eq!(result.path, "${UNDEFINED_VAR}/path");
    }

    #[test]
    fn test_local_path_clone() {
        let local_path = LocalPath::from("/original/path");
        let cloned = local_path.clone();
        assert_eq!(local_path.path, cloned.path);
    }

    #[test]
    fn test_local_path_partial_eq() {
        let path1 = LocalPath::from("/same/path");
        let path2 = LocalPath::from("/same/path");
        let path3 = LocalPath::from("/different/path");

        assert_eq!(path1, path2);
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_local_path_debug() {
        let local_path = LocalPath::from("/debug/path");
        let debug_str = format!("{:?}", local_path);
        assert!(debug_str.contains("LocalPath"));
        assert!(debug_str.contains("/debug/path"));
    }
}
