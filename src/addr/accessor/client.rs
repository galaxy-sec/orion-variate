use std::time::Duration;

use crate::addr::access_ctrl::UnitCtrl;

use reqwest::{Client, ClientBuilder, Proxy};

pub fn create_http_client_by_ctrl(ctrl: Option<UnitCtrl>) -> reqwest::Client {
    // 使用 UnitCtrl 中的超时配置创建客户端
    let mut builder = if let Some(timeout) = ctrl.as_ref().map(|x| x.timeout().clone()).flatten() {
        ClientBuilder::new()
            .connect_timeout(timeout.connect_duration())
            .read_timeout(timeout.read_duration())
            .timeout(timeout.total_duration())
            .tcp_keepalive(Duration::from_secs(60))
            .pool_idle_timeout(Duration::from_secs(90))
    } else {
        ClientBuilder::new()
    };
    if let Some(proxy) = ctrl.as_ref().map(|x| x.proxy().clone()).flatten() {
        if let Ok(proxy) = Proxy::all(proxy.url().as_str()) {
            builder = builder.proxy(proxy);
        } else {
            tracing::warn!("无效的代理设置: {}", proxy.url());
        }
    }
    builder.build().unwrap_or_else(|e| {
        tracing::error!("创建HTTP客户端失败: {}", e);
        reqwest::Client::new()
    })
}
