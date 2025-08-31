use derive_more::Deref;
use getset::{Getters, WithSetters};
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};

use crate::vars::types::UpperKey;

use super::{
    EnvDict, EnvEvalable, ValueDict, VarCollection, definition::Mutability, dict::ValueMap,
    types::ValueType,
};

pub type OriginMap = IndexMap<UpperKey, OriginValue>;

impl EnvEvalable<OriginMap> for OriginMap {
    fn env_eval(self, dict: &EnvDict) -> OriginMap {
        let mut cur_dict = dict.clone();
        let mut vmap = OriginMap::new();
        for (k, v) in self {
            let e_v = v.env_eval(&cur_dict);
            if !cur_dict.contains_key(&k) {
                cur_dict.insert(k.clone(), e_v.value.clone());
            }
            vmap.insert(k, e_v);
        }
        vmap
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, WithSetters)]
#[getset(get = "pub")]
pub struct OriginValue {
    origin: Option<String>,
    value: ValueType,
    /// 替换原有的 immutable: Option<bool>
    #[getset(get = "pub", set_with = "pub")]
    #[serde(default, skip_serializing_if = "Mutability::is_default")]
    mutability: Mutability,
}

impl EnvEvalable<OriginValue> for OriginValue {
    fn env_eval(self, dict: &EnvDict) -> OriginValue {
        Self {
            origin: self.origin,
            value: self.value.env_eval(dict),
            mutability: self.mutability,
        }
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Deref, Default)]
pub struct OriginDict {
    dict: OriginMap,
}
impl EnvEvalable<OriginDict> for OriginDict {
    fn env_eval(self, dict: &EnvDict) -> OriginDict {
        Self {
            dict: self.dict.env_eval(dict),
        }
    }
}

impl From<ValueType> for OriginValue {
    fn from(value: ValueType) -> Self {
        Self {
            value,
            origin: None,
            mutability: Mutability::default(),
        }
    }
}
impl From<&str> for OriginValue {
    fn from(value: &str) -> Self {
        Self {
            origin: None,
            value: ValueType::from(value),
            mutability: Mutability::default(),
        }
    }
}

impl OriginValue {
    pub fn with_origin<S: Into<String>>(mut self, origin: S) -> Self {
        self.origin = Some(origin.into());
        self
    }
    pub fn is_mutable(&self) -> bool {
        match self.mutability {
            Mutability::Immutable => false,
            Mutability::System | Mutability::Module => true,
        }
    }
}

impl From<ValueDict> for OriginDict {
    fn from(value: ValueDict) -> Self {
        let mut dict = OriginMap::new();
        for (k, v) in value.dict() {
            dict.insert(k.clone(), OriginValue::from(v.clone()));
        }
        Self { dict }
    }
}
impl From<VarCollection> for OriginDict {
    fn from(value: VarCollection) -> Self {
        let mut dict = OriginMap::new();
        for item in value.immutable_vars() {
            dict.insert(
                item.name().to_string().into(),
                OriginValue::from(item.value().clone()).with_mutability(item.mutability().clone()),
            );
        }
        for item in value.system_vars() {
            dict.insert(
                item.name().to_string().into(),
                OriginValue::from(item.value().clone()).with_mutability(item.mutability().clone()),
            );
        }
        for item in value.module_vars() {
            dict.insert(
                item.name().to_string().into(),
                OriginValue::from(item.value().clone()).with_mutability(item.mutability().clone()),
            );
        }

        Self { dict }
    }
}

impl OriginDict {
    pub fn new() -> Self {
        Self {
            dict: OriginMap::new(),
        }
    }

