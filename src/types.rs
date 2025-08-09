use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{
    addr::{AddrResult, Address, accessor::rename_path},
    update::{DownloadOptions, UploadOptions},
    vars::VarCollection,
};
use getset::{CloneGetters, CopyGetters, Getters, MutGetters, Setters, WithSetters};

#[derive(
    Clone,
    Debug,
    Getters,
    Setters,
    WithSetters,
    MutGetters,
    CopyGetters,
    CloneGetters,
    Default,
    PartialEq,
)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addr::{AddrResult, Address};
    use crate::update::{DownloadOptions, UploadOptions};
    use crate::vars::VarCollection;
    use async_trait::async_trait;
    use std::path::PathBuf;

    // UpdateUnit 结构体测试
    #[test]
    fn test_update_unit_new() {
        let position = PathBuf::from("/test/path");
        let vars = VarCollection::default();
        let unit = UpdateUnit::new(position.clone(), vars.clone());

        assert_eq!(*unit.position(), position);
        assert_eq!(unit.vars(), Some(&vars));
    }

    #[test]
    fn test_update_unit_default() {
        let unit = UpdateUnit::default();
        assert_eq!(*unit.position(), PathBuf::new());
        assert!(unit.vars().is_none());
    }

    #[test]
    fn test_update_unit_from_pathbuf() {
        let path = PathBuf::from("/test/path");
        let unit: UpdateUnit = path.clone().into();

        assert_eq!(*unit.position(), path);
        assert!(unit.vars().is_none());
    }

    #[test]
    fn test_update_unit_position_getter() {
        let position = PathBuf::from("/test/path");
        let unit = UpdateUnit::new(position.clone(), VarCollection::default());

        assert_eq!(*unit.position(), position);
    }

    #[test]
    fn test_update_unit_vars_getter() {
        let vars = VarCollection::default();
        let unit = UpdateUnit::new(PathBuf::from("/test/path"), vars.clone());

        assert_eq!(unit.vars(), Some(&vars));
    }

    #[test]
    fn test_update_unit_vars_getter_none() {
        let unit: UpdateUnit = PathBuf::from("/test/path").into();
        assert!(unit.vars().is_none());
    }

    #[test]
    fn test_update_unit_set_position() {
        let mut unit = UpdateUnit::new(PathBuf::from("/old/path"), VarCollection::default());
        let new_position = PathBuf::from("/new/path");

        unit.set_position(new_position.clone());
        assert_eq!(*unit.position(), new_position);
    }

    #[test]
    fn test_update_unit_with_position() {
        let unit = UpdateUnit::default();
        let new_position = PathBuf::from("/new/path");

        let modified_unit = unit.with_position(new_position.clone());
        assert_eq!(*modified_unit.position(), new_position);
    }

    #[test]
    fn test_update_unit_position_mut() {
        let mut unit = UpdateUnit::new(PathBuf::from("/old/path"), VarCollection::default());

        unit.position_mut().push("subdir");
        assert_eq!(*unit.position(), PathBuf::from("/old/path/subdir"));
    }

    #[test]
    fn test_update_unit_clone() {
        let original = UpdateUnit::new(PathBuf::from("/test/path"), VarCollection::default());
        let cloned = original.clone();

        assert_eq!(*original.position(), *cloned.position());
        assert_eq!(original.vars(), cloned.vars());
    }

    #[test]
    fn test_update_unit_partial_eq() {
        let path = PathBuf::from("/test/path");
        let vars = VarCollection::default();

        let unit1 = UpdateUnit::new(path.clone(), vars.clone());
        let unit2 = UpdateUnit::new(path.clone(), vars.clone());
        let unit3 = UpdateUnit::new(PathBuf::from("/different/path"), vars.clone());

        assert_eq!(unit1, unit2);
        assert_ne!(unit1, unit3);
    }

    #[test]
    fn test_update_unit_debug() {
        let unit = UpdateUnit::new(PathBuf::from("/test/path"), VarCollection::default());
        let debug_str = format!("{unit:?}");

        assert!(debug_str.contains("UpdateUnit"));
        assert!(debug_str.contains("/test/path"));
    }

    // ResourceUploader trait 的模拟实现和测试
    struct MockUploader;

    #[async_trait]
    impl ResourceUploader for MockUploader {
        async fn upload_from_local(
            &self,
            _source: &Address,
            dest: &Path,
            _options: &UploadOptions,
        ) -> AddrResult<UpdateUnit> {
            // 模拟上传操作
            Ok(UpdateUnit::new(
                dest.to_path_buf(),
                VarCollection::default(),
            ))
        }
    }

    #[tokio::test]
    async fn test_resource_uploader_upload() {
        let uploader = MockUploader;
        let source = Address::Local(crate::addr::LocalPath::from("/local/source"));
        let dest = Path::new("/remote/dest");
        let options = UploadOptions::default();

        let result = uploader.upload_from_local(&source, dest, &options).await;

        assert!(result.is_ok());
        let unit = result.unwrap();
        assert_eq!(*unit.position(), PathBuf::from("/remote/dest"));
    }

    // ResourceDownloader trait 的模拟实现和测试
    struct MockDownloader;

    #[async_trait]
    impl ResourceDownloader for MockDownloader {
        async fn download_to_local(
            &self,
            _source: &Address,
            dest: &Path,
            _options: &DownloadOptions,
        ) -> AddrResult<UpdateUnit> {
            // 模拟下载操作
            Ok(UpdateUnit::new(
                dest.to_path_buf(),
                VarCollection::default(),
            ))
        }
    }

    #[tokio::test]
    async fn test_resource_downloader_download() {
        let downloader = MockDownloader;
        let source = Address::Local(crate::addr::LocalPath::from("/remote/source"));
        let dest = Path::new("/local/dest");
        let options = DownloadOptions::default();

        let result = downloader.download_to_local(&source, dest, &options).await;

        assert!(result.is_ok());
        let unit = result.unwrap();
        assert_eq!(*unit.position(), PathBuf::from("/local/dest"));
    }

    // 边界情况测试
    #[test]
    fn test_update_unit_empty_path() {
        let unit = UpdateUnit::new(PathBuf::new(), VarCollection::default());
        assert!(unit.position().as_os_str().is_empty());
    }

    #[tokio::test]
    async fn test_uploader_with_different_addresses() {
        let uploader = MockUploader;
        let options = UploadOptions::default();

        let addresses = vec![
            Address::Local(crate::addr::LocalPath::from("/local/path")),
            Address::Http(crate::addr::HttpResource::from("https://example.com/file")),
        ];

        for addr in addresses {
            let dest = Path::new("/dest");
            let result = uploader.upload_from_local(&addr, dest, &options).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_downloader_with_different_addresses() {
        let downloader = MockDownloader;
        let options = DownloadOptions::default();

        let addresses = vec![
            Address::Local(crate::addr::LocalPath::from("/local/path")),
            Address::Http(crate::addr::HttpResource::from("https://example.com/file")),
        ];

        for addr in addresses {
            let dest = Path::new("/dest");
            let result = downloader.download_to_local(&addr, dest, &options).await;
            assert!(result.is_ok());
        }
    }

    // 性能测试（可选）
    #[test]
    fn test_update_unit_clone_performance() {
        let unit = UpdateUnit::new(PathBuf::from("/test/path"), VarCollection::default());

        // 测试克隆性能
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = unit.clone();
        }
        let duration = start.elapsed();

        // 断言克隆操作应该在合理时间内完成
        assert!(duration.as_millis() < 100);
    }
}
