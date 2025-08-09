use std::fmt::Display;

use winnow::error::{ContextError, ErrMode};

pub struct WinnowErrorEx(ErrMode<ContextError>);

impl Display for WinnowErrorEx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut context_vec: Vec<String> = match &self.0 {
            ErrMode::Incomplete(_) => {
                write!(f, "Incomplete input:",)?;
                Vec::new()
            }
            ErrMode::Backtrack(err) => {
                write!(f, "backtrack : ")?;
                if let Some(cause) = err.cause() {
                    write!(f, "cause: {cause}")?;
                }
                collect_context(err)
            }
            ErrMode::Cut(err) => {
                write!(f, "cut: ")?;
                if let Some(cause) = err.cause() {
                    write!(f, "cause: {cause}")?;
                }
                collect_context(err)
            }
        };
        context_vec.reverse();
        writeln!(f, "parse context:",)?;
        for context in context_vec {
            write!(f, "{context}::",)?;
        }
        Ok(())
    }
}

fn collect_context(err: &ContextError) -> Vec<String> {
    let mut context_vec = Vec::new();
    let current = err;

    for context in current.context() {
        match context {
            winnow::error::StrContext::Label(value) => {
                context_vec.push(value.to_string());
            }
            winnow::error::StrContext::Expected(value) => {
                context_vec.push(value.to_string());
            }
            _ => {}
        }
    }
    context_vec
}
impl From<ErrMode<ContextError>> for WinnowErrorEx {
    fn from(err: ErrMode<ContextError>) -> Self {
        WinnowErrorEx(err)
    }
}
pub fn err_code_prompt(code: &str) -> String {
    let take_len = if code.len() > 200 { 200 } else { code.len() };
    if let Some((left, _right)) = code.split_at_checked(take_len) {
        return format!("{left}...",);
    }
    "".to_string()
}

use derive_more::From;
use orion_error::{ErrorCode, StructError, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;
#[derive(Clone, Debug, Serialize, PartialEq, Error, From)]
pub enum TplReason {
    #[error("brief:{0}")]
    Brief(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

impl ErrorCode for TplReason {
    fn error_code(&self) -> i32 {
        match self {
            TplReason::Brief(_) => 500,
            TplReason::Uvs(r) => r.error_code(),
        }
    }
}

pub type TplResult<T> = Result<T, StructError<TplReason>>;
pub type TplError = StructError<TplReason>;

#[cfg(test)]
mod tests {
    use super::*;
    use orion_error::StructError;
    use winnow::error::{ContextError, ErrMode, Needed};

    #[test]
    fn test_winnow_error_ex_display_incomplete() {
        let err_mode = ErrMode::Incomplete(Needed::new(5));
        let error_ex = WinnowErrorEx::from(err_mode);
        let display = format!("{error_ex}");
        assert!(display.contains("Incomplete input:"));
        assert!(display.contains("parse context:"));
    }

    #[test]
    fn test_winnow_error_ex_display_backtrack() {
        // Create a simple ContextError for testing
        let context_error = ContextError::default();
        let err_mode = ErrMode::Backtrack(context_error);
        let error_ex = WinnowErrorEx::from(err_mode);
        let display = format!("{error_ex}");
        assert!(display.contains("backtrack :"));
        assert!(display.contains("parse context:"));
    }

    #[test]
    fn test_winnow_error_ex_display_cut() {
        // Create a simple ContextError for testing
        let context_error = ContextError::default();
        let err_mode = ErrMode::Cut(context_error);
        let error_ex = WinnowErrorEx::from(err_mode);
        let display = format!("{error_ex}");
        assert!(display.contains("cut:"));
        assert!(display.contains("parse context:"));
    }

    #[test]
    fn test_collect_context_empty() {
        let context_error = ContextError::default();
        let context_vec = collect_context(&context_error);
        assert_eq!(context_vec.len(), 0);
    }

    #[test]
    fn test_from_err_mode() {
        let err_mode = ErrMode::Incomplete(Needed::new(10));
        let error_ex: WinnowErrorEx = WinnowErrorEx::from(err_mode);
        assert!(matches!(error_ex.0, ErrMode::Incomplete(_)));
    }

    #[test]
    fn test_err_code_prompt_short_string() {
        let code = "short code";
        let result = err_code_prompt(code);
        assert_eq!(result, "short code...");
    }

    #[test]
    fn test_err_code_prompt_long_string() {
        let code = "a".repeat(300);
        let result = err_code_prompt(&code);
        assert_eq!(result.len(), 203); // 200 characters + "..."
        assert!(result.ends_with("..."));
        assert_eq!(result, "a".repeat(200) + "...");
    }

    #[test]
    fn test_err_code_prompt_empty_string() {
        let code = "";
        let result = err_code_prompt(code);
        assert_eq!(result, "...");
    }

    #[test]
    fn test_err_code_prompt_exact_200_chars() {
        let code = "a".repeat(200);
        let result = err_code_prompt(&code);
        assert_eq!(result, code + "...");
    }

    #[test]
    fn test_tpl_reason_brief() {
        let reason = TplReason::Brief("test brief".to_string());
        assert_eq!(reason.error_code(), 500);
        assert_eq!(format!("{reason}",), "brief:test brief");
    }

    #[test]
    fn test_tpl_reason_uvs() {
        let brief_reason = TplReason::Brief("test".to_string());
        let uvs_reason = TplReason::Uvs(orion_error::UvsReason::core_conf("test"));

        assert_ne!(brief_reason.error_code(), uvs_reason.error_code());
    }

    #[test]
    fn test_tpl_result_ok() {
        let result: TplResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_tpl_result_err() {
        let error = StructError::new(
            TplReason::Brief("test error".to_string()),
            None,
            None,
            vec![],
        );
        let result: TplResult<i32> = Err(error);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(StructError::new(
                TplReason::Brief("test error".to_string()),
                None,
                None,
                vec![],
            ))
        );
    }

    #[test]
    fn test_tpl_error() {
        let error: TplError = StructError::new(
            TplReason::Brief("test error".to_string()),
            None,
            None,
            vec![],
        );
        assert_eq!(error.reason().error_code(), 500);
        assert_eq!(format!("{}", error.reason()), "brief:test error");
    }

    #[test]
    fn test_tpl_reason_partial_eq() {
        let reason1 = TplReason::Brief("test".to_string());
        let reason2 = TplReason::Brief("test".to_string());
        let reason3 = TplReason::Brief("different".to_string());

        assert_eq!(reason1, reason2);
        assert_ne!(reason1, reason3);
    }

    #[test]
    fn test_tpl_reason_clone() {
        let reason = TplReason::Brief("test".to_string());
        let cloned = reason.clone();
        assert_eq!(reason, cloned);
    }

    #[test]
    fn test_tpl_reason_debug() {
        let reason = TplReason::Brief("test".to_string());
        let debug_str = format!("{reason:?}");
        assert!(debug_str.contains("Brief"));
        assert!(debug_str.contains("test"));
    }
}
