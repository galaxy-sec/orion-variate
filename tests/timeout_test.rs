use orion_variate::addr::proxy::create_http_client_with_timeouts;
use orion_variate::update::DownloadOptions;
use std::time::Duration;

#[test]
fn test_download_options_timeout_config() {
    // 测试默认配置
    let options = DownloadOptions::for_test();
    assert_eq!(options.connect_timeout(10), 30);
    assert_eq!(options.read_timeout(10), 60);
    assert_eq!(options.total_timeout(10), 300);
    assert_eq!(options.max_retries(10), 3);

    // 测试自定义超时期望值功能
    let opts = DownloadOptions::for_test()
        .with_connect_timeout(15)
        .with_read_timeout(45)
        .with_total_timeout(600)
        .with_max_retries(5);

    assert_eq!(opts.connect_timeout(0), 15);
    assert_eq!(opts.read_timeout(0), 45);
    assert_eq!(opts.total_timeout(0), 600);
    assert_eq!(opts.max_retries(0), 5);
}

#[test]
fn test_http_large_file_timeout() {
    let options = DownloadOptions::for_test().with_http_large_file_timeout();

    assert_eq!(options.connect_timeout(0), 60);
    assert_eq!(options.read_timeout(0), 300);
    assert_eq!(options.total_timeout(0), 3600);
    assert_eq!(options.max_retries(0), 5);
}

#[test]
fn test_git_operation_timeout() {
    let options = DownloadOptions::for_test().with_git_operation_timeout();

    assert_eq!(options.connect_timeout(0), 120);
    assert_eq!(options.read_timeout(0), 180);
    assert_eq!(options.total_timeout(0), 1800);
    assert_eq!(options.max_retries(0), 2);
}

#[tokio::test]
async fn test_http_client_creation() {
    let _client = create_http_client_with_timeouts(
        Duration::from_secs(1),
        Duration::from_secs(5),
        Duration::from_secs(5),
    );
}
