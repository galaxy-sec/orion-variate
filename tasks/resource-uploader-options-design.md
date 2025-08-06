# ResourceUploader UploadOptions 设计方案（已优化）

## 背景
当前 `ResourceUploader` trait 的 `upload_from_local` 方法使用 `DownloadOptions` 作为参数类型，这在语义上不合理。本方案提供一个简化且实用的 `UploadOptions` 设计，专门针对HTTP上传场景进行优化。

## 问题分析
1. **语义不一致**: `upload_from_local` 使用 `DownloadOptions` 造成理解困难
2. **功能缺失**: 缺乏针对HTTP上传的特定配置选项
3. **过度设计**: 初期版本过于复杂，引入了大量不必要的选项

## 简化设计目标
- 聚焦HTTP上传的核心需求
- 提供简洁实用的配置选项
- 保持向后兼容性
- 易于使用和理解

## 优化后的设计方案

### 1. 核心结构

```rust
/// HTTP方法枚举，支持不同的上传方式
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Put,    // PUT请求用于二进制上传 (HTTP/RESTful)
    Post,   // POST请求用于form-data上传
    Patch,  // PATCH请求用于部分更新
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Put  // 默认使用PUT，符合RESTful设计
    }
}

/// 简化后的上传参数配置
#[derive(Clone, Debug, Default)]
pub struct UploadOptions {
    /// HTTP上传方法：PUT/POST/PATCH
    http_method: HttpMethod,
    /// 是否在上传前进行压缩
    compression: bool,
    /// 附加的元数据（将转换为HTTP headers）
    metadata: ValueDict,
}
```

### 2. 使用方法

#### 2.1 创建方式
```rust
// 默认配置（PUT方法，不压缩）
let opts = UploadOptions::new();

// 指定HTTP方法
let opts = UploadOptions::with_method(HttpMethod::Post);

// 链式配置
let opts = UploadOptions::new()
    .method(HttpMethod::Post)
    .compression(true)
    .metadata("Content-Type".into(), "application/json".into());
```

#### 2.2 API设计
```rust
impl UploadOptions {
    /// 创建默认配置
    pub fn new() -> Self

    /// 指定HTTP方法创建
    pub fn with_method(method: HttpMethod) -> Self

    /// 设置HTTP方法
    pub fn method(mut self, method: HttpMethod) -> Self

    /// 启用/禁用压缩
    pub fn compression(mut self, enable: bool) -> Self

    /// 添加自定义元数据
    pub fn metadata(mut self, key: String, value: String) -> Self

    /// 获取HTTP方法
    pub fn http_method(&self) -> &HttpMethod

    /// 是否启用压缩
    pub fn compression_enabled(&self) -> bool
}
```

### 3. 兼容性处理


#### 3.2 测试配置
```rust
impl UploadOptions {
    /// 测试专用配置
    pub fn for_test() -> Self {
        Self {
            http_method: HttpMethod::Post,
            compression: false,
            metadata: ValueDict::default(),
        }
    }
}
```

### 4. 使用场景示例

### 4.1 典型的文件上传
```rust
use my_crate::update::{UploadOptions, HttpMethod};

let opts = UploadOptions::new()
    .method(HttpMethod::Post)
    .compression(true)
    .metadata("X-Custom-Version".into(), "1.0.0".into());

let uploader = GitAccessor::new();
uploader.upload_from_local(&addr, &path, &opts).await?;
```

### 4.2 简单的默认上传
```rust
let uploader = HttpAccessor::new();
uploader.upload_from_local(&addr, &path, &UploadOptions::new()).await?;
```

## 实现更新要求

### 需要修改的模块
1. `src/addr/accessor/git.rs` – 添加PUT上传支持
2. `src/addr/accessor/http.rs` – 实现UploadOptions配置
3. `src/addr/accessor/local.rs` – 简化配置（可忽略HTTP配置）
4. `src/addr/accessor/core.rs` – 接口更新
5. `src/addr/accessor/universal.rs` – 接口更新

### 测试更新
更新所有测试用例，从使用 `DownloadOptions::default()` 改为 `UploadOptions::new()` 或 `UploadOptions::for
