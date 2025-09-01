use std::env;

use winnow::{Parser, token::take_until};

use super::EnvDict;

fn until_beg<'i>(s: &mut &'i str) -> winnow::Result<&'i str> {
    let data = take_until(0.., "${").parse_next(s)?;
    "${".parse_next(s)?;
    Ok(data)
}
fn until_name<'i>(s: &mut &'i str) -> winnow::Result<&'i str> {
    let data = take_until(0.., ":").parse_next(s)?;
    ":".parse_next(s)?;
    Ok(data)
}
fn until_name_default<'i>(s: &mut &'i str) -> winnow::Result<Vec<&'i str>> {
    let mut data: Vec<&str> = Vec::new();
    if let Ok(ok_data) = until_name.parse_next(s) {
        data.push(ok_data)
    }
    let last = take_until(0.., "}").parse_next(s)?;
    "}".parse_next(s)?;
    data.push(last);
    Ok(data)
}

pub fn expand_env_vars(dict: &EnvDict, input: &str) -> String {
    let mut out = String::new();
    let mut data = input;
    while !data.is_empty() {
        match until_beg.parse_next(&mut data) {
            Ok(ok_data) => {
                out.push_str(ok_data);
            }
            Err(_e) => {
                out.push_str(data);
                return out;
            }
        }
        match until_name_default.parse_next(&mut data) {
            Ok(vecs) => match vecs.len() {
                1 => {
                    if let Some(found) = dict.get(vecs[0]) {
                        out.push_str(found.to_string().as_str());
                    } else if let Ok(found) = env::var(vecs[0]) {
                        out.push_str(found.as_str());
                    } else {
                        out.push_str(format!("${{{}}}", vecs[0]).as_str());
                    }
                }
                2 => {
                    if let Some(found) = dict.get(vecs[0]) {
                        out.push_str(found.to_string().as_str());
                    } else if let Ok(found) = env::var(vecs[0]) {
                        out.push_str(found.as_str());
                    } else {
                        out.push_str(vecs[1]);
                    }
                }
                _ => {
                    panic!()
                }
            },
            Err(_) => {
                out.push_str("${");
                out.push_str(data);
                return out;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{
        tools::get_repo_name,
        vars::{EnvDict, ValueType, env_eval::expand_env_vars},
    };

    #[test]
    fn test_get_last_segment() {
        // 测试HTTP URL
        assert_eq!(
            get_repo_name("https://github.com/user/repo.git"),
            Some("repo.git".to_string())
        );

        // 测试HTTPS URL
        assert_eq!(
            get_repo_name("https://github.com/user/repo"),
            Some("repo".to_string())
        );

        // 测试SSH格式Git地址
        assert_eq!(
            get_repo_name("git@github.com:user/repo.git"),
            Some("repo.git".to_string())
        );

        // 测试SSH格式不带.git后缀
        assert_eq!(
            get_repo_name("git@gitlab.com:group/subgroup/repo"),
            Some("repo".to_string())
        );

        // 测试无效URL
        assert_eq!(get_repo_name("not_a_url"), None);
    }

    #[test]
    fn test_basic_expansion() {
        unsafe { env::set_var("HOME", "/home/user") };
        assert_eq!(
            expand_env_vars(&EnvDict::default(), "${HOME}/bin"),
            "/home/user/bin"
        );
    }

    #[test]
    fn test_multiple_variables() {
        unsafe { env::set_var("USER", "john") };
        unsafe { env::set_var("APP", "myapp") };
        let mut dict = EnvDict::new();
        dict.insert("USER", ValueType::from("galaxy"));
        dict.insert("APP", ValueType::from("galaxy"));
        assert_eq!(
            expand_env_vars(&EnvDict::default(), "/opt/${APP}/bin/${USER}"),
            "/opt/myapp/bin/john"
        );
        assert_eq!(
            expand_env_vars(&dict, "/opt/${APP}/bin/${USER}"),
            "/opt/galaxy/bin/galaxy"
        );
    }

    #[test]
    fn test_undefined_variable() {
        unsafe { env::remove_var("UNDEFINED_VAR") };
        assert_eq!(
            expand_env_vars(&EnvDict::default(), "Path: ${UNDEFINED_VAR}/data"),
            "Path: ${UNDEFINED_VAR}/data"
        );
    }

    #[test]
    fn test_nested_braces() {
        unsafe { env::set_var("VAR", "value") };
        assert_eq!(expand_env_vars(&EnvDict::default(), "${VAR}}"), "value}");
        assert_eq!(expand_env_vars(&EnvDict::default(), "${VAR}}}"), "value}}");
    }

    #[test]
    fn test_unclosed_brace() {
        unsafe { env::set_var("HOME", "/home/user") };
        assert_eq!(expand_env_vars(&EnvDict::default(), "${HOME"), "${HOME");
        assert_eq!(
            expand_env_vars(&EnvDict::default(), "${HOME${USER"),
            "${HOME${USER"
        );
    }

    #[test]
    fn test_empty_variable_name() {
        assert_eq!(expand_env_vars(&EnvDict::default(), "${}"), "${}");
    }

    #[test]
    fn test_special_characters() {
        unsafe { env::set_var("VAR_WITH_UNDERSCORE", "ok") };
        assert_eq!(
            expand_env_vars(&EnvDict::default(), "${VAR_WITH_UNDERSCORE}"),
            "ok"
        );
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(expand_env_vars(&EnvDict::default(), ""), "");
        assert_eq!(
            expand_env_vars(&EnvDict::default(), "no variables"),
            "no variables"
        );
        assert_eq!(expand_env_vars(&EnvDict::default(), "$"), "$");
        assert_eq!(expand_env_vars(&EnvDict::default(), "${"), "${");
        assert_eq!(expand_env_vars(&EnvDict::default(), "}"), "}");
        assert_eq!(expand_env_vars(&EnvDict::default(), "${}"), "${}");
    }

    #[test]
    fn test_consecutive_variables() {
        unsafe { env::set_var("A", "1") };
        unsafe { env::set_var("B", "2") };
        assert_eq!(expand_env_vars(&EnvDict::default(), "${A}${B}"), "12");
    }
    #[test]
    fn test_default_value() {
        unsafe { env::remove_var("DEFAULT_TEST_VAR") };
        // 确保环境变量真的被移除了
        assert!(env::var("DEFAULT_TEST_VAR").is_err());
        assert_eq!(
            expand_env_vars(&EnvDict::default(), "Hello ${DEFAULT_TEST_VAR:World}"),
            "Hello World"
        );
    }

    #[test]
    fn test_default_value_with_existing_variable() {
        unsafe { env::set_var("DEFAULT_TEST_VAR1", "Galaxy") };
        assert_eq!(
            expand_env_vars(&EnvDict::default(), "Hello ${DEFAULT_TEST_VAR1:World}"),
            "Hello Galaxy"
        );
    }

    #[test]
    fn test_default_value_with_dict() {
        let mut dict = EnvDict::new();
        dict.insert("DEFAULT_TEST_VAR", ValueType::from("DictValue"));
        unsafe { env::set_var("DEFAULT_TEST_VAR", "EnvValue") };
        assert_eq!(
            expand_env_vars(&dict, "Hello ${DEFAULT_TEST_VAR:World}"),
            "Hello DictValue"
        );
    }

    #[test]
    fn test_default_value_with_dict_but_no_env() {
        let dict = EnvDict::new();
        unsafe { env::remove_var("DEFAULT_TEST_VAR") };
        // 确保环境变量真的被移除了
        assert!(env::var("DEFAULT_TEST_VAR").is_err());
        assert_eq!(
            expand_env_vars(&dict, "Hello ${DEFAULT_TEST_VAR:World}"),
            "Hello World"
        );
    }
}
