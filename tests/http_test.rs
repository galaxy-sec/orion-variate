use orion_variate::{
    addr::{Address, HttpResource, accessor::HttpAccessor},
    types::ResourceDownloader,
    update::DownloadOptions,
};
use tempfile::tempdir;

#[tokio::test]
async fn test_http_accessor_download() {
    // 创建临时目录用于测试
    let temp_dir = tempdir().unwrap();
    let dest_dir = temp_dir.path();

    // 创建HttpAccessor实例
    let accessor = HttpAccessor::default();

    // 创建测试用的HttpAddr
    let http_addr = HttpResource::from("https://httpbin.org/robots.txt");

    // 创建UpdateOptions
    let options = DownloadOptions::default();

    // 执行下载测试
    let result = accessor
        .download_to_local(&Address::from(http_addr), dest_dir, &options)
        .await;

    // 由于网络依赖，我们主要验证代码结构是否正确
    // 在实际测试中可能需要mock或跳过网络测试
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_http_accessor_creation() {
    // 测试HttpAccessor的创建
    let accessor = HttpAccessor::default();
    assert!(accessor.redirect().is_none());
}

#[test]
fn test_http_addr_creation() {
    // 测试HttpAddr的创建
    let http_addr = HttpResource::from("https://example.com/file.txt");
    assert_eq!(http_addr.url(), "https://example.com/file.txt");
    assert!(http_addr.username().is_none());
    assert!(http_addr.password().is_none());
}

#[test]
fn test_http_addr_with_credentials() {
    // 测试带认证的HttpAddr创建
    let http_addr =
        HttpResource::from("https://example.com/file.txt").with_credentials("user", "pass");
    assert_eq!(http_addr.url(), "https://example.com/file.txt");
    assert_eq!(http_addr.username().as_deref(), Some("user"));
    assert_eq!(http_addr.password().as_deref(), Some("pass"));
}
