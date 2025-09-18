use orion_error::{ErrorOwe, ErrorWith};
use winnow::{
    ModalResult, Parser,
    ascii::{line_ending, till_line_ending},
    combinator::{fail, opt},
    error::{StrContext, StrContextValue},
    token::take_while,
};

pub struct YmlComment {}
impl YmlComment {
    pub fn remove(code: &str) -> TplResult<String> {
        remove_comment(code)
    }
}

use crate::tpl::{TplReason, TplResult};

use super::super::error::{WinnowErrorEx, err_code_prompt};
#[derive(Debug)]
pub enum YmlStatus {
    Comment,
    Code,
    StringDouble,
    StringSingle,
    // Track YAML block scalar context (| or >); `indent` is detected from the
    // first non-empty content line following the indicator and used to know
    // when the block ends.
    BlockData { indent: Option<usize> },
}
pub fn ignore_comment_line(status: &mut YmlStatus, input: &mut &str) -> ModalResult<String> {
    let mut out = String::new();
    let mut line = String::new();
    loop {
        if input.is_empty() {
            // Flush any remaining buffered content when reaching EOF
            if !line.trim().is_empty() {
                out.push_str(&line);
            }
            break;
        }
        match status {
            YmlStatus::Code => {
                let code = take_while(0.., |c| {
                    c != '"'
                        && c != '|'
                        && c != '>'
                        && c != '\''
                        && c != '#'
                        && c != '\n'
                        && c != '\r'
                })
                .parse_next(input)?;

                if opt(line_ending).parse_next(input)?.is_some() {
                    // Preserve original blank lines: always emit the current line
                    // followed by a line ending, even if it's only whitespace.
                    line += code;
                    out += line.as_str();
                    out += "\n";
                    line = String::new();
                    continue;
                }

                if opt("#").parse_next(input)?.is_some() {
                    if !code.trim().is_empty() {
                        line += code;
                    }
                    *status = YmlStatus::Comment;
                    continue;
                }
                line += code;
                if input.is_empty() {
                    // EOF with pending code but no trailing newline.
                    // Flush the remainder so the last line is not lost.
                    if !line.trim().is_empty() {
                        out.push_str(&line);
                        line.clear();
                    }
                    break;
                }
                // Block scalar start: | or > with optional chomping/indent modifiers, then line ending
                // Examples: |, |-, |+, >, >2, >-
                let mut indicator: Option<char> = None;
                if opt("|").parse_next(input)?.is_some() {
                    indicator = Some('|');
                } else if opt(">").parse_next(input)?.is_some() {
                    indicator = Some('>');
                }
                if let Some(ind) = indicator {
                    // Peek ahead to the end of line without consuming input
                    let s = *input;
                    let eol_pos = s.find(['\n', '\r']);
                    let mods = match eol_pos {
                        Some(p) => &s[..p],
                        None => s,
                    };
                    let is_valid = mods.chars().all(|ch| {
                        ch.is_ascii_whitespace() || ch == '+' || ch == '-' || ch.is_ascii_digit()
                    });
                    if is_valid {
                        // Consume the modifiers portion
                        let consume_len = mods.len();
                        line.push(ind);
                        line.push_str(mods);
                        *input = &s[consume_len..];
                        // Start of block scalar only if followed by a real line ending
                        if opt(line_ending).parse_next(input)?.is_some() {
                            line.push('\n');
                            *status = YmlStatus::BlockData { indent: None };
                            continue;
                        } else {
                            continue;
                        }
                    } else {
                        // Not a block scalar indicator, treat as plain char
                        line.push(ind);
                        continue;
                    }
                }
                // Double-quoted string
                if opt("\"").parse_next(input)?.is_some() {
                    line.push('"');
                    *status = YmlStatus::StringDouble;
                    continue;
                }
                // Single-quoted string
                if opt("\'").parse_next(input)?.is_some() {
                    line.push('\'');
                    *status = YmlStatus::StringSingle;
                    continue;
                }
                if opt("#").parse_next(input)?.is_some() {
                    *status = YmlStatus::Comment;
                    continue;
                }
                return fail.context(wn_desc("end-code")).parse_next(input);
            }
            YmlStatus::BlockData { indent } => {
                // Read one visual line of the block scalar, preserve Unicode and
                // detect block termination when indentation drops below baseline.
                let s = *input;
                if s.is_empty() {
                    continue;
                }

                // Locate end-of-line; support both "\n" and "\r\n". Normalize as "\n".
                let (line_str, eol_len) = if let Some(nl_pos) = s.find('\n') {
                    let mut end = nl_pos;
                    if nl_pos > 0 && s.as_bytes()[nl_pos - 1] == b'\r' {
                        end -= 1;
                    }
                    (&s[..end], (nl_pos + 1) - end)
                } else {
                    (s, 0)
                };

                let current_indent = line_str.chars().take_while(|c| *c == ' ').count();
                if indent.is_none() && !line_str.trim().is_empty() {
                    *indent = Some(current_indent);
                }
                let baseline = indent.unwrap_or(0);

                // If this non-empty line is less indented than baseline, block ends.
                if !line_str.trim().is_empty() && current_indent < baseline {
                    *status = YmlStatus::Code;
                    // Do not consume; let Code branch handle this line in the next loop.
                    continue;
                }

                // Consume this line and append it, keeping a normalized newline if present.
                let consume_len = line_str.len() + eol_len;
                line.push_str(line_str);
                if eol_len > 0 {
                    line.push('\n');
                }
                *input = &s[consume_len..];
            }

            YmlStatus::StringDouble => {
                // Read until an unescaped double quote, preserving Unicode correctly.
                let s = *input;
                let mut end_idx = None;
                let mut escaped = false;
                for (i, ch) in s.char_indices() {
                    match ch {
                        '\\' if !escaped => {
                            escaped = true;
                            line.push('\\');
                        }
                        '"' if !escaped => {
                            line.push('"');
                            end_idx = Some(i + ch.len_utf8());
                            break;
                        }
                        '\n' | '\r' => {
                            line.push(ch);
                            escaped = false;
                        }
                        _ => {
                            line.push(ch);
                            escaped = false;
                        }
                    }
                }
                let idx = end_idx.unwrap_or(s.len());
                *input = &s[idx..];
                *status = YmlStatus::Code;
            }
            YmlStatus::StringSingle => {
                // Read until a single quote that is not part of a doubled '' escape
                let s = *input;
                let mut chars = s.char_indices().peekable();
                let mut end_idx = None;
                while let Some((i, ch)) = chars.next() {
                    if ch == '\'' {
                        if let Some((_, next_ch)) = chars.peek()
                            && *next_ch == '\''
                        {
                            // Escaped quote: append one and skip the next
                            line.push('\'');
                            let _ = chars.next(); // consume the escape partner
                            continue;
                        }
                        // Closing quote
                        end_idx = Some(i + ch.len_utf8());
                        line.push('\'');
                        break;
                    } else {
                        line.push(ch);
                    }
                }
                let idx = end_idx.unwrap_or(s.len());
                *input = &s[idx..];
                *status = YmlStatus::Code;
            }

            YmlStatus::Comment => {
                let _ = till_line_ending
                    .context(wn_desc("comment-line"))
                    .parse_next(input)?;
                let has_eol = opt(line_ending)
                    .context(wn_desc("comment-line_ending"))
                    .parse_next(input)?
                    .is_some();
                // If this was an inline comment (there is already some code in `line`),
                // we preserve the line ending. If the line contained only a comment,
                // we drop it entirely and clear any buffer to avoid phantom blank lines.
                if has_eol {
                    if !line.trim().is_empty() {
                        line.push('\n');
                        out += &line;
                    }
                    line.clear();
                } else {
                    // No trailing EOL (EOF). Keep code if present, without adding a newline.
                    if !line.trim().is_empty() {
                        out += &line;
                    }
                    line.clear();
                }
                *status = YmlStatus::Code;
            }
        }
    }
    Ok(out)
}
#[inline(always)]
pub fn wn_desc(desc: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::Description(desc))
}

