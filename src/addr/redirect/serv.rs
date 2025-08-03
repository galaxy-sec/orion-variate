use std::{path::PathBuf, rc::Rc};

use getset::Getters;
use orion_common::serde::Yamlable;
use orion_error::{ErrorOwe, ErrorWith};

use crate::addr::{
    AddrError,
    redirect::{
        auth::Auth,
        unit::{DirectPath, Unit},
    },
};

use super::rule::Rule;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct DirectServ {
    units: Vec<Unit>,
    enable: bool,
}

pub type ServHandle = Rc<DirectServ>;

impl DirectServ {
    pub fn new(units: Vec<Unit>, enable: bool) -> Self {
        Self { units, enable }
    }
    pub fn redirect(&self, url: &str) -> DirectPath {
        let mut path = DirectPath::Origin(url.to_string());
        for unit in &self.units {
            path = unit.proxy(path.path());
            if path.is_proxy() {
                break;
            }
        }
        path
    }
    pub fn from_rule(rule: Rule, auth: Option<Auth>) -> Self {
        let unit = Unit::new(vec![rule], auth);
        Self::new(vec![unit], true)
    }
}
impl TryFrom<&PathBuf> for DirectServ {
    type Error = AddrError;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        Ok(DirectServ::from_yml(value).owe_res().with(value)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_serv_serialization_basic() {
        let serv = DirectServ::new(vec![], false);
        let serialized = serde_yaml::to_string(&serv).unwrap();
        let deserialized: DirectServ = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.units().len(), 0);
        assert!(!deserialized.enable());
    }

