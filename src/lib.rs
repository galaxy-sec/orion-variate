//! 通用工具库

pub mod opt;
pub mod vars;

// Re-export commonly used items from `vars` at the crate root for ergonomic imports
#[deprecated]
pub use vars::EnvEvalable;
pub use vars::{
    CwdGuard, EnvChecker, EnvDict, EnvEvaluable, Mutability, OriginDict, OriginValue, UpperKey,
    ValueConstraint, ValueDict, ValueObj, ValueType, ValueVec, VarCollection, VarDefinition,
    VarToValue, extract_env_var_names, find_project_define, find_project_define_base,
    find_project_root, find_project_root_from, setup_start_env_vars,
};