pub fn remove_comment(code: &str) -> TplResult<String> {
    let mut xcode = code;
    let pure_code = ignore_comment(&mut xcode)
        .map_err(WinnowErrorEx::from)
        .owe(TplReason::Brief("yml comment error".into()))
        .position(err_code_prompt(code))
        .want("remove comment");
    match pure_code {
        Err(e) => {
            println!("code:\n{xcode}");
            println!("{e}");
            Err(e)
        }
        Ok(o) => Ok(o),
    }
}

pub fn ignore_comment(input: &mut &str) -> ModalResult<String> {
    let mut status = YmlStatus::Code;
    let mut out = String::new();
    loop {
        if input.is_empty() {
            break;
        }
        //let mut line = till_line_ending.parse_next(input)?;
        let code = ignore_comment_line(&mut status, input)?;
        // Always append processed code; `ignore_comment_line` already
        // handles whether to keep or drop blank lines and comment-only lines.
        out += code.as_str();
    }
    Ok(out)
}

#[cfg(test)]
mod tests {

    use std::{fs::read_to_string, path::PathBuf};

    use fs_extra::file::write_all;
    use orion_error::TestAssert;

    use super::remove_comment;

    #[test]
    fn test_case1() {
        let data = r#"
hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In
        "#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
        assert!(!_codes.contains("#"));
    }

