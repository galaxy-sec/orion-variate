use derive_more::From;

use super::{CommentFmt, LabelCoverter, TplResult};
#[derive(Debug, Clone, PartialEq, From)]
pub enum CustTmplLabel {
    None,
    Setting(LabelCoverter),
}

impl CustTmplLabel {
    pub fn convert(&self, cfmt: &CommentFmt, code: String) -> TplResult<String> {
        match self {
            CustTmplLabel::None => Ok(code),
            CustTmplLabel::Setting(t) => t.convert(cfmt, code),
        }
    }
    pub fn restore(&self, code: String) -> TplResult<String> {
        match self {
            CustTmplLabel::None => Ok(code),
            CustTmplLabel::Setting(t) => t.restore(code),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tpl::{comment::CommentFmt, covert::LabelCoverter};

    // CustTmplLabel 基础功能测试
    mod basic_tests {
        use super::*;

        #[test]
        fn test_cust_tmpl_label_none_creation() {
            let label = CustTmplLabel::None;
            match label {
                CustTmplLabel::None => {}
                CustTmplLabel::Setting(_) => panic!("Expected None variant"),
            }
        }

        #[test]
        fn test_cust_tmpl_label_setting_creation() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            match label {
                CustTmplLabel::None => panic!("Expected Setting variant"),
                CustTmplLabel::Setting(_) => {}
            }
        }

        #[test]
        fn test_cust_tmpl_label_from_label_coverter() {
            let converter = LabelCoverter::new(
                ("${", "}"),  // orion labels
                ("<%", "%>"), // target labels
            );
            let label: CustTmplLabel = converter.into();
            match label {
                CustTmplLabel::None => panic!("Expected Setting variant"),
                CustTmplLabel::Setting(_) => {}
            }
        }
    }

    // None 变体的测试
    mod none_variant_tests {
        use super::*;

        #[test]
        fn test_none_convert_passthrough() {
            let label = CustTmplLabel::None;
            let cfmt = CommentFmt::UnNeed;
            let code = "Hello World".to_string();

            let result = label.convert(&cfmt, code.clone()).unwrap();
            assert_eq!(result, code);
        }

        #[test]
        fn test_none_restore_passthrough() {
            let label = CustTmplLabel::None;
            let code = "Hello World".to_string();

            let result = label.restore(code.clone()).unwrap();
            assert_eq!(result, code);
        }

        #[test]
        fn test_none_convert_with_cstyle_comments() {
            let label = CustTmplLabel::None;
            let cfmt = CommentFmt::CStyle;
            let code = "int x = 5; // comment\n".to_string();

            let result = label.convert(&cfmt, code.clone()).unwrap();
            assert_eq!(result, code);
        }

        #[test]
        fn test_none_convert_with_yml_comments() {
            let label = CustTmplLabel::None;
            let cfmt = CommentFmt::Yml;
            let code = "key: value # comment\n".to_string();

            let result = label.convert(&cfmt, code.clone()).unwrap();
            assert_eq!(result, code);
        }

        #[test]
        fn test_none_convert_with_complex_code() {
            let label = CustTmplLabel::None;
            let cfmt = CommentFmt::CStyle;
            let code = r#"
function test() {
    /* multi-line comment */
    let x = 5; // single line comment
    return x;
}
            "#
            .to_string();

            let result = label.convert(&cfmt, code.clone()).unwrap();
            assert_eq!(result, code);
        }
    }

    // Setting 变体的测试
    mod setting_variant_tests {
        use super::*;

        fn create_test_converter() -> LabelCoverter {
            LabelCoverter::new(
                ("{{", "}}"), // orion template labels
                ("{%", "%}"), // target template labels
            )
        }

        #[test]
        fn test_setting_convert_simple_labels() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let code = "Hello {{ name }} World".to_string();

            let result = label.convert(&cfmt, code).unwrap();
            assert_eq!(result, "Hello {% name %} World");
        }

        #[test]
        fn test_setting_restore_simple_labels() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let code = "Hello {% name %} World".to_string();

            let result = label.restore(code).unwrap();
            assert_eq!(result, "Hello {{ name }} World");
        }

        #[test]
        fn test_setting_convert_multiple_labels() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let code = "{{ title }}: {{ content }}".to_string();

            let result = label.convert(&cfmt, code).unwrap();
            assert_eq!(result, "{% title %}: {% content %}");
        }

        #[test]
        fn test_setting_restore_multiple_labels() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let code = "{% title %}: {% content %}".to_string();

            let result = label.restore(code).unwrap();
            assert_eq!(result, "{{ title }}: {{ content }}");
        }

        #[test]
        fn test_setting_convert_multiline_code() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let code = r#"
{{ header }}
{{ content }}
{{ footer }}
            "#
            .to_string();

            let result = label.convert(&cfmt, code).unwrap();
            let expected = r#"
{% header %}
{% content %}
{% footer %}
            "#;
            assert_eq!(result, expected);
        }

        #[test]
        fn test_setting_convert_with_cstyle_comments() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::CStyle;
            let code = r#"
