use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    addr::{AddrResult, Address, accessor::rename_path},
    update::{DownloadOptions, UploadOptions},
    vars::VarCollection,
};
use getset::{CloneGetters, CopyGetters, Getters, MutGetters, Setters, WithSetters};

#[derive(Clone, Getters, Setters, WithSetters, MutGetters, CopyGetters, CloneGetters, Default)]
pub struct UpdateUnit {
    #[getset(get = "pub", set = "pub", get_mut, set_with)]
    pub position: PathBuf,
    pub vars: Option<VarCollection>,
}
impl UpdateUnit {
    pub fn new(position: PathBuf, vars: VarCollection) -> Self {
        Self {
            position,
            vars: Some(vars),
        }
    }
    pub fn vars(&self) -> Option<&VarCollection> {
        self.vars.as_ref()
    }
}
impl From<PathBuf> for UpdateUnit {
    fn from(value: PathBuf) -> Self {
        Self {
            vars: None,
            position: value,
        }
    }
}

#[async_trait]
pub trait ResourceUploader {
    async fn upload_from_local(
        &self,
        source: &Address,
        dest: &Path,
        options: &UploadOptions,
    ) -> AddrResult<UpdateUnit>;
}
#[async_trait]
pub trait ResourceDownloader {
    async fn download_to_local(
        &self,
        source: &Address,
        dest: &Path,
        options: &DownloadOptions,
    ) -> AddrResult<UpdateUnit>;
    async fn download_rename(
        &self,
        addr: &Address,
        path: &Path,
        name: &str,
        options: &DownloadOptions,
    ) -> AddrResult<UpdateUnit> {
        let mut target = self.download_to_local(addr, path, options).await?;
        let path = rename_path(target.position(), name)?;
        target.set_position(path);
        Ok(target)
    }
}
