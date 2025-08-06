use async_trait::async_trait;
use derive_more::From;

use crate::addr::{AddrResult, Address};
use crate::types::{ResourceDownloader, ResourceUploader, UpdateUnit};
use crate::update::UpdateOptions;
use std::path::Path;

use super::git::GitAccessor;
use super::http::HttpAccessor;
use super::local::LocalAccessor;

/// 地址访问器，提供统一的地址更新接口
///
/// 这个结构体封装了不同类型的地址（Git、HTTP、Local），
/// 并提供统一的更新操作接口
#[derive(Debug, Clone, From)]
pub enum AddrAccessor {
    Git(GitAccessor),
    Http(HttpAccessor),
    Local(LocalAccessor),
}

#[async_trait]
impl ResourceDownloader for AddrAccessor {
    async fn download_to_local(
        &self,
        addr: &Address,
        path: &Path,
        up_options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        match self {
            AddrAccessor::Git(o) => o.download_to_local(addr, path, up_options).await,
            AddrAccessor::Http(o) => o.download_to_local(addr, path, up_options).await,
            AddrAccessor::Local(o) => o.download_to_local(addr, path, up_options).await,
        }
    }
}

#[async_trait]
impl ResourceUploader for AddrAccessor {
    async fn upload_from_local(
        &self,
        addr: &Address,
        path: &Path,
        options: &UpdateOptions,
    ) -> AddrResult<UpdateUnit> {
        match self {
            AddrAccessor::Git(o) => o.upload_from_local(addr, path, options).await,
            AddrAccessor::Http(o) => o.upload_from_local(addr, path, options).await,
            AddrAccessor::Local(o) => o.upload_from_local(addr, path, options).await,
        }
    }
}