    pub fn insert<S: Into<UpperKey>>(&mut self, k: S, v: ValueType) -> Option<OriginValue> {
        self.dict.insert(k.into(), OriginValue::from(v))
    }
    pub fn set_source<S: Into<String> + Clone>(&mut self, lable: S) {
        for x in self.dict.values_mut() {
            if x.origin().is_none() {
                x.origin = Some(lable.clone().into());
            }
        }
    }
    pub fn with_origin<S: Into<String> + Clone>(mut self, lable: S) -> Self {
        for x in self.dict.values_mut() {
            if x.origin().is_none() {
                x.origin = Some(lable.clone().into());
            }
        }
        self
    }
    pub fn merge(&mut self, other: &Self) {
        for (k, v) in other.iter() {
            if let Some(x) = self.get(k) {
                //replace orion value;
                if x.is_mutable() {
                    self.dict.insert(k.clone(), v.clone());
                }
            } else {
                self.dict.insert(k.clone(), v.clone());
            }
        }
    }
    pub fn export_value(&self) -> ValueMap {
        let mut map = ValueMap::new();
        for (k, v) in &self.dict {
            map.insert(k.clone(), v.value().clone());
        }
        map
    }
    pub fn export_dict(&self) -> ValueDict {
        ValueDict::from(self.export_value())
    }
    pub fn export_origin(&self) -> OriginMap {
        let mut map = OriginMap::new();
        for (k, v) in &self.dict {
            map.insert(k.clone(), v.clone());
        }
        map
    }
    pub fn ucase_get<S: AsRef<str>>(&self, key: S) -> Option<&OriginValue> {
        let upper_key = UpperKey::from(key.as_ref());
        self.dict.get(&upper_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vars::dict::ValueDict;

    #[test]
    fn test_origin_value_from_value_type() {
        let value = ValueType::from("test_value");
        let origin_value = OriginValue::from(value);
        assert_eq!(origin_value.origin().as_ref(), None);
        assert_eq!(origin_value.value(), &ValueType::from("test_value"));
    }

    #[test]
    fn test_origin_value_from_str() {
        let origin_value = OriginValue::from("test_string");
        assert_eq!(origin_value.origin().as_ref(), None);
        assert_eq!(origin_value.value(), &ValueType::from("test_string"));
    }

    #[test]
    fn test_origin_value_with_origin() {
        let origin_value = OriginValue::from("test_value").with_origin("test_origin");
        assert_eq!(
            origin_value.origin().as_ref(),
            Some(&"test_origin".to_string())
        );
        assert_eq!(origin_value.value(), &ValueType::from("test_value"));
    }

    #[test]
    fn test_origin_value_env_eval() {
        let mut env_dict = EnvDict::new();
        env_dict.insert("TEST_VAR", "replaced_value".into());

        let origin_value = OriginValue::from("prefix_${TEST_VAR}_suffix");
        let evaluated = origin_value.env_eval(&env_dict);

        assert_eq!(evaluated.origin().as_ref(), None);
        assert_eq!(
            evaluated.value(),
            &ValueType::from("prefix_replaced_value_suffix")
        );
    }

    #[test]
    fn test_origin_value_env_eval_with_origin() {
        let mut env_dict = EnvDict::new();
        env_dict.insert("TEST_VAR", "replaced_value".into());

        let origin_value =
            OriginValue::from("prefix_${TEST_VAR}_suffix").with_origin("test_origin");
        let evaluated = origin_value.env_eval(&env_dict);

        assert_eq!(
            evaluated.origin().as_ref(),
            Some(&"test_origin".to_string())
        );
        assert_eq!(
            evaluated.value(),
            &ValueType::from("prefix_replaced_value_suffix")
        );
    }

    #[test]
    fn test_origin_dict_new() {
        let dict = OriginDict::new();
        assert!(dict.is_empty());
        assert_eq!(dict.len(), 0);
    }

    #[test]
    fn test_origin_dict_insert() {
        let mut dict = OriginDict::new();
        let result = dict.insert("key1", ValueType::from("value1"));
        assert_eq!(result, None);

        let result = dict.insert("key1", ValueType::from("new_value"));
        assert!(result.is_some());

        assert_eq!(dict.len(), 1);
        assert_eq!(
            dict.get("KEY1").unwrap().value(),
            &ValueType::from("new_value")
        );
    }

    #[test]
    fn test_origin_dict_set_source() {
        let mut dict = OriginDict::new();
        dict.insert("key1", ValueType::from("value1"));
        dict.insert("key2", ValueType::from("value2"));

        dict.set_source("new_source");

        assert_eq!(
            dict.get("KEY1").unwrap().origin().as_ref(),
            Some(&"new_source".to_string())
        );
        assert_eq!(
            dict.get("KEY2").unwrap().origin().as_ref(),
            Some(&"new_source".to_string())
        );
    }

    #[test]
    fn test_origin_dict_merge() {
        let mut dict1 = OriginDict::new();
        dict1.insert("key1", ValueType::from("value1"));
        dict1.insert("key2", ValueType::from("value2"));

        let mut dict2 = OriginDict::new();
        dict2.insert("key2", ValueType::from("new_value2"));
        dict2.insert("key3", ValueType::from("value3"));

        dict1.merge(&dict2);

        assert_eq!(dict1.len(), 3);
        assert_eq!(
            dict1.get("KEY1").unwrap().value(),
            &ValueType::from("value1")
        );
        assert_eq!(
            dict1.get("KEY2").unwrap().value(),
            &ValueType::from("new_value2")
        );
        assert_eq!(
            dict1.get("KEY3").unwrap().value(),
            &ValueType::from("value3")
        );
    }

    #[test]
    fn test_origin_dict_export_value() {
        let mut dict = OriginDict::new();
        dict.insert("key1", ValueType::from("value1"));
        dict.insert("key2", ValueType::from("value2"));

        let value_map = dict.export_value();

        assert_eq!(value_map.len(), 2);
        assert_eq!(value_map.get("KEY1"), Some(&ValueType::from("value1")));
        assert_eq!(value_map.get("KEY2"), Some(&ValueType::from("value2")));
    }

    #[test]
    fn test_origin_dict_export_dict() {
        let mut dict = OriginDict::new();
        dict.insert("key1", ValueType::from("value1"));
        dict.insert("key2", ValueType::from("value2"));

        let value_dict = dict.export_dict();

        assert_eq!(value_dict.len(), 2);
        assert_eq!(value_dict.get("KEY1"), Some(&ValueType::from("value1")));
        assert_eq!(value_dict.get("KEY2"), Some(&ValueType::from("value2")));
    }

    #[test]
    fn test_origin_dict_export_origin() {
        let mut dict = OriginDict::new();
        dict.insert("key1", ValueType::from("value1"));
        dict.insert("key2", ValueType::from("value2"));

        // 先设置source
        dict.set_source("origin1");

        let origin_map = dict.export_origin();

        assert_eq!(origin_map.len(), 2);
        assert_eq!(
            origin_map.get("KEY1").unwrap().origin().as_ref(),
            Some(&"origin1".to_string())
        );
        assert_eq!(
            origin_map.get("KEY2").unwrap().origin().as_ref(),
            Some(&"origin1".to_string())
        );
    }

    #[test]
    fn test_origin_dict_from_value_dict() {
        let mut value_dict = ValueDict::new();
        value_dict.insert("key1", ValueType::from("value1"));
        value_dict.insert("key2", ValueType::from("value2"));

        let origin_dict = OriginDict::from(value_dict);

        assert_eq!(origin_dict.len(), 2);
        assert_eq!(
            origin_dict.get("KEY1").unwrap().value(),
            &ValueType::from("value1")
        );
        assert_eq!(
            origin_dict.get("KEY2").unwrap().value(),
            &ValueType::from("value2")
        );
        assert_eq!(origin_dict.get("KEY1").unwrap().origin().as_ref(), None);
        assert_eq!(origin_dict.get("KEY2").unwrap().origin().as_ref(), None);
    }

    #[test]
    fn test_origin_map_env_eval() {
        let mut origin_map = OriginMap::new();
        origin_map.insert(
            "key1".to_string().into(),
            OriginValue::from("prefix_${TEST_VAR}_suffix"),
        );
        origin_map.insert(
            "key2".to_string().into(),
            OriginValue::from("static_value").with_origin("test_origin"),
        );

        let mut env_dict = EnvDict::new();
        env_dict.insert("TEST_VAR", "replaced_value".into());

        let evaluated_map = origin_map.env_eval(&env_dict);

        assert_eq!(evaluated_map.len(), 2);
        assert_eq!(
            evaluated_map.get("KEY1").unwrap().value(),
            &ValueType::from("prefix_replaced_value_suffix")
        );
        assert_eq!(evaluated_map.get("KEY1").unwrap().origin().as_ref(), None);
        assert_eq!(
            evaluated_map.get("KEY2").unwrap().value(),
            &ValueType::from("static_value")
        );
        assert_eq!(
            evaluated_map.get("KEY2").unwrap().origin().as_ref(),
            Some(&"test_origin".to_string())
        );
    }

    #[test]
    fn test_origin_value_clone() {
        let origin_value = OriginValue::from("test_value").with_origin("test_origin");
        let cloned = origin_value.clone();

        assert_eq!(cloned, origin_value);
        assert_eq!(cloned.origin().as_ref(), Some(&"test_origin".to_string()));
        assert_eq!(cloned.value(), &ValueType::from("test_value"));
    }

    #[test]
    fn test_origin_dict_clone() {
        let mut dict = OriginDict::new();
        dict.insert("key1", ValueType::from("value1"));
        dict.insert("key2", ValueType::from("value2"));

        // 设置source来测试origin
        dict.set_source("origin1");

        let cloned = dict.clone();

        assert_eq!(cloned, dict);
        assert_eq!(cloned.len(), 2);
        assert_eq!(
            cloned.get("KEY1").unwrap().origin().as_ref(),
            Some(&"origin1".to_string())
        );
        assert_eq!(
            cloned.get("KEY2").unwrap().origin().as_ref(),
            Some(&"origin1".to_string())
        );
    }

    #[test]
    fn test_origin_value_debug() {
        let origin_value = OriginValue::from("test_value").with_origin("test_origin");
        let debug_str = format!("{origin_value:?}");

        assert!(debug_str.contains("OriginValue"));
        assert!(debug_str.contains("test_origin"));
        assert!(debug_str.contains("test_value"));
    }

    #[test]
    fn test_origin_dict_debug() {
        let mut dict = OriginDict::new();
        dict.insert("key1", ValueType::from("value1"));

        let debug_str = format!("{dict:?}");

        assert!(debug_str.contains("OriginDict"));
        assert!(debug_str.contains("KEY1"));
        assert!(debug_str.contains("value1"));
    }

    #[test]
    fn test_origin_dict_default() {
        let dict: OriginDict = OriginDict::default();
        assert!(dict.is_empty());
        assert_eq!(dict.len(), 0);
    }

    #[test]
    fn test_origin_dict_deref() {
        let mut dict = OriginDict::new();
        dict.insert("key1", ValueType::from("value1"));
        dict.insert("key2", ValueType::from("value2"));

        // Test deref to OriginMap
        let map: &OriginMap = &dict;
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("KEY1"));
        assert!(map.contains_key("KEY2"));
    }

    #[test]
    fn test_origin_dict_partial_eq() {
        let mut dict1 = OriginDict::new();
        dict1.insert("key1", ValueType::from("value1"));
        dict1.insert("key2", ValueType::from("value2"));

        let mut dict2 = OriginDict::new();
        dict2.insert("key1", ValueType::from("value1"));
        dict2.insert("key2", ValueType::from("value2"));

        let mut dict3 = OriginDict::new();
        dict3.insert("key1", ValueType::from("value1"));
        dict3.insert("key2", ValueType::from("different_value"));

        assert_eq!(dict1, dict2);
        assert_ne!(dict1, dict3);
    }

    #[test]
    fn test_origin_value_partial_eq() {
        let value1 = OriginValue::from("test_value").with_origin("test_origin");
        let value2 = OriginValue::from("test_value").with_origin("test_origin");
        let value3 = OriginValue::from("different_value").with_origin("test_origin");
        let value4 = OriginValue::from("test_value").with_origin("different_origin");

        assert_eq!(value1, value2);
        assert_ne!(value1, value3);
        assert_ne!(value1, value4);
    }
}

#[cfg(test)]
mod change_scope_tests {
    use super::*;
    use crate::vars::definition::Mutability;

