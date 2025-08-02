use std::rc::Rc;

use getset::Getters;

use crate::addr::proxy::{
    auth::Auth,
    unit::{ProxyPath, Unit},
};

use super::rule::{self, Rule};

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Serv {
    units: Vec<Unit>,
    enable: bool,
}

pub type ServHandle = Rc<Serv>;

impl Serv {
    pub fn new(units: Vec<Unit>, enable: bool) -> Self {
        Self { units, enable }
    }
    pub fn proxy(&self, url: &str) -> ProxyPath {
        let mut path = ProxyPath::Origin(url.to_string());
        for unit in &self.units {
            path = unit.proxy(path.path());
            if path.is_proxy() {
                break;
            }
        }
        path
    }
    pub fn from_rule(rule: Rule, auth: Option<Auth>) -> Self {
        let unit = Unit::new(vec![rule], auth);
        Self::new(vec![unit], true)
    }
}
