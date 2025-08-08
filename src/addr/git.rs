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
mod tests {}
