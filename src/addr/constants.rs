//! 地址相关常量定义模块
//!
//! 该模块集中管理地址处理过程中使用的所有常量，
//! 包括Git仓库、HTTP资源、本地路径等的默认配置和常用值。

/// Git仓库相关常量
pub mod git {
    /// Git仓库默认分支名
    pub const DEFAULT_BRANCH: &str = "main";

    /// Git仓库默认远程名称
    pub const DEFAULT_REMOTE: &str = "origin";

    /// 常用Git托管服务域名
    pub const GITHUB_DOMAIN: &str = "github.com";
    pub const GITLAB_DOMAIN: &str = "gitlab.com";
    pub const GITEA_DOMAIN: &str = "gitea.io";

    /// Git配置文件名
    pub const GIT_CONFIG_FILE: &str = ".git/config";
    pub const GIT_CREDENTIALS_FILE: &str = ".git-credentials";

    /// SSH密钥默认路径
    pub const SSH_KEY_FILE: &str = ".ssh/id_rsa";
    pub const SSH_CONFIG_FILE: &str = ".ssh/config";

    /// Git协议前缀
    pub const HTTPS_PREFIX: &str = "https://";
    pub const SSH_PREFIX: &str = "git@";
    pub const GIT_PROTOCOL: &str = "git://";
}

/// HTTP资源相关常量
pub mod http {
    /// 默认HTTP连接超时时间（秒）
    pub const DEFAULT_TIMEOUT: u64 = 30;

    /// 默认重试次数
    pub const DEFAULT_RETRIES: u32 = 3;

    /// 默认User-Agent
    pub const DEFAULT_USER_AGENT: &str = "orion-variate/1.0";

    /// 常用HTTP状态码
    pub mod status {
        pub const OK: u16 = 200;
        pub const NOT_FOUND: u16 = 404;
        pub const UNAUTHORIZED: u16 = 401;
        pub const FORBIDDEN: u16 = 403;
        pub const INTERNAL_ERROR: u16 = 500;
    }

    /// 常用文件扩展名
    pub mod extensions {
        pub const ZIP: &str = "zip";
        pub const TAR: &str = "tar";
        pub const TAR_GZ: &str = "tar.gz";
        pub const TAR_XZ: &str = "tar.xz";
        pub const TAR_BZ2: &str = "tar.bz2";
        pub const JSON: &str = "json";
        pub const YAML: &str = "yaml";
        pub const YML: &str = "yml";
        pub const XML: &str = "xml";
    }

    /// 常用MIME类型
    pub mod mime {
        pub const ZIP: &str = "application/zip";
        pub const TAR: &str = "application/x-tar";
        pub const GZIP: &str = "application/gzip";
        pub const JSON: &str = "application/json";
        pub const YAML: &str = "application/yaml";
        pub const XML: &str = "application/xml";
        pub const OCTET_STREAM: &str = "application/octet-stream";
    }
}

/// 本地路径相关常量
pub mod local {
    /// 默认临时目录名
    pub const TEMP_DIR: &str = ".tmp";

    /// 默认缓存目录名
    pub const CACHE_DIR: &str = ".cache";

    /// 配置文件名
    pub const CONFIG_FILE: &str = "config.yaml";
    pub const CONFIG_YML_FILE: &str = "config.yml";
    pub const CONFIG_JSON_FILE: &str = "config.json";

    /// 锁文件名
    pub const LOCK_FILE: &str = ".lock";

    /// 隐藏目录前缀
    pub const HIDDEN_PREFIX: &str = ".";

    /// 路径分隔符
    pub const PATH_SEPARATOR: char = '/';
    #[cfg(target_os = "windows")]
    pub const ALT_PATH_SEPARATOR: char = '\\';
}

/// 重定向相关常量
pub mod redirect {
    /// 默认重定向配置文件名
    pub const CONFIG_FILE: &str = "redirect-rules.yaml";
    pub const CONFIG_YML_FILE: &str = "redirect-rules.yml";

    /// 环境变量前缀
    pub const ENV_PREFIX: &str = "ORION_VARIATE_";

    /// 最大重定向次数
    pub const MAX_REDIRECTS: u32 = 10;

    /// 重定向规则匹配模式
    pub mod patterns {
        pub const WILDCARD: &str = "*";
        pub const EXACT: &str = "=";
        pub const PREFIX: &str = "^";
        pub const SUFFIX: &str = "$";
    }
}

/// 环境变量相关常量
pub mod env {
    /// Git相关环境变量
    pub const GITHUB_TOKEN: &str = "GITHUB_TOKEN";
    pub const GITLAB_TOKEN: &str = "GITLAB_TOKEN";
    pub const GITEA_TOKEN: &str = "GITEA_TOKEN";
    pub const GIT_USERNAME: &str = "GIT_USERNAME";
    pub const GIT_PASSWORD: &str = "GIT_PASSWORD";

    /// HTTP代理相关环境变量
    pub const HTTP_PROXY: &str = "HTTP_PROXY";
    pub const HTTPS_PROXY: &str = "HTTPS_PROXY";
    pub const NO_PROXY: &str = "NO_PROXY";