    #[test]
    fn test_serv_serialization_with_units() {
        let auth = Some(Auth::new("test_user", "test_pass"));
        let rules = vec![
            Rule::new("https://github.com/*", "https://mirror.github.com/"),
            Rule::new("https://gitlab.com/*", "https://mirror.gitlab.com/"),
        ];
        let unit = Unit::new(rules, auth);
        let serv = DirectServ::new(vec![unit], true);

        let serialized = serde_json::to_string(&serv).unwrap();
        let deserialized: DirectServ = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.units().len(), 1);
        assert!(deserialized.enable());
        assert_eq!(deserialized.units()[0].rules().len(), 2);
        assert!(deserialized.units()[0].auth().is_some());
    }

    #[test]
    fn test_serv_serialization_yaml_format() {

        let yaml_content = r#"
units:
  - rules:
      - pattern: "https://example.com/*"
        target: "https://proxy.com/"
enable: true
"#;

        let deserialized: DirectServ = serde_yaml::from_str(yaml_content).unwrap();
        assert!(deserialized.enable());
        assert_eq!(deserialized.units().len(), 1);
        assert_eq!(
            deserialized.units()[0].rules()[0].pattern(),
            "https://example.com/*"
        );
    }

    #[test]
    fn test_serv_from_rule_serialization() {
        let rule = Rule::new("https://test.com/*", "https://redirect.com/");
        let auth = Some(Auth::new("admin", "secret"));
        let serv = DirectServ::from_rule(rule, auth);

        let serialized = serde_json::to_string_pretty(&serv).unwrap();
        let deserialized: DirectServ = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.enable());
        assert_eq!(deserialized.units().len(), 1);
        assert_eq!(deserialized.units()[0].rules().len(), 1);
        assert!(deserialized.units()[0].auth().is_some());
    }

    #[test]
    fn test_serv_multiple_units_serialization() {
        let unit1 = Unit::new(
            vec![Rule::new("https://api1.com/*", "https://proxy1.com/")],
            Some(Auth::new("user1", "pass1")),
        );

        let unit2 = Unit::new(
            vec![Rule::new("https://api2.com/*", "https://proxy2.com/")],
            None,
        );

        let unit3 = Unit::new(
            vec![
                Rule::new("https://api3.com/v1/*", "https://proxy3.com/v1/"),
                Rule::new("https://api3.com/v2/*", "https://proxy3.com/v2/"),
            ],
            Some(Auth::new("user3", "pass3")),
        );

        let serv = DirectServ::new(vec![unit1, unit2, unit3], true);

        let serialized = serde_yaml::to_string(&serv).unwrap();
        let deserialized: DirectServ = serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(deserialized.units().len(), 3);
        assert!(deserialized.enable());
    }

    #[test]
    fn test_serv_complex_yaml_structure() {
        let yaml_content = r#"
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://ghproxy.com/"
      - pattern: "https://raw.githubusercontent.com/*"
        target: "https://raw.ghproxy.com/"
    auth:
      username: "proxy_user"
      password: "proxy_pass"
  - rules:
      - pattern: "https://npmjs.com/*"
        target: "https://npmmirror.com/"
      - pattern: "https://registry.npmjs.org/*"
        target: "https://registry.npmmirror.com/"
enable: true
"#;

        let deserialized: DirectServ = serde_yaml::from_str(yaml_content).unwrap();

        assert_eq!(deserialized.units().len(), 2);
        assert!(deserialized.enable());

        let first_unit = &deserialized.units()[0];
        assert_eq!(first_unit.rules().len(), 2);
        assert_eq!(first_unit.rules()[0].pattern(), "https://github.com/*");
        assert_eq!(first_unit.rules()[0].target(), "https://ghproxy.com/");
        assert!(first_unit.auth().is_some());
        assert_eq!(first_unit.auth().as_ref().unwrap().username(), "proxy_user");

        let second_unit = &deserialized.units()[1];
        assert_eq!(second_unit.rules().len(), 2);
        assert!(second_unit.auth().is_none());
    }

    #[test]
    fn test_serv_empty_yaml() {
        let yaml_content = r#"
units: []
enable: false
"#;

        let deserialized: DirectServ = serde_yaml::from_str(yaml_content).unwrap();

        assert_eq!(deserialized.units().len(), 0);
        assert!(!deserialized.enable());
    }

    #[test]
    fn test_serv_json_format() {
        let json_content = r#"
{
  "units": [
    {
      "rules": [
        {
          "pattern": "https://test.example.com/*",
          "target": "https://proxy.example.com/"
        }
      ],
      "auth": {
        "username": "testuser",
        "password": "testpass"
      }
    }
  ],
  "enable": true
}
"#;

        let deserialized: DirectServ = serde_json::from_str(json_content).unwrap();

        assert_eq!(deserialized.units().len(), 1);
        assert!(deserialized.enable());
        assert_eq!(
            deserialized.units()[0].rules()[0].pattern(),
            "https://test.example.com/*"
        );
    }

    #[test]
    fn test_serv_redirect_functionality() {
        let rules = vec![Rule::new("https://github.com/*", "https://mirror.com/")];
        let unit = Unit::new(rules, None);
        let serv = DirectServ::new(vec![unit], true);

        let result = serv.redirect("https://github.com/user/repo");
        match result {
            DirectPath::Proxy(path, _) => {
                assert_eq!(path, "https://mirror.com/user/repo");
            }
            DirectPath::Origin(_) => panic!("Expected proxy path"),
        }
    }

    #[test]
    fn test_serv_no_redirect_match() {
        let rules = vec![Rule::new("https://github.com/*", "https://mirror.com/")];
        let unit = Unit::new(rules, None);
        let serv = DirectServ::new(vec![unit], true);

        let result = serv.redirect("https://gitlab.com/user/repo");
        match result {
            DirectPath::Origin(path) => {
                assert_eq!(path, "https://gitlab.com/user/repo");
            }
            DirectPath::Proxy(_, _) => panic!("Expected origin path"),
        }
    }

    #[test]
    fn test_serv_file_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_serv.yml");

        let rules = vec![Rule::new(
            "https://file-test.com/*",
            "https://file-proxy.com/",
        )];
        let unit = Unit::new(rules, Some(Auth::new("file_user", "file_pass")));
        let original_serv = DirectServ::new(vec![unit], true);

        // 写入文件
        original_serv.save_yml(&file_path).unwrap();

        // 从文件读取
        let loaded_serv = DirectServ::try_from(&file_path).unwrap();

        assert_eq!(loaded_serv.units().len(), original_serv.units().len());
        assert_eq!(loaded_serv.enable(), original_serv.enable());
    }
}
