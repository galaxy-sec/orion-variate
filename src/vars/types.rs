use std::{collections::HashMap, net::IpAddr};

use super::{ValueDict, env_eval::expand_env_vars};
use serde_derive::{Deserialize, Serialize};

pub type EnvDict = ValueDict;
pub trait EnvEvalable<T> {
    fn env_eval(self, dict: &EnvDict) -> T;
}

impl EnvEvalable<String> for String {
    fn env_eval(self, dict: &EnvDict) -> String {
        expand_env_vars(dict, self.as_str())
    }
}

impl EnvEvalable<Option<String>> for Option<String> {
    fn env_eval(self, dict: &EnvDict) -> Option<String> {
        self.map(|x| expand_env_vars(dict, x.as_str()))
    }
}

pub type ValueObj = HashMap<String, ValueType>;
pub type ValueVec = Vec<ValueType>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ValueType {
    String(String),
    Bool(bool),
    Number(u64),
    Float(f64),
    Ip(IpAddr),
    Obj(ValueObj),
    List(ValueVec),
}
impl EnvEvalable<ValueType> for ValueType {
    fn env_eval(self, dict: &EnvDict) -> ValueType {
        match self {
            ValueType::String(v) => ValueType::String(v.env_eval(dict)),
            _ => self,
        }
    }
}

impl From<&str> for ValueType {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
impl From<bool> for ValueType {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<u64> for ValueType {
    fn from(value: u64) -> Self {
        Self::Number(value)
    }
}
impl From<f64> for ValueType {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

#[cfg(test)]
mod tests {
    use super::ValueType;
    use serde_yaml;

    #[test]
    fn test_modlist_deserialization() {
        let yaml_data = r#"
        - name: core-module
          version: 1.2.3
          dependencies:
            - lib-utils@5.6.0
        - name: network-layer
          version: 2.0.0
        "#;

        // 反序列化为 ValueType 枚举
        let decoded: ValueType = serde_yaml::from_str(yaml_data).unwrap();

        println!("\n🔧 Modlist 反序列化结果:\n{:#?}", decoded);
    }
    use super::*;
    use orion_error::TestAssert;
    use serde_json;

    #[test]
    fn test_from_modlist() {
        let data = r#"
- name: redis_mock
  addr:
    path: ./example/modules/redis_mock
  model: arm-mac14-host
- name: mysql_mock
  addr:
    path: ./example/modules/mysql_mock
  model: arm-mac14-host
"#;

        let decoded: ValueType = serde_yaml::from_str(data).unwrap();
        if let ValueType::List(mods) = decoded {
            if let Some(ValueType::Obj(first_mod)) = mods.first() {
                assert_eq!(
                    first_mod.get("name"),
                    Some(&ValueType::String("redis_mock".into()))
                );
            }
        }
    }
    #[test]
    fn test_value_obj_serialization() {
        // 混合类型测试数据
        let mut complex_obj = ValueObj::new();
        complex_obj.insert("user".into(), ValueType::String("Alice".into()));
        complex_obj.insert("age".into(), ValueType::Number(30));
        complex_obj.insert(
            "preferences".into(),
            ValueType::String("{\"theme\":\"dark\"}".into()),
        );

        // 序列化演示
        let json_output = serde_json::to_string_pretty(&complex_obj).unwrap();
        let yaml_output = serde_yaml::to_string(&complex_obj).unwrap();

        println!("\n✅ 混合类型序列化测试:\n");
        println!("📦 原始对象: {:#?}", complex_obj);
        println!("📜 JSON 输出:\n{}", json_output);
        println!("🎯 YAML 输出:\n{}", yaml_output);

        // 验证往返序列化
        let json_roundtrip: ValueObj = serde_json::from_str(&json_output).unwrap();
        let yaml_roundtrip: ValueObj = serde_yaml::from_str(&yaml_output).unwrap();

        assert_eq!(complex_obj, json_roundtrip, "JSON 往返序列化不一致");
        assert_eq!(complex_obj, yaml_roundtrip, "YAML 往返序列化不一致");
        let mut obj = ValueObj::new();
        obj.insert("string".to_string(), ValueType::String("test".into()));
        obj.insert("number".to_string(), ValueType::Number(42));
        obj.insert("boolean".to_string(), ValueType::Bool(true));

        let json = serde_json::to_string(&obj).assert();
        println!("{json:#}");
        let decoded: ValueObj = serde_json::from_str(&json).unwrap();

        assert_eq!(obj["string"], decoded["string"]);
        assert_eq!(obj["number"], decoded["number"]);
        assert_eq!(obj["boolean"], decoded["boolean"]);

        // YAML 序列化测试
        let yaml = serde_yaml::to_string(&obj).unwrap();
        println!("{yaml:#}");
        let yaml_decoded: ValueObj = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(obj["string"], yaml_decoded["string"]);
        assert_eq!(obj["number"], yaml_decoded["number"]);
        assert_eq!(obj["boolean"], yaml_decoded["boolean"]);
    }
}
