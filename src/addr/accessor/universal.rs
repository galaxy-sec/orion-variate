use async_trait::async_trait;

use crate::addr::redirect::serv::RedirectService;
use crate::addr::{AddrResult, Address};
use crate::types::{ResourceDownloader, ResourceUploader, UpdateUnit};
use crate::update::UpdateOptions;
use std::path::Path;

use super::git::GitAccessor;
use super::http::HttpAccessor;
use super::local::LocalAccessor;

/// 统一地址访问器配置
#[derive(Debug, Clone, Default)]
pub struct UniversalConfig {
    /// 重定向服务配置
    pub redirect: Option<RedirectService>,
}

impl UniversalConfig {
    pub fn with_redirect(mut self, redirect: RedirectService) -> Self {
        self.redirect = Some(redirect);
        self
    }
}

/// 统一地址访问器
///
/// 提供统一的地址访问接口，根据地址类型自动选择合适的底层访问器
pub struct UniversalAccessor {
    git: GitAccessor,
    http: HttpAccessor,
    local: LocalAccessor,
    config: UniversalConfig,
}

impl UniversalAccessor {
    /// 创建新的统一地址访问器
    pub fn new(config: UniversalConfig) -> Self {
        let git = GitAccessor::default()
            .with_redirect(config.redirect.clone())
            .with_proxy_from_env();
        let http = HttpAccessor::default().with_redirect(config.redirect.clone());
        let local = LocalAccessor::default();

        Self {
            git,
            http,
            local,
            config,
        }
    }

    /// 获取配置
    pub fn config(&self) -> &UniversalConfig {
        &self.config
    }

    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut UniversalConfig {
        &mut self.config
    }
}

#[async_trait]
impl ResourceDownloader for UniversalAccessor {
    async fn download_to_local(
        &self,
        addr: &Address,
        path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        match addr {
            Address::Git(_) => self.git.download_to_local(addr, path, options).await,
            Address::Http(_) => self.http.download_to_local(addr, path, options).await,
            Address::Local(_) => self.local.download_to_local(addr, path, options).await,
        }
    }
}

#[async_trait]
impl ResourceUploader for UniversalAccessor {
    async fn upload_from_local(
        &self,
        addr: &Address,
        path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        match addr {
            Address::Git(_) => self.git.upload_from_local(addr, path, options).await,
            Address::Http(_) => self.http.upload_from_local(addr, path, options).await,
            Address::Local(_) => self.local.upload_from_local(addr, path, options).await,
        }
    }
}

impl Clone for UniversalAccessor {
    fn clone(&self) -> Self {
        Self {
            git: self.git.clone(),
            http: self.http.clone(),
            local: self.local.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use orion_error::TestAssert;

    use super::*;
    use crate::addr::{Address, GitRepository};

    #[tokio::test]
    async fn test_select_accessor() {
        let config = UniversalConfig::default();
        let accessor = UniversalAccessor::new(config);

        let dest_path = PathBuf::from("./tmp/hello-word.git");
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path).assert();
        }
        let git_addr = Address::Git(GitRepository::from(
            "https://github.com/galaxy-sec/hello-word.git",
        ));
        accessor
            .download_to_local(
                &git_addr,
                &PathBuf::from("./tmp/"),
                &UpdateOptions::default(),
            )
            .await
            .assert();
    }
}
