use getset::Getters;
use indexmap::IndexMap;
use orion_conf::StorageLoadEvent;
use serde_derive::{Deserialize, Serialize};

use super::{ValueDict, VarDefinition, definition::ChangeScope};

#[derive(Getters, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[getset(get = "pub")]
//#[serde(transparent)]
pub struct VarCollection {
    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "immutable")]
    immutable_vars: Vec<VarDefinition>,

    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        rename = "public",
        alias = "vars"
    )]
    public_vars: Vec<VarDefinition>,

    #[serde(default, skip_serializing_if = "Vec::is_empty", rename = "model")]
    modul_vars: Vec<VarDefinition>,
}
impl StorageLoadEvent for VarCollection {
    fn loaded_event_do(&mut self) {
        self.mark_vars_scope();
    }
}
impl VarCollection {
    pub fn define(vars: Vec<VarDefinition>) -> Self {
        let mut immutable_vars = Vec::new();
        let mut public_vars = Vec::new();
        let mut modul_vars = Vec::new();

        for v in vars {
            match v.scope() {
                ChangeScope::Immutable => {
                    immutable_vars.push(v);
                }
                ChangeScope::Public => {
                    public_vars.push(v);
                }
                ChangeScope::Model => modul_vars.push(v),
            }
        }
        Self {
            immutable_vars,
            public_vars,
            modul_vars,
        }
    }
    pub fn mark_vars_scope(&mut self) {
        for var in self.immutable_vars.iter_mut() {
            var.set_scope(ChangeScope::Immutable);
        }
        for var in self.public_vars.iter_mut() {
            var.set_scope(ChangeScope::Public);
        }
        for var in self.modul_vars.iter_mut() {
            var.set_scope(ChangeScope::Model);
        }
    }

    pub fn value_dict(&self) -> ValueDict {
        let mut dict = ValueDict::new();
        for var in self.immutable_vars() {
            dict.insert(var.name().to_string(), var.value().clone()); // 可能需要 into() 转换
        }
        for var in self.public_vars() {
            dict.insert(var.name().to_string(), var.value().clone()); // 可能需要 into() 转换
        }
        for var in self.modul_vars() {
            dict.insert(var.name().to_string(), var.value().clone()); // 可能需要 into() 转换
        }
        dict
    }
    // 基于VarType的name进行合并，相同的name会被覆盖
    pub fn merge(self, other: VarCollection) -> Self {
        let immutable_vars = merge_vec(self.immutable_vars, other.immutable_vars, false);
        let public_vars = merge_vec(self.public_vars, other.public_vars, true);
        let modul_vars = merge_vec(self.modul_vars, other.modul_vars, true);
        Self {
            immutable_vars,
            public_vars,
            modul_vars,
        }
    }

    /*
    fn eval_import(self, dict: &mut ValueDict) -> Self {
        let mut vars = Vec::new();
        for v in self.vars {
            let e_v = v.value().clone().env_eval(dict);
            dict.insert(v.name(), e_v.clone());
            vars.push(VarDefinition::from((v.name().as_str(), e_v)));
        }
        Self { vars }
    }
    */
}
fn merge_vec(
    my: Vec<VarDefinition>,
    other: Vec<VarDefinition>,
    is_over: bool,
) -> Vec<VarDefinition> {
    let mut target = Vec::new();
    let mut merged = IndexMap::new();
    for var in my {
        //immutable_vars.push(var)
        merged.insert(var.name().clone(), var);
    }
    for var in other {
        if is_over || !merged.contains_key(var.name()) {
            merged.insert(var.name().clone(), var);
        }
    }
    for var in merged.into_values() {
        target.push(var);
    }
    target
}

#[cfg(test)]
mod tests {
    use crate::vars::ValueType;
    use crate::vars::definition::ChangeScope;

    use super::*;
    use serde_json;
    use serde_yaml;

