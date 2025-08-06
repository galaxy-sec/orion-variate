# Addr模块详细日志记录设计方案

## 背景
当前addr模块的日志记录较为简单，主要存在以下问题：
- 日志级别使用不够细致，缺少debug/trace级别的详细日志
- 日志信息不够结构化，难以进行问题追踪和性能分析
- 缺少关键操作的耗时记录
- 错误日志缺少足够的上下文信息
- 没有统一的日志格式规范

## 目标
- 建立完整的日志级别体系（trace/debug/info/warn/error）
- 提供结构化的日志输出，便于分析和监控
- 记录关键操作的耗时和状态
- 为调试和故障排查提供足够的信息
- 保持高性能，避免过度日志影响性能

## 设计方案

### 1. 日志级别定义

#### 1.1 Trace级别
- 最详细的调试信息
- 函数入口/出口参数
- 内部状态变化
- 性能关键路径的详细步骤

#### 1.2 Debug级别
- 重要操作的开始和结束
- 配置加载和解析
- 网络请求的发送和响应
- 文件系统操作

#### 1.3 Info级别
- 服务启动/停止
- 重要配置变更
- 成功的操作完成
- 定期状态报告

#### 1.4 Warn级别
- 非关键错误
- 降级处理
- 配置警告
- 性能警告

#### 1.5 Error级别
- 操作失败
- 配置错误
- 网络错误
- 系统错误

### 2. 结构化日志字段

#### 2.1 通用字段
```rust
{
    "timestamp": "2024-01-01T12:00:00.123Z",
    "level": "INFO",
    "target": "addr::git",
    "module": "git_accessor",
    "file": "src/addr/accessor/git.rs",
    "line": 123,
    "span": {
        "name": "clone_repository",
        "repo_url": "https://github.com/user/repo.git",
        "target_path": "/tmp/repo"
    }
}
```

#### 2.2 操作相关字段
```rust
{
    "operation": "git_clone",
    "duration_ms": 1250,
    "status": "success",
    "bytes_transferred": 1024000,
    "retry_count": 0
}
```

#### 2.3 错误相关字段
```rust
{
    "error_type": "GitAuthentication",
    "error_code": 2005,
    "error_message": "认证失败",
    "context": {
        "repo_url": "https://github.com/user/repo.git",
        "auth_method": "token"
    }
}
```

### 3. 具体实现方案

#### 3.1 日志宏封装
```rust
use tracing::{debug, error, info, instrument, trace, warn};

/// 为Git操作添加详细的日志记录
#[instrument(
    skip(addr, path),
    fields(
        repo_url = %addr.repo(),
        branch = ?addr.branch(),
        tag = ?addr.tag(),
        target_path = %path.display()
    )
)]
pub async fn clone_repository(&self, addr: &GitRepository, path: &Path) -> AddrResult<()> {
    let start = std::time::Instant::now();
    
    info!(
        operation = "git_clone_start",
        "开始克隆Git仓库: {} -> {}",
        addr.repo(),
        path.display()
    );
    
    trace!("验证仓库URL格式: {}", addr.repo());
    
    match self.perform_clone(addr, path).await {
        Ok(_) => {
            let duration = start.elapsed();
            info!(
                operation = "git_clone_success",
                duration_ms = duration.as_millis(),
                "Git仓库克隆成功"
            );
            Ok(())
        }
        Err(e) => {
            let duration = start.elapsed();
            error!(
                operation = "git_clone_error",
                duration_ms = duration.as_millis(),
                error_type = %e.error_code(),
                error_message = %e,
                "Git仓库克隆失败"
            );
            Err(e)
        }
    }
}
```

#### 3.2 HTTP访问器日志
```rust
#[instrument(
    skip(self, addr, path),
    fields(
        url = %addr.url(),
        method = "download",
        target_path = %path.display()
    )
)]
pub async fn download_to_local(
    &self,
    addr: &HttpResource,
    path: &Path,
    options: &UpdateOptions,
) -> AddrResult<()> {
    let start = std::time::Instant::now();
    
    info!(
        operation = "http_download_start",
        url = %addr.url(),
        "开始下载HTTP资源"
    );
    
    debug!("创建HTTP客户端");
    let client = create_http_client();
    
    trace!("发送HTTP请求");
    let response = client.get(addr.url()).send().await
        .map_err(|e| {
            error!(
                operation = "http_request_error",
                error = %e,
                url = %addr.url(),
                "HTTP请求失败"
            );
            AddrError::from(AddrReason::HttpRequest(e.to_string()))
        })?;
    
    let status = response.status();
    debug!(
        operation = "http_response_received",
        status = status.as_u16(),
        "收到HTTP响应"
    );
    
    if !status.is_success() {
        error!(
            operation = "http_error_response",
            status = status.as_u16(),
            url = %addr.url(),
            "HTTP请求返回错误状态"
        );
        return Err(AddrError::from(AddrReason::HttpServerError { 
            status: status.as_u16() 
        }));
    }
    
    let content_length = response.content_length();
    debug!(
        operation = "http_content_info",
        content_length = content_length,
        "获取资源大小信息"
    );
    
    // 下载逻辑...
    
    let duration = start.elapsed();
    info!(
        operation = "http_download_success",
        duration_ms = duration.as_millis(),
        bytes_downloaded = total_bytes,
        "HTTP资源下载成功"
    );
    
    Ok(())
}
```

