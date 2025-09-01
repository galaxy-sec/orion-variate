use std::{collections::HashMap, path::Path};

use crate::{tpl::TplResult, vars::types::UpperKey};
use derive_getters::Getters;
use derive_more::{Deref, From};
use indexmap::IndexMap;
use orion_conf::Yamlable;
use orion_error::ErrorOwe;
use serde_derive::{Deserialize, Serialize};

use super::{
    EnvDict,
    types::{EnvEvalable, ValueType},
};

pub type ValueMap = IndexMap<UpperKey, ValueType>;

impl EnvEvalable<ValueMap> for ValueMap {
    fn env_eval(self, dict: &EnvDict) -> ValueMap {
        let mut cur_dict = dict.clone();
        let mut vmap = ValueMap::new();
        for (k, v) in self {
            let e_v = v.env_eval(&cur_dict);
            if !cur_dict.contains_key(&k) {
                cur_dict.insert(k.clone(), e_v.clone());
            }
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
impl From<HashMap<String, String>> for ValueDict {
    fn from(map: HashMap<String, String>) -> Self {
        let mut vmap = ValueMap::new();
        for (k, v) in map {
            vmap.insert(UpperKey::from(k), ValueType::from(v));
        }
        Self { dict: vmap }
    }
}
impl ValueDict {
    pub fn new() -> Self {
        Self {
            dict: ValueMap::new(),
        }
    }

    pub fn insert<S: Into<UpperKey>>(&mut self, k: S, v: ValueType) -> Option<ValueType> {
        self.dict.insert(k.into(), v)
    }
    pub fn merge(&mut self, other: &ValueDict) {
        for (k, v) in other.iter() {
            if !self.contains_key(k) {
                self.dict.insert(k.clone(), v.clone());
            }
        }
    }

    /// 以大小写不敏感的方式获取值
    ///
    /// # 参数
    /// * `key` - 要查找的键（可以是任何大小写）
    ///
    /// # 返回值
    /// 返回对应值的引用，如果不存在则返回 None
    ///
    /// # 示例
    /// ```
    /// use orion_variate::vars::ValueDict;
    /// use orion_variate::vars::ValueType;
    ///
    /// let mut dict = ValueDict::new();
    /// dict.insert("Hello", ValueType::from("world"));
    ///
    /// assert_eq!(dict.ucase_get("hello"), Some(&ValueType::from("world")));
    /// assert_eq!(dict.ucase_get("HELLO"), Some(&ValueType::from("world")));
    /// assert_eq!(dict.ucase_get("nonexistent"), None);
    /// ```
    pub fn ucase_get<S: AsRef<str>>(&self, key: S) -> Option<&ValueType> {
        let upper_key = UpperKey::from(key.as_ref());
        self.dict.get(&upper_key)
    }

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
        value_map.insert(UpperKey::from("key1"), ValueType::from("value1"));
        value_map.insert(
            UpperKey::from("KEY2"),
            ValueType::from("${ENV_KEY1}-${KEY1}"),
        );
        value_map.insert(
            UpperKey::from("KEY3"),
            ValueType::from("${ENV_KEY2}-${KEY2}"),
        );
        value_map.insert(
            UpperKey::from("key4"),
            ValueType::from("${undefined_key:default_value}"),
        );

        // 执行env_eval
        let result = value_map.env_eval(&env_dict);

        // 验证结果
        assert_eq!(result.get("KEY1"), Some(&ValueType::from("value1")));
        assert_eq!(
            result.get("KEY2"),
            Some(&ValueType::from("env_value1-value1"))
        );
        assert_eq!(
            result.get("KEY3"),
            Some(&ValueType::from("env_value2-env_value1-value1"))
        );
        assert_eq!(result.get("KEY4"), Some(&ValueType::from("default_value")));
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
        env_dict.insert("HOST".to_string(), ValueType::from("example.com"));
        env_dict.insert("PORT".to_string(), ValueType::from("8080"));

        // 创建ValueMap，使用环境变量和字典变量
        let mut value_map = ValueMap::new();
        value_map.insert(UpperKey::from("config1"), ValueType::from("${HOST}"));
        value_map.insert(UpperKey::from("config2"), ValueType::from("${ENV_HOST}"));
        value_map.insert(
            UpperKey::from("config3"),
            ValueType::from("prefix_${HOST}_suffix"),
        );
        value_map.insert(
            UpperKey::from("config4"),
            ValueType::from("prefix_${ENV_HOST}_suffix"),
        );

        // 执行env_eval
        let result = value_map.env_eval(&env_dict);

        // 验证结果
        assert_eq!(result.get("CONFIG1"), Some(&ValueType::from("example.com")));
        assert_eq!(
            result.get("CONFIG2"),
            Some(&ValueType::from("env.example.com"))
        );
        assert_eq!(
            result.get("CONFIG3"),
            Some(&ValueType::from("prefix_example.com_suffix"))
        );
        assert_eq!(
            result.get("CONFIG4"),
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

    #[test]
    fn test_ucase_get() {
        let mut dict = ValueDict::new();
        dict.insert("Hello", ValueType::from("world"));
        dict.insert("WORLD", ValueType::from("hello"));
        dict.insert("CamelCase", ValueType::from("value"));

        // 测试大小写不敏感查找
        assert_eq!(dict.ucase_get("hello"), Some(&ValueType::from("world")));
        assert_eq!(dict.ucase_get("HELLO"), Some(&ValueType::from("world")));
        assert_eq!(dict.ucase_get("Hello"), Some(&ValueType::from("world")));

        // 测试不同键的查找
        assert_eq!(dict.ucase_get("world"), Some(&ValueType::from("hello")));
        assert_eq!(dict.ucase_get("World"), Some(&ValueType::from("hello")));
        assert_eq!(dict.ucase_get("WORLD"), Some(&ValueType::from("hello")));

        // 测试驼峰命名查找
        assert_eq!(dict.ucase_get("camelcase"), Some(&ValueType::from("value")));
        assert_eq!(dict.ucase_get("CAMELCASE"), Some(&ValueType::from("value")));
        assert_eq!(dict.ucase_get("CamelCase"), Some(&ValueType::from("value")));

        // 测试不存在的键
        assert_eq!(dict.ucase_get("nonexistent"), None);
        assert_eq!(dict.ucase_get(""), None);
    }

    #[test]
    fn test_ucase_get_with_special_chars() {
        let mut dict = ValueDict::new();
        dict.insert("key-with-dashes", ValueType::from("dashed"));
        dict.insert("key_with_underscores", ValueType::from("underscored"));
        dict.insert("key.with.dots", ValueType::from("dotted"));

        // 测试包含特殊字符的键
        assert_eq!(
            dict.ucase_get("key-with-dashes"),
            Some(&ValueType::from("dashed"))
        );
        assert_eq!(
            dict.ucase_get("KEY-WITH-DASHES"),
            Some(&ValueType::from("dashed"))
        );

        assert_eq!(
            dict.ucase_get("key_with_underscores"),
            Some(&ValueType::from("underscored"))
        );
        assert_eq!(
            dict.ucase_get("KEY_WITH_UNDERSCORES"),
            Some(&ValueType::from("underscored"))
        );

        assert_eq!(
            dict.ucase_get("key.with.dots"),
            Some(&ValueType::from("dotted"))
        );
        assert_eq!(
            dict.ucase_get("KEY.WITH.DOTS"),
            Some(&ValueType::from("dotted"))
        );
    }

    #[test]
    fn test_ucase_get_edge_cases() {
        let mut dict = ValueDict::new();

        // 测试空字典
        assert_eq!(dict.ucase_get("any"), None);

        // 插入空字符串键
        dict.insert("", ValueType::from("empty"));
        assert_eq!(dict.ucase_get(""), Some(&ValueType::from("empty")));
        assert_eq!(dict.ucase_get("  "), None);

        // 测试 Unicode 字符
        dict.insert("café", ValueType::from("coffee"));
        assert_eq!(dict.ucase_get("CAFÉ"), Some(&ValueType::from("coffee")));
        assert_eq!(dict.ucase_get("café"), Some(&ValueType::from("coffee")));

        // 测试数字键
        dict.insert("123", ValueType::from("number"));
        assert_eq!(dict.ucase_get("123"), Some(&ValueType::from("number")));
        assert_eq!(dict.ucase_get("123"), Some(&ValueType::from("number")));
    }

    #[test]
    fn test_dict_yaml_block_serialization() {
        // 创建包含多行块数据的 ValueDict
        let mut dict = ValueDict::new();
        dict.insert(
            "block_text",
            ValueType::from(
                "This is a multi-line\ntext block that preserves\nline breaks and formatting\n",
            ),
        );
        dict.insert(
            "indented_block",
            ValueType::from(
                "This block has indentation\nthat should be preserved\nacross multiple lines\n",
            ),
        );
        dict.insert("special_chars", ValueType::from("Contains special characters:\n- Tabs:\t\n- Quotes: \"hello\"\n- Backslashes: \\n\\r\\t"));

        // 序列化为 YAML
        let yaml_output = serde_yaml::to_string(&dict).unwrap();
        println!("YAML 输出:\n{}", yaml_output);

        // 验证序列化结果包含 "|" 符号（YAML 块格式）
        assert!(
            yaml_output.contains("|"),
            "YAML 输出应该包含 '|' 符号表示块数据"
        );
        assert!(
            yaml_output.contains("This is a multi-line"),
            "YAML 输出应该包含多行文本内容"
        );
        assert!(
            yaml_output.contains("line breaks and formatting"),
            "YAML 输出应该包含完整的块文本内容"
        );

        // 反序列化测试 - 从字符串创建 YAML 块数据
        let yaml_with_blocks = r#"
BLOCK_TEXT: |
  This is a multi-line
  text block that preserves
  line breaks and formatting

INDENTED_BLOCK: |
    This block has indentation
    that should be preserved
    across multiple lines

SPECIAL_CHARS: "Contains special characters:\n- Tabs:\t\n- Quotes: \"hello\"\n- Backslashes: \\n\\r\\t"
"#;

        // 反序列化 YAML
        let deserialized_dict: ValueDict = serde_yaml::from_str(yaml_with_blocks).unwrap();

        // 验证反序列化结果
        assert_eq!(
            deserialized_dict.ucase_get("BLOCK_TEXT"),
            Some(&ValueType::from(
                "This is a multi-line\ntext block that preserves\nline breaks and formatting\n"
            ))
        );
        assert_eq!(
            deserialized_dict.ucase_get("INDENTED_BLOCK"),
            Some(&ValueType::from(
                "This block has indentation\nthat should be preserved\nacross multiple lines\n"
            ))
        );
        assert_eq!(
            deserialized_dict.ucase_get("SPECIAL_CHARS"),
            Some(&ValueType::from(
                "Contains special characters:\n- Tabs:\t\n- Quotes: \"hello\"\n- Backslashes: \\n\\r\\t"
            ))
        );

        // 往返一致性测试
        let roundtrip_yaml = serde_yaml::to_string(&deserialized_dict).unwrap();
        let roundtrip_dict: ValueDict = serde_yaml::from_str(&roundtrip_yaml).unwrap();

        // 验证往返序列化后数据保持一致
        assert_eq!(
            deserialized_dict, roundtrip_dict,
            "往返序列化后数据应该保持一致"
        );

        // 验证块数据在往返过程中格式保持
        assert_eq!(
            roundtrip_dict.ucase_get("BLOCK_TEXT"),
            Some(&ValueType::from(
                "This is a multi-line\ntext block that preserves\nline breaks and formatting\n"
            ))
        );
        assert_eq!(
            roundtrip_dict.ucase_get("INDENTED_BLOCK"),
            Some(&ValueType::from(
                "This block has indentation\nthat should be preserved\nacross multiple lines\n"
            ))
        );
        assert_eq!(
            roundtrip_dict.ucase_get("SPECIAL_CHARS"),
            Some(&ValueType::from(
                "Contains special characters:\n- Tabs:\t\n- Quotes: \"hello\"\n- Backslashes: \\n\\r\\t"
            ))
        );

        println!("往返序列化测试通过！块数据格式在序列化/反序列化过程中保持正确。");
    }
}
