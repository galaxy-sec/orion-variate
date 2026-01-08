mod collection;
mod constraint;
mod definition;
mod dict;
mod env_eval;
mod error;
mod global;
mod origin;
mod parse;
mod types;
pub use collection::VarCollection;
pub use constraint::{ValueConstraint, ValueScope};
pub use definition::{Mutability, VarDefinition, VarToValue};
pub use dict::ValueDict;
pub use global::{
    CwdGuard, find_project_define as find_project_root,
    find_project_define_base as find_project_root_from, setup_start_env_vars,
};
pub use origin::OriginDict;
pub use origin::OriginValue;
pub use types::EnvDict;
pub use types::EnvEvaluable;
// 向后兼容别名
pub use global::find_project_define;
pub use global::find_project_define_base;
#[deprecated]
pub use types::EnvEvaluable as EnvEvalable;
pub use types::{UpperKey, ValueObj, ValueType, ValueVec};
