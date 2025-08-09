use std::{
    env::{self, current_dir},
    path::PathBuf,
};

use log::info;
use orion_error::{ErrorOwe, ErrorWith};

use super::error::VarsResult;

pub fn setup_start_env_vars() -> VarsResult<()> {
    unsafe { std::env::set_var("GXL_OS_SYS", format_os_sys().as_str()) };
    let start_root = current_dir().owe_sys().want("get current dir")?;
    unsafe { std::env::set_var("GXL_START_ROOT", start_root.display().to_string()) };
    let prj_root = find_project_define().unwrap_or(PathBuf::from("UNDEFIN"));
    unsafe { std::env::set_var("GXL_PRJ_ROOT", format!("{}", prj_root.display())) };
    Ok(())
}

fn get_os_info() -> (String, String, u64) {
    let info = os_info::get();
    let os_type = match info.os_type() {
        os_info::Type::Macos => "macos".to_string(),
        _ => info.os_type().to_string().to_lowercase(),
    };

    let arch = info.architecture().unwrap_or("unknown").to_string();
    let ver_major = match info.version() {
        os_info::Version::Semantic(major, _, _) => *major,
        _ => 0,
    };

    (arch, os_type, ver_major)
}

fn format_os_sys() -> String {
    let (arch, os_type, ver_major) = get_os_info();
    format!("{arch}_{os_type}_{ver_major}",)
}

/// 从当前目录开始向上查找 _gal/project.toml 文件
/// 如果找到则返回其绝对路径的PathBuf，未找到则返回None
pub fn find_project_define() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

    loop {
        let project_file = current_dir.join("_gal").join("project.toml");
        if project_file.exists() {
            //let project_root = current_dir.clone();
            return Some(current_dir);
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // 已到达根目录
        }
    }

    None
}
pub struct WorkDir {
    original_dir: PathBuf,
}

impl WorkDir {
    #[allow(dead_code)]
    pub fn change<S: Into<PathBuf>>(target_dir: S) -> std::io::Result<Self> {
        let original_dir = env::current_dir()?;
        let target = target_dir.into();
        info!("set current dir:{}", target.display());
        env::set_current_dir(target)?;
        Ok(Self { original_dir })
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        info!("set current dir:{}", self.original_dir.display());
        if let Err(e) = env::set_current_dir(&self.original_dir) {
            log::error!("Failed to restore directory: {e}",);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use tempfile::TempDir;

    use crate::vars::global::{WorkDir, find_project_define, get_os_info, setup_start_env_vars};

    #[test]
    fn test_get_os_info() {
        let (arch, os_type, _ver_major) = get_os_info();

        // 验证返回的信息不为空
        assert!(!arch.is_empty());
        assert!(!os_type.is_empty());

        // 验证架构名称是有效的
        let valid_archs = ["x86_64", "x86", "arm64", "aarch64", "unknown"];
        assert!(valid_archs.contains(&arch.as_str()));
    }

    #[test]
    fn test_setup_start_env_vars() {
        // 保存原始环境变量
        let original_gxl_os_sys = env::var("GXL_OS_SYS");
        let original_gxl_start_root = env::var("GXL_START_ROOT");
        let original_gxl_prj_root = env::var("GXL_PRJ_ROOT");

        // 清理环境变量以确保测试准确性
        unsafe {
            env::remove_var("GXL_OS_SYS");
            env::remove_var("GXL_START_ROOT");
            env::remove_var("GXL_PRJ_ROOT");
        }

        // 调用函数设置环境变量
        let result = setup_start_env_vars();
        assert!(result.is_ok(), "setup_start_env_vars failed: {result:?}");

        // 验证环境变量已设置
        assert!(env::var("GXL_OS_SYS").is_ok());
        assert!(env::var("GXL_START_ROOT").is_ok());
        assert!(env::var("GXL_PRJ_ROOT").is_ok());

        // 验证GXL_OS_SYS格式
        let gxl_os_sys = env::var("GXL_OS_SYS").unwrap();
        let parts: Vec<&str> = gxl_os_sys.split('_').collect();
        assert!(parts.len() >= 3);

        // 验证GXL_START_ROOT是有效路径
        let gxl_start_root = env::var("GXL_START_ROOT").unwrap();
        assert!(std::path::Path::new(&gxl_start_root).exists());

        // 恢复原始环境变量
        unsafe {
            if let Ok(val) = original_gxl_os_sys {
                env::set_var("GXL_OS_SYS", val);
            } else {
                env::remove_var("GXL_OS_SYS");
            }

            if let Ok(val) = original_gxl_start_root {
                env::set_var("GXL_START_ROOT", val);
            } else {
                env::remove_var("GXL_START_ROOT");
            }

            if let Ok(val) = original_gxl_prj_root {
                env::set_var("GXL_PRJ_ROOT", val);
            } else {
                env::remove_var("GXL_PRJ_ROOT");
            }
        }
    }

    // 辅助函数：标准化路径比较，处理macOS上的/private前缀问题
    fn normalize_path_for_comparison(path: &std::path::Path) -> std::path::PathBuf {
        use std::path::Path;

        let path_str = path.to_string_lossy();

        // 如果路径以/private开头，移除这个前缀
        if let Some(stripped) = path_str.strip_prefix("/private") {
            Path::new(stripped).to_path_buf()
        } else {
            path.to_path_buf()
        }
    }

    // 辅助函数：比较两个路径是否相等（考虑macOS路径标准化）
    fn assert_paths_eq(path1: &std::path::Path, path2: &std::path::Path) {
        let normalized1 = normalize_path_for_comparison(path1);
        let normalized2 = normalize_path_for_comparison(path2);
        assert_eq!(
            normalized1, normalized2,
            "Path comparison failed: {path1:?} vs {path2:?}"
        );
    }

    #[test]
    fn test_work_dir_with_relative_path() {
        let original_dir = env::current_dir().expect("Failed to get current dir");

        {
            // 使用相对路径创建WorkDir
            let _work_dir = WorkDir::change(".").expect("Failed to change directory");

            // 验证工作目录仍然是当前目录
            assert_paths_eq(&env::current_dir().unwrap(), &original_dir);
        }

        // 验证工作目录已恢复
        assert_paths_eq(&env::current_dir().unwrap(), &original_dir);
    }

    #[test]
    fn test_work_dir_creation_and_restoration() {
        let original_dir = env::current_dir().expect("Failed to get current dir");
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        {
            // 创建WorkDir实例，改变工作目录
            let _work_dir = WorkDir::change(temp_dir.path()).expect("Failed to change directory");

            // 验证工作目录已改变
            assert_paths_eq(&env::current_dir().unwrap(), temp_dir.path());
        }

        // 验证工作目录已恢复
        assert_paths_eq(&env::current_dir().unwrap(), &original_dir);
    }

    #[test]
    fn test_find_project_define_with_deep_nesting() {
        // 创建深层嵌套的目录结构
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let deep_dir = temp_dir.path().join("level1").join("level2").join("level3");
        std::fs::create_dir_all(&deep_dir).expect("Failed to create deep directory structure");

        // 在根目录创建project.toml
        let gal_dir = temp_dir.path().join("_gal");
        std::fs::create_dir(&gal_dir).expect("Failed to create _gal dir");
        let project_file = gal_dir.join("project.toml");
        std::fs::write(&project_file, "").expect("Failed to create project.toml");

        // 在深层目录中查找
        let _wd = WorkDir::change(&deep_dir).expect("Failed to change directory");

        let result = find_project_define();
        assert!(result.is_some());
        assert_paths_eq(&result.unwrap(), temp_dir.path());
    }
}
