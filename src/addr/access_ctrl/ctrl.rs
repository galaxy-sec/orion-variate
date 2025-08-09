use crate::{
    addr::{access_ctrl::auth::AuthConfig, proxy::ProxyConfig},
    timeout::TimeoutConfig,
};
use getset::Getters;

#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct UnitCtrl {
    auth: Option<AuthConfig>,
    timeout: Option<TimeoutConfig>,
    proxy: Option<ProxyConfig>,
}
impl UnitCtrl {
    pub fn new(
        auth: Option<AuthConfig>,
        timeout: Option<TimeoutConfig>,
        proxy: Option<ProxyConfig>,
    ) -> Self {
        Self {
            auth,
            timeout,
            proxy,
        }
    }
}
