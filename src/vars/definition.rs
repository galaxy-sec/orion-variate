use getset::{Getters, WithSetters};
use serde_derive::{Deserialize, Serialize};

use super::ValueType;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Getters, WithSetters)]
#[getset(get = "pub")]
pub struct VarDefinition {
    name: String,
    value: ValueType,
    #[getset(set_with = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    desp: Option<String>,
    #[getset(set_with = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    immutable: Option<bool>,
}
impl VarDefinition {
    pub fn is_mutable(&self) -> bool {
        let immutable = self.immutable.clone().unwrap_or(false);
        !immutable
    }
}
impl From<(&str, &str)> for VarDefinition {
    fn from(value: (&str, &str)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
            immutable: None,
        }
    }
}
impl From<(&str, bool)> for VarDefinition {
    fn from(value: (&str, bool)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
            immutable: None,
        }
    }
}
impl From<(&str, u64)> for VarDefinition {
    fn from(value: (&str, u64)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
            immutable: None,
        }
    }
}
impl From<(&str, f64)> for VarDefinition {
    fn from(value: (&str, f64)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
            immutable: None,
        }
    }
}

impl From<(&str, ValueType)> for VarDefinition {
    fn from(value: (&str, ValueType)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: value.1,
            immutable: None,
        }
    }
}
