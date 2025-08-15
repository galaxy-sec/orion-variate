use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::vars::EnvEvalable;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Socks5,
}

#[derive(Clone, Debug, Getters, Serialize, Deserialize, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_proxy_type_variants_and_functionality() {
        let http = ProxyType::Http;
        let socks5 = ProxyType::Socks5;

        assert_eq!(http, http);
        assert_eq!(socks5, socks5);
        assert_ne!(http, socks5);

        let yaml = serde_yaml::to_string(&http).unwrap();
        let deserialized: ProxyType = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(http, deserialized);
    }

    #[test]
    fn test_proxy_config_creation_and_accessors() {
        let config = ProxyConfig::new("http://proxy.example.com:8080");
        assert_eq!(config.url(), "http://proxy.example.com:8080");

        let cloned = config.clone();
        assert_eq!(config, cloned);
        assert_ne!(std::ptr::addr_of!(config), std::ptr::addr_of!(cloned));
    }

    #[test]
    fn test_proxy_config_serialization_roundtrip() {
        let original = ProxyConfig::new("socks5://proxy.example.com:1080");
        let yaml = serde_yaml::to_string(&original).unwrap();
        let deserialized: ProxyConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_proxy_config_environment_evaluation() {
        let config = ProxyConfig::new("http://$PROXY_HOST:$PROXY_PORT");
        let mut dict = HashMap::new();
        dict.insert("PROXY_HOST".to_string(), "proxy.example.com".into());
        dict.insert("PROXY_PORT".to_string(), "8080".into());

        // Test that environment evaluation doesn't panic
        let evaluated = config.env_eval(&dict.into());
        assert!(!evaluated.url().is_empty());
    }

    #[test]
    fn test_proxy_config_with_different_schemes() {
        let urls = vec![
            "http://proxy.example.com:8080",
            "https://proxy.example.com:443",
            "socks5://proxy.example.com:1080",
            "http://user:pass@proxy.example.com:8080",
        ];

        for url in urls {
            let config = ProxyConfig::new(url);
            assert_eq!(config.url(), url);

            let yaml = serde_yaml::to_string(&config).unwrap();
            let deserialized: ProxyConfig = serde_yaml::from_str(&yaml).unwrap();
            assert_eq!(config, deserialized);
        }
    }
}