/* This is a comment */
{{ variable }} /* another comment */
int x = 5;
            "#
            .to_string();

            let result = label.convert(&cfmt, code).unwrap();
            // Comments should be removed, labels converted
            assert!(result.contains("{% variable %}"));
            assert!(!result.contains("/*"));
            assert!(!result.contains("comment"));
        }

        #[test]
        fn test_setting_convert_with_yml_comments() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::Yml;
            let code = r#"
key: {{ value }} # comment
other: {{ other_value }}
            "#
            .to_string();

            let result = label.convert(&cfmt, code).unwrap();
            // Comments should be removed, labels converted
            assert!(result.contains("{% value %}"));
            assert!(!result.contains("# comment"));
        }

        #[test]
        fn test_setting_convert_empty_code() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let code = "".to_string();

            let result = label.convert(&cfmt, code).unwrap();
            assert_eq!(result, "");
        }

        #[test]
        fn test_setting_restore_empty_code() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let code = "".to_string();

            let result = label.restore(code).unwrap();
            assert_eq!(result, "");
        }

        #[test]
        fn test_setting_convert_no_labels() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let code = "Just plain text".to_string();

            let result = label.convert(&cfmt, code).unwrap();
            assert_eq!(result, "Just plain text");
        }

        #[test]
        fn test_setting_restore_no_labels() {
            let converter = create_test_converter();
            let label = CustTmplLabel::Setting(converter);
            let code = "Just plain text".to_string();

            let result = label.restore(code).unwrap();
            assert_eq!(result, "Just plain text");
        }
    }

    // 边界情况和错误处理测试
    mod edge_case_tests {
        use super::*;

        #[test]
        fn test_convert_very_long_code() {
            let converter = LabelCoverter::new(
                ("${", "}"),  // orion labels
                ("<%", "%>"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;

            // 创建一个长字符串
            let long_content = "Hello ${name} ".repeat(100);
            let code = long_content.to_string();

            let result = label.convert(&cfmt, code).unwrap();

            // Basic check that conversion worked
            assert!(!result.is_empty());
            assert!(!result.contains("${"));
        }

        #[test]
        fn test_convert_special_characters() {
            let converter = LabelCoverter::new(
                ("[[", "]]"), // orion labels
                ("((", "))"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let code = "Test [[var]] with 中文 and 🚀 emojis".to_string();

            let result = label.convert(&cfmt, code).unwrap();
            assert_eq!(result, "Test ((var)) with 中文 and 🚀 emojis");
        }

        #[test]
        fn test_convert_nested_label_patterns() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let code = "{{ outer {{ inner }} }}".to_string();

            let result = label.convert(&cfmt, code).unwrap();
            // Should convert from outside to inside
            assert_eq!(result, "{% outer {% inner %} %}");
        }

        #[test]
        fn test_convert_with_line_breaks() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let code = "{{ name }}\n{{ value }}".to_string();

            let result = label.convert(&cfmt, code).unwrap();
            assert_eq!(result, "{% name %}\n{% value %}");
        }

        #[test]
        fn test_restore_with_line_breaks() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let code = "{% name %}\n{% value %}".to_string();

            let result = label.restore(code).unwrap();
            assert_eq!(result, "{{ name }}\n{{ value }}");
        }
    }

    // Trait 实现测试
    mod trait_tests {
        use super::*;

        #[test]
        fn test_cust_tmpl_label_debug_none() {
            let label = CustTmplLabel::None;
            let debug_str = format!("{label:?}");
            assert_eq!(debug_str, "None");
        }

        #[test]
        fn test_cust_tmpl_label_debug_setting() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let debug_str = format!("{label:?}");
            assert!(debug_str.contains("Setting"));
            assert!(debug_str.contains("LabelCoverter"));
        }

        #[test]
        fn test_cust_tmpl_label_clone_none() {
            let label = CustTmplLabel::None;
            let cloned = label.clone();
            match cloned {
                CustTmplLabel::None => {}
                _ => panic!("Expected None variant"),
            }
        }

        #[test]
        fn test_cust_tmpl_label_clone_setting() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter.clone());
            let cloned = label.clone();

            match cloned {
                CustTmplLabel::Setting(_) => {}
                _ => panic!("Expected Setting variant"),
            }
        }

        #[test]
        fn test_cust_tmpl_label_partial_eq_none() {
            let label1 = CustTmplLabel::None;
            let label2 = CustTmplLabel::None;
            assert_eq!(label1, label2);
        }

        #[test]
        fn test_cust_tmpl_label_partial_eq_setting() {
            let converter1 = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let converter2 = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label1 = CustTmplLabel::Setting(converter1);
            let label2 = CustTmplLabel::Setting(converter2);
            assert_eq!(label1, label2);
        }

        #[test]
        fn test_cust_tmpl_label_partial_eq_none_setting() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label1 = CustTmplLabel::None;
            let label2 = CustTmplLabel::Setting(converter);
            assert_ne!(label1, label2);
        }

        #[test]
        fn test_cust_tmpl_label_partial_eq_different_settings() {
            let converter1 = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let converter2 = LabelCoverter::new(
                ("${", "}"),  // orion labels
                ("<%", "%>"), // target labels
            );
            let label1 = CustTmplLabel::Setting(converter1);
            let label2 = CustTmplLabel::Setting(converter2);
            assert_ne!(label1, label2);
        }

        #[test]
        fn test_cust_tmpl_label_from_label_coverter() {
            let converter = LabelCoverter::new(
                ("[[", "]]"), // orion labels
                ("((", "))"), // target labels
            );
            let label: CustTmplLabel = converter.clone().into();

            match label {
                CustTmplLabel::Setting(c) => {
                    // Verify the converter is the same
                    // Note: We can't directly compare LabelCoverter due to private fields,
                    // but we can test its behavior
                    let cfmt = CommentFmt::UnNeed;
                    let test_code = "[[ test ]]".to_string();
                    let result = c.convert(&cfmt, test_code).unwrap();
                    assert_eq!(result, "(( test ))");
                }
                _ => panic!("Expected Setting variant"),
            }
        }
    }

    // 集成测试
    mod integration_tests {
        use super::*;

        #[test]
        fn test_roundtrip_conversion() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let original_code = r#"
function {{ name }}() {
    let {{ var }} = "{{ value }}";
    return {{ var }};
}
            "#
            .to_string();

            // Convert orion to target format
            let converted = label.convert(&cfmt, original_code.clone()).unwrap();

            // Restore back to orion format
            let restored = label.restore(converted).unwrap();

            // Should be back to original (note: comment removal may affect formatting)
            // Check that labels are restored correctly and content is preserved
            assert!(restored.contains("{{ name }}"));
            assert!(restored.contains("{{ var }}"));
            assert!(restored.contains("{{ value }}"));
            assert!(restored.contains("function"));
            assert!(restored.contains("return"));
        }

        #[test]
        fn test_roundtrip_with_comments_simple() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::CStyle;
            let original_code = "/* comment */ {{ name }}".to_string();

            // Convert (comments will be removed)
            let converted = label.convert(&cfmt, original_code.clone()).unwrap();

            // Restore (no comments to restore)
            let restored = label.restore(converted).unwrap();

            // Should have original labels, no comments
            // Note: comment removal might affect spacing, so we trim
            assert_eq!(restored.trim(), "{{ name }}");
        }

        #[test]
        fn test_roundtrip_with_comments() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::CStyle;
            let original_code = r#"
