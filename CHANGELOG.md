# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

API 命名与导出统一（均已提供兼容别名，建议尽快迁移）
- WorkDir → CwdGuard（结构体与方法保持不变；保留 `type WorkDir = CwdGuard`）
- ValueDict::ucase_get → get_case_insensitive（保留旧方法为 `#[deprecated]` 别名）
- ValueType::update_by_str → update_from_str；type_name → variant_name（保留旧方法别名）
- EnvEvalable → EnvEvaluable（并同时 re-export 新旧名称）
- 为项目根查找提供更直观别名：
  - find_project_define_base → find_project_root_from（原函数仍可用）
  - find_project_define → find_project_root（原函数仍可用）

行为与一致性改进
- 统一 VarCollection 内部命名：`system_vars/module_vars`，修正文档与注释；合并策略说明为“后者覆盖前者”。
- Mutability::is_default 与默认枚举保持一致（Module），序列化跳过策略更清晰。

对外导出
- 在 crate 根（lib.rs）re-export 常用类型与函数，简化 `use` 路径。

迁移建议
- 立即切换到新名称；旧名称将于后续次要版本中标记为不可用后移除。

