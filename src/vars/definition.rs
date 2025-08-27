use std::default;

use getset::{Getters, WithSetters};
use serde_derive::{Deserialize, Serialize};

use super::ValueType;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum ChangeScope {
    /// 不可变变量，不允许任何修改
    Immutable,
    /// 公开可变变量，允许在任何上下文中修改
    Public,
    /// 模块级可变变量，只在同一模块内允许修改
    #[default]
    Model,
}

impl ChangeScope {
    /// 检查是否为默认值，用于序列化优化
    pub fn is_default(&self) -> bool {
        matches!(self, ChangeScope::Public)
    }

    /// 创建不可变作用域
    pub fn immutable() -> Self {
        ChangeScope::Immutable
    }

    /// 创建公开可变作用域
    pub fn public() -> Self {
        ChangeScope::Public
    }

    /// 创建模块级可变作用域
    pub fn model() -> Self {
        ChangeScope::Model
    }

    /// 为向后兼容，从旧格式的 Option<bool> 转换
    pub fn from_immutable_flag(immutable: Option<bool>) -> Self {
        match immutable {
            Some(true) => ChangeScope::Immutable,
            Some(false) | None => ChangeScope::Public,
        }
    }

    /// 转换为旧格式，用于向后兼容
    pub fn to_immutable_flag(&self) -> Option<bool> {
        match self {
            ChangeScope::Immutable => Some(true),
            ChangeScope::Public | ChangeScope::Model => Some(false),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Getters, WithSetters)]
#[getset(get = "pub")]
pub struct VarDefinition {
    name: String,
    value: ValueType,
    #[getset(set_with = "pub")]
    #[serde(skip_serializing_if = "Option::is_none")]
    desp: Option<String>,
    /// 替换原有的 immutable: Option<bool>
    #[getset(get = "pub", set_with = "pub")]
    #[serde(default, skip_serializing_if = "ChangeScope::is_default")]
    scope: ChangeScope,
}
impl VarDefinition {
    pub fn is_mutable(&self) -> bool {
        match self.scope {
            ChangeScope::Immutable => false,
            ChangeScope::Public | ChangeScope::Model => true,
        }
    }
}
impl From<(&str, &str)> for VarDefinition {
    fn from(value: (&str, &str)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
            scope: ChangeScope::default(),
        }
    }
}
impl From<(&str, bool)> for VarDefinition {
    fn from(value: (&str, bool)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
            scope: ChangeScope::default(),
        }
    }
}
impl From<(&str, u64)> for VarDefinition {
    fn from(value: (&str, u64)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
            scope: ChangeScope::default(),
        }
    }
}
impl From<(&str, f64)> for VarDefinition {
    fn from(value: (&str, f64)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: ValueType::from(value.1),
            scope: ChangeScope::default(),
        }
    }
}

impl From<(&str, ValueType)> for VarDefinition {
    fn from(value: (&str, ValueType)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desp: None,
            value: value.1,
            scope: ChangeScope::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_scope_factory_methods() {
        assert_eq!(ChangeScope::immutable(), ChangeScope::Immutable);
        assert_eq!(ChangeScope::public(), ChangeScope::Public);
        assert_eq!(ChangeScope::model(), ChangeScope::Model);
    }

    #[test]
    fn test_change_scope_from_immutable_flag() {
        assert_eq!(
            ChangeScope::from_immutable_flag(Some(true)),
            ChangeScope::Immutable
        );
        assert_eq!(
            ChangeScope::from_immutable_flag(Some(false)),
            ChangeScope::Public
        );
        assert_eq!(ChangeScope::from_immutable_flag(None), ChangeScope::Public);
    }

    #[test]
    fn test_change_scope_to_immutable_flag() {
        assert_eq!(ChangeScope::Immutable.to_immutable_flag(), Some(true));
        assert_eq!(ChangeScope::Public.to_immutable_flag(), Some(false));
        assert_eq!(ChangeScope::Model.to_immutable_flag(), Some(false));
    }

    #[test]
    fn test_var_definition_is_mutable() {
        let immutable_var = VarDefinition {
            name: "test".to_string(),
            desp: None,
            value: ValueType::from("value"),
            scope: ChangeScope::Immutable,
        };
        assert!(!immutable_var.is_mutable());

        let public_var = VarDefinition {
            name: "test".to_string(),
            desp: None,
            value: ValueType::from("value"),
            scope: ChangeScope::Public,
        };
        assert!(public_var.is_mutable());

        let model_var = VarDefinition {
            name: "test".to_string(),
            desp: None,
            value: ValueType::from("value"),
            scope: ChangeScope::Model,
        };
        assert!(model_var.is_mutable());
    }

    #[test]
    fn test_var_definition_from_tuple() {
        let var = VarDefinition::from(("test_name", "test_value"));
        assert_eq!(var.name(), "test_name");
        assert_eq!(var.value(), &ValueType::from("test_value"));
        assert_eq!(var.scope(), &ChangeScope::Model);
        assert!(var.is_mutable());
    }

    #[test]
    fn test_var_definition_scope_getter_setter() {
        let mut var = VarDefinition::from(("test", "value"));
        assert_eq!(var.scope(), &ChangeScope::Model);

        var = var.with_scope(ChangeScope::Immutable);
        assert_eq!(var.scope(), &ChangeScope::Immutable);
        assert!(!var.is_mutable());

        var = var.with_scope(ChangeScope::Model);
        assert_eq!(var.scope(), &ChangeScope::Model);
        assert!(var.is_mutable());
    }

    #[test]
    fn test_var_definition_serialization() {
        let var = VarDefinition {
            name: "test".to_string(),
            desp: None,
            value: ValueType::from("value"),
            scope: ChangeScope::Public,
        };

        // 默认的 Public scope 应该被跳过序列化
        let json = serde_json::to_string(&var).unwrap();
        assert!(!json.contains("scope"));

        // Non-Default scope 应该被序列化
        let var_immutable = VarDefinition {
            name: "test".to_string(),
            desp: None,
            value: ValueType::from("value"),
            scope: ChangeScope::Immutable,
        };

        let json_immutable = serde_json::to_string(&var_immutable).unwrap();
        assert!(json_immutable.contains("scope"));
    }
}
