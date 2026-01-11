use std::{
    fmt::{Display, Formatter},
    net::IpAddr,
};

use crate::vars::{
    error::{VarsReason, VarsResult},
    parse::{take_value_map, take_value_vec},
};

use super::{
    ValueDict,
    env_eval::{expand_env_vars, extract_env_var_names},
};
use derive_more::From;
use indexmap::IndexMap;
use orion_error::{ErrorOwe, ErrorWith};
use serde_derive::{Deserialize, Serialize};
use winnow::Parser;

pub type EnvDict = ValueDict;
pub trait EnvEvaluable<T> {
    fn env_eval(self, dict: &EnvDict) -> T;
}

/// Trait to check if a value contains environment variable placeholders
/// that need evaluation (e.g., `${VAR_NAME}` or `${VAR_NAME:default}`)
pub trait EnvChecker {
    /// Returns true if the value contains environment variable placeholders
    fn needs_env_eval(&self) -> bool;

    /// Returns a list of all environment variable names found in the value
    /// For `${VAR:default}` syntax, only returns the variable name without the default value
    fn list_env_vars(&self) -> Vec<String>;
}

impl EnvChecker for String {
    fn needs_env_eval(&self) -> bool {
        self.contains("${")
    }

    fn list_env_vars(&self) -> Vec<String> {
        extract_env_var_names(self)
    }
}

impl EnvChecker for &str {
    fn needs_env_eval(&self) -> bool {
        self.contains("${")
    }

    fn list_env_vars(&self) -> Vec<String> {
        extract_env_var_names(self)
    }
}

impl EnvChecker for Option<String> {
    fn needs_env_eval(&self) -> bool {
        self.as_ref().is_some_and(|s| s.needs_env_eval())
    }

    fn list_env_vars(&self) -> Vec<String> {
        self.as_ref().map_or(Vec::new(), |s| s.list_env_vars())
    }
}

impl EnvChecker for Option<&str> {
    fn needs_env_eval(&self) -> bool {
        self.is_some_and(|s| s.needs_env_eval())
    }

    fn list_env_vars(&self) -> Vec<String> {
        self.map_or(Vec::new(), |s| s.list_env_vars())
    }
}

impl EnvEvaluable<String> for String {
    fn env_eval(self, dict: &EnvDict) -> String {
        expand_env_vars(dict, self.as_str())
    }
}

impl EnvEvaluable<Option<String>> for Option<String> {
    fn env_eval(self, dict: &EnvDict) -> Option<String> {
        self.map(|x| expand_env_vars(dict, x.as_str()))
    }
}

pub type ValueObj = IndexMap<String, ValueType>;
pub type ValueVec = Vec<ValueType>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UpperKey(String);