/* Function header */
function {{ name }}() {
    /* Variable declaration */
    let {{ var }} = "{{ value }}"; /* inline comment */
    return {{ var }};
}
            "#
            .to_string();

            // Convert (comments will be removed)
            let converted = label.convert(&cfmt, original_code.clone()).unwrap();

            // Restore (no comments to restore)
            let restored = label.restore(converted).unwrap();

            // Should be original code without comments, with original labels
            // Just verify that the core content is preserved
            assert!(restored.contains("{{ name }}"));
            assert!(restored.contains("{{ var }}"));
            assert!(restored.contains("{{ value }}"));
            assert!(restored.contains("function"));
            assert!(restored.contains("return"));
            assert!(!restored.contains("/*"));
            assert!(!restored.contains("comment"));
        }

        #[test]
        fn test_multiple_labels_roundtrip() {
            let converter = LabelCoverter::new(
                ("${", "}"),  // orion labels
                ("<%", "%>"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);
            let cfmt = CommentFmt::UnNeed;
            let original = "${title}: ${content} | ${date}".to_string();

            let converted = label.convert(&cfmt, original.clone()).unwrap();
            let restored = label.restore(converted).unwrap();

            assert_eq!(restored, original);
        }

        #[test]
        fn test_different_comment_formats() {
            let converter = LabelCoverter::new(
                ("{{", "}}"), // orion labels
                ("{%", "%}"), // target labels
            );
            let label = CustTmplLabel::Setting(converter);

            let test_cases = vec![
                (CommentFmt::UnNeed, "Simple {{ code }}"),
                (CommentFmt::CStyle, "/* comment */ {{ code }}"),
                (CommentFmt::Yml, "key: {{ value }} # comment"),
            ];

            for (cfmt, code) in test_cases {
                let code_string = code.to_string();

                // All operations should succeed without panicking
                let result = label.convert(&cfmt, code_string.clone());
                assert!(
                    result.is_ok(),
                    "Conversion failed for {cfmt:?} with code: {code}",
                );

                let result_str = result.unwrap();

                // For UnNeed format, labels should be converted
                if cfmt == CommentFmt::UnNeed {
                    assert!(
                        result_str.contains("{%"),
                        "UnNeed format should contain converted labels"
                    );
                    assert!(
                        !result_str.contains("{{"),
                        "UnNeed format should not contain original labels"
                    );
                }
                // For comment formats, just ensure operation succeeded
                // The exact behavior depends on comment removal implementation
            }
        }
    }
}