    #[test]
    fn test_origin_value_is_mutable() {
        let immutable_value = OriginValue {
            origin: None,
            value: ValueType::from("test"),
            mutability: Mutability::Immutable,
        };
        assert!(!immutable_value.is_mutable());

        let public_value = OriginValue {
            origin: None,
            value: ValueType::from("test"),
            mutability: Mutability::System,
        };
        assert!(public_value.is_mutable());

        let model_value = OriginValue {
            origin: None,
            value: ValueType::from("test"),
            mutability: Mutability::Module,
        };
        assert!(model_value.is_mutable());
    }

    #[test]
    fn test_origin_value_from_value_type() {
        let value = OriginValue::from(ValueType::from("test"));
        assert_eq!(value.mutability(), &Mutability::Module);
        assert!(value.is_mutable());
    }

    #[test]
    fn test_origin_value_from_str() {
        let value = OriginValue::from("test");
        assert_eq!(value.mutability(), &Mutability::Module);
        assert!(value.is_mutable());
    }

    #[test]
    fn test_origin_value_scope_getter_setter() {
        let mut value = OriginValue::from("test");
        assert_eq!(value.mutability(), &Mutability::Module);

        value = value.with_mutability(Mutability::Immutable);
        assert_eq!(value.mutability(), &Mutability::Immutable);
        assert!(!value.is_mutable());

        value = value.with_mutability(Mutability::Module);
        assert_eq!(value.mutability(), &Mutability::Module);
        assert!(value.is_mutable());
    }