    #[test]
    fn test_define_classification() {
        // 创建测试变量
        let vars = vec![
            VarDefinition::from(("immutable_var", "immutable_value"))
                .with_scope(ChangeScope::Immutable),
            VarDefinition::from(("public_var", "public_value")).with_scope(ChangeScope::Public),
            VarDefinition::from(("model_var", "model_value")).with_scope(ChangeScope::Model),
        ];

        let collection = VarCollection::define(vars);

        // 验证分类正确性
        assert_eq!(collection.immutable_vars().len(), 1);
        assert_eq!(collection.immutable_vars()[0].name(), "immutable_var");

        assert_eq!(collection.public_vars().len(), 1);
        assert_eq!(collection.public_vars()[0].name(), "public_var");

        assert_eq!(collection.modul_vars().len(), 1);
        assert_eq!(collection.modul_vars()[0].name(), "model_var");
    }

    #[test]
    fn test_value_dict_generation() {
        let vars = vec![
            VarDefinition::from(("immutable_var", "immutable_value"))
                .with_scope(ChangeScope::Immutable),
            VarDefinition::from(("public_var", "public_value")).with_scope(ChangeScope::Public),
            VarDefinition::from(("model_var", "model_value")).with_scope(ChangeScope::Model),
            VarDefinition::from(("numeric_var", 42u64)).with_scope(ChangeScope::Public),
        ];

        let collection = VarCollection::define(vars);
        let dict = collection.value_dict();

        // 验证字典包含所有变量
        assert_eq!(dict.len(), 4);
        assert_eq!(
            dict.get("immutable_var"),
            Some(&ValueType::from("immutable_value"))
        );
        assert_eq!(
            dict.get("public_var"),
            Some(&ValueType::from("public_value"))
        );
        assert_eq!(dict.get("model_var"), Some(&ValueType::from("model_value")));
        assert_eq!(dict.get("numeric_var"), Some(&ValueType::from(42u64)));
    }

    #[test]
    fn test_merge_collections() {
        let vars1 = vec![
            VarDefinition::from(("var1", "value1_from_1")).with_scope(ChangeScope::Public),
            VarDefinition::from(("var2", "value2_from_1")).with_scope(ChangeScope::Immutable),
            VarDefinition::from(("unique_to_1", "unique")).with_scope(ChangeScope::Model),
        ];

        let vars2 = vec![
            VarDefinition::from(("var1", "value1_from_2")).with_scope(ChangeScope::Public),
            VarDefinition::from(("var3", "value3_from_2")).with_scope(ChangeScope::Model),
            VarDefinition::from(("unique_to_2", "unique2")).with_scope(ChangeScope::Public),
        ];

        let collection1 = VarCollection::define(vars1);
        let collection2 = VarCollection::define(vars2);

        let merged = collection1.merge(collection2);

        // 验证合并结果
        assert_eq!(merged.public_vars().len(), 2); // unique_to_1, unique_to_2
        assert_eq!(merged.immutable_vars().len(), 1); // var2
        assert_eq!(merged.modul_vars().len(), 2); // var3, unique_to_1

        // 验证重复变量被正确处理
        let dict = merged.value_dict();
        assert_eq!(dict.get("var1"), Some(&ValueType::from("value1_from_2"))); // 第一个集合的值优先
        assert_eq!(dict.get("var2"), Some(&ValueType::from("value2_from_1")));
        assert_eq!(dict.get("var3"), Some(&ValueType::from("value3_from_2")));
        assert_eq!(dict.get("unique_to_1"), Some(&ValueType::from("unique")));
        assert_eq!(dict.get("unique_to_2"), Some(&ValueType::from("unique2")));
    }

    #[test]
    fn test_serialization_deserialization() {
        let vars = vec![
            VarDefinition::from(("string_var", "hello")).with_scope(ChangeScope::Immutable),
            VarDefinition::from(("bool_var", true)).with_scope(ChangeScope::Public),
            // 注释掉 model 变量以测试空字段跳过
            // VarDefinition::from(("number_var", 42u64)).with_scope(ChangeScope::Model),
        ];

        let original = VarCollection::define(vars);

        // 测试 JSON 序列化/反序列化
        let json = serde_json::to_string(&original).unwrap();
        let mut deserialized: VarCollection = serde_json::from_str(&json).unwrap();
        deserialized.mark_vars_scope();
        assert_eq!(original, deserialized);

        // 测试 YAML 序列化/反序列化
        let yaml = serde_yaml::to_string(&original).unwrap();
        println!("{yaml:#}");
        let mut deserialized_yaml: VarCollection = serde_yaml::from_str(&yaml).unwrap();
        deserialized_yaml.mark_vars_scope();
        assert_eq!(original, deserialized_yaml);

        // 验证序列化优化：空的字段应该被跳过
        // 首先检查 model_vars 是否为空
        assert_eq!(original.modul_vars().len(), 0, "model_vars should be empty");
        // model_vars 为空，应该被跳过
        assert!(
            !json.contains("\"model\""),
            "model field should be skipped in serialization"
        );
        // immutable_vars 不为空，应该包含
        assert!(
            json.contains("\"immutable\""),
            "immutable field should be included in serialization"
        );

        // 调试输出
        println!("JSON output: {}", json);
    }

