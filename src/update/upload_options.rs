use crate::vars::{ValueDict, ValueType};
use std::str::FromStr;

/// HTTP methods supported for upload operations
#[derive(Debug, Clone, PartialEq, derive_more::Display)]
pub enum HttpMethod {
    #[display("PUT")]
    /// PUT request for binary upload
    Put,
    #[display("POST")]
    /// POST request for form-data upload
    Post,
    #[display("PATCH")]
    /// PATCH request for partial updates
    Patch,
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Put
    }
}

/// 转换错误类型
#[derive(Debug, thiserror::Error)]
#[error("无效的HTTP方法: {0}")]
pub struct ParseHttpMethodError(String);

impl FromStr for HttpMethod {
    type Err = ParseHttpMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PUT" => Ok(HttpMethod::Put),
            "POST" => Ok(HttpMethod::Post),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(ParseHttpMethodError(s.to_string())),
        }
    }
}

impl TryFrom<&str> for HttpMethod {
    type Error = ParseHttpMethodError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        HttpMethod::from_str(value)
    }
}

impl TryFrom<String> for HttpMethod {
    type Error = ParseHttpMethodError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        HttpMethod::from_str(&value)
    }
}

/// Options for controlling upload operations, primarily focused on HTTP uploads
#[derive(Clone, Debug, Default)]
pub struct UploadOptions {
    /// HTTP method to use for upload
    http_method: HttpMethod,
    /// Whether to compress the resource before upload
    compression: bool,
    /// Additional metadata to include in upload headers
    metadata: ValueDict,
}

impl UploadOptions {
    /// Create new UploadOptions with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create UploadOptions for specific HTTP method
    pub fn with_method(method: HttpMethod) -> Self {
        Self {
            http_method: method,
            compression: false,
            metadata: ValueDict::default(),
        }
    }

    /// Set HTTP method for upload
    pub fn method(mut self, method: HttpMethod) -> Self {
        self.http_method = method;
        self
    }

    /// Enable or disable compression
    pub fn compression(mut self, enable: bool) -> Self {
        self.compression = enable;
        self
    }

    /// Add metadata to upload
    /// 添加自定义元数据
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata
            .insert(key.into(), ValueType::String(value.into()));
        self
    }

    /// Get the HTTP method
    pub fn http_method(&self) -> &HttpMethod {
        &self.http_method
    }

    /// Check if compression is enabled
    pub fn compression_enabled(&self) -> bool {
        self.compression
    }

    /// Get metadata
    pub fn metadata_dict(&self) -> &ValueDict {
        &self.metadata
    }

    /// Create for testing purposes
    pub fn for_test() -> Self {
        Self {
            http_method: HttpMethod::Put,
            compression: false,
            metadata: ValueDict::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_from_str_valid_cases() {
        assert_eq!(HttpMethod::from_str("PUT").unwrap(), HttpMethod::Put);
        assert_eq!(HttpMethod::from_str("put").unwrap(), HttpMethod::Put);
        assert_eq!(HttpMethod::from_str("Put").unwrap(), HttpMethod::Put);

        assert_eq!(HttpMethod::from_str("POST").unwrap(), HttpMethod::Post);
        assert_eq!(HttpMethod::from_str("post").unwrap(), HttpMethod::Post);
        assert_eq!(HttpMethod::from_str("Post").unwrap(), HttpMethod::Post);

        assert_eq!(HttpMethod::from_str("PATCH").unwrap(), HttpMethod::Patch);
        assert_eq!(HttpMethod::from_str("patch").unwrap(), HttpMethod::Patch);
        assert_eq!(HttpMethod::from_str("Patch").unwrap(), HttpMethod::Patch);
    }

    #[test]
    fn test_from_str_invalid_cases() {
        assert!(HttpMethod::from_str("GET").is_err());
        assert!(HttpMethod::from_str("DELETE").is_err());
        assert!(HttpMethod::from_str("INVALID").is_err());
        assert!(HttpMethod::from_str("").is_err());
        assert!(HttpMethod::from_str("  PUT  ").is_err()); // 包含空格
    }

    #[test]
    fn test_try_from_str() {
        assert_eq!(HttpMethod::try_from("PUT").unwrap(), HttpMethod::Put);
        assert_eq!(HttpMethod::try_from("Post").unwrap(), HttpMethod::Post);
        assert_eq!(HttpMethod::try_from("patch").unwrap(), HttpMethod::Patch);
    }

    #[test]
    fn test_try_from_string() {
        assert_eq!(
            HttpMethod::try_from(String::from("PUT")).unwrap(),
            HttpMethod::Put
        );
        assert_eq!(
            HttpMethod::try_from(String::from("POST")).unwrap(),
            HttpMethod::Post
        );
        assert_eq!(
            HttpMethod::try_from(String::from("PATCH")).unwrap(),
            HttpMethod::Patch
        );

        assert!(HttpMethod::try_from(String::from("DELETE")).is_err());
        assert!(HttpMethod::try_from(String::from("")).is_err());
    }

    #[test]
    fn test_parse_http_method_error() {
        let err = HttpMethod::from_str("INVALID").unwrap_err();
        assert_eq!(err.to_string(), "无效的HTTP方法: INVALID");

        let err = HttpMethod::from_str("").unwrap_err();
        assert_eq!(err.to_string(), "无效的HTTP方法: ");
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::Put.to_string(), "PUT");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
        assert_eq!(HttpMethod::Patch.to_string(), "PATCH");
    }
}

impl From<(usize, ValueDict)> for UploadOptions {
    fn from((method, values): (usize, ValueDict)) -> Self {
        let http_method = match method {
            0 => HttpMethod::Put,
            1 => HttpMethod::Post,
            2 => HttpMethod::Patch,
            _ => HttpMethod::Put,
        };

        Self {
            http_method,
            compression: false,
            metadata: values,
        }
    }
}

impl From<crate::update::DownloadOptions> for UploadOptions {
    fn from(_: crate::update::DownloadOptions) -> Self {
        Self::default()
    }
}
