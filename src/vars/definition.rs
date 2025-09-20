use getset::{Getters, Setters, WithSetters};
use serde_derive::{Deserialize, Serialize};

use super::ValueType;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum Mutability {
    /// 不可变变量，不允许任何修改
    Immutable,
    /// 公开可变变量，允许在任何上下文中修改
    System,
    /// 模块级可变变量，只在同一模块内允许修改
    #[default]
    Module,
}

impl Mutability {
    /// 检查是否为默认值，用于序列化优化
    pub fn is_default(&self) -> bool {
        matches!(self, Mutability::Module)
    }

    /// 创建不可变作用域
    pub fn immutable() -> Self {
        Mutability::Immutable
    }

    /// 创建公开可变作用域
    pub fn system() -> Self {
        Mutability::System
    }

    /// 创建模块级可变作用域
    pub fn module() -> Self {
        Mutability::Module
    }

    #[deprecated(note = "renamed to module() for clarity")]
    pub fn model() -> Self {
        Self::Module
    }

    /// 为向后兼容，从旧格式的 Option<bool> 转换
    pub fn from_immutable_flag(immutable: Option<bool>) -> Self {
        match immutable {
            Some(true) => Mutability::Immutable,
            Some(false) | None => Mutability::System,
        }
    }

    /// 转换为旧格式，用于向后兼容
    pub fn to_immutable_flag(&self) -> Option<bool> {
        match self {
            Mutability::Immutable => Some(true),
            Mutability::System | Mutability::Module => Some(false),
        }
    }
}
pub trait VarToValue<T> {
    fn to_val(&self) -> T;
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Getters, WithSetters, Setters)]
#[getset(get = "pub")]
pub struct VarDefinition {
    name: String,
    value: ValueType,
    #[getset(set_with = "pub")]
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "desc",
        alias = "desp"
    )]
    desc: Option<String>,
    #[getset(get = "pub", set_with = "pub", set = "pub")]
    #[serde(default, skip)]
    mutability: Mutability,
}
impl VarDefinition {
    pub fn is_mutable(&self) -> bool {
        match self.mutability {
            Mutability::Immutable => false,
            Mutability::System | Mutability::Module => true,
        }
    }
    pub fn with_mut_immutable(mut self) -> Self {
        self.mutability = Mutability::Immutable;
        self
    }
    pub fn with_mut_system(mut self) -> Self {
        self.mutability = Mutability::System;
        self
    }
    pub fn with_mut_module(mut self) -> Self {
        self.mutability = Mutability::Module;
        self
    }
}

impl VarDefinition {
    /// 兼容旧 API：保留 `desp()` 访问器
    #[deprecated(note = "renamed to desc()")]
    pub fn desp(&self) -> &Option<String> {
        self.desc()
    }
}
impl From<(&str, &str)> for VarDefinition {
    fn from(value: (&str, &str)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desc: None,
            value: ValueType::from(value.1),
            mutability: Mutability::default(),
        }
    }
}
impl From<(&str, bool)> for VarDefinition {
    fn from(value: (&str, bool)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desc: None,
            value: ValueType::from(value.1),
            mutability: Mutability::default(),
        }
    }
}
impl From<(&str, u64)> for VarDefinition {
    fn from(value: (&str, u64)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desc: None,
            value: ValueType::from(value.1),
            mutability: Mutability::default(),
        }
    }
}
impl From<(&str, f64)> for VarDefinition {
    fn from(value: (&str, f64)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desc: None,
            value: ValueType::from(value.1),
            mutability: Mutability::default(),
        }
    }
}

impl From<(&str, ValueType)> for VarDefinition {
    fn from(value: (&str, ValueType)) -> Self {
        VarDefinition {
            name: value.0.to_string(),
            desc: None,
            value: value.1,
            mutability: Mutability::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_scope_factory_methods() {
        assert_eq!(Mutability::immutable(), Mutability::Immutable);
        assert_eq!(Mutability::system(), Mutability::System);
        assert_eq!(Mutability::module(), Mutability::Module);
    }

    #[test]
    fn test_change_scope_from_immutable_flag() {
        assert_eq!(
            Mutability::from_immutable_flag(Some(true)),
            Mutability::Immutable
        );
        assert_eq!(
            Mutability::from_immutable_flag(Some(false)),
            Mutability::System
        );
        assert_eq!(Mutability::from_immutable_flag(None), Mutability::System);
    }

    #[test]
    fn test_change_scope_to_immutable_flag() {
        assert_eq!(Mutability::Immutable.to_immutable_flag(), Some(true));
        assert_eq!(Mutability::System.to_immutable_flag(), Some(false));
        assert_eq!(Mutability::Module.to_immutable_flag(), Some(false));
    }

    #[test]
    fn test_var_definition_is_mutable() {
        let immutable_var = VarDefinition {
            name: "test".to_string(),
            desc: None,
            value: ValueType::from("value"),
            mutability: Mutability::Immutable,
        };
        assert!(!immutable_var.is_mutable());

        let public_var = VarDefinition {
            name: "test".to_string(),
            desc: None,
            value: ValueType::from("value"),
            mutability: Mutability::System,
        };
        assert!(public_var.is_mutable());

        let model_var = VarDefinition {
            name: "test".to_string(),
            desc: None,
            value: ValueType::from("value"),
            mutability: Mutability::Module,
        };
        assert!(model_var.is_mutable());
    }

    #[test]
    fn test_var_definition_from_tuple() {
        let var = VarDefinition::from(("test_name", "test_value"));
        assert_eq!(var.name(), "test_name");
        assert_eq!(var.value(), &ValueType::from("test_value"));
        assert_eq!(var.mutability(), &Mutability::Module);
        assert!(var.is_mutable());
    }

    #[test]
    fn test_var_definition_scope_getter_setter() {
        let mut var = VarDefinition::from(("test", "value"));
        assert_eq!(var.mutability(), &Mutability::Module);

        var = var.with_mutability(Mutability::Immutable);
        assert_eq!(var.mutability(), &Mutability::Immutable);
        assert!(!var.is_mutable());

        var = var.with_mutability(Mutability::Module);
        assert_eq!(var.mutability(), &Mutability::Module);
        assert!(var.is_mutable());
    }

    #[test]
    fn test_var_definition_serialization() {
        let var = VarDefinition {
            name: "test".to_string(),
            desc: None,
            value: ValueType::from("value"),
            mutability: Mutability::System,
        };

        // scope 应该被跳过序列化
        let json = serde_json::to_string(&var).unwrap();
        assert!(!json.contains("scope"));

        // Non-Default scope 应该被序列化
        let var_immutable = VarDefinition {
            name: "test".to_string(),
            desc: None,
            value: ValueType::from("value"),
            mutability: Mutability::Immutable,
        };

        let json_immutable = serde_json::to_string(&var_immutable).unwrap();
        assert!(!json_immutable.contains("scope"));
    }
}
