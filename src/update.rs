use derive_more::From;

use crate::vars::ValueDict;

mod upload_options;
pub use upload_options::*;

//use super::predule::*;
/// Defines the duration for which updates are kept or applied.
///
/// Currently, only project-level duration is supported.
#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum KeepDuration {
    /// Keep or apply updates at the project level.
    #[default]
    DurProj,
}

/// Defines the scope levels for updates, determining how broadly changes are applied.
///
/// The scope levels are ordered from most specific (`Element`) to broadest (`Host`),
/// with `Project` as the default.
#[derive(Debug, From, Clone, Default, PartialEq)]
pub enum UpdateScope {
    #[default]
    None,
    RemoteCache,
}

impl From<(usize, ValueDict)> for DownloadOptions {
    fn from(value: (usize, ValueDict)) -> Self {
        match value.0 {
            0 => Self::new(UpdateScope::None, value.1),
            1 => Self::new(UpdateScope::RemoteCache, value.1),
            _ => Self::new(UpdateScope::RemoteCache, value.1),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DownloadOptions {
    scope_level: UpdateScope,
    values: ValueDict,
}
impl DownloadOptions {
    pub fn new(scope_level: UpdateScope, values: ValueDict) -> Self {
        Self {
            scope_level,
            values,
        }
    }

    pub fn for_test() -> Self {
        Self {
            scope_level: UpdateScope::RemoteCache,
            values: ValueDict::default(),
        }
    }

    pub fn clean_cache(&self) -> bool {
        match self.scope_level {
            UpdateScope::None => false,
            UpdateScope::RemoteCache => true,
        }
    }
    pub fn reuse_cache(&self) -> bool {
        match self.scope_level {
            UpdateScope::None => true,
            UpdateScope::RemoteCache => false,
        }
    }

    pub fn clean_git_cache(&self) -> bool {
        self.clean_cache()
    }

    pub fn scope_level(&self) -> &UpdateScope {
        &self.scope_level
    }

    pub fn values(&self) -> &ValueDict {
        &self.values
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vars::ValueDict;

    #[test]
    fn test_keep_duration_default() {
        // 测试KeepDuration的默认值
        let duration = KeepDuration::default();
        assert_eq!(duration, KeepDuration::DurProj);
    }

    #[test]
    fn test_keep_duration_clone() {
        // 测试KeepDuration的克隆
        let duration = KeepDuration::DurProj;
        let cloned = duration.clone();
        assert_eq!(duration, cloned);
    }

    #[test]
    fn test_keep_duration_debug() {
        // 测试KeepDuration的调试格式
        let duration = KeepDuration::DurProj;
        let debug_str = format!("{:?}", duration);
        assert!(debug_str.contains("DurProj"));
    }

    #[test]
    fn test_keep_duration_partial_eq() {
        // 测试KeepDuration的部分相等性
        let duration1 = KeepDuration::DurProj;
        let duration2 = KeepDuration::DurProj;
        assert_eq!(duration1, duration2);
    }

    #[test]
    fn test_update_scope_default() {
        // 测试UpdateScope的默认值
        let scope = UpdateScope::default();
        assert_eq!(scope, UpdateScope::None);
    }

    #[test]
    fn test_update_scope_clone() {
        // 测试UpdateScope的克隆
        let scope = UpdateScope::None;
        let cloned = scope.clone();
        assert_eq!(scope, cloned);

        let scope = UpdateScope::RemoteCache;
        let cloned = scope.clone();
        assert_eq!(scope, cloned);
    }

    #[test]
    fn test_update_scope_debug() {
        // 测试UpdateScope的调试格式
        let scope = UpdateScope::None;
        let debug_str = format!("{:?}", scope);
        assert!(debug_str.contains("None"));

        let scope = UpdateScope::RemoteCache;
        let debug_str = format!("{:?}", scope);
        assert!(debug_str.contains("RemoteCache"));
    }

    #[test]
    fn test_update_scope_partial_eq() {
        // 测试UpdateScope的部分相等性
        let scope1 = UpdateScope::None;
        let scope2 = UpdateScope::None;
        assert_eq!(scope1, scope2);

        let scope1 = UpdateScope::RemoteCache;
        let scope2 = UpdateScope::RemoteCache;
        assert_eq!(scope1, scope2);

        let scope1 = UpdateScope::None;
        let scope2 = UpdateScope::RemoteCache;
        assert_ne!(scope1, scope2);
    }

    #[test]
    fn test_download_options_new() {
        // 测试DownloadOptions的构造函数
        let scope = UpdateScope::None;
        let values = ValueDict::default();
        let options = DownloadOptions::new(scope.clone(), values.clone());

        assert_eq!(*options.scope_level(), scope);
        assert_eq!(*options.values(), values);
    }

    #[test]
    fn test_download_options_for_test() {
        // 测试DownloadOptions的for_test方法
        let options = DownloadOptions::for_test();
        assert_eq!(*options.scope_level(), UpdateScope::RemoteCache);
        assert_eq!(*options.values(), ValueDict::default());
    }

    #[test]
    fn test_download_options_default() {
        // 测试DownloadOptions的默认值
        let options = DownloadOptions::default();
        assert_eq!(*options.scope_level(), UpdateScope::None);
        assert_eq!(*options.values(), ValueDict::default());
    }

    #[test]
    fn test_download_options_clean_cache() {
        // 测试clean_cache方法
        let options_none = DownloadOptions::new(UpdateScope::None, ValueDict::default());
        assert!(!options_none.clean_cache());

        let options_remote = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());
        assert!(options_remote.clean_cache());
    }

    #[test]
    fn test_download_options_reuse_cache() {
        // 测试reuse_cache方法
        let options_none = DownloadOptions::new(UpdateScope::None, ValueDict::default());
        assert!(options_none.reuse_cache());

        let options_remote = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());
        assert!(!options_remote.reuse_cache());
    }

    #[test]
    fn test_download_options_clean_git_cache() {
        // 测试clean_git_cache方法
        let options_none = DownloadOptions::new(UpdateScope::None, ValueDict::default());
        assert!(!options_none.clean_git_cache());

        let options_remote = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());
        assert!(options_remote.clean_git_cache());

        // 验证clean_git_cache与clean_cache行为一致
        assert_eq!(options_none.clean_git_cache(), options_none.clean_cache());
        assert_eq!(
            options_remote.clean_git_cache(),
            options_remote.clean_cache()
        );
    }

    #[test]
    fn test_download_options_scope_level() {
        // 测试scope_level方法
        let options = DownloadOptions::new(UpdateScope::None, ValueDict::default());
        assert_eq!(*options.scope_level(), UpdateScope::None);

        let options = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());
        assert_eq!(*options.scope_level(), UpdateScope::RemoteCache);
    }