    #[test]
    fn test_origin_value_with_origin() {
        let value = OriginValue::from("test").with_origin("test_source");
        assert_eq!(value.origin(), &Some("test_source".to_string()));
        assert_eq!(value.mutability(), &Mutability::Module);
    }

    #[test]
    fn test_origin_value_env_eval() {
        let mut env_dict = EnvDict::new();
        env_dict.insert("TEST_VAR".to_string(), ValueType::from("replaced"));

        let value = OriginValue {
            origin: Some("test_origin".to_string()),
            value: ValueType::from("prefix_${TEST_VAR}_suffix"),
            mutability: Mutability::Immutable,
        };

        let evaluated = value.env_eval(&env_dict);
        assert_eq!(evaluated.origin(), &Some("test_origin".to_string()));
        assert_eq!(
            evaluated.value(),
            &ValueType::from("prefix_replaced_suffix")
        );
        assert_eq!(evaluated.mutability(), &Mutability::Immutable);
        assert!(!evaluated.is_mutable());
    }

    #[test]
    fn test_origin_value_serialization() {
        let value = OriginValue {
            origin: Some("test_origin".to_string()),
            value: ValueType::from("test_value"),
            mutability: Mutability::System,
        };

        // 默认的 Public scope 应该被跳过序列化
        let json = serde_json::to_string(&value).unwrap();
        assert!(!json.contains("scope"));

        // Non-Default scope 应该被序列化
        let immutable_value = OriginValue {
            origin: Some("test_origin".to_string()),
            value: ValueType::from("test_value"),
            mutability: Mutability::Immutable,
        };

        let json_immutable = serde_json::to_string(&immutable_value).unwrap();
        assert!(json_immutable.contains("mutability"));
    }
}
