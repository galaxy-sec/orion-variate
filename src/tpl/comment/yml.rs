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
    BlockData,
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
                    c != '"' && c != '|' && c != '>' && c != '\'' && c != '#' && c != '\n' && c != '\r'
                })
                .parse_next(input)?;

                if opt(line_ending).parse_next(input)?.is_some() {
                    line += code;
                    if !line.trim().is_empty() {
                        out += line.as_str();
                        out += "\n";
                        line = String::new();
                    }
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
                    let is_valid = mods
                        .chars()
                        .all(|ch| ch.is_ascii_whitespace() || ch == '+' || ch == '-' || ch.is_ascii_digit());
                    if is_valid {
                        // Consume the modifiers portion
                        let consume_len = mods.len();
                        line.push(ind);
                        line.push_str(mods);
                        *input = &s[consume_len..];
                        // Start of block scalar only if followed by a real line ending
                        if opt(line_ending).parse_next(input)?.is_some() {
                            line.push('\n');
                            *status = YmlStatus::BlockData;
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
            YmlStatus::BlockData => match till_line_ending.parse_next(input) {
                Ok(data) => {
                    if data.trim().is_empty() {
                        *status = YmlStatus::Code;
                    } else {
                        line += data;
                    }
                }
                Err(e) => return Err(e),
            },

            YmlStatus::StringDouble => {
                // Read until an unescaped double quote (handles \" sequences)
                let s = *input;
                let mut idx = 0;
                let bytes = s.as_bytes();
                let mut escaped = false;
                while idx < bytes.len() {
                    let ch = bytes[idx] as char;
                    if ch == '\n' || ch == '\r' {
                        // allow EOL inside double quoted in YAML (it is allowed with escaping), flush and continue
                        line.push(ch);
                        escaped = false;
                        idx += 1;
                        continue;
                    }
                    if ch == '"' && !escaped {
                        // include closing quote and consume
                        line.push('"');
                        idx += 1;
                        *input = &s[idx..];
                        *status = YmlStatus::Code;
                        break;
                    }
                    if ch == '\\' && !escaped {
                        escaped = true;
                        line.push(ch);
                        idx += 1;
                        continue;
                    }
                    escaped = false;
                    line.push(ch);
                    idx += 1;
                }
                if idx >= bytes.len() {
                    // Unterminated string: consume all
                    *input = &s[idx..];
                    *status = YmlStatus::Code;
                }
            }
            YmlStatus::StringSingle => {
                // Read until a single quote that is not part of a doubled '' escape
                let s = *input;
                let mut chars = s.char_indices().peekable();
                let mut end_idx = None;
                while let Some((i, ch)) = chars.next() {
                    if ch == '\'' {
                        if let Some((_, next_ch)) = chars.peek() {
                            if *next_ch == '\'' {
                                // Escaped quote: append one and skip the next
                                line.push('\'');
                                let _ = chars.next(); // consume the escape partner
                                continue;
                            }
                        }
                        // Closing quote
                        end_idx = Some(i + ch.len_utf8());
                        line.push('\'');
                        break;
                    } else {
                        line.push(ch);
                    }
                }
                let idx = end_idx.unwrap_or_else(|| s.len());
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
                if has_eol {
                    line += "\n";
                }
                if !line.trim().is_empty() {
                    out += &line;
                    line = String::new();
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
        if !code.trim().is_empty() {
            out += code.as_str();
            if opt(line_ending).parse_next(input)?.is_some() {
                //out += "\n";
            }
        }
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
        let base_path = PathBuf::from("./test/data/yml");
        std::fs::create_dir_all(&base_path).assert();

        let codes = remove_comment(YAML_DATA).assert();
        let excpet = r#"vector:
  customConfigNamespace: ""
    sinks:
      vlogs:
        request:
          headers:
            ProjectID: "0"

extraObjects: []
"#;
        assert_eq!(codes, excpet);
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

    const YAML_DATA: &str = r#"
vector:
  customConfigNamespace: ""
    sinks:
      vlogs:
        request:
          headers:
            ProjectID: "0"

# -- Add extra specs dynamically to this chart
extraObjects: []
"#;
}
