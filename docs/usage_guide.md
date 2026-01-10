# Orion Variate 使用指南

## 📋 概述

**orion-variate** 是一个用于大型项目的变量管理 Rust 库，提供以下核心功能：

- 多类型值系统（String、Bool、Number、Float、IP、Object、List）
- 大小写不敏感的字典访问
- 环境变量插值（支持 `${VAR}` 和 `${VAR:default}` 语法）
- 变量可变性控制（Immutable、System、Module 三级作用域）
- 工作目录 RAII 守卫
- 项目根目录查找
- 序列化/反序列化支持（JSON、YAML、TOML、INI）

---

## 🚀 快速开始

### 1. 添加依赖

```toml
[dependencies]
orion-variate = "0.10"
```

### 2. 基本用法

```rust
use orion_variate::{ValueDict, ValueType, CwdGuard};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建大小写不敏感的字典
    let mut dict = ValueDict::new();
    dict.insert("Host", ValueType::from("example.com"));
    dict.insert("Port", ValueType::from(8080u64));

    // 大小写不敏感查询
    assert_eq!(dict.get_case_insensitive("HOST").unwrap().to_string(), "example.com");
    assert_eq!(dict.get_case_insensitive("port").unwrap().to_string(), "8080");

    // 工作目录守卫（自动恢复）
    {
        let _guard = CwdGuard::change("/tmp")?;
        // 在这个作用域内，当前目录是 /tmp
    } // Drop 时自动恢复到原目录

    Ok(())
}
```

---

## 📦 核心类型详解

### ValueType - 多类型值枚举

支持 7 种类型的值：

```rust
use orion_variate::{ValueType, ValueObj, ValueVec};
use std::net::IpAddr;

// String
let s = ValueType::String("hello".to_string());
let s = ValueType::from("hello");  // 更简洁

// Bool
let b = ValueType::Bool(true);
let b = ValueType::from(true);

// Number (u64)
let n = ValueType::Number(42);
let n = ValueType::from(42u64);

// Float (f64)
let f = ValueType::Float(3.14);
let f = ValueType::from(3.14);

// IP地址
let ip = ValueType::Ip("127.0.0.1".parse::<IpAddr>()?);

// Object (键值对)
let mut obj = ValueObj::new();
obj.insert("name".to_string(), ValueType::from("Alice"));
obj.insert("age".to_string(), ValueType::from(30u64));
let o = ValueType::Obj(obj);

// List (数组)
let list = ValueVec::from([
    ValueType::from("item1"),
    ValueType::from("item2"),
]);
let l = ValueType::List(list);
```

**ValueType 常用方法：**

```rust
let mut value = ValueType::from("hello");

// 获取类型名称
assert_eq!(value.variant_name(), "String");

// 获取长度
assert_eq!(value.len(), 5);

// 检查是否为空
assert!(!value.is_empty());

// 从字符串更新值（保持类型不变）
let mut num = ValueType::from(100u64);
num.update_from_str("42")?;  // 更新为 42
assert_eq!(num, ValueType::from(42u64));

// 显示值
println!("{}", value);  // 输出: hello
```

---

### ValueDict - 大小写不敏感字典

```rust
use orion_variate::{ValueDict, ValueType};

let mut dict = ValueDict::new();

// 插入值（键会自动转为大写存储）
dict.insert("Host", ValueType::from("example.com"));
dict.insert("PORT", ValueType::from(8080u64));

// 大小写不敏感查询
assert_eq!(dict.get_case_insensitive("host"), Some(&ValueType::from("example.com")));
assert_eq!(dict.get_case_insensitive("HoSt"), Some(&ValueType::from("example.com")));
assert_eq!(dict.get_case_insensitive("port"), Some(&ValueType::from(8080u64)));

// 合并字典（不覆盖已存在的键）
let mut dict2 = ValueDict::new();
dict2.insert("timeout", ValueType::from(30u64));
dict.merge(&dict2);

// 序列化/反序列化
let json = serde_json::to_string(&dict)?;
let yaml = serde_yaml::to_string(&dict)?;
let loaded: ValueDict = serde_json::from_str(&json)?;
```