    #[test]
    fn test_download_options_values() {
        // 测试values方法
        let values = ValueDict::default();
        let options = DownloadOptions::new(UpdateScope::None, values.clone());
        assert_eq!(*options.values(), values);
    }

    #[test]
    fn test_download_options_clone() {
        // 测试DownloadOptions的克隆
        let options = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());
        let cloned = options.clone();

        assert_eq!(*options.scope_level(), *cloned.scope_level());
        assert_eq!(*options.values(), *cloned.values());
    }

    #[test]
    fn test_download_options_debug() {
        // 测试DownloadOptions的调试格式
        let options = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("DownloadOptions"));
        assert!(debug_str.contains("RemoteCache"));
    }

    #[test]
    fn test_from_tuple_for_download_options() {
        // 测试从元组转换到DownloadOptions
        let values = ValueDict::default();

        // 测试索引0对应UpdateScope::None
        let tuple = (0, values.clone());
        let options: DownloadOptions = tuple.into();
        assert_eq!(*options.scope_level(), UpdateScope::None);
        assert_eq!(*options.values(), values);

        // 测试索引1对应UpdateScope::RemoteCache
        let tuple = (1, values.clone());
        let options: DownloadOptions = tuple.into();
        assert_eq!(*options.scope_level(), UpdateScope::RemoteCache);
        assert_eq!(*options.values(), values);

        // 测试其他索引默认对应UpdateScope::RemoteCache
        let tuple = (2, values.clone());
        let options: DownloadOptions = tuple.into();
        assert_eq!(*options.scope_level(), UpdateScope::RemoteCache);
        assert_eq!(*options.values(), values);

        let tuple = (100, values);
        let options: DownloadOptions = tuple.into();
        assert_eq!(*options.scope_level(), UpdateScope::RemoteCache);
    }

    #[test]
    fn test_cache_behavior_consistency() {
        // 测试缓存行为的一致性
        let options_none = DownloadOptions::new(UpdateScope::None, ValueDict::default());
        let options_remote = DownloadOptions::new(UpdateScope::RemoteCache, ValueDict::default());

        // 对于None范围：不清理缓存，但重用缓存
        assert!(!options_none.clean_cache());
        assert!(options_none.reuse_cache());

        // 对于RemoteCache范围：清理缓存，不重用缓存
        assert!(options_remote.clean_cache());
        assert!(!options_remote.reuse_cache());

        // 验证clean_cache和reuse_cache的互斥性
        assert_ne!(options_none.clean_cache(), options_none.reuse_cache());
        assert_ne!(options_remote.clean_cache(), options_remote.reuse_cache());
    }

    #[test]
    fn test_download_options_with_different_values() {
        // 测试使用不同ValueDict的DownloadOptions
        let values1 = ValueDict::default();
        let values2 = ValueDict::default();

        let options1 = DownloadOptions::new(UpdateScope::None, values1);
        let options2 = DownloadOptions::new(UpdateScope::RemoteCache, values2);

        // 验证不同的scope_level
        assert_ne!(*options1.scope_level(), *options2.scope_level());

        // 验证不同的values（即使都是default，但应该是不同的实例）
        // 注意：这里我们只验证引用，因为ValueDict可能实现了PartialEq
        assert_eq!(*options1.values(), *options2.values());
    }

    #[test]
    fn test_scope_level_variants() {
        // 测试UpdateScope的所有变体
        let scopes = vec![UpdateScope::None, UpdateScope::RemoteCache];

        for scope in scopes {
            let options = DownloadOptions::new(scope.clone(), ValueDict::default());
            assert_eq!(*options.scope_level(), scope);

            // 验证每个scope的缓存行为
            match scope {
                UpdateScope::None => {
                    assert!(!options.clean_cache());
                    assert!(options.reuse_cache());
                }
                UpdateScope::RemoteCache => {
                    assert!(options.clean_cache());
                    assert!(!options.reuse_cache());
                }
            }
        }
    }

    #[test]
    fn test_from_trait_edge_cases() {
        // 测试From trait的边界情况
        let values = ValueDict::default();

        // 测试usize的最大值
        let tuple = (usize::MAX, values.clone());
        let options: DownloadOptions = tuple.into();
        assert_eq!(*options.scope_level(), UpdateScope::RemoteCache);
        assert_eq!(*options.values(), values);

        // 测试usize为0
        let tuple = (0, values.clone());
        let options: DownloadOptions = tuple.into();
        assert_eq!(*options.scope_level(), UpdateScope::None);
        assert_eq!(*options.values(), values);
    }
}
