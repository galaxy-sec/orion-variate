# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.8] - 2026-01-11

### Fixed

- **Critical bug fix**: Fixed `env_eval` incorrectly parsing environment variable values containing `://` (e.g., URLs, database connection strings)
  - The parser previously treated any `:` as a default value separator, even when it appeared outside the `${}` syntax
  - This caused incorrect replacement when variable values contained protocols like `postgresql://`, `https://`, etc.
  - Example of the bug:
    ```rust
    let mut dict = EnvDict::new();
    dict.insert("DB_URL", "postgresql://localhost/mydb");
    "${DB_URL}".env_eval(&dict); // Was broken, now fixed
    ```
  - The fix ensures `:` is only treated as a separator when it appears within the `${}` brackets
  - Added comprehensive tests to prevent regression

### Added

- New test case `test_url_with_protocol` for verifying URL handling in environment variable values
- New test case `test_url_with_protocol_complex` for testing multiple variables with URL values
- Example program `examples/test_multiline_json.rs` demonstrating JSON configuration with URL environment variables

## [0.10.7] - 2026-01-11

### Added

- 新增 `EnvChecker` trait，用于环境变量检查与提取
  - `needs_env_eval(&self) -> bool` - 检查是否包含环境变量占位符（如 `${VAR}`）
  - `list_env_vars(&self) -> Vec<String>` - 提取所有环境变量名称
- 为以下类型实现 `EnvChecker` trait：
  - `String` 和 `&str` - 直接检查和提取
  - `Option<String>` 和 `Option<&str>` - 递归处理 Some 值
  - `ValueType` - 智能递归处理所有变体（String、Obj、List）
- 新增 `extract_env_var_names()` 公共辅助函数，从字符串中提取环境变量名
- `ValueType::env_eval()` 现在支持递归求值（Obj、List、String）
- 在 crate 根导出 `EnvChecker` trait 和 `extract_env_var_names` 函数
- 新增使用文档：
  - `docs/env_checker_usage.md` - 完整使用指南
  - `docs/env_checker_summary.md` - 功能快速参考
  - `docs/env_checker_str_support.md` - &str 支持说明
  - `docs/changelog_guide.md` - CHANGELOG 维护指南
- 新增 12 个测试用例，覆盖所有 EnvChecker 功能（总计 114 个测试）

```rust
// 可以从 vars 模块导入
use orion_variate::vars::EnvChecker;
// 也可以直接从 crate 根导入
use orion_variate::{EnvChecker, extract_env_var_names};

// 检查是否需要求值
let path: &str = "${HOME}/bin";
assert!(path.needs_env_eval());

// 提取所有环境变量名
let config = "Server: ${HOST}:${PORT}";
assert_eq!(config.list_env_vars(), vec!["HOST", "PORT"]);

// 使用辅助函数
let vars = extract_env_var_names("${VAR1}/${VAR2}");
assert_eq!(vars, vec!["VAR1", "VAR2"]);

// 递归处理复杂结构
let complex = ValueType::List(vec![
    ValueType::String("${VAR1}".to_string()),
    ValueType::Obj(obj_with_vars),
]);
let all_vars = complex.list_env_vars(); // 自动收集所有嵌套变量
```

### Changed

- `ValueType::env_eval()` 从仅处理 String 变体改为递归处理 Obj 和 List 变体
- 在 crate 根（lib.rs）re-export 常用类型与函数，简化 `use` 路径
- 统一 VarCollection 内部命名为 `system_vars/module_vars`，修正文档与注释
- 合并策略说明更新为"后者覆盖前者"
- `Mutability::is_default` 与默认枚举保持一致（Module），序列化跳过策略更清晰
- 代码质量改进：修复所有 clippy 警告，使用更现代的 Rust 惯用法

### Deprecated

- `WorkDir` → 使用 `CwdGuard` 代替（结构体与方法保持不变；保留 `type WorkDir = CwdGuard` 别名）
- `ValueDict::ucase_get` → 使用 `get_case_insensitive` 代替（保留旧方法为 `#[deprecated]` 别名）
- `ValueType::update_by_str` → 使用 `update_from_str` 代替（保留旧方法别名）
- `ValueType::type_name` → 使用 `variant_name` 代替（保留旧方法别名）
- `EnvEvalable` → 使用 `EnvEvaluable` 代替（并同时 re-export 新旧名称）
- `find_project_define_base` → 使用 `find_project_root_from` 代替（原函数仍可用）
- `find_project_define` → 使用 `find_project_root` 代替（原函数仍可用）

**迁移建议**: 建议尽快切换到新名称；旧名称将于后续次要版本中标记为不可用后移除。