    #[test]
    fn test_serialization_field_optimization() {
        // 测试 skip_serializing_if 逻辑
        let empty_collection = VarCollection::default();
        let json = serde_json::to_string(&empty_collection).unwrap();

        // 空集合应该序列化为空对象 {}
        assert_eq!(json, "{}");

        // 只有 public 变量的集合
        let vars =
            vec![VarDefinition::from(("public_var", "value")).with_scope(ChangeScope::Public)];
        let public_only = VarCollection::define(vars);
        let json_public = serde_json::to_string(&public_only).unwrap();

        // 应该只包含 public 字段
        assert!(json_public.contains("\"public\""));
        assert!(!json_public.contains("\"immutable\""));
        assert!(!json_public.contains("\"model\""));
    }

    #[test]
    fn test_empty_collection() {
        let empty_vars = vec![];
        let collection = VarCollection::define(empty_vars);

        assert_eq!(collection.immutable_vars().len(), 0);
        assert_eq!(collection.public_vars().len(), 0);
        assert_eq!(collection.modul_vars().len(), 0);

        let dict = collection.value_dict();
        assert_eq!(dict.len(), 0);
    }

    #[test]
    fn test_duplicate_variable_names() {
        let vars = vec![
            VarDefinition::from(("duplicate", "first")).with_scope(ChangeScope::Immutable),
            VarDefinition::from(("duplicate", "second")).with_scope(ChangeScope::Public),
            VarDefinition::from(("duplicate", "third")).with_scope(ChangeScope::Model),
        ];

        let collection = VarCollection::define(vars);

        // 验证每个作用域都有一个重复名称的变量
        assert_eq!(collection.immutable_vars().len(), 1);
        assert_eq!(collection.public_vars().len(), 1);
        assert_eq!(collection.modul_vars().len(), 1);

        // 验证 value_dict 包含所有变量（尽管名称相同，value_dict 是 IndexMap，后插入的会覆盖先插入的）
        let dict = collection.value_dict();
        // 由于 value_dict 按 immutable -> public -> model 的顺序插入，model 会覆盖前面同名的
        assert_eq!(dict.get("duplicate"), Some(&ValueType::from("third")));
    }

    #[test]
    fn test_special_characters_in_names() {
        let vars = vec![
            VarDefinition::from(("normal_name", "normal")).with_scope(ChangeScope::Public),
            VarDefinition::from(("name-with-dashes", "dashed")).with_scope(ChangeScope::Public),
            VarDefinition::from(("name_with_underscores", "underscored"))
                .with_scope(ChangeScope::Public),
            VarDefinition::from(("name.with.dots", "dotted")).with_scope(ChangeScope::Public),
        ];

        let collection = VarCollection::define(vars);
        let dict = collection.value_dict();

        // 验证特殊字符名称能正确处理
        assert_eq!(dict.get("normal_name"), Some(&ValueType::from("normal")));
        assert_eq!(
            dict.get("name-with-dashes"),
            Some(&ValueType::from("dashed"))
        );
        assert_eq!(
            dict.get("name_with_underscores"),
            Some(&ValueType::from("underscored"))
        );
        assert_eq!(dict.get("name.with.dots"), Some(&ValueType::from("dotted")));
    }

    #[test]
    fn test_default_collection() {
        let default_collection = VarCollection::default();

        assert_eq!(default_collection.immutable_vars().len(), 0);
        assert_eq!(default_collection.public_vars().len(), 0);
        assert_eq!(default_collection.modul_vars().len(), 0);

        let dict = default_collection.value_dict();
        assert_eq!(dict.len(), 0);

        // 测试序列化
        let json = serde_json::to_string(&default_collection).unwrap();
        assert_eq!(json, "{}");
    }
}
