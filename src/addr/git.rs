use crate::vars::EnvEvalable;
use crate::{predule::*, vars::EnvDict};
use getset::{Getters, Setters, WithSetters};
use home::home_dir;

///
/// 支持通过SSH和HTTPS协议访问Git仓库
///
/// # Token认证示例
///

#[derive(Clone, Debug, Serialize, Deserialize, Default, Getters, Setters, WithSetters)]
#[getset(get = "pub", set = "pub")]
#[serde(rename = "git")]
pub struct GitRepository {
    repo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    res: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    // 新增：SSH私钥路径
    #[serde(skip_serializing_if = "Option::is_none")]
    ssh_key: Option<String>,
    // 新增：SSH密钥密码
    #[serde(skip_serializing_if = "Option::is_none")]
    ssh_passphrase: Option<String>,
    // 新增：Token认证（用于HTTPS协议）
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    // 新增：用户名（用于Token认证）
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
}

impl PartialEq for GitRepository {
    fn eq(&self, other: &Self) -> bool {
        self.repo == other.repo
    }
}
impl EnvEvalable<GitRepository> for GitRepository {
    fn env_eval(self, dict: &EnvDict) -> GitRepository {
        Self {
            repo: self.repo.env_eval(dict),
            res: self.res.env_eval(dict),
            tag: self.tag.env_eval(dict),
            branch: self.branch.env_eval(dict),
            rev: self.rev.env_eval(dict),
            path: self.path.env_eval(dict),
            ssh_key: self.ssh_key.env_eval(dict),
            ssh_passphrase: self.ssh_passphrase.env_eval(dict),
            token: self.token.env_eval(dict),
            username: self.username.env_eval(dict),
        }
    }
}

