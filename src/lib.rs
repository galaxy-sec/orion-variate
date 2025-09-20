//! 通用工具库

pub mod opt;
pub mod vars;

// Re-export commonly used items from `vars` at the crate root for ergonomic imports
pub use vars::{
    CwdGuard, EnvDict, EnvEvalable, EnvEvaluable, Mutability, OriginDict, OriginValue, UpperKey,
    ValueConstraint, ValueDict, ValueObj, ValueType, ValueVec, VarCollection, VarDefinition,
    VarToValue, find_project_define, find_project_define_base, find_project_root,
    find_project_root_from, setup_start_env_vars,
};
