use crate::vars::{ValueDict, ValueType};

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