impl UpperKey {
    fn new<S: Into<String>>(key: S) -> Self {
        Self(key.into().to_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<S: Into<String>> From<S> for UpperKey {
    fn from(key: S) -> Self {
        Self::new(key)
    }
}

impl std::borrow::Borrow<str> for UpperKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<String> for UpperKey {
    fn borrow(&self) -> &String {
        &self.0
    }
}

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

impl EnvChecker for ValueType {
    fn needs_env_eval(&self) -> bool {
        match self {
            ValueType::String(s) => s.needs_env_eval(),
            ValueType::Obj(obj) => obj.values().any(|v| v.needs_env_eval()),
            ValueType::List(list) => list.iter().any(|v| v.needs_env_eval()),
            // Other types (Bool, Number, Float, Ip) don't contain env vars
            _ => false,
        }
    }

    fn list_env_vars(&self) -> Vec<String> {
        match self {
            ValueType::String(s) => s.list_env_vars(),
            ValueType::Obj(obj) => obj.values().flat_map(|v| v.list_env_vars()).collect(),
            ValueType::List(list) => list.iter().flat_map(|v| v.list_env_vars()).collect(),
            _ => Vec::new(),
        }
    }
}

impl EnvEvaluable<ValueType> for ValueType {
    fn env_eval(self, dict: &EnvDict) -> ValueType {
        match self {
            ValueType::String(v) => ValueType::String(v.env_eval(dict)),
            ValueType::Obj(obj) => ValueType::Obj(
                obj.into_iter()
                    .map(|(k, v)| (k, v.env_eval(dict)))
                    .collect(),
            ),
            ValueType::List(list) => {
                ValueType::List(list.into_iter().map(|v| v.env_eval(dict)).collect())
            }
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

    pub fn variant_name(&self) -> &'static str {
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
    pub fn update_from_str(&mut self, s: &str) -> VarsResult<()> {
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

    #[deprecated(note = "renamed to variant_name()")]
    pub fn type_name(&self) -> &'static str {
        self.variant_name()
    }

    #[deprecated(note = "renamed to update_from_str()")]
    pub fn update_by_str(&mut self, s: &str) -> VarsResult<()> {
        self.update_from_str(s)
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
        if let ValueType::List(mods) = decoded
            && let Some(ValueType::Obj(first_mod)) = mods.first()
        {
            assert_eq!(
                first_mod.get("name"),
                Some(&ValueType::String("redis_mock".into()))
            );
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
        assert_eq!(s.variant_name(), "String");

        let b = ValueType::Bool(true);
        assert_eq!(b.variant_name(), "Bool");

        let n = ValueType::Number(42);
        assert_eq!(n.variant_name(), "Number");

        let f = ValueType::Float(4.14);
        assert_eq!(f.variant_name(), "Float");

        let ip = ValueType::Ip("127.0.0.1".parse().unwrap());
        assert_eq!(ip.variant_name(), "Ip");

        let obj = ValueType::Obj(ValueObj::new());
        assert_eq!(obj.variant_name(), "Obj");

        let list = ValueType::List(ValueVec::new());
        assert_eq!(list.variant_name(), "List");
    }

    #[test]
    fn test_update_from_str() {
        // 测试 String 类型更新
        let mut string_val = ValueType::String("old".to_string());
        string_val.update_from_str("new").unwrap();
        assert_eq!(string_val, ValueType::String("new".to_string()));

        // 测试 Bool 类型更新
        let mut bool_val = ValueType::Bool(false);
        bool_val.update_from_str("true").unwrap();
        assert_eq!(bool_val, ValueType::Bool(true));

        // 测试无效 Bool 值
        let mut bool_val = ValueType::Bool(false);
        assert!(bool_val.update_from_str("invalid").is_err());

        // 测试 Number 类型更新
        let mut number_val = ValueType::Number(10);
        number_val.update_from_str("42").unwrap();
        assert_eq!(number_val, ValueType::Number(42));

        // 测试无效 Number 值
        let mut number_val = ValueType::Number(10);
        assert!(number_val.update_from_str("invalid").is_err());

        // 测试 Float 类型更新
        let mut float_val = ValueType::Float(1.5);
        float_val.update_from_str("3.24").unwrap();
        assert_eq!(float_val, ValueType::Float(3.24));

        // 测试无效 Float 值
        let mut float_val = ValueType::Float(1.5);
        assert!(float_val.update_from_str("invalid").is_err());

        // 测试 IP 类型更新
        let mut ip_val = ValueType::Ip("127.0.0.1".parse().unwrap());
        ip_val.update_from_str("192.168.1.1").unwrap();
        assert_eq!(ip_val, ValueType::Ip("192.168.1.1".parse().unwrap()));

        // 测试无效 IP 值
        let mut ip_val = ValueType::Ip("127.0.0.1".parse().unwrap());
        assert!(ip_val.update_from_str("invalid").is_err());

        // 测试 Obj 类型更新
        let mut obj_val = ValueType::Obj(ValueObj::new());
        obj_val.update_from_str("{key: \"value\"}").unwrap();
        let mut expected_obj = ValueObj::new();
        expected_obj.insert("key".to_string(), ValueType::String("value".to_string()));
        assert_eq!(obj_val, ValueType::Obj(expected_obj));

        // 测试无效 Obj 值
        let mut obj_val = ValueType::Obj(ValueObj::new());
        assert!(obj_val.update_from_str("invalid").is_err());

        // 测试 List 类型更新
        let mut list_val = ValueType::List(ValueVec::new());
        list_val.update_from_str("[\"item1\", \"item2\"]").unwrap();
        let expected_list = ValueVec::from([
            ValueType::String("item1".to_string()),
            ValueType::String("item2".to_string()),
        ]);
        assert_eq!(list_val, ValueType::List(expected_list));

        // 测试无效 List 值
        let mut list_val = ValueType::List(ValueVec::new());
        assert!(list_val.update_from_str("invalid").is_err());
    }

    #[test]
    fn test_env_checker_string() {
        use super::EnvChecker;

        // 包含环境变量的字符串
        let with_env = String::from("Hello ${USER}");
        assert!(with_env.needs_env_eval());

        // 不包含环境变量的字符串
        let without_env = String::from("Hello World");
        assert!(!without_env.needs_env_eval());

        // 包含默认值语法的环境变量
        let with_default = String::from("${VAR:default}");
        assert!(with_default.needs_env_eval());
    }

    #[test]
    fn test_env_checker_str() {
        use super::EnvChecker;

        // 包含环境变量的 &str
        let with_env: &str = "Hello ${USER}";
        assert!(with_env.needs_env_eval());

        // 不包含环境变量的 &str
        let without_env: &str = "Hello World";
        assert!(!without_env.needs_env_eval());

        // 包含默认值语法的环境变量
        let with_default: &str = "${VAR:default}";
        assert!(with_default.needs_env_eval());

        // 多个变量
        let multi: &str = "${VAR1} and ${VAR2}";
        assert!(multi.needs_env_eval());
    }

    #[test]
    fn test_env_checker_option_string() {
        use super::EnvChecker;

        let some_with_env = Some(String::from("${VAR}"));
        assert!(some_with_env.needs_env_eval());

        let some_without_env = Some(String::from("plain text"));
        assert!(!some_without_env.needs_env_eval());

        let none: Option<String> = None;
        assert!(!none.needs_env_eval());
    }

    #[test]
    fn test_env_checker_option_str() {
        use super::EnvChecker;

        let some_with_env: Option<&str> = Some("${VAR}");
        assert!(some_with_env.needs_env_eval());

        let some_without_env: Option<&str> = Some("plain text");
        assert!(!some_without_env.needs_env_eval());

        let none: Option<&str> = None;
        assert!(!none.needs_env_eval());
    }

    #[test]
    fn test_env_checker_value_type() {
        use super::EnvChecker;

        // String 类型
        let string_with_env = ValueType::String("path: ${HOME}".to_string());
        assert!(string_with_env.needs_env_eval());

        let string_without_env = ValueType::String("plain".to_string());
        assert!(!string_without_env.needs_env_eval());

        // Bool, Number 等类型不包含环境变量
        assert!(!ValueType::Bool(true).needs_env_eval());
        assert!(!ValueType::Number(42).needs_env_eval());
        assert!(!ValueType::Float(2.5).needs_env_eval());

        // List 类型 - 递归检查
        let list_with_env = ValueType::List(vec![
            ValueType::String("${VAR1}".to_string()),
            ValueType::Number(42),
        ]);
        assert!(list_with_env.needs_env_eval());

        let list_without_env = ValueType::List(vec![
            ValueType::String("plain".to_string()),
            ValueType::Number(42),
        ]);
        assert!(!list_without_env.needs_env_eval());

        // Obj 类型 - 递归检查
        let mut obj_with_env = ValueObj::new();
        obj_with_env.insert("key1".to_string(), ValueType::String("${VAR}".to_string()));
        obj_with_env.insert("key2".to_string(), ValueType::Number(42));
        assert!(ValueType::Obj(obj_with_env).needs_env_eval());

        let mut obj_without_env = ValueObj::new();
        obj_without_env.insert("key1".to_string(), ValueType::String("plain".to_string()));
        assert!(!ValueType::Obj(obj_without_env).needs_env_eval());
    }

    #[test]
    fn test_env_eval_with_checker() {
        use super::{EnvChecker, EnvEvaluable};

        let mut dict = EnvDict::new();
        dict.insert("USER", ValueType::from("alice"));
        dict.insert("APP", ValueType::from("myapp"));

        // 只对需要求值的字符串进行求值
        let text = String::from("Hello ${USER}, welcome to ${APP}!");
        if text.needs_env_eval() {
            let result = text.env_eval(&dict);
            assert_eq!(result, "Hello alice, welcome to myapp!");
        }

        // 不需要求值的可以跳过
        let plain = String::from("No variables here");
        assert!(!plain.needs_env_eval());
    }

    #[test]
    fn test_value_type_env_eval_recursive() {
        use super::EnvEvaluable;

        let mut dict = EnvDict::new();
        dict.insert("VAR1", ValueType::from("value1"));
        dict.insert("VAR2", ValueType::from("value2"));

        // 测试 List 递归求值
        let list = ValueType::List(vec![
            ValueType::String("${VAR1}".to_string()),
            ValueType::String("${VAR2}".to_string()),
            ValueType::Number(42),
        ]);

        let evaluated = list.env_eval(&dict);
        if let ValueType::List(items) = evaluated {
            assert_eq!(items[0], ValueType::String("value1".to_string()));
            assert_eq!(items[1], ValueType::String("value2".to_string()));
            assert_eq!(items[2], ValueType::Number(42));
        } else {
            panic!("Expected List type");
        }

        // 测试 Obj 递归求值
        let mut obj = ValueObj::new();
        obj.insert(
            "field1".to_string(),
            ValueType::String("${VAR1}".to_string()),
        );
        obj.insert("field2".to_string(), ValueType::Number(100));

        let evaluated_obj = ValueType::Obj(obj).env_eval(&dict);
        if let ValueType::Obj(fields) = evaluated_obj {
            assert_eq!(
                fields.get("field1"),
                Some(&ValueType::String("value1".to_string()))
            );
            assert_eq!(fields.get("field2"), Some(&ValueType::Number(100)));
        } else {
            panic!("Expected Obj type");
        }
    }

    #[test]
    fn test_list_env_vars_string() {
        use super::EnvChecker;

        let text = String::from("Hello ${USER}, path is ${HOME}/bin");
        let vars = text.list_env_vars();
        assert_eq!(vars, vec!["USER", "HOME"]);
    }

    #[test]
    fn test_list_env_vars_str() {
        use super::EnvChecker;

        let text: &str = "Hello ${USER}, path is ${HOME}/bin";
        let vars = text.list_env_vars();
        assert_eq!(vars, vec!["USER", "HOME"]);

        // 空字符串
        let empty: &str = "";
        assert!(empty.list_env_vars().is_empty());

        // 无变量
        let plain: &str = "no variables here";
        assert!(plain.list_env_vars().is_empty());
    }

    #[test]
    fn test_list_env_vars_string_with_default() {
        use super::EnvChecker;

        let text = String::from("${VAR1:default1} and ${VAR2} and ${VAR3:default3}");
        let vars = text.list_env_vars();
        assert_eq!(vars, vec!["VAR1", "VAR2", "VAR3"]);
    }

    #[test]
    fn test_list_env_vars_str_with_default() {
        use super::EnvChecker;

        let text: &str = "${VAR1:default1} and ${VAR2} and ${VAR3:default3}";
        let vars = text.list_env_vars();
        assert_eq!(vars, vec!["VAR1", "VAR2", "VAR3"]);
    }

    #[test]
    fn test_list_env_vars_option_string() {
        use super::EnvChecker;

        let some_text = Some(String::from("${APP}/${VERSION}"));
        assert_eq!(some_text.list_env_vars(), vec!["APP", "VERSION"]);

        let none_text: Option<String> = None;
        assert!(none_text.list_env_vars().is_empty());
    }

    #[test]
    fn test_list_env_vars_option_str() {
        use super::EnvChecker;

        let some_text: Option<&str> = Some("${APP}/${VERSION}");
        assert_eq!(some_text.list_env_vars(), vec!["APP", "VERSION"]);

        let none_text: Option<&str> = None;
        assert!(none_text.list_env_vars().is_empty());
    }

    #[test]
    fn test_list_env_vars_value_type() {
        use super::EnvChecker;

        // String 类型
        let str_val = ValueType::String("${HOME}/bin".to_string());
        assert_eq!(str_val.list_env_vars(), vec!["HOME"]);

        // List 类型 - 递归收集
        let list_val = ValueType::List(vec![
            ValueType::String("${VAR1}".to_string()),
            ValueType::Number(42),
            ValueType::String("${VAR2}/${VAR3}".to_string()),
        ]);
        assert_eq!(list_val.list_env_vars(), vec!["VAR1", "VAR2", "VAR3"]);

        // Obj 类型 - 递归收集
        let mut obj = ValueObj::new();
        obj.insert(
            "path".to_string(),
            ValueType::String("${APP_DIR}/data".to_string()),
        );
        obj.insert("port".to_string(), ValueType::Number(8080));
        obj.insert(
            "config".to_string(),
            ValueType::String("${CONFIG_FILE}".to_string()),
        );
        let obj_val = ValueType::Obj(obj);
        let vars = obj_val.list_env_vars();
        // 注意：因为 IndexMap 保持插入顺序
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"APP_DIR".to_string()));
        assert!(vars.contains(&"CONFIG_FILE".to_string()));

        // 不包含变量的类型
        assert!(ValueType::Bool(true).list_env_vars().is_empty());
        assert!(ValueType::Number(42).list_env_vars().is_empty());
    }

    #[test]
    fn test_list_env_vars_nested_structures() {
        use super::EnvChecker;

        // 嵌套的复杂结构
        let mut inner_obj = ValueObj::new();
        inner_obj.insert(
            "inner1".to_string(),
            ValueType::String("${INNER_VAR}".to_string()),
        );

        let mut outer_obj = ValueObj::new();
        outer_obj.insert(
            "outer1".to_string(),
            ValueType::String("${OUTER_VAR}".to_string()),
        );
        outer_obj.insert("nested".to_string(), ValueType::Obj(inner_obj));

        let list_with_obj = ValueType::List(vec![
            ValueType::String("${LIST_VAR}".to_string()),
            ValueType::Obj(outer_obj),
        ]);

        let vars = list_with_obj.list_env_vars();
        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"LIST_VAR".to_string()));
        assert!(vars.contains(&"OUTER_VAR".to_string()));
        assert!(vars.contains(&"INNER_VAR".to_string()));
    }
}