---

### VarDefinition - 变量定义

```rust
use orion_variate::{VarDefinition, ValueType, Mutability};

// 从元组创建
let var = VarDefinition::from(("db_host", "localhost"));

// 带描述的变量
let var = VarDefinition::from(("db_port", 5432u64))
    .with_desc(Some("数据库端口".to_string()));

// 设置可变性
let immutable_var = VarDefinition::from(("api_key", "secret"))
    .with_mut_immutable();  // 不可变

let system_var = VarDefinition::from(("log_level", "info"))
    .with_mut_system();  // 系统级可变

let module_var = VarDefinition::from(("cache_size", 1024u64))
    .with_mut_module();  // 模块级可变（默认）

// 检查可变性
assert!(!immutable_var.is_mutable());
assert!(system_var.is_mutable());
```

---

### Mutability - 可变性枚举

三级可变性控制：

```rust
use orion_variate::Mutability;

// Immutable - 不可变，不允许任何修改
let immutable = Mutability::Immutable;

// System - 系统级可变，允许在任何上下文中修改
let system = Mutability::System;

// Module - 模块级可变，只在同一模块内允许修改（默认）
let module = Mutability::Module;  // 这是默认值

// 工厂方法
let m1 = Mutability::immutable();
let m2 = Mutability::system();
let m3 = Mutability::module();
```

---

### VarCollection - 变量集合

按可变性分类管理变量：

```rust
use orion_variate::{VarCollection, VarDefinition, Mutability, ValueDict};

// 创建变量列表
let vars = vec![
    VarDefinition::from(("app_name", "MyApp"))
        .with_mut_immutable(),
    VarDefinition::from(("version", "1.0.0"))
        .with_mut_immutable(),
    VarDefinition::from(("debug", true))
        .with_mut_system(),
    VarDefinition::from(("cache_ttl", 300u64))
        .with_mut_module(),
];

// 创建集合（自动分类）
let collection = VarCollection::define(vars);

// 访问各类变量
println!("Immutable vars: {}", collection.immutable_vars().len());
println!("System vars: {}", collection.system_vars().len());
println!("Module vars: {}", collection.module_vars().len());

// 导出为字典
let dict: ValueDict = collection.value_dict();

// 合并集合
let collection1 = VarCollection::define(vec![/* ... */]);
let collection2 = VarCollection::define(vec![/* ... */]);
let merged = collection1.merge(collection2);  // 后者覆盖前者
```

**序列化示例（YAML）：**

```yaml
immutable:
  - name: app_name
    value: MyApp
  - name: version
    value: "1.0.0"

system:
  - name: debug
    value: true

module:
  - name: cache_ttl
    value: 300
```

---

## 🌍 环境变量插值

### EnvEvaluable Trait

支持 `${VAR}` 和 `${VAR:default}` 语法：

```rust
use orion_variate::{EnvDict, EnvEvaluable, ValueType, ValueDict};
use std::env;

// 设置环境变量
env::set_var("APP_ENV", "production");
env::set_var("APP_PORT", "8080");

// 创建环境字典
let mut env_dict = EnvDict::new();
env_dict.insert("host", ValueType::from("example.com"));
env_dict.insert("timeout", ValueType::from(30u64));

// String 插值
let template = "Server: ${HOST}:${APP_PORT}".to_string();
let result = template.env_eval(&env_dict);
// 结果: "Server: example.com:8080"

// 带默认值的插值
let template2 = "DB: ${DB_HOST:localhost}:${DB_PORT:5432}".to_string();
let result2 = template2.env_eval(&env_dict);
// 如果 DB_HOST 和 DB_PORT 未定义，使用默认值
// 结果: "DB: localhost:5432"

// ValueType 插值
let value = ValueType::from("Path: ${HOME}/data");
let evaluated = value.env_eval(&env_dict);

// ValueDict 插值
let mut dict = ValueDict::new();
dict.insert("url", ValueType::from("http://${HOST}:${APP_PORT}/api"));
let evaluated_dict = dict.env_eval(&env_dict);
```