#### 3.3 验证日志
```rust
#[instrument(skip(self))]
pub fn validate(&self) -> AddrResult<()> {
    let mut errors = Vec::new();
    
    debug!(operation = "validation_start", "开始地址验证");
    
    // 验证URL格式
    if let Some(http) = self.http() {
        trace!("验证HTTP资源: {}", http.url());
        match Url::parse(http.url()) {
            Ok(_) => trace!("HTTP URL格式有效"),
            Err(e) => {
                warn!(
                    operation = "url_validation_failed",
                    url = %http.url(),
                    error = %e,
                    "HTTP URL格式无效"
                );
                errors.push(ValidationError::new(
                    "url",
                    &format!("无效的URL格式: {}", e),
                    "INVALID_URL_FORMAT"
                ));
            }
        }
    }
    
    // 验证Git配置
    if let Some(git) = self.git() {
        trace!("验证Git仓库配置: {}", git.repo());
        
        if git.tag().is_some() && git.branch().is_some() {
            warn!(
                operation = "git_config_warning",
                repo = %git.repo(),
                "同时指定了tag和branch，将优先使用tag"
            );
        }
    }
    
    if errors.is_empty() {
        debug!(operation = "validation_success", "地址验证成功");
        Ok(())
    } else {
        error!(
            operation = "validation_failed",
            error_count = errors.len(),
            "地址验证失败"
        );
        Err(AddrError::from(AddrReason::Validation(errors)))
    }
}
```

### 4. 性能考虑

#### 4.1 日志级别控制
```rust
/// 条件日志宏，避免在高级别日志时产生开销
macro_rules! log_if_enabled {
    ($level:expr, $($arg:tt)*) => {
        if log::log_enabled!($level) {
            log::log!($level, $($arg)*);
        }
    };
}
```

#### 4.2 异步日志
- 使用`tracing`的异步日志功能
- 避免在热路径中同步日志
- 使用`spawn_blocking`处理文件日志

### 5. 配置支持

#### 5.1 日志配置结构
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 全局日志级别
    pub level: LogLevel,
    
    /// 模块特定日志级别
    pub modules: HashMap<String, LogLevel>,
    
    /// 是否启用结构化日志
    pub structured: bool,
    
    /// 是否包含文件位置信息
    pub include_location: bool,
    
    /// 是否包含span信息
    pub include_span: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {\    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
```

#### 5.2 环境变量配置
```bash
# 设置全局日志级别
export ORION_LOG_LEVEL=debug

# 设置模块特定日志级别
export ORION_ADDR_LOG_LEVEL=trace
export ORION_HTTP_LOG_LEVEL=info

# 启用结构化日志
export ORION_STRUCTURED_LOG=true
```

### 6. 监控和诊断

#### 6.1 性能指标
- 操作耗时统计
- 错误率监控
- 重试次数统计
- 网络流量统计

#### 6.2 诊断工具
- 日志采样
- 错误聚合
- 性能分析
- 调试模式

### 7. 测试策略

#### 7.1 日志测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;
    
    #[test]
    #[traced_test]
    fn test_git_clone_logging() {
        let accessor = GitAccessor::new();
        let addr = GitRepository::from("https://github.com/test/repo.git");
        
        // 测试日志输出
        let result = accessor.clone_repository(&addr, Path::new("/tmp/test"));
        
        // 验证日志包含预期内容
        assert!(logs_contain("开始克隆Git仓库"));
    }
}
```

#### 7.2 性能测试
- 日志开销测试
- 内存使用测试
- 并发安全测试

## 实施步骤

1. **设计阶段**（当前）
   - 完成详细日志设计方案
   - 评审设计方案

2. **实现阶段**
   - 添加`tracing`依赖
   - 实现日志配置结构
   - 为每个模块添加详细日志
   - 实现性能监控

3. **测试阶段**
   - 编写日志测试
   - 性能测试
   - 集成测试

4. **文档阶段**
   - 更新配置文档
   - 添加日志指南
   - 更新示例代码

## 预期结果

- 完整的日志级别体系
- 结构化的日志输出
- 详细的操作记录
- 便于调试和监控
- 高性能的日志系统

## 风险与考虑

- **性能影响**：需要平衡日志详细度和性能
- **存储成本**：结构化日志可能增加存储需求
- **配置复杂性**：需要简化配置使用
- **向后兼容性**：确保不影响现有日志使用