# Download Timeout Control Design (下载超时控制设计方案)

## 问题分析

目前项目中的download功能存在以下超时相关问题：

1. **固定超时设置**：当前使用30秒的固定超时，对于大文件下载或慢速网络环境不够用
2. **缺乏重试机制**：网络临时中断时，没有自动重试机制
3. **无进度超时**：长时间下载过程中，没有确认数据传输的超时保护
4. **操作区分粒度粗**：HTTP下载和Git下载使用同一超时设置，没有针对性优化

## 解决方案

### 1. 分层超时控制架构

```
┌─────────────────────────────────────────┐
│        Download Timeout Manager         │
├─────────────────────────────────────────┤
│   Connection Timeout   │  Read Timeout  │
│   (连接阶段超时)        │  (数据传输超时) │
├─────────────────────────────────────────┤
│   Retry Policy Layer   │                │
│   (重试策略层)          │                │
└─────────────────────────────────────────┘
```

### 2. 具体实现方案

#### 2.1 新的超时配置结构

```rust
/// 下载超时配置
#[derive(Debug, Clone)]
pub struct DownloadTimeoutConfig {
    /// 连接超时时间 (默认: 30秒)
    pub connect_timeout: Duration,
    /// 读取超时时间 (默认: 60秒)
    pub read_timeout: Duration,
    /// 总操作超时时间 (默认: 5分钟)
    pub total_timeout: Duration,
    /// 是否启用超时重试
    pub retry_on_timeout: bool,
    /// 重试次数 (默认: 3次)
    pub max_retries: u32,
    /// 重试间隔时间 (默认: 2秒)
    pub retry_interval: Duration,
}

impl Default for DownloadTimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
            total_timeout: Duration::from_secs(300), // 5分钟
            retry_on_timeout: true,
            max_retries: 3,
            retry_interval: Duration::from_secs(2),
        }
    }
}
```

#### 2.2 针对不同类型的专属配置

```rust
impl DownloadTimeoutConfig {
    /// HTTP模式配置 (适用于小文件)
    pub fn http_simple() -> Self {
        Self::default()
    }

    /// HTTP大文件模式配置
    pub fn http_large_file() -> Self {
        Self {
            read_timeout: Duration::from_secs(300),
            total_timeout: Duration::from_secs(3600), // 1小时
            max_retries: 5,
            ..Default::default()
        }
    }

    /// Git操作配置
    pub fn git_operation() -> Self {
        Self {
            connect_timeout: Duration::from_secs(120),
            read_timeout: Duration::from_secs(180),
            total_timeout: Duration::from_secs(1800), // 30分钟
            max_retries: 2,
            retry_interval: Duration::from_secs(10),
            ..Default::default()
        }
    }
}
```

### 3. 集成到现有系统

#### 3.1 扩展DownloadOptions

为现有的`DownloadOptions`结构增加超时配置：

```rust
#[derive(Debug, Clone, Default)]
pub struct DownloadOptions {
    scope_level: UpdateScope,
    values: ValueDict,
    timeout_config: Option<DownloadTimeoutConfig>, // 新增
}

impl DownloadOptions {
    /// 为有特定超时需求的场景创建自定义配置
    pub fn with_timeout_config(mut self, config: DownloadTimeoutConfig) -> Self {
        self.timeout_config = Some(config);
        self
    }

    pub fn timeout_config(&self) -> DownloadTimeoutConfig {
        self.timeout_config.clone().unwrap_or_else(|| {
            // 根据Address类型自动选择配置
            match self.scope_level {
                UpdateScope::None => DownloadTimeoutConfig::default(),
                UpdateScope::RemoteCache => DownloadTimeoutConfig::http_simple(),
            }
        })
    }
}
```

#### 3.2 HTTP客户端增强版创建函数

