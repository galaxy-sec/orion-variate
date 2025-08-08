use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 下载超时配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TimeoutConfig {
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
    /// 读取/写入超时时间（秒）
    pub read_timeout: u64,
    /// 总操作超时时间（秒）
    pub total_timeout: u64,
    /// 重试次数
    pub max_retries: u32,
    /// 重试间隔时间（秒）
    pub retry_interval: u64,
    /// 是否在超时时启用重试
    pub retry_on_timeout: bool,
}

impl TimeoutConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// HTTP模式配置（适用于小文件）
    pub fn http_simple() -> Self {
        Self {
            connect_timeout: 30,
            read_timeout: 60,
            total_timeout: 300,
            max_retries: 3,
            retry_interval: 2,
            retry_on_timeout: true,
        }
    }

    /// HTTP大文件模式配置
    pub fn http_large_file() -> Self {
        Self {
            connect_timeout: 60,
            read_timeout: 300,
            total_timeout: 3600,
            max_retries: 5,
            retry_interval: 5,
            retry_on_timeout: true,
        }
    }

    /// Git操作配置
    pub fn git_operation() -> Self {
        Self {
            connect_timeout: 120,
            read_timeout: 180,
            total_timeout: 1800,
            max_retries: 2,
            retry_interval: 10,
            retry_on_timeout: true,
        }
    }

    /// 转为Duration格式
    pub fn connect_duration(&self) -> Duration {
        Duration::from_secs(self.connect_timeout)
    }

    pub fn read_duration(&self) -> Duration {
        Duration::from_secs(self.read_timeout)
    }

    pub fn total_duration(&self) -> Duration {
        Duration::from_secs(self.total_timeout)
    }

    pub fn retry_interval_duration(&self) -> Duration {
        Duration::from_secs(self.retry_interval)
    }

    /// 验证配置有效性
    pub fn validate(&self) -> bool {
        self.connect_timeout > 0
            && self.read_timeout > 0
            && self.total_timeout > 0
            && self.max_retries > 0
            && self.retry_interval > 0
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: 30,
            read_timeout: 60,
            total_timeout: 300,
            max_retries: 3,
            retry_interval: 2,
            retry_on_timeout: true,
        }
    }
}

/// 下载进度监控器
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    last_activity: std::time::Instant,
    timeout_duration: Duration,
    total_downloaded: u64,
    total_expected: Option<u64>,
}

impl ProgressTracker {
    /// 创建新的进度跟踪器
    pub fn new(timeout_duration: Duration) -> Self {
        Self {
            last_activity: std::time::Instant::now(),
            timeout_duration,
            total_downloaded: 0,
            total_expected: None,
        }
    }

    /// 更新进度信息
    pub fn update(&mut self, bytes: u64, total: Option<u64>) {
        self.last_activity = std::time::Instant::now();
        self.total_downloaded = bytes;
        self.total_expected = total;
    }

    pub fn reset(&mut self) {
        self.last_activity = std::time::Instant::now();
    }

    /// 检查是否超时
    pub fn has_timed_out(&self) -> bool {
        self.last_activity.elapsed() > self.timeout_duration
    }

    /// 获取下载进度百分比
    pub fn progress_percent(&self) -> Option<f64> {
        match self.total_expected {
            Some(total) if total > 0 => Some((self.total_downloaded as f64 / total as f64) * 100.0),
            _ => None,
        }
    }

    /// 获取已下载字节数
    pub fn downloaded(&self) -> u64 {
        self.total_downloaded
    }

    /// 获取总期望字节数
    pub fn total_expected(&self) -> Option<u64> {
        self.total_expected
    }
}

// 辅助函数
fn get_env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn get_env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|s| match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_default_config() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connect_timeout, 30);
        assert_eq!(config.read_timeout, 60);
        assert_eq!(config.total_timeout, 300);
    }

    #[test]
    fn test_duration_conversions() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connect_duration(), Duration::from_secs(30));
        assert_eq!(config.read_duration(), Duration::from_secs(60));
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new(Duration::from_millis(100));
        assert!(!tracker.has_timed_out());
        assert_eq!(tracker.progress_percent(), None);

        tracker.update(50, Some(100));
        assert_eq!(tracker.progress_percent(), Some(50.0));
        assert_eq!(tracker.downloaded(), 50);

        std::thread::sleep(Duration::from_millis(110));
        assert!(tracker.has_timed_out());
    }

    #[test]
    fn test_config_validation() {
        let config = TimeoutConfig {
            connect_timeout: 0,
            ..Default::default()
        };
        assert!(!config.validate());

        let config = TimeoutConfig::default();
        assert!(config.validate());
    }
}
