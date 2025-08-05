//! 地址配置验证模块
//!
//! 提供地址配置验证功能，确保地址格式正确且可访问

use crate::addr::{constants };
use std::path::Path;
use url::Url;

use super::{Address, GitRepository, HttpResource, LocalPath};

/// 地址验证结果
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// 验证错误类型
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// 错误字段
    pub field: String,
    /// 错误描述
    pub message: String,
    /// 错误代码
    pub code: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str, code: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            code: code.to_string(),
        }
    }
}

/// 地址验证trait
pub trait Validate {
    /// 验证地址配置
    fn validate(&self) -> ValidationResult;
    
    /// 验证地址是否可访问
    fn is_accessible(&self) -> bool;
}

impl Validate for Address {
    fn validate(&self) -> ValidationResult {
        match self {
            Address::Git(repo) => repo.validate(),
            Address::Http(resource) => resource.validate(),
            Address::Local(path) => path.validate(),
        }
    }

    fn is_accessible(&self) -> bool {
        match self {
            Address::Git(repo) => repo.is_accessible(),
            Address::Http(resource) => resource.is_accessible(),
            Address::Local(path) => path.is_accessible(),
        }
    }
}

impl Validate for GitRepository {
    fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // 验证仓库地址
        if self.repo().is_empty() {
            errors.push(ValidationError::new(
                "repo",
                "仓库地址不能为空",
                "EMPTY_REPO",
            ));
        } else if !is_valid_git_url(self.repo()) {
            errors.push(ValidationError::new(
                "repo",
                "无效的Git仓库地址格式",
                "INVALID_GIT_URL",
            ));
        }

        // 验证SSH密钥路径
        if let Some(ssh_key) = &self.ssh_key() {
            if !Path::new(ssh_key).exists() {
                errors.push(ValidationError::new(
                    "ssh_key",
                    &format!("SSH密钥文件不存在: {}", ssh_key),
                    "SSH_KEY_NOT_FOUND",
                ));
            }
        }

        // 验证认证配置
        if self.token().is_some() && self.username().is_none() {
            errors.push(ValidationError::new(
                "username",
                "使用Token认证时必须提供用户名",
                "MISSING_USERNAME",
            ));
        }

        // 验证版本标识符
        let version_count = [
            self.tag().as_ref(),
            self.branch().as_ref(),
            self.rev().as_ref(),
        ]
        .iter()
        .filter(|x| x.is_some())
        .count();