    /// 配置相关环境变量
    pub const CONFIG_PATH: &str = "ORION_VARIATE_CONFIG";
    pub const REDIRECT_RULES_PATH: &str = "ORION_VARIATE_REDIRECT_RULES";
    pub const CACHE_DIR: &str = "ORION_VARIATE_CACHE_DIR";
    pub const TEMP_DIR: &str = "ORION_VARIATE_TEMP_DIR";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_constants() {
        assert_eq!(git::DEFAULT_BRANCH, "main");
        assert_eq!(git::DEFAULT_REMOTE, "origin");
        assert_eq!(git::GITHUB_DOMAIN, "github.com");
        assert_eq!(git::GITLAB_DOMAIN, "gitlab.com");
        assert_eq!(git::GITEA_DOMAIN, "gitea.io");
        assert_eq!(git::GIT_CONFIG_FILE, ".git/config");
        assert_eq!(git::GIT_CREDENTIALS_FILE, ".git-credentials");
        assert_eq!(git::SSH_KEY_FILE, ".ssh/id_rsa");
        assert_eq!(git::SSH_CONFIG_FILE, ".ssh/config");
        assert_eq!(git::HTTPS_PREFIX, "https://");
        assert_eq!(git::SSH_PREFIX, "git@");
        assert_eq!(git::GIT_PROTOCOL, "git://");
    }

    #[test]
    fn test_http_constants() {
        assert_eq!(http::DEFAULT_TIMEOUT, 30);
        assert_eq!(http::DEFAULT_RETRIES, 3);
        assert_eq!(http::DEFAULT_USER_AGENT, "orion-variate/1.0");
        assert_eq!(http::status::OK, 200);
        assert_eq!(http::status::NOT_FOUND, 404);
        assert_eq!(http::status::UNAUTHORIZED, 401);
        assert_eq!(http::status::FORBIDDEN, 403);
        assert_eq!(http::status::INTERNAL_ERROR, 500);
        assert_eq!(http::extensions::ZIP, "zip");
        assert_eq!(http::extensions::TAR, "tar");
        assert_eq!(http::extensions::TAR_GZ, "tar.gz");
        assert_eq!(http::extensions::TAR_XZ, "tar.xz");
        assert_eq!(http::extensions::TAR_BZ2, "tar.bz2");
        assert_eq!(http::extensions::JSON, "json");
        assert_eq!(http::extensions::YAML, "yaml");
        assert_eq!(http::extensions::YML, "yml");
        assert_eq!(http::extensions::XML, "xml");
        assert_eq!(http::mime::ZIP, "application/zip");
        assert_eq!(http::mime::TAR, "application/x-tar");
        assert_eq!(http::mime::GZIP, "application/gzip");
        assert_eq!(http::mime::JSON, "application/json");
        assert_eq!(http::mime::YAML, "application/yaml");
        assert_eq!(http::mime::XML, "application/xml");
        assert_eq!(http::mime::OCTET_STREAM, "application/octet-stream");
    }

    #[test]
    fn test_local_constants() {
        assert_eq!(local::TEMP_DIR, ".tmp");
        assert_eq!(local::CACHE_DIR, ".cache");
        assert_eq!(local::CONFIG_FILE, "config.yaml");
        assert_eq!(local::CONFIG_YML_FILE, "config.yml");
        assert_eq!(local::CONFIG_JSON_FILE, "config.json");
        assert_eq!(local::LOCK_FILE, ".lock");
        assert_eq!(local::HIDDEN_PREFIX, ".");
        assert_eq!(local::PATH_SEPARATOR, '/');
        #[cfg(target_os = "windows")]
        assert_eq!(local::ALT_PATH_SEPARATOR, '\\');
    }

    #[test]
    fn test_redirect_constants() {
        assert_eq!(redirect::CONFIG_FILE, "redirect-rules.yaml");
        assert_eq!(redirect::CONFIG_YML_FILE, "redirect-rules.yml");
        assert_eq!(redirect::ENV_PREFIX, "ORION_VARIATE_");
        assert_eq!(redirect::MAX_REDIRECTS, 10);
        assert_eq!(redirect::patterns::WILDCARD, "*");
        assert_eq!(redirect::patterns::EXACT, "=");
        assert_eq!(redirect::patterns::PREFIX, "^");
        assert_eq!(redirect::patterns::SUFFIX, "$");
    }

    #[test]
    fn test_env_constants() {
        assert_eq!(env::GITHUB_TOKEN, "GITHUB_TOKEN");
        assert_eq!(env::GITLAB_TOKEN, "GITLAB_TOKEN");
        assert_eq!(env::GITEA_TOKEN, "GITEA_TOKEN");
        assert_eq!(env::GIT_USERNAME, "GIT_USERNAME");
        assert_eq!(env::GIT_PASSWORD, "GIT_PASSWORD");
        assert_eq!(env::HTTP_PROXY, "HTTP_PROXY");
        assert_eq!(env::HTTPS_PROXY, "HTTPS_PROXY");
        assert_eq!(env::NO_PROXY, "NO_PROXY");
        assert_eq!(env::CONFIG_PATH, "ORION_VARIATE_CONFIG");
        assert_eq!(env::REDIRECT_RULES_PATH, "ORION_VARIATE_REDIRECT_RULES");
        assert_eq!(env::CACHE_DIR, "ORION_VARIATE_CACHE_DIR");
        assert_eq!(env::TEMP_DIR, "ORION_VARIATE_TEMP_DIR");
    }

    #[test]
    fn test_constants_are_static() {
        // 确保所有常量都是静态生命周期
        let _git_branch: &'static str = git::DEFAULT_BRANCH;
        let _http_timeout: &'static str = http::DEFAULT_USER_AGENT;
        let _local_separator: char = local::PATH_SEPARATOR;
        let _redirect_config: &'static str = redirect::CONFIG_FILE;
        let _env_token: &'static str = env::GITHUB_TOKEN;
    }
}
