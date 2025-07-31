use getset::Getters;
use serde_derive::{Deserialize, Serialize};
#[derive(Debug,Clone,Serialize,Deserialize,Getters)]
#[getset(get = "pub")]
pub struct Auth {
    username : String,
    password : String,
}

impl Auth {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}