    #[test]
    fn test_case2() {
        let data = r#"
            # Ranking of 1998 home runs
            ---
            - Mark McGwire
            - Sammy Sosa
            - Ken Griffey

            # Team ranking
            ---
            - Chicago Cubs
            - St Louis Cardinals
        "#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
        assert!(!_codes.contains("#"));
    }

    #[test]
    fn test_case4() {
        let data = r#"
    ---
    hr: # 1998 hr ranking
        - Mark McGwire
        - Sammy Sosa
    rbi:
        # 1998 rbi ranking
        - Sammy Sosa
        - Ken Griffey
        "#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
        assert!(!_codes.contains("#"));
    }

    #[test]
    fn test_case5() {
        let data = r#"
    ---
    unicode: "Sosa did fine.\u263A"
    control: "\b1998\t1999\t2000\n"
    hex esc: "\x0d\x0a is \r\n"

    single: '"Howdy!" he cried.'
    quoted: ' # Not a ''comment''.'
    tie-fighter: '|\-*-/|'
        "#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
        assert!(_codes.contains("#"));
    }

    #[test]
    fn test_case6() {
        let data = r#"
    ---
    application specific tag: !something |
     The #semantics of the tag
     above may be different for
     different documents.

    # hello
    galaxy is ok

        "#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
        assert!(_codes.contains("#"));
    }

    #[test]
    fn test_case7() {
        let data = r#"
global:
    imageRegistry: ""
    ## E.g.
    ## imagePullSecrets:
    ##   - myRegistryKeySecretName
    ##
    imagePullSecrets: []
    ## Security parameters
    ##
    security:
    ## @param global.security.allowInsecureImages Allows skipping image verification
    ##
    allowInsecureImages: false
            imageRegistry: ""
        "#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
    }

    #[test]
    fn test_case8() {
        let data = r#"hello
# xxxabc"#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
    }
    //          regex: (\d+);((([0-9]+?)(\.|$)){4})
    #[test]
    fn test_case9() {
        //test this :
        //tag: !something |
        //regex

        let data = r#"regex: (\d+);((([0-9]+?)(\.|$)){4})"#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
    }
    #[test]
    fn test_case10() {
        let data = r#"
tunable:
# -- See the [property reference documentation](https://docs.redpanda.com/docs/reference/cluster-
log_segment_size_min: 16777216 # 16 mb
# -- See the property reference documentation.
log_segment_size_max: 268435456 # 256 mb
# -- See the property reference documentation.
compacted_log_segment_size: 67108864 # 64 mb
# -- See the property reference documentation.
max_compacted_log_segment_size: 536870912 # 512 mb
# -- See the property reference documentation.
kafka_connection_rate_limit: 1000
           "#;
        let _codes = remove_comment(data).assert();
        println!("{_codes}",);
    }

    #[test]
    fn test_inline_comment_eof_no_newline() {
        let data = "key: value # comment";
        let codes = remove_comment(data).assert();
        assert_eq!(codes, "key: value ");
    }

    #[test]
    fn test_crlf_line_endings() {
        let data = "a: 1\r\nb: 2 # x\r\nc: 3\r\n";
        let codes = remove_comment(data).assert();
        assert_eq!(codes, "a: 1\nb: 2 \nc: 3\n");
    }

    #[test]
    fn test_block_scalar_chomp_indicator() {
        let data = r#"key: |-
  first # not comment
  second
end: ok
"#;
        let codes = remove_comment(data).assert();
        // '#' inside block should be preserved
        assert!(codes.contains("first # not comment"));
        assert!(codes.contains("second"));
        assert!(codes.contains("key: |-\n"));
        assert!(codes.contains("end: ok"));
    }

    #[test]
    fn test_block_scalar_folded() {
        let data = r#"key: >
  line one # keep
  line two
"#;
        let codes = remove_comment(data).assert();
        assert!(codes.contains("line one # keep"));
        assert!(codes.contains("line two"));
        assert!(codes.contains("key: >\n"));
    }