**查找优先级：**

1. 先查找 `EnvDict` 中的变量
2. 如果未找到，查找系统环境变量
3. 如果都未找到，使用默认值（如果提供）
4. 否则保持原样 `${VAR}`

---

## 📁 OriginDict - 带来源追踪的字典

```rust
use orion_variate::{OriginDict, OriginValue, ValueType};

// 创建 OriginDict
let mut dict = OriginDict::new();

// 插入值
dict.insert("key1", ValueType::from("value1"));
dict.insert("key2", ValueType::from("value2"));

// 设置来源标签
dict.set_source("config.yaml");

// 访问值及其来源
if let Some(origin_val) = dict.get_case_insensitive("key1") {
    println!("Value: {}", origin_val.value());
    println!("Origin: {:?}", origin_val.origin());
    println!("Is mutable: {}", origin_val.is_mutable());
}

// 带来源的值
let value = OriginValue::from("data")
    .with_origin("user_input")
    .with_mutability(Mutability::Immutable);

// 合并字典（遵循可变性规则）
let mut dict1 = OriginDict::new();
dict1.insert("key", ValueType::from("original"));

let mut dict2 = OriginDict::new();
dict2.insert("key", ValueType::from("updated"));

dict1.merge(&dict2);  // 只有可变的值会被覆盖

// 导出为 ValueDict
let value_dict = dict.export_dict();
```

---

## 🗂️ 项目管理工具

### 查找项目根目录

```rust
use orion_variate::{find_project_root, find_project_root_from};
use std::path::PathBuf;

// 从当前目录开始向上查找 _gal/project.toml
if let Some(root) = find_project_root() {
    println!("Project root: {}", root.display());
}

// 从指定目录开始查找
let base = PathBuf::from("/path/to/subdir");
if let Some(root) = find_project_root_from(base) {
    println!("Found project at: {}", root.display());
}
```

### 设置启动环境变量

```rust
use orion_variate::setup_start_env_vars;

// 自动设置以下环境变量：
// - GXL_OS_SYS: 操作系统信息 (如 "arm64_macos_14")
// - GXL_START_ROOT: 启动时的工作目录
// - GXL_PRJ_ROOT: 项目根目录
setup_start_env_vars()?;

// 之后可以在环境变量中访问
println!("OS: {}", std::env::var("GXL_OS_SYS").unwrap());
```

---

## 🔐 CwdGuard - 工作目录守卫

RAII 模式的目录切换：

```rust
use orion_variate::CwdGuard;
use std::env;

let original = env::current_dir()?;

{
    // 进入新目录
    let _guard = CwdGuard::change("/tmp")?;
    println!("Current dir: {}", env::current_dir()?.display());
    // 输出: Current dir: /tmp

    // 可以进行各种操作...

} // _guard 被 drop，自动恢复到原目录

println!("Back to: {}", env::current_dir()?.display());
// 输出: Back to: <original>
```

---

## 🔧 实用工具

### OptionFrom Trait

方便的类型转换：

```rust
use orion_variate::opt::OptionFrom;
use std::path::PathBuf;

// &str -> Option<String>
let s: Option<String> = "hello".to_opt();

// String -> Option<String>
let s: Option<String> = "world".to_string().to_opt();

// &str -> Option<PathBuf>
let p: Option<PathBuf> = "/path/to/file".to_opt();
```

### ValueConstraint - 值约束

```rust
use orion_variate::{ValueConstraint, ValueScope};

// 锁定值（不允许修改）
let locked = ValueConstraint::Locked;

// 范围约束
let scope = ValueConstraint::scope(1, 100);  // 值必须在 [1, 100] 范围内
```

---

## 📝 完整示例

### 示例 1：配置管理系统

