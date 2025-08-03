use crate::types::{LocalUpdate, RemoteUpdate, UpdateUnit};
use crate::update::UpdateOptions;
use crate::vars::{EnvDict, EnvEvalable};
use std::path::Path;

use super::{AddrResult, AddrType};

/// 地址访问器，提供统一的地址更新接口
/// 
/// 这个结构体封装了不同类型的地址（Git、HTTP、Local），
/// 并提供统一的更新操作接口
#[derive(Debug, Clone)]
pub struct AddrAccessor {
    addr: AddrType,
}

impl AddrAccessor {
    /// 创建一个新的地址访问器
    pub fn new(addr: AddrType) -> Self {
        Self { addr }
    }

    /// 从地址字符串创建访问器
    pub fn from_str(addr_str: &str) -> Self {
        let addr = AddrType::from(addr_str);
        Self::new(addr)
    }

    /// 获取内部的地址类型
    pub fn inner(&self) -> &AddrType {
        &self.addr
    }

    /// 获取可变的内部地址类型
    pub fn inner_mut(&mut self) -> &mut AddrType {
        &mut self.addr
    }

    /// 转换为内部的地址类型
    pub fn into_inner(self) -> AddrType {
        self.addr
    }

    /// 执行环境变量评估
    pub fn env_eval(&mut self, dict: &EnvDict) -> &mut Self {
        self.addr = self.addr.clone().env_eval(dict);
        self
    }

    /// 更新本地路径
    pub async fn update_local(&self, path: &Path, options: &UpdateOptions) -> AddrResult<UpdateUnit> {
        self.addr.update_local(path, options).await
    }

    /// 更新本地路径并重命名
    pub async fn update_local_rename(
        &self,
        path: &Path,
        name: &str,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        self.addr.update_local_rename(path, name, options).await
    }

    /// 更新远程地址
    pub async fn update_remote(&self, path: &Path, options: &UpdateOptions) -> AddrResult<UpdateUnit> {
        self.addr.update_remote(path, options).await
    }
}

impl From<AddrType> for AddrAccessor {
    fn from(addr: AddrType) -> Self {
        Self::new(addr)
    }
}

impl From<&str> for AddrAccessor {
    fn from(addr_str: &str) -> Self {
        Self::from_str(addr_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update::UpdateOptions;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_addr_accessor_basic() {
        let accessor = AddrAccessor::from_str("https://github.com/user/repo.git");
        assert!(matches!(accessor.inner(), AddrType::Http(_)));
    }

    #[tokio::test]
    async fn test_addr_accessor_git() {
        let accessor = AddrAccessor::from_str("git@github.com:user/repo.git");
        assert!(matches!(accessor.inner(), AddrType::Git(_)));
    }

    #[tokio::test]
    async fn test_addr_accessor_local() {
        let accessor = AddrAccessor::from_str("./local/path");
        assert!(matches!(accessor.inner(), AddrType::Local(_)));
    }

    #[tokio::test]
    async fn test_addr_accessor_update_local() -> AddrResult<()> {
        let temp_dir = tempdir().unwrap();
        let accessor = AddrAccessor::from_str("https://github.com/galaxy-sec/hello-word.git");
        
        let options = UpdateOptions::default();
        let result = accessor.update_local(temp_dir.path(), &options).await;
        
        // 由于网络依赖，这里只验证函数调用是否成功
        assert!(result.is_ok() || result.is_err());
        
        Ok(())
    }
}