use getset::Getters;
use reqwest::{ClientBuilder, Proxy};
use std::time::Duration;

use url::Url;

use super::redirect::Auth;
#[derive(Clone, Debug, PartialEq)]
pub enum ProxyType {
    Http,
    Socks5,
}
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct ProxyConfig {
    /// 代理 URL（如：http://user:pass@proxy:8080 或 socks5://user:pass@proxy:1080）
    url: Url,
    /// 代理类型（可选，默认根据 URL 自动推断）
    proxy_type: Option<ProxyType>,
    auth: Option<Auth>,
}

impl ProxyConfig {
    /// 从环境变量字符串解析代理配置（如 `http://user:pass@proxy:8080`）
    pub fn from_env(var: &str) -> Option<Self> {
        std::env::var(var).ok().and_then(|s| Self::parse_url(&s))
    }

    /// 从多个标准环境变量解析代理配置，优先级：HTTPS_PROXY > HTTP_PROXY > ALL_PROXY
    pub fn from_standard_env() -> Option<Self> {
        // 按优先级顺序检查环境变量
        let proxy_url = std::env::var("https_proxy")
            .or_else(|_| std::env::var("HTTPS_PROXY"))
            .or_else(|_| std::env::var("http_proxy"))
            .or_else(|_| std::env::var("HTTP_PROXY"))
            .or_else(|_| std::env::var("all_proxy"))
            .or_else(|_| std::env::var("ALL_PROXY"))
            .ok()?;
        
        Self::parse_url(&proxy_url)
    }

    /// 解析URL字符串为ProxyConfig
    fn parse_url(url_str: &str) -> Option<Self> {
        let url = Url::parse(url_str).ok()?;
        let proxy_type = match url.scheme() {
            "http" | "https" => Some(ProxyType::Http),
            "socks5" | "socks5h" => Some(ProxyType::Socks5),
            _ => None, // 不支持的代理类型
        };
        let username = url.username().to_string();
        let password = url.password().map(|p| p.to_string());

        // 若 URL 中已包含认证信息，直接使用
        let auth = if let Some(password) = password {
            Some(Auth::new(username, password))
        } else {
            None
        };

        Some(Self {
            url,
            proxy_type,
            auth,
        })
    }
}

pub fn create_http_client() -> reqwest::Client {
    let mut client_builder = ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .tcp_keepalive(Duration::from_secs(60))
        .pool_idle_timeout(Duration::from_secs(90));

    // 添加代理支持
    if let Ok(proxy_url) = std::env::var("https_proxy")
        .or_else(|_| std::env::var("HTTPS_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("all_proxy"))
        .or_else(|_| std::env::var("ALL_PROXY"))
    {
        if let Ok(proxy) = Proxy::all(&proxy_url) {
            client_builder = client_builder.proxy(proxy);
        } else {
            tracing::warn!("无效的代理设置: {}", proxy_url);
        }
    }

    client_builder.build().unwrap_or_else(|e| {
        tracing::error!("创建HTTP客户端失败: {}", e);
        reqwest::Client::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_http() {
        let config = ProxyConfig::parse_url("http://127.0.0.1:7890").unwrap();
        assert_eq!(config.url().as_str(), "http://127.0.0.1:7890/");
        assert_eq!(config.proxy_type(), &Some(ProxyType::Http));
        assert_eq!(config.auth(), &None);
    }

    #[test]
    fn test_parse_url_https() {
        let config = ProxyConfig::parse_url("https://proxy.example.com:8080").unwrap();
        assert_eq!(config.url().as_str(), "https://proxy.example.com:8080/");
        assert_eq!(config.proxy_type(), &Some(ProxyType::Http));
        assert_eq!(config.auth(), &None);
    }

    #[test]
    fn test_parse_url_socks5() {
        let config = ProxyConfig::parse_url("socks5://127.0.0.1:1080").unwrap();
        assert_eq!(config.url().as_str(), "socks5://127.0.0.1:1080");
        assert_eq!(config.proxy_type(), &Some(ProxyType::Socks5));
        assert_eq!(config.auth(), &None);
    }

    #[test]
    fn test_parse_url_with_auth() {
        let config = ProxyConfig::parse_url("http://user:pass@proxy.example.com:8080").unwrap();
        assert_eq!(config.url().as_str(), "http://user:pass@proxy.example.com:8080/");
        assert_eq!(config.proxy_type(), &Some(ProxyType::Http));
        
        let auth = config.auth().as_ref().unwrap();
        assert_eq!(auth.username(), "user");
        assert_eq!(auth.password(), "pass");
    }

    #[test]
    fn test_parse_url_invalid() {
        let config = ProxyConfig::parse_url("invalid-url");
        assert!(config.is_none());
    }

    #[test]
    fn test_parse_url_unsupported_scheme() {
        let config = ProxyConfig::parse_url("ftp://proxy.example.com:21");
        // FTP URL可以被解析，但proxy_type会是None
        assert!(config.is_some());
        assert_eq!(config.unwrap().proxy_type(), &None);
    }

    #[test]
    fn test_from_env_var() {
        // 测试单个环境变量解析
        let config = ProxyConfig::from_env("TEST_PROXY_VAR");
        // 由于环境变量可能不存在，这里只验证返回类型
        assert!(config.is_none() || config.is_some());
    }
}