    #[test]
    fn test_double_quoted_escaped_quotes() {
        let data = r#"msg: "He said \"Hi\" # not comment""#;
        let codes = remove_comment(data).assert();
        // '#' is inside quotes; should not be treated as a comment
        assert!(codes.contains("# not comment"));
    }
    #[test]
    fn test_file_case1() {
        let base_path = PathBuf::from("./tests/data/yml");
        std::fs::create_dir_all(&base_path).assert();

        let val_file = base_path.join("values.yaml");
        let data = r#"
        tunable:
        # -- See the [property reference documentation](https://docs.redpanda.com/docs/reference/cluster-
        log_segment_size_min: 16777216 # 16 mb
        # -- See the property reference documentation.
        log_segment_size_max: 268435456 # 256 mb
        # -- See the property reference documentation.
        compacted_log_segment_size: 67108864 # 64 mb
        # -- See the property reference documentation.
        max_compacted_log_segment_size: 536870912 # 512 mb
        # -- See the property reference documentation.
        kafka_connection_rate_limit: 1000
                   "#;
        std::fs::write(&val_file, data).assert();

        let out_file = PathBuf::from("./tests/data/yml/_values.yaml");
        let yml = read_to_string(&val_file).assert();
        let codes = remove_comment(yml.as_str()).assert();
        write_all(out_file, codes.as_str()).assert();
    }
    #[test]
    fn test_yaml_case2() {
        let in_file = PathBuf::from("./tests/data/yml/case2_in.yml");
        let out_file = PathBuf::from("./tests/data/yml/case2_out.yml");
        let in_yml = read_to_string(&in_file).assert();
        let out_yml = read_to_string(&out_file).assert();
        let codes = remove_comment(in_yml.as_str()).assert();
        println!("{codes:#}");
        assert_eq!(out_yml, codes.as_str());
    }

    #[test]
    fn test_yaml_case3() {
        let in_file = PathBuf::from("./tests/data/yml/case3_in.yml");
        let out_file = PathBuf::from("./tests/data/yml/case3_out.yml");
        let in_yml = read_to_string(&in_file).assert();
        let out_yml = read_to_string(&out_file).assert();
        let codes = remove_comment(in_yml.as_str()).assert();
        println!("{codes:#}");
        assert_eq!(out_yml, codes.as_str());
    }

    #[test]
    fn test_yaml_case4() {
        let in_file = PathBuf::from("./tests/data/yml/case4_in.yml");
        let out_file = PathBuf::from("./tests/data/yml/case4_out.yml");
        let in_yml = read_to_string(&in_file).assert();
        let out_yml = read_to_string(&out_file).assert();
        let codes = remove_comment(in_yml.as_str()).assert();
        println!("{codes:#}");
        assert_eq!(out_yml, codes.as_str());
    }

    #[test]
    fn debug_chinese_encoding() {
        println!("=== 调试中文字符编码问题 ===");

        // 读取输入文件
        let in_file = PathBuf::from("./tests/data/yml/case4_in.yml");
        let out_file = PathBuf::from("./tests/data/yml/case4_out.yml");
        let input_content = read_to_string(&in_file).assert();
        let expected_content = read_to_string(&out_file).assert();

        // 检查文件内容是否相同
        if input_content == expected_content {
            println!("✓ 输入文件和期望输出文件内容完全相同");
        } else {
            println!("✗ 输入文件和期望输出文件内容不同");
        }

        // 检查实际处理结果
        println!("=== 测试YAML处理函数 ===");
        match remove_comment(&input_content) {
            Ok(processed) => {
                println!("处理后的内容:");
                println!("{}", processed);

                // 检查处理结果与期望输出的差异
                if processed == expected_content {
                    println!("✓ 处理结果与期望输出完全匹配");
                } else {
                    println!("✗ 处理结果与期望输出不匹配");

                    // 找出差异
                    let processed_lines: Vec<&str> = processed.lines().collect();
                    let expected_lines: Vec<&str> = expected_content.lines().collect();

                    for (i, (p_line, e_line)) in processed_lines
                        .iter()
                        .zip(expected_lines.iter())
                        .enumerate()
                    {
                        if p_line != e_line {
                            println!("第{}行差异:", i + 1);
                            println!("实际:   '{}'", p_line);
                            println!("期望:   '{}'", e_line);

                            // 检查字符级别的差异
                            let p_chars: Vec<char> = p_line.chars().collect();
                            let e_chars: Vec<char> = e_line.chars().collect();

                            for (j, (p_char, e_char)) in
                                p_chars.iter().zip(e_chars.iter()).enumerate()
                            {
                                if p_char != e_char {
                                    println!(
                                        "  字符{}差异: 实际='{}' (U+{:04X}), 期望='{}' (U+{:04X})",
                                        j, p_char, *p_char as u32, e_char, *e_char as u32
                                    );
                                }
                            }
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("处理失败: {}", e);
            }
        }
    }
}
