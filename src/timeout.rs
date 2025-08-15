use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 下载超时配置结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TimeoutConfig {
    /// 连接超时时间（秒）
    pub connect_timeout: u64,
    /// 读取/写入超时时间（秒）
    pub read_timeout: u64,
    /// 总操作超时时间（秒）
    pub total_timeout: u64,
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
        }
    }

    /// HTTP大文件模式配置
    pub fn http_large_file() -> Self {
        Self {
            connect_timeout: 60,
            read_timeout: 300,
            total_timeout: 3600,
        }
    }

    /// Git操作配置
    pub fn git_operation() -> Self {
        Self {
            connect_timeout: 120,
            read_timeout: 180,
            total_timeout: 1800,
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

    /// 验证配置有效性
    pub fn validate(&self) -> bool {
        self.connect_timeout > 0 && self.read_timeout > 0 && self.total_timeout > 0
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: 30,
            read_timeout: 60,
            total_timeout: 300,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // TimeoutConfig 测试
    mod timeout_config {
        use super::*;

        #[test]
        fn test_default_config() {
            let config = TimeoutConfig::default();
            assert_eq!(config.connect_timeout, 30);
            assert_eq!(config.read_timeout, 60);
            assert_eq!(config.total_timeout, 300);
        }

        #[test]
        fn test_new_config() {
            let config = TimeoutConfig::new();
            assert_eq!(config, TimeoutConfig::default());
        }

        #[test]
        fn test_http_simple_config() {
            let config = TimeoutConfig::http_simple();
            assert_eq!(config.connect_timeout, 30);
            assert_eq!(config.read_timeout, 60);
            assert_eq!(config.total_timeout, 300);
        }

        #[test]
        fn test_http_large_file_config() {
            let config = TimeoutConfig::http_large_file();
            assert_eq!(config.connect_timeout, 60);
            assert_eq!(config.read_timeout, 300);
            assert_eq!(config.total_timeout, 3600);
        }

        #[test]
        fn test_git_operation_config() {
            let config = TimeoutConfig::git_operation();
            assert_eq!(config.connect_timeout, 120);
            assert_eq!(config.read_timeout, 180);
            assert_eq!(config.total_timeout, 1800);
        }

        #[test]
        fn test_connect_duration() {
            let config = TimeoutConfig {
                connect_timeout: 45,
                read_timeout: 90,
                total_timeout: 180,
            };
            assert_eq!(config.connect_duration(), Duration::from_secs(45));
        }

        #[test]
        fn test_read_duration() {
            let config = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 120,
                total_timeout: 300,
            };
            assert_eq!(config.read_duration(), Duration::from_secs(120));
        }

        #[test]
        fn test_total_duration() {
            let config = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 60,
                total_timeout: 600,
            };
            assert_eq!(config.total_duration(), Duration::from_secs(600));
        }

        #[test]
        fn test_duration_conversions_zero_values() {
            let config = TimeoutConfig {
                connect_timeout: 0,
                read_timeout: 0,
                total_timeout: 0,
            };
            assert_eq!(config.connect_duration(), Duration::from_secs(0));
            assert_eq!(config.read_duration(), Duration::from_secs(0));
            assert_eq!(config.total_duration(), Duration::from_secs(0));
        }

        #[test]
        fn test_duration_conversions_large_values() {
            let config = TimeoutConfig {
                connect_timeout: u64::MAX,
                read_timeout: u64::MAX,
                total_timeout: u64::MAX,
            };
            assert_eq!(config.connect_duration(), Duration::from_secs(u64::MAX));
            assert_eq!(config.read_duration(), Duration::from_secs(u64::MAX));
            assert_eq!(config.total_duration(), Duration::from_secs(u64::MAX));
        }

        #[test]
        fn test_validation_valid_config() {
            let config = TimeoutConfig {
                connect_timeout: 10,
                read_timeout: 20,
                total_timeout: 30,
            };
            assert!(config.validate());
        }

        #[test]
        fn test_validation_zero_connect_timeout() {
            let config = TimeoutConfig {
                connect_timeout: 0,
                read_timeout: 60,
                total_timeout: 300,
            };
            assert!(!config.validate());
        }

        #[test]
        fn test_validation_zero_read_timeout() {
            let config = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 0,
                total_timeout: 300,
            };
            assert!(!config.validate());
        }

        #[test]
        fn test_validation_zero_total_timeout() {
            let config = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 60,
                total_timeout: 0,
            };
            assert!(!config.validate());
        }

        #[test]
        fn test_validation_all_zero_values() {
            let config = TimeoutConfig {
                connect_timeout: 0,
                read_timeout: 0,
                total_timeout: 0,
            };
            assert!(!config.validate());
        }

        #[test]
        fn test_validation_edge_case_one_second() {
            let config = TimeoutConfig {
                connect_timeout: 1,
                read_timeout: 1,
                total_timeout: 1,
            };
            assert!(config.validate());
        }

        #[test]
        fn test_partial_eq() {
            let config1 = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 60,
                total_timeout: 300,
            };
            let config2 = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 60,
                total_timeout: 300,
            };
            let config3 = TimeoutConfig {
                connect_timeout: 31,
                read_timeout: 60,
                total_timeout: 300,
            };

            assert_eq!(config1, config2);
            assert_ne!(config1, config3);
        }

        #[test]
        fn test_clone() {
            let config = TimeoutConfig {
                connect_timeout: 45,
                read_timeout: 90,
                total_timeout: 180,
            };
            let cloned_config = config.clone();
            assert_eq!(config, cloned_config);
        }

        #[test]
        fn test_debug_format() {
            let config = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 60,
                total_timeout: 300,
            };
            let debug_str = format!("{config:?}");
            assert!(debug_str.contains("connect_timeout: 30"));
            assert!(debug_str.contains("read_timeout: 60"));
            assert!(debug_str.contains("total_timeout: 300"));
        }
    }

    // ProgressTracker 测试
    mod progress_tracker {
        use super::*;

        #[test]
        fn test_new_tracker() {
            let duration = Duration::from_secs(10);
            let tracker = ProgressTracker::new(duration);

            assert_eq!(tracker.timeout_duration, duration);
            assert_eq!(tracker.total_downloaded, 0);
            assert_eq!(tracker.total_expected, None);
            assert!(!tracker.has_timed_out());
        }

        #[test]
        fn test_update_with_total() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(50, Some(100));

            assert_eq!(tracker.downloaded(), 50);
            assert_eq!(tracker.total_expected(), Some(100));
            assert_eq!(tracker.progress_percent(), Some(50.0));
            assert!(!tracker.has_timed_out());
        }

        #[test]
        fn test_update_without_total() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(50, None);

            assert_eq!(tracker.downloaded(), 50);
            assert_eq!(tracker.total_expected(), None);
            assert_eq!(tracker.progress_percent(), None);
            assert!(!tracker.has_timed_out());
        }

        #[test]
        fn test_progress_percent_zero_total() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(50, Some(0));

            assert_eq!(tracker.progress_percent(), None);
        }

        #[test]
        fn test_progress_percent_complete() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(100, Some(100));

            assert_eq!(tracker.progress_percent(), Some(100.0));
        }

        #[test]
        fn test_progress_percent_half() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(50, Some(100));

            assert_eq!(tracker.progress_percent(), Some(50.0));
        }

        #[test]
        fn test_progress_percent_partial() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(33, Some(100));

            let percent = tracker.progress_percent().unwrap();
            assert!((percent - 33.0).abs() < 0.01);
        }

        #[test]
        fn test_progress_percent_large_numbers() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(u64::MAX / 2, Some(u64::MAX));

            let percent = tracker.progress_percent().unwrap();
            assert!((percent - 50.0).abs() < 0.01);
        }

        #[test]
        fn test_reset() {
            let mut tracker = ProgressTracker::new(Duration::from_millis(100));

            // 更新进度
            tracker.update(50, Some(100));
            assert_eq!(tracker.downloaded(), 50);

            // 等待一段时间
            std::thread::sleep(Duration::from_millis(50));
            let old_time = tracker.last_activity;

            // 重置
            tracker.reset();
            assert_eq!(tracker.downloaded(), 50); // 下载字节数不变
            assert_eq!(tracker.total_expected(), Some(100)); // 总期望字节数不变
            assert!(tracker.last_activity > old_time); // 时间被重置
            assert!(!tracker.has_timed_out()); // 重置后不应该超时
        }

        #[test]
        fn test_timeout_behavior() {
            let mut tracker = ProgressTracker::new(Duration::from_millis(50));

            // 初始状态不应该超时
            assert!(!tracker.has_timed_out());

            // 更新进度
            tracker.update(10, Some(100));
            assert!(!tracker.has_timed_out());

            // 等待超时
            std::thread::sleep(Duration::from_millis(60));
            assert!(tracker.has_timed_out());

            // 更新进度后应该重置超时状态
            tracker.update(20, Some(100));
            assert!(!tracker.has_timed_out());
        }

        #[test]
        fn test_zero_timeout_duration() {
            let tracker = ProgressTracker::new(Duration::from_millis(1));

            // 立即检查，可能还没超时
            let _timed_out = tracker.has_timed_out();
            // 短暂等待后应该超时
            std::thread::sleep(Duration::from_millis(2));
            assert!(tracker.has_timed_out());
        }

        #[test]
        fn test_large_timeout_duration() {
            let tracker = ProgressTracker::new(Duration::from_secs(3600)); // 1小时
            assert!(!tracker.has_timed_out());
        }

        #[test]
        fn test_multiple_updates() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));

            // 多次更新进度
            tracker.update(10, Some(100));
            assert_eq!(tracker.progress_percent(), Some(10.0));

            tracker.update(25, Some(100));
            assert_eq!(tracker.progress_percent(), Some(25.0));

            tracker.update(75, Some(100));
            assert_eq!(tracker.progress_percent(), Some(75.0));

            tracker.update(100, Some(100));
            assert_eq!(tracker.progress_percent(), Some(100.0));

            assert!(!tracker.has_timed_out());
        }

        #[test]
        fn test_update_changes_total_expected() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));

            tracker.update(50, Some(100));
            assert_eq!(tracker.total_expected(), Some(100));
            assert_eq!(tracker.progress_percent(), Some(50.0));

            // 更新时改变总期望大小
            tracker.update(50, Some(200));
            assert_eq!(tracker.total_expected(), Some(200));
            assert_eq!(tracker.progress_percent(), Some(25.0));
        }

        #[test]
        fn test_clone_tracker() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(50, Some(100));

            let cloned_tracker = tracker.clone();

            assert_eq!(cloned_tracker.downloaded(), 50);
            assert_eq!(cloned_tracker.total_expected(), Some(100));
            assert_eq!(cloned_tracker.progress_percent(), Some(50.0));
            assert!(!cloned_tracker.has_timed_out());
        }

        #[test]
        fn test_debug_format_tracker() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(10));
            tracker.update(50, Some(100));

            let debug_str = format!("{tracker:?}");
            assert!(debug_str.contains("last_activity"));
            assert!(debug_str.contains("timeout_duration"));
            assert!(debug_str.contains("total_downloaded"));
            assert!(debug_str.contains("total_expected"));
        }

        #[test]
        fn test_tracker_behavior_under_load() {
            let mut tracker = ProgressTracker::new(Duration::from_millis(100));

            // 模拟频繁更新
            for i in 1..=100 {
                tracker.update(i, Some(100));
                assert_eq!(tracker.downloaded(), i);

                // 使用近似比较来处理浮点数精度问题
                let percent = tracker.progress_percent().unwrap();
                assert!(
                    (percent - i as f64).abs() < 0.0001,
                    "Expected {}, got {} for i={}",
                    i as f64,
                    percent,
                    i
                );

                assert!(!tracker.has_timed_out());

                // 每次更新后短暂等待，确保不会超时
                std::thread::sleep(Duration::from_millis(1));
            }
        }

        #[test]
        fn test_edge_case_max_values() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(1));

            tracker.update(u64::MAX, Some(u64::MAX));
            assert_eq!(tracker.downloaded(), u64::MAX);
            assert_eq!(tracker.total_expected(), Some(u64::MAX));
            assert_eq!(tracker.progress_percent(), Some(100.0));
        }

        #[test]
        fn test_edge_case_zero_values() {
            let mut tracker = ProgressTracker::new(Duration::from_secs(1));

            tracker.update(0, Some(0));
            assert_eq!(tracker.downloaded(), 0);
            assert_eq!(tracker.total_expected(), Some(0));
            assert_eq!(tracker.progress_percent(), None);

            tracker.update(0, None);
            assert_eq!(tracker.downloaded(), 0);
            assert_eq!(tracker.total_expected(), None);
            assert_eq!(tracker.progress_percent(), None);
        }
    }

    // 序列化/反序列化测试
    mod serde_tests {
        use super::*;

        #[test]
        fn test_serialize_yaml() {
            let config = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 60,
                total_timeout: 300,
            };

            let yaml = serde_yaml::to_string(&config).unwrap();
            assert!(yaml.contains("connect-timeout: 30"));
            assert!(yaml.contains("read-timeout: 60"));
            assert!(yaml.contains("total-timeout: 300"));
        }

        #[test]
        fn test_deserialize_yaml() {
            let yaml = r#"
connect-timeout: 45
read-timeout: 90
total-timeout: 180
"#;

            let config: TimeoutConfig = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(config.connect_timeout, 45);
            assert_eq!(config.read_timeout, 90);
            assert_eq!(config.total_timeout, 180);
        }

        #[test]
        fn test_serialize_json() {
            let config = TimeoutConfig {
                connect_timeout: 30,
                read_timeout: 60,
                total_timeout: 300,
            };

            let json = serde_json::to_string(&config).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

            assert_eq!(parsed["connect-timeout"], 30);
            assert_eq!(parsed["read-timeout"], 60);
            assert_eq!(parsed["total-timeout"], 300);
        }

        #[test]
        fn test_deserialize_json() {
            let json = r#"
{
    "connect-timeout": 45,
    "read-timeout": 90,
    "total-timeout": 180
}
"#;

            let config: TimeoutConfig = serde_json::from_str(json).unwrap();
            assert_eq!(config.connect_timeout, 45);
            assert_eq!(config.read_timeout, 90);
            assert_eq!(config.total_timeout, 180);
        }

        #[test]
        fn test_serialize_roundtrip_yaml() {
            let original = TimeoutConfig {
                connect_timeout: 120,
                read_timeout: 240,
                total_timeout: 600,
            };

            let yaml = serde_yaml::to_string(&original).unwrap();
            let deserialized: TimeoutConfig = serde_yaml::from_str(&yaml).unwrap();

            assert_eq!(original, deserialized);
        }

        #[test]
        fn test_serialize_roundtrip_json() {
            let original = TimeoutConfig {
                connect_timeout: 120,
                read_timeout: 240,
                total_timeout: 600,
            };

            let json = serde_json::to_string(&original).unwrap();
            let deserialized: TimeoutConfig = serde_json::from_str(&json).unwrap();

            assert_eq!(original, deserialized);
        }

        #[test]
        fn test_deserialize_missing_field() {
            let yaml = "connect-timeout: 30\nread-timeout: 60";
            let result: Result<TimeoutConfig, _> = serde_yaml::from_str(yaml);
            assert!(result.is_err());
        }

        #[test]
        fn test_deserialize_extra_field() {
            let yaml = r#"
connect-timeout: 30
read-timeout: 60
total-timeout: 300
extra-field: "ignored"
"#;

            let result: Result<TimeoutConfig, _> = serde_yaml::from_str(yaml);
            // 这应该成功，因为serde默认忽略额外字段
            assert!(result.is_ok());
        }
    }
}