impl GitRepository {
    pub fn from<S: Into<String>>(repo: S) -> Self {
        Self {
            repo: repo.into(),
            ..Default::default()
        }
    }
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tag = Some(tag.into());
        self
    }
    pub fn with_opt_tag(mut self, tag: Option<String>) -> Self {
        self.tag = tag;
        self
    }
    pub fn with_branch<S: Into<String>>(mut self, branch: S) -> Self {
        self.branch = Some(branch.into());
        self
    }
    pub fn with_opt_branch(mut self, branch: Option<String>) -> Self {
        self.branch = branch;
        self
    }
    pub fn with_rev<S: Into<String>>(mut self, rev: S) -> Self {
        self.rev = Some(rev.into());
        self
    }
    pub fn with_path<S: Into<String>>(mut self, path: S) -> Self {
        self.path = Some(path.into());
        self
    }
    // 新增：设置SSH私钥
    pub fn with_ssh_key<S: Into<String>>(mut self, ssh_key: S) -> Self {
        self.ssh_key = Some(ssh_key.into());
        self
    }
    // 新增：设置SSH密钥密码
    pub fn with_ssh_passphrase<S: Into<String>>(mut self, ssh_passphrase: S) -> Self {
        self.ssh_passphrase = Some(ssh_passphrase.into());
        self
    }
    // 新增：设置Token认证
    pub fn with_token<S: Into<String>>(mut self, token: S) -> Self {
        self.token = Some(token.into());
        self
    }
    // 新增：设置用户名（用于Token认证）
    pub fn with_username<S: Into<String>>(mut self, username: S) -> Self {
        self.username = Some(username.into());
        self
    }
    // 新增：设置Token认证（可选）
    pub fn with_opt_token(mut self, token: Option<String>) -> Self {
        self.token = token;
        self
    }
    // 新增：设置用户名（可选）
    pub fn with_opt_username(mut self, username: Option<String>) -> Self {
        self.username = username;
        self
    }

    /// 为GitHub设置Token认证（便捷方法）
    /// GitHub使用用户名+Token作为密码的方式
    pub fn with_github_token<S: Into<String>>(mut self, token: S) -> Self {
        self.username = Some("git".to_string());
        self.token = Some(token.into());
        self
    }

    /// 为GitLab设置Token认证（便捷方法）
    /// GitLab可以使用"oauth2"作为用户名，Token作为密码
    pub fn with_gitlab_token<S: Into<String>>(mut self, token: S) -> Self {
        self.username = Some("oauth2".to_string());
        self.token = Some(token.into());
        self
    }

    /// 为Gitea设置Token认证（便捷方法）
    /// Gitea可以使用Token作为密码
    pub fn with_gitea_token<S: Into<String>>(mut self, token: S) -> Self {
        self.username = Some("git".to_string());
        self.token = Some(token.into());
        self
    }

    /// 从环境变量读取Token认证
    ///
    /// # Arguments
    /// * `env_var` - 环境变量名，例如 "GITHUB_TOKEN"
    pub fn with_env_token(mut self, env_var: &str) -> Self {
        if let Ok(token) = std::env::var(env_var) {
            self.token = Some(token);
        }
        self
    }

    /// 从环境变量读取GitHub Token认证
    pub fn with_github_env_token(self) -> Self {
        self.with_env_token("GITHUB_TOKEN")
    }

    /// 从环境变量读取GitLab Token认证
    pub fn with_gitlab_env_token(mut self) -> Self {
        if let Ok(token) = std::env::var("GITLAB_TOKEN") {
            self.username = Some("oauth2".to_string());
            self.token = Some(token);
        }
        self
    }

    /// 从环境变量读取Gitea Token认证
    pub fn with_gitea_env_token(self) -> Self {
        self.with_env_token("GITEA_TOKEN")
    }

    /// 从~/.git-credentials文件读取token
    pub fn with_git_credentials(mut self) -> Self {
        if let Some(credentials) = Self::read_git_credentials() {
            for (url, username, token) in credentials {
                if self.repo.starts_with(&url) {
                    self.username = Some(username);
                    self.token = Some(token);
                    break;
                }
            }
        }
        self
    }
    /// 读取~/.git-credentials文件
    pub fn read_git_credentials() -> Option<Vec<(String, String, String)>> {
        use std::fs;
        use std::io::{BufRead, BufReader};

        let home = home_dir()?;
        let credentials_path = home.join(".git-credentials");

        if !credentials_path.exists() {
            return None;
        }

        let file = fs::File::open(credentials_path).ok()?;
        let reader = BufReader::new(file);
        let mut credentials = Vec::new();

        for line in reader.lines().map_while(Result::ok) {
            //if let Ok(line) = line {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 使用URL解析来正确提取各个部分
            if let Ok(url) = url::Url::parse(line) {
                let host = url.host_str()?;
                let scheme = url.scheme();
                let path = url.path();

                // 构建基础URL用于匹配
                let base_url = format!("{scheme}://{host}{path}");

                // 提取用户名和密码
                let username = url.username();
                if !username.is_empty()
                    && let Some(password) = url.password()
                {
                    credentials.push((base_url, username.to_string(), password.to_string()));
                }
                //}
            }
        }

        if credentials.is_empty() {
            None
        } else {
            Some(credentials)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_git_repository_from() {
        let repo = GitRepository::from("https://github.com/user/repo.git");
        assert_eq!(repo.repo(), "https://github.com/user/repo.git");
        assert!(repo.tag().is_none());
        assert!(repo.branch().is_none());
        assert!(repo.token().is_none());
    }

    #[test]
    fn test_git_repository_with_tag() {
        let repo = GitRepository::from("https://github.com/user/repo.git").with_tag("v1.0.0");
        assert_eq!(repo.tag().as_ref(), Some(&"v1.0.0".to_string()));
    }

    #[test]
    fn test_git_repository_with_opt_tag() {
        let repo1 = GitRepository::from("https://github.com/user/repo.git")
            .with_opt_tag(Some("v1.0.0".to_string()));
        assert_eq!(repo1.tag().as_ref(), Some(&"v1.0.0".to_string()));

        let repo2 = GitRepository::from("https://github.com/user/repo.git").with_opt_tag(None);
        assert!(repo2.tag().is_none());
    }

    #[test]
    fn test_git_repository_with_branch() {
        let repo = GitRepository::from("https://github.com/user/repo.git").with_branch("main");
        assert_eq!(repo.branch().as_ref(), Some(&"main".to_string()));
    }

    #[test]
    fn test_git_repository_with_opt_branch() {
        let repo1 = GitRepository::from("https://github.com/user/repo.git")
            .with_opt_branch(Some("main".to_string()));
        assert_eq!(repo1.branch().as_ref(), Some(&"main".to_string()));

        let repo2 = GitRepository::from("https://github.com/user/repo.git").with_opt_branch(None);
        assert!(repo2.branch().is_none());
    }

    #[test]
    fn test_git_repository_with_rev() {
        let repo = GitRepository::from("https://github.com/user/repo.git").with_rev("abc123");
        assert_eq!(repo.rev().as_ref(), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_git_repository_with_path() {
        let repo = GitRepository::from("https://github.com/user/repo.git").with_path("subdir");
        assert_eq!(repo.path().as_ref(), Some(&"subdir".to_string()));
    }

    #[test]
    fn test_git_repository_with_ssh_key() {
        let repo = GitRepository::from("git@github.com:user/repo.git").with_ssh_key("/path/to/key");
        assert_eq!(repo.ssh_key().as_ref(), Some(&"/path/to/key".to_string()));
    }

    #[test]
    fn test_git_repository_with_ssh_passphrase() {
        let repo =
            GitRepository::from("git@github.com:user/repo.git").with_ssh_passphrase("secret");
        assert_eq!(repo.ssh_passphrase().as_ref(), Some(&"secret".to_string()));
    }

    #[test]
    fn test_git_repository_with_token() {
        let repo = GitRepository::from("https://github.com/user/repo.git").with_token("token123");
        assert_eq!(repo.token().as_ref(), Some(&"token123".to_string()));
    }

    #[test]
    fn test_git_repository_with_username() {
        let repo = GitRepository::from("https://github.com/user/repo.git").with_username("user");
        assert_eq!(repo.username().as_ref(), Some(&"user".to_string()));
    }

    #[test]
    fn test_git_repository_with_opt_token() {
        let repo1 = GitRepository::from("https://github.com/user/repo.git")
            .with_opt_token(Some("token123".to_string()));
        assert_eq!(repo1.token().as_ref(), Some(&"token123".to_string()));

        let repo2 = GitRepository::from("https://github.com/user/repo.git").with_opt_token(None);
        assert!(repo2.token().is_none());
    }

    #[test]
    fn test_git_repository_with_opt_username() {
        let repo1 = GitRepository::from("https://github.com/user/repo.git")
            .with_opt_username(Some("user".to_string()));
        assert_eq!(repo1.username().as_ref(), Some(&"user".to_string()));

        let repo2 = GitRepository::from("https://github.com/user/repo.git").with_opt_username(None);
        assert!(repo2.username().is_none());
    }

    #[test]
    fn test_git_repository_with_github_token() {
        let repo =
            GitRepository::from("https://github.com/user/repo.git").with_github_token("ghp_token");
        assert_eq!(repo.username().as_ref(), Some(&"git".to_string()));
        assert_eq!(repo.token().as_ref(), Some(&"ghp_token".to_string()));
    }

    #[test]
    fn test_git_repository_with_gitlab_token() {
        let repo = GitRepository::from("https://gitlab.com/user/repo.git")
            .with_gitlab_token("glpat_token");
        assert_eq!(repo.username().as_ref(), Some(&"oauth2".to_string()));
        assert_eq!(repo.token().as_ref(), Some(&"glpat_token".to_string()));
    }

    #[test]
    fn test_git_repository_with_gitea_token() {
        let repo =
            GitRepository::from("https://gitea.com/user/repo.git").with_gitea_token("gitea_token");
        assert_eq!(repo.username().as_ref(), Some(&"git".to_string()));
        assert_eq!(repo.token().as_ref(), Some(&"gitea_token".to_string()));
    }

    #[test]
    fn test_read_git_credentials_valid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "https://user:token@github.com").unwrap();
        writeln!(temp_file, "https://oauth2:token@gitlab.com/user/repo.git").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        temp_file.flush().unwrap();

        let credentials = GitRepository::read_git_credentials_from_path(temp_file.path());
        assert!(credentials.is_some());
        let creds = credentials.unwrap();
        assert_eq!(creds.len(), 2);

        assert_eq!(creds[0].0, "https://github.com/");
        assert_eq!(creds[0].1, "user");
        assert_eq!(creds[0].2, "token");

        assert_eq!(creds[1].0, "https://gitlab.com/user/repo.git");
        assert_eq!(creds[1].1, "oauth2");
        assert_eq!(creds[1].2, "token");
    }

    #[test]
    fn test_read_git_credentials_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let credentials = GitRepository::read_git_credentials_from_path(temp_file.path());
        assert!(credentials.is_none());
    }

    #[test]
    fn test_read_git_credentials_invalid_url() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid-url").unwrap();
        temp_file.flush().unwrap();

        let credentials = GitRepository::read_git_credentials_from_path(temp_file.path());
        assert!(credentials.is_none());
    }

    #[test]
    fn test_git_repository_with_git_credentials() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "https://user:token@github.com/user/repo.git").unwrap();
        temp_file.flush().unwrap();

        let repo = GitRepository::from("https://github.com/user/repo.git")
            .with_git_credentials_from_path(temp_file.path());

        assert_eq!(repo.username().as_ref(), Some(&"user".to_string()));
        assert_eq!(repo.token().as_ref(), Some(&"token".to_string()));
    }

    #[test]
    fn test_git_repository_with_git_credentials_no_match() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "https://user:token@gitlab.com/user/repo.git").unwrap();
        temp_file.flush().unwrap();

        let repo = GitRepository::from("https://github.com/user/repo.git")
            .with_git_credentials_from_path(temp_file.path());

        assert!(repo.username().is_none());
        assert!(repo.token().is_none());
    }

    // Helper methods for testing
    impl GitRepository {
        fn read_git_credentials_from_path(
            path: &std::path::Path,
        ) -> Option<Vec<(String, String, String)>> {
            use std::fs;
            use std::io::{BufRead, BufReader};

            if !path.exists() {
                return None;
            }

            let file = fs::File::open(path).ok()?;
            let reader = BufReader::new(file);
            let mut credentials = Vec::new();

            for line in reader.lines().map_while(Result::ok) {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Ok(url) = url::Url::parse(line) {
                    let host = url.host_str()?;
                    let scheme = url.scheme();
                    let path = url.path();

                    let base_url = format!("{scheme}://{host}{path}");

                    let username = url.username();
                    if !username.is_empty()
                        && let Some(password) = url.password()
                    {
                        credentials.push((base_url, username.to_string(), password.to_string()));
                    }
                }
            }

            if credentials.is_empty() {
                None
            } else {
                Some(credentials)
            }
        }

        fn with_git_credentials_from_path(mut self, path: &std::path::Path) -> Self {
            if let Some(credentials) = Self::read_git_credentials_from_path(path) {
                for (url, username, token) in credentials {
                    if self.repo.starts_with(&url) {
                        self.username = Some(username);
                        self.token = Some(token);
                        break;
                    }
                }
            }
            self
        }
    }
}
