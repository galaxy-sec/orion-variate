use std::{
    fmt::{Display, Formatter},
    net::IpAddr,
};

use crate::vars::{
    error::{VarsReason, VarsResult},
    parse::{take_value_map, take_value_vec},
};

use super::{ValueDict, env_eval::expand_env_vars};
use derive_more::From;
use indexmap::IndexMap;
use orion_error::{ErrorOwe, ErrorWith};
use serde_derive::{Deserialize, Serialize};
use winnow::Parser;

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

pub type ValueObj = IndexMap<String, ValueType>;
pub type ValueVec = Vec<ValueType>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, From)]
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

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::String(v) => write!(f, "{v}"),
            ValueType::Bool(v) => write!(f, "{v}"),
            ValueType::Number(v) => write!(f, "{v}"),
            ValueType::Float(v) => write!(f, "{v}"),
            ValueType::Ip(v) => write!(f, "{v}"),
            ValueType::Obj(_) => write!(f, "obj..."),
            ValueType::List(_) => write!(f, "list..."),
        }
    }
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

impl ValueType {
    pub fn len(&self) -> usize {
        match self {
            ValueType::String(s) => s.len(),
            ValueType::List(v) => v.len(),
            ValueType::Obj(m) => m.len(),
            _ => 1,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            ValueType::String(s) => s.is_empty(),
            ValueType::List(v) => v.is_empty(),
            ValueType::Obj(m) => m.is_empty(),
            _ => false,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            ValueType::String(_) => "String",
            ValueType::Bool(_) => "Bool",
            ValueType::Number(_) => "Number",
            ValueType::Float(_) => "Float",
            ValueType::Ip(_) => "Ip",
            ValueType::Obj(_) => "Obj",
            ValueType::List(_) => "List",
        }
    }
    pub fn update_by_str(&mut self, s: &str) -> VarsResult<()> {
        let mut input = s;
        match self {
            ValueType::String(x) => *x = s.to_string(),
            ValueType::Bool(x) => *x = s.parse().owe(VarsReason::Format).with(s.to_string())?,
            ValueType::Number(x) => *x = s.parse().owe(VarsReason::Format).with(s.to_string())?,
            ValueType::Float(x) => *x = s.parse().owe(VarsReason::Format).with(s.to_string())?,
            ValueType::Ip(x) => *x = s.parse().owe(VarsReason::Format).with(s.to_string())?,
            ValueType::Obj(x) => {
                *x = take_value_map
                    .parse_next(&mut input)
                    .owe(VarsReason::Format)
                    .with(s.to_string())?
            }
            ValueType::List(x) => {
                *x = take_value_vec
                    .parse_next(&mut input)
                    .owe(VarsReason::Format)
                    .with(s.to_string())?
            }
        }
        Ok(())
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

        println!("\n🔧 Modlist 反序列化结果:\n{decoded:#?}",);
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
        println!("📦 原始对象: {complex_obj:#?}",);
        println!("📜 JSON 输出:\n{json_output}",);
        println!("🎯 YAML 输出:\n{yaml_output}",);

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
    #[test]
    fn test_value_type_len() {
        let s = ValueType::String("hello".to_string());
        assert_eq!(s.len(), 5);

        let l = ValueType::List(vec![
            ValueType::String("a".to_string()),
            ValueType::String("b".to_string()),
        ]);
        assert_eq!(l.len(), 2);

        let mut obj = ValueObj::new();
        obj.insert("key1".to_string(), ValueType::String("value1".to_string()));
        obj.insert("key2".to_string(), ValueType::String("value2".to_string()));
        let o = ValueType::Obj(obj);
        assert_eq!(o.len(), 2);

        let b = ValueType::Bool(true);
        assert_eq!(b.len(), 1);

        let n = ValueType::Number(42);
        assert_eq!(n.len(), 1);
    }

    #[test]
    fn test_value_type_name() {
        let s = ValueType::String("hello".to_string());
        assert_eq!(s.type_name(), "String");

        let b = ValueType::Bool(true);
        assert_eq!(b.type_name(), "Bool");

        let n = ValueType::Number(42);
        assert_eq!(n.type_name(), "Number");

        let f = ValueType::Float(4.14);
        assert_eq!(f.type_name(), "Float");

        let ip = ValueType::Ip("127.0.0.1".parse().unwrap());
        assert_eq!(ip.type_name(), "Ip");

        let obj = ValueType::Obj(ValueObj::new());
        assert_eq!(obj.type_name(), "Obj");

        let list = ValueType::List(ValueVec::new());
        assert_eq!(list.type_name(), "List");
    }

    #[test]
    fn test_update_by_str() {
        // 测试 String 类型更新
        let mut string_val = ValueType::String("old".to_string());
        string_val.update_by_str("new").unwrap();
        assert_eq!(string_val, ValueType::String("new".to_string()));

        // 测试 Bool 类型更新
        let mut bool_val = ValueType::Bool(false);
        bool_val.update_by_str("true").unwrap();
        assert_eq!(bool_val, ValueType::Bool(true));

        // 测试无效 Bool 值
        let mut bool_val = ValueType::Bool(false);
        assert!(bool_val.update_by_str("invalid").is_err());

        // 测试 Number 类型更新
        let mut number_val = ValueType::Number(10);
        number_val.update_by_str("42").unwrap();
        assert_eq!(number_val, ValueType::Number(42));

        // 测试无效 Number 值
        let mut number_val = ValueType::Number(10);
        assert!(number_val.update_by_str("invalid").is_err());

        // 测试 Float 类型更新
        let mut float_val = ValueType::Float(1.5);
        float_val.update_by_str("3.24").unwrap();
        assert_eq!(float_val, ValueType::Float(3.24));

        // 测试无效 Float 值
        let mut float_val = ValueType::Float(1.5);
        assert!(float_val.update_by_str("invalid").is_err());

        // 测试 IP 类型更新
        let mut ip_val = ValueType::Ip("127.0.0.1".parse().unwrap());
        ip_val.update_by_str("192.168.1.1").unwrap();
        assert_eq!(ip_val, ValueType::Ip("192.168.1.1".parse().unwrap()));

        // 测试无效 IP 值
        let mut ip_val = ValueType::Ip("127.0.0.1".parse().unwrap());
        assert!(ip_val.update_by_str("invalid").is_err());

        // 测试 Obj 类型更新
        let mut obj_val = ValueType::Obj(ValueObj::new());
        obj_val.update_by_str("{key: \"value\"}").unwrap();
        let mut expected_obj = ValueObj::new();
        expected_obj.insert("key".to_string(), ValueType::String("value".to_string()));
        assert_eq!(obj_val, ValueType::Obj(expected_obj));

        // 测试无效 Obj 值
        let mut obj_val = ValueType::Obj(ValueObj::new());
        assert!(obj_val.update_by_str("invalid").is_err());

        // 测试 List 类型更新
        let mut list_val = ValueType::List(ValueVec::new());
        list_val.update_by_str("[\"item1\", \"item2\"]").unwrap();
        let expected_list = ValueVec::from([
            ValueType::String("item1".to_string()),
            ValueType::String("item2".to_string()),
        ]);
        assert_eq!(list_val, ValueType::List(expected_list));

        // 测试无效 List 值
        let mut list_val = ValueType::List(ValueVec::new());
        assert!(list_val.update_by_str("invalid").is_err());
    }
}
