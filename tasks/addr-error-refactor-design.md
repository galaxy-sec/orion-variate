# Addr模块错误类型重构设计方案（简化版）

## 背景
当前addr模块的错误处理存在以下问题：
- `AddrReason`枚举定义了过多的错误变体，上层无法有效处理
- 错误分类过于细致，导致使用复杂
- 需要简化错误类型，同时保留关键信息

## 目标
- 将错误类型减少到5-7个核心类别
- 每个错误类型包含详细的上下文信息
- 保持向后兼容性
- 简化上层错误处理逻辑

## 简化设计方案

### 1. 核心错误类型结构

```rust
use derive_more::From;
use orion_error::{ErrorCode, StructError, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum AddrReason {
    // 通用错误 - 用于无法归类的错误
    #[error("{0}")]
    Generic(String),
    
    // 上游错误 - 保持兼容性
    #[error("{0}")]
    Uvs(UvsReason),
    
    // 网络相关错误（Git、HTTP等）
    #[error("网络错误: {0}")]
    Network(String),
    
    // 权限和认证错误
    #[error("权限错误: {0}")]
    Permission(String),
    
    // 资源未找到错误
    #[error("资源未找到: {0}")]
    NotFound(String),
}

// 验证错误类型（作为Configuration错误的具体实现）
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidationError {
    /// 错误字段
    pub field: String,
    /// 错误描述
    pub message: String,
    /// 错误代码
    pub code: String,
}

impl ValidationError {
    pub fn new(field: &str, message: &str, code: &str) -> Self {
        Self {
            field: field.to_string(),
            message: message.to_string(),
            code: code.to_string(),
        }
    }
}

impl ErrorCode for AddrReason {
    fn error_code(&self) -> i32 {
        match self {
            AddrReason::Generic(_) => 1000,
            AddrReason::Uvs(r) => r.error_code(),
            AddrReason::Network(_) => 3000,
            AddrReason::Permission(_) => 5000,
            AddrReason::NotFound(_) => 6000,
        }
    }
}

pub type AddrResult<T> = Result<T, StructError<AddrReason>>;
pub type AddrError = StructError<AddrReason>;
```


### 2. 辅助创建函数

```rust
impl AddrReason {
    /// 创建配置错误（包含验证错误）
    pub fn config_error(message: &str, validation_errors: Option<Vec<ValidationError>>) -> Self {
        if let Some(errors) = validation_errors {
            let details = serde_json::to_string(&errors).unwrap_or_default();
            Self::Configuration(format!("{}: {}", message, details))
        } else {
            Self::Configuration(message.to_string())
        }
    }
    
    /// 创建网络错误
    pub fn network_error(operation: &str, details: &str) -> Self {
        Self::Network(format!("{}: {}", operation, details))
    }
    
    /// 创建权限错误
    pub fn permission_error(resource: &str, details: &str) -> Self {
        Self::Permission(format!("访问 {} 失败: {}", resource, details))
    }
    
    /// 创建资源未找到错误
    pub fn not_found_error(resource_type: &str, identifier: &str) -> Self {
        Self::NotFound(format!("{} '{}' 未找到", resource_type, identifier))
    }
}
```

### 4. 使用示例

#### 4.1 地址解析
```rust
impl FromStr for Address {
    type Err = AddrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        
        if s.is_empty() {
            return Err(AddrReason::config_error("地址不能为空", None).into());
        }
        
        // 解析逻辑...
        
        Err(AddrReason::config_error(
            &format!("无效的地址格式: {}", s),
            None
        ).into())
    }
}
```

#### 4.2 Git操作
```rust
impl GitAccessor {
    async fn clone_repository(&self, url: &str, path: &Path) -> AddrResult<()> {
        git2::Repository::clone(url, path)
            .map_err(|e| match e.code() {
                git2::ErrorCode::NotFound => {
                    AddrReason::not_found_error("Git仓库", url).into()
                }
                git2::ErrorCode::Auth => {
                    AddrReason::permission_error(url, "认证失败").into()
                }
                _ => AddrReason::network_error("Git克隆", &e.message().to_string()).into(),
            })?
            .with_operation("clone_repository")
            .with_resource(url)
    }
}
```

#### 4.3 验证错误
```rust
impl Address {
    pub fn validate(&self) -> AddrResult<()> {
        let mut errors = Vec::new();
        
        // 验证逻辑...
        if let Some(git) = self.git() {
            if git.tag().is_some() && git.branch().is_some() {
                errors.push(ValidationError::new(
                    "version", 
                    "不能同时指定tag和branch", 
                    "CONFLICTING_VERSIONS"
                ));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(AddrReason::config_error("地址配置验证失败", Some(errors)).into())
        }
    }
}
```

### 5. 上层处理策略

上层可以根据错误类型进行统一处理：

```rust
match error.reason() {
    AddrReason::Configuration(_) => {
        // 配置错误：提示用户检查配置
        log::warn!("配置错误: {}", error);
        // 提供配置修复建议
    }
    AddrReason::Network(_) => {
        // 网络错误：重试或提示检查网络
        log::error!("网络错误: {}", error);
        // 重试逻辑
    }
    AddrReason::Permission(_) => {
        // 权限错误：提示用户检查权限
        log::error!("权限错误: {}", error);
        // 权限检查建议
    }
    AddrReason::NotFound(_) => {
        // 资源未找到：提示用户检查资源是否存在
        log::warn!("资源未找到: {}", error);
    }
    AddrReason::FileSystem(_) => {
        // 文件系统错误：提示检查磁盘空间、路径等
        log::error!("文件系统错误: {}", error);
    }
    _ => {
        // 其他错误：记录并向上传递
        log::error!("未知错误: {}", error);
    }
}
```

### 6. 向后兼容性

- 保留`AddrResult`和`AddrError`类型别名
- 保留`Uvs(UvsReason)`变体
- 提供从旧错误到新错误的转换函数

### 7. 测试策略

#### 7.1 单元测试
- 测试每个错误类型的创建和序列化
- 测试错误上下文添加
- 测试辅助函数

#### 7.2 集成测试
- 测试错误在模块间的传递
- 测试上层错误处理逻辑
- 验证向后兼容性

## 实施步骤

1. **设计确认**（当前）
   - 确认简化后的错误类型设计
   - 评审上下文信息方案

2. **实现阶段**
   - 重构`AddrReason`枚举（减少到7个变体）
   - 实现错误上下文功能
   - 更新辅助创建函数
   - 修改现有代码使用新的错误类型

3. **测试阶段**
   - 编写单元测试
   - 更新集成测试
   - 验证上层处理逻辑

4. **文档更新**
   - 更新API文档
   - 添加错误处理最佳实践
   - 更新示例代码

## 预期结果

- 错误类型从20+减少到7个核心类别
- 上层处理逻辑大幅简化
- 通过上下文信息保留足够的调试信息
- 保持向后兼容性
- 更清晰的错误处理策略