```rust
/// 根据超时配置创建HTTP客户端
pub fn create_http_client_with_config(config: &DownloadTimeoutConfig) -> reqwest::Client {
    let client_builder = ClientBuilder::new()
        .connect_timeout(config.connect_timeout)
        .timeout(config.total_timeout)
        .tcp_keepalive(Duration::from_secs(60))
        .pool_idle_timeout(Duration::from_secs(90));

    client_builder.build().unwrap_or_else(|e| {
        log::error!("创建HTTP客户端失败: {}", e);
        // 降级使用基础配置
        reqwest::Client::new()
    })
}
```

### 4. 实现超时重试机制

#### 4.1 带重试的下载包装器

```rust
### 4.1 带重试的下载包装器

```rust
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

/// 带有超时重试逻辑的下载包装器
pub async fn download_with_retry<F, Fut, T>(
    operation: F,
    config: &DownloadTimeoutConfig,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    let mut attempts = 0;
    let start_time = Instant::now();

    loop {
        attempts += 1;
        
        match tokio::time::timeout(config.total_timeout, operation()).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(e)) => {
                if attempts >= config.max_retries && !config.retry_on_timeout {
                    return Err(e);
                }
                
                tracing::warn!(
                    "下载失败 (尝试 {} / {})，将在 {}s 后重试: {}",
                    attempts,
                    config.max_retries,
                    config.retry_interval.as_secs(),
                    e
                );
                
                if start_time.elapsed() + config.retry_interval > config.total_timeout {
                    return Err("操作超时".into());
                }
                
                tokio::time::sleep(config.retry_interval).await;
            }
            Err(_timeout) => {
                if attempts >= config.max_retries {
                    return Err("操作超时且重试次数耗尽".into());
                }
                
                tracing::warn!(
                    "下载超时 (尝试 {} / {})，将在 {}s 后重试",
                    attempts,
                    config.max_retries,
                    config.retry_interval.as_secs()
                );
                
                if start_time.elapsed() + config.retry_interval > config.total_timeout {
                    return Err("操作总超时".into());
                }
                
                tokio::time::sleep(config.retry_interval).await;
            }
        }
    }
}
```

### 4.2 进度监控超时

```rust
/// 带进度监控的下载器
pub struct ProgressDownloadTracker {
    last_activity: Instant,
    timeout_interval: Duration,
    progress_callback: Box<dyn Fn(u64, u64) + Send + Sync>,
}

impl ProgressDownloadTracker {
    pub fn new(timeout_interval: Duration, callback: impl Fn(u64, u64) + Send + Sync + 'static) -> Self {
        Self {
            last_activity: Instant::now(),
            timeout_interval,
            progress_callback: Box::new(callback),
        }
    }
    
    pub fn update_progress(&mut self, downloaded: u64, total: u64) {
        self.last_activity = Instant::now();
        (self.progress_callback)(downloaded, total);
    }
    
    pub fn check_timeout(&self) -> bool {
        self.last_activity.elapsed() > self.timeout_interval
    }
}
```

### 5. 配置暴露和用法
#### 5.1 从环境变量读取配置

```rust
impl DownloadTimeoutConfig {
    pub fn from_env() -> Self {
        let connect_timeout = std::env::var("ORION_CONNECT_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30));
            
        let read_timeout = std::env::var("ORION_READ_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(60));
            
        let max_retries = std::env::var("ORION_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);
            
        Self {
            connect_timeout,
            read_timeout,
            max_retries,
            ..Default::default()
        }
    }
}
```

### 6. 与现有代码的集成
#### 6.1 HttpAccessor中的集成
在`download`函数中替换现有超时逻辑：

```rust
pub async fn download(
    &self,
    addr: &HttpResource,
    dest_path: &Path,
    options: &DownloadOptions,
) -> AddrResult<PathBuf> {
    let timeout_config = options.timeout_config();
    
    // 使用定制化客户端
    let client = create_http_client_with_config(&timeout_config);
    
    // 集成带重试的下载逻辑
    download_with_retry(
        || async { /* 实际的下载逻辑 */ },
        &timeout_config,
    ).await
}
```

### 7. 测试策略
#### 7.1 超时测试
- 验证超时配置正确生效
- 测试重试机制在不同failure类型下的行为
- 验证