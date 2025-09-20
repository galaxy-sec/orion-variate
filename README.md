# Orion Variate

[![CI](https://github.com/galaxy-sec/orion-variate/workflows/CI/badge.svg)](https://github.com/galaxy-sec/orion-variate/actions)
[![Coverage Status](https://codecov.io/gh/galaxy-sec/orion-variate/branch/main/graph/badge.svg)](https://codecov.io/gh/galaxy-sec/orion-variate)
[![crates.io](https://img.shields.io/crates/v/orion-variate.svg)](https://crates.io/crates/orion-variate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个 Rust 库，提供变量解析与扩展（大小写不敏感字典、环境变量插值）、值类型解析、以及便捷的工作目录守卫等工具。

## 快速开始

```rust
use orion_variate::{ValueDict, ValueType, CwdGuard};

// 1) 大小写不敏感的字典访问
let mut dict = ValueDict::new();
dict.insert("Host", ValueType::from("example.com"));
assert_eq!(dict.get_case_insensitive("HOST").unwrap().to_string(), "example.com");

// 2) 工作目录守卫（RAII）
let _guard = CwdGuard::change(".")?; // Drop 时自动恢复
```

更多类型与工具可直接从 crate 根导入：

```rust
use orion_variate::{VarCollection, VarDefinition, ValueType, EnvDict};
```

## 命名更新（重要）
- WorkDir → CwdGuard（已提供兼容别名 `WorkDir`）
- ValueDict::ucase_get → get_case_insensitive（保留兼容）
- ValueType::update_by_str → update_from_str；type_name → variant_name（保留兼容）
- EnvEvalable → EnvEvaluable（同时导出新旧别名）
- 提供更直观的项目根查询别名：find_project_root(_from)（原始函数仍可用）

详见 CHANGELOG.md 获取完整列表与迁移建议。
