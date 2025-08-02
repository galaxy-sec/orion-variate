use getset::Getters;
use serde_derive::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Getters, PartialEq)]
#[getset(get = "pub")]
pub struct Auth {
    username: String,
    password: String,
}

impl Auth {
    pub fn new<S: Into<String>>(username: S, password: S) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
    pub fn make_example() -> Self {
        Self {
            username: "galaxy".into(),
            password: "this-is-password".into(),
        }
    }
}