        if version_count > 1 {
            errors.push(ValidationError::new(
                "version",
                "不能同时指定tag、branch和rev中的多个",
                "CONFLICTING_VERSIONS",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_accessible(&self) -> bool {
        // 简化的可访问性检查
        // 实际实现可能需要网络连接测试
        is_valid_git_url(self.repo())
    }
}

impl Validate for HttpResource {
    fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // 验证URL格式
        if self.url().is_empty() {
            errors.push(ValidationError::new(
                "url",
                "URL不能为空",
                "EMPTY_URL",
            ));
        } else if let Err(e) = Url::parse(self.url()) {
            errors.push(ValidationError::new(
                "url",
                &format!("无效的URL格式: {}", e),
                "INVALID_URL",
            ));
        }

        // 验证认证信息
        if self.username().is_some() && self.password().is_none() {
            errors.push(ValidationError::new(
                "password",
                "提供用户名时必须提供密码",
                "MISSING_PASSWORD",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_accessible(&self) -> bool {
        // 简化的可访问性检查
        Url::parse(self.url()).is_ok()
    }
}

impl Validate for LocalPath {
    fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();

        // 验证路径格式
        let path_str = self.path();
        if path_str.is_empty() {
            errors.push(ValidationError::new(
                "path",
                "本地路径不能为空",
                "EMPTY_PATH",
            ));
        } else {
            let path = Path::new(path_str);
            
            // 检查路径是否包含非法字符
            if path_str.contains("\\") && cfg!(not(target_os = "windows")) {
                errors.push(ValidationError::new(
                    "path",
                    "在非Windows系统上使用了反斜杠路径分隔符",
                    "INVALID_PATH_SEPARATOR",
                ));
            }

            // 检查相对路径
            if path.is_relative() && !path_str.starts_with("./") && !path_str.starts_with("../") {
                errors.push(ValidationError::new(
                    "path",
                    "相对路径应以./或../开头",
                    "INVALID_RELATIVE_PATH",
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn is_accessible(&self) -> bool {
        Path::new(self.path()).exists()
    }
}

/// 验证Git URL格式
fn is_valid_git_url(url: &str) -> bool {
    // HTTPS格式
    if url.starts_with(constants::git::HTTPS_PREFIX) && url.ends_with(".git") {
        return Url::parse(url).is_ok();
    }

    // SSH格式 (git@host:repo.git)
    if url.starts_with(constants::git::SSH_PREFIX) && url.contains(':') && url.ends_with(".git") {
        return true;
    }

    // Git协议格式
    if url.starts_with(constants::git::GIT_PROTOCOL) && url.ends_with(".git") {
        return Url::parse(url).is_ok();
    }

    // 简化的GitHub/GitLab等格式
    url.contains("github.com") || url.contains("gitlab.com") || url.contains("gitea.com")
}

/// 批量验证多个地址
pub fn validate_addresses(addresses: &[Address]) -> ValidationResult {
    let mut all_errors = Vec::new();

    for (index, addr) in addresses.iter().enumerate() {
        if let Err(errors) = addr.validate() {
            for error in errors {
                all_errors.push(ValidationError::new(
                    &format!("address[{}].{}", index, error.field),
                    &error.message,
                    &error.code,
                ));
            }
        }
    }

    if all_errors.is_empty() {
        Ok(())
    } else {
        Err(all_errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_repository_validation() {
        let repo = GitRepository::from("https://github.com/user/repo.git");
        assert!(repo.validate().is_ok());

        let invalid_repo = GitRepository::from("");
        assert!(invalid_repo.validate().is_err());

        let ssh_repo = GitRepository::from("git@github.com:user/repo.git");
        assert!(ssh_repo.validate().is_ok());
    }

    #[test]
    fn test_http_resource_validation() {
        let resource = HttpResource::from("https://example.com/file.zip");
        assert!(resource.validate().is_ok());

        let invalid_resource = HttpResource::from("invalid-url");
        assert!(invalid_resource.validate().is_err());

        let empty_resource = HttpResource::from("");
        assert!(empty_resource.validate().is_err());
    }

    #[test]
    fn test_local_path_validation() {
        let path = LocalPath::from("./relative/path");
        assert!(path.validate().is_ok());

        let absolute_path = LocalPath::from("/absolute/path");
        assert!(absolute_path.validate().is_ok());

        let invalid_path = LocalPath::from("");
        assert!(invalid_path.validate().is_err());
    }

    #[test]
    fn test_address_validation() {
        let git_addr = Address::Git(GitRepository::from("https://github.com/user/repo.git"));
        assert!(git_addr.validate().is_ok());

        let http_addr = Address::Http(HttpResource::from("https://example.com/file.zip"));
        assert!(http_addr.validate().is_ok());

        let local_addr = Address::Local(LocalPath::from("./path"));
        assert!(local_addr.validate().is_ok());
    }

    #[test]
    fn test_batch_validation() {
        let addresses = vec![
            Address::Git(GitRepository::from("https://github.com/user/repo.git")),
            Address::Http(HttpResource::from("https://example.com/file.zip")),
        ];

        assert!(validate_addresses(&addresses).is_ok());

        let invalid_addresses = vec![
            Address::Git(GitRepository::from("")),
            Address::Http(HttpResource::from("invalid-url")),
        ];

        assert!(validate_addresses(&invalid_addresses).is_err());
    }

    #[test]
    fn test_is_valid_git_url() {
        assert!(is_valid_git_url("https://github.com/user/repo.git"));
        assert!(is_valid_git_url("git@github.com:user/repo.git"));
        assert!(is_valid_git_url("git://github.com/user/repo.git"));
        assert!(is_valid_git_url("https://gitlab.com/user/repo.git"));
        assert!(!is_valid_git_url("invalid-url"));
        assert!(!is_valid_git_url(""));
    }
}