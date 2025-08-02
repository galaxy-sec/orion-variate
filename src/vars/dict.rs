use std::path::Path;

use crate::tpl::TplResult;
use derive_getters::Getters;
use derive_more::{Deref, From};
use indexmap::IndexMap;
use orion_common::serde::Yamlable;
use orion_error::ErrorOwe;
use serde_derive::{Deserialize, Serialize};

use super::{
    EnvDict,
    types::{EnvEvalable, ValueType},
};

pub type ValueMap = IndexMap<String, ValueType>;

impl EnvEvalable<ValueMap> for ValueMap {
    fn env_eval(self, dict: &EnvDict) -> ValueMap {
        let mut cur_dict = dict.clone();
        let mut vmap = ValueMap::new();
        for (k, v) in self {
            let e_v = v.env_eval(&cur_dict);
            cur_dict.insert(k.clone(), e_v.clone());
            vmap.insert(k, e_v);
        }
        vmap
    }
}

impl EnvEvalable<ValueDict> for ValueDict {
    fn env_eval(mut self, dict: &EnvDict) -> ValueDict {
        self.dict = self.dict.env_eval(dict);
        self
    }
}

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Deref, Default, From)]
#[serde(transparent)]
pub struct ValueDict {
    dict: ValueMap,
}
impl ValueDict {
    pub fn new() -> Self {
        Self {
            dict: ValueMap::new(),
        }
    }

    pub fn insert<S: Into<String>>(&mut self, k: S, v: ValueType) -> Option<ValueType> {
        self.dict.insert(k.into(), v)
    }
    pub fn merge(&mut self, other: &ValueDict) {
        for (k, v) in other.iter() {
            if !self.contains_key(k) {
                self.dict.insert(k.clone(), v.clone());
            }
        }
    }
    /*
    pub fn env_eval(self, dict: &EnvDict) -> Self {
        let mut map = ValueMap::new();
        for (k, v) in self.dict {
            let e_v = v.env_eval(dict);
            map.insert(k, e_v);
        }
        Self { dict: map }
    }
    */
    pub fn eval_from_file(dict: &EnvDict, file_path: &Path) -> TplResult<Self> {
        //let mut cur_dict = dict.clone();
        let ins = ValueDict::from_yml(file_path).owe_res()?;
        Ok(ins.env_eval(dict))
    }

    /*
    fn eval_import(self, dict: &mut ValueDict) -> Self {
        let mut map = ValueMap::new();
        for (k, v) in self.dict {
            let e_v = v.env_eval(dict);
            dict.insert(k.clone(), e_v.clone());
            map.insert(k, e_v);
        }
        Self { dict: map }
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use tempfile::NamedTempFile;

    // 辅助函数：创建临时测试文件
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{content}",).unwrap();
        file
    }
    #[test]
    fn test_eval_from_file_basic() {
        // 准备测试数据
        let file = create_temp_file(
            r#"
        key1: "value1"
        key2:  ${key1}
        key3:  ${key2}
        "#,
        );

        // 准备环境字典
        let env_dict = EnvDict::new();

        // 执行方法
        let result = ValueDict::eval_from_file(&env_dict, file.path()).unwrap();

        // 验证结果
        assert_eq!(result.get("key1"), Some(&ValueType::from("value1")));
        assert_eq!(result.get("key2"), Some(&ValueType::from("value1")));
        assert_eq!(result.get("key3"), Some(&ValueType::from("value1")));
    }

    #[test]
    fn test_dict_toml_serialization() {
        let mut dict = ValueDict::new();
        dict.insert("key1".to_string(), ValueType::from("value1"));
        dict.insert("key2".to_string(), ValueType::from(42));
        let content = toml::to_string(&dict).unwrap();
        println!("{content}",);

        let loaded: ValueDict = toml::from_str(content.as_str()).unwrap();
        assert_eq!(dict, loaded);

        let content = serde_yaml::to_string(&dict).unwrap();
        println!("{content}",);

        let content = serde_json::to_string(&dict).unwrap();
        println!("{content}",);
    }

    #[test]
    fn test_value_map_env_eval() {
        // 创建环境字典
        let mut env_dict = EnvDict::new();
        env_dict.insert("env_key1".to_string(), ValueType::from("env_value1"));
        env_dict.insert("ENV_KEY2".to_string(), ValueType::from("env_value2"));

        // 创建ValueMap
        let mut value_map = ValueMap::new();
        value_map.insert("key1".to_string(), ValueType::from("value1"));
        value_map.insert("KEY2".to_string(), ValueType::from("${env_key1}-${key1}"));
        value_map.insert("KEY3".to_string(), ValueType::from("${ENV_KEY2}-${KEY2}"));
        value_map.insert(
            "key4".to_string(),
            ValueType::from("${undefined_key:default_value}"),
        );

        // 执行env_eval
        let result = value_map.env_eval(&env_dict);

        // 验证结果
        assert_eq!(result.get("key1"), Some(&ValueType::from("value1")));
        assert_eq!(
            result.get("KEY2"),
            Some(&ValueType::from("env_value1-value1"))
        );
        assert_eq!(
            result.get("KEY3"),
            Some(&ValueType::from("env_value2-env_value1-value1"))
        );
        assert_eq!(result.get("key4"), Some(&ValueType::from("default_value")));
    }

    #[test]
    fn test_value_map_env_eval_single_var() {
        // 设置环境变量
        unsafe {
            std::env::set_var("ENV_HOST", "env.example.com");
        }
        unsafe {
            std::env::set_var("ENV_PORT", "9090");
        }

        // 创建环境字典
        let mut env_dict = EnvDict::new();
        env_dict.insert("host".to_string(), ValueType::from("example.com"));
        env_dict.insert("port".to_string(), ValueType::from("8080"));

        // 创建ValueMap，使用环境变量和字典变量
        let mut value_map = ValueMap::new();
        value_map.insert("config1".to_string(), ValueType::from("${host}"));
        value_map.insert("config2".to_string(), ValueType::from("${ENV_HOST}"));
        value_map.insert(
            "config3".to_string(),
            ValueType::from("prefix_${host}_suffix"),
        );
        value_map.insert(
            "config4".to_string(),
            ValueType::from("prefix_${ENV_HOST}_suffix"),
        );

        // 执行env_eval
        let result = value_map.env_eval(&env_dict);

        // 验证结果
        assert_eq!(result.get("config1"), Some(&ValueType::from("example.com")));
        assert_eq!(
            result.get("config2"),
            Some(&ValueType::from("env.example.com"))
        );
        assert_eq!(
            result.get("config3"),
            Some(&ValueType::from("prefix_example.com_suffix"))
        );
        assert_eq!(
            result.get("config4"),
            Some(&ValueType::from("prefix_env.example.com_suffix"))
        );

        // 清理环境变量
        unsafe {
            std::env::remove_var("ENV_HOST");
        }
        unsafe {
            std::env::remove_var("ENV_PORT");
        }
    }
}