```rust
use orion_variate::*;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 设置环境
    env::set_var("APP_ENV", "production");
    env::set_var("DB_HOST", "db.example.com");

    // 2. 定义配置变量
    let config_vars = vec![
        VarDefinition::from(("app_name", "MyService"))
            .with_mut_immutable()
            .with_desc(Some("应用名称".into())),

        VarDefinition::from(("version", "1.0.0"))
            .with_mut_immutable(),

        VarDefinition::from(("env", "${APP_ENV}"))
            .with_mut_system(),

        VarDefinition::from(("db_url", "postgres://${DB_HOST}:5432/mydb"))
            .with_mut_system(),

        VarDefinition::from(("max_connections", 100u64))
            .with_mut_module(),
    ];

    // 3. 创建变量集合
    let collection = VarCollection::define(config_vars);

    // 4. 导出并评估环境变量
    let env_dict = EnvDict::new();
    let config_dict = collection.value_dict().env_eval(&env_dict);

    // 5. 使用配置
    println!("App: {}", config_dict.get_case_insensitive("app_name").unwrap());
    println!("Environment: {}", config_dict.get_case_insensitive("env").unwrap());
    println!("Database: {}", config_dict.get_case_insensitive("db_url").unwrap());

    // 6. 序列化配置
    let yaml = serde_yaml::to_string(&collection)?;
    println!("\n配置 YAML:\n{}", yaml);

    Ok(())
}
```

### 示例 2：多来源配置合并

```rust
use orion_variate::*;

fn load_config() -> Result<OriginDict, Box<dyn std::error::Error>> {
    // 1. 加载默认配置
    let mut defaults = OriginDict::new();
    defaults.insert("timeout", ValueType::from(30u64));
    defaults.insert("retries", ValueType::from(3u64));
    defaults.set_source("defaults");

    // 2. 加载用户配置
    let mut user_config = OriginDict::new();
    user_config.insert("timeout", ValueType::from(60u64));  // 覆盖默认值
    user_config.insert("api_key", ValueType::from("user-secret"));
    user_config.set_source("user_config.yaml");

    // 3. 合并配置
    defaults.merge(&user_config);

    // 4. 检查来源
    if let Some(val) = defaults.get_case_insensitive("timeout") {
        println!("timeout: {} (from: {:?})", val.value(), val.origin());
        // 输出: timeout: 60 (from: Some("user_config.yaml"))
    }

    Ok(defaults)
}
```

---

## 📚 API 兼容性说明

以下是已更名但保留了兼容别名的 API：

| 旧名称 | 新名称 | 状态 |
|--------|--------|------|
| `WorkDir` | `CwdGuard` | ✅ 兼容别名可用 |
| `ucase_get()` | `get_case_insensitive()` | ✅ 兼容别名可用 |
| `type_name()` | `variant_name()` | ✅ 兼容别名可用 |
| `update_by_str()` | `update_from_str()` | ✅ 兼容别名可用 |
| `EnvEvalable` | `EnvEvaluable` | ✅ 兼容别名可用 |
| `find_project_define()` | `find_project_root()` | ✅ 兼容别名可用 |
| `find_project_define_base()` | `find_project_root_from()` | ✅ 兼容别名可用 |

**建议：** 尽快迁移到新名称，旧名称将在未来版本中移除。

---

## 🧪 测试支持

所有主要类型都支持完整的序列化/反序列化测试：

```rust
#[cfg(test)]
mod tests {
    use orion_variate::*;

    #[test]
    fn test_round_trip() {
        let mut dict = ValueDict::new();
        dict.insert("key", ValueType::from("value"));

        // JSON 往返
        let json = serde_json::to_string(&dict).unwrap();
        let loaded: ValueDict = serde_json::from_str(&json).unwrap();
        assert_eq!(dict, loaded);

        // YAML 往返
        let yaml = serde_yaml::to_string(&dict).unwrap();
        let loaded: ValueDict = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(dict, loaded);
    }
}
```

---

## 🔗 相关链接

- **GitHub**: https://github.com/galaxy-sec/orion-variate
- **crates.io**: https://crates.io/crates/orion-variate
- **文档**: https://docs.rs/orion-variate
- **License**: MIT

---

## 版本信息

当前文档对应版本: v0.10.2

更新日志请参考: [CHANGELOG.md](../CHANGELOG.md)
