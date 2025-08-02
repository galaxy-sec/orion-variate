use getset::Getters;
use serde_derive::{Deserialize, Serialize};
use wildmatch::WildMatch;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Rule {
    #[serde(skip)]
    matchs: WildMatch,
    pattern: String,
    target: String,
}

impl Rule {
    pub fn new<S: AsRef<str>, S2: Into<String>>(matchs: S, target: S2) -> Self {
        let pattern = matchs.as_ref().to_string();
        Self {
            matchs: WildMatch::new(&pattern),
            pattern,
            target: target.into(),
        }
    }
    pub fn replace(&self, input: &str) -> Option<String> {
        if self.matchs.matches(input) {
            // 找到模式中的通配符位置
            if let Some(star_idx) = self.pattern.find('*') {
                let prefix = &self.pattern[..star_idx];
                if input.starts_with(prefix) {
                    let suffix = &input[prefix.len()..];
                    return Some(format!("{}{suffix}", self.target));
                }
            }
            // 如果没有通配符或者精确匹配，直接替换整个字符串
            None
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule() {
        let rule = Rule::new(
            "https://github.com/galaxy-sec/galaxy-flow*",
            "https://gflow.com",
        );
        let url = rule.replace("https://github.com/galaxy-sec/galaxy-flow/releases/download/v0.8.5/galaxy-flow-v0.8.5-aarch64-apple-darwin.tar.gz");
        assert_eq!(url,Some("https://gflow.com/releases/download/v0.8.5/galaxy-flow-v0.8.5-aarch64-apple-darwin.tar.gz".to_string()));
    }

    #[test]
    fn test_rule_no_match() {
        let rule = Rule::new(
            "https://github.com/galaxy-sec/galaxy-flow*",
            "https://gflow.com",
        );
        let url = rule.replace("https://example.com/other/path");
        assert_eq!(url, None);
    }

    #[test]
    fn test_rule_exact_match() {
        let rule = Rule::new(
            "https://github.com/galaxy-sec/galaxy-flow*",
            "https://gflow.com",
        );
        let url = rule.replace("https://github.com/galaxy-sec/galaxy-flow");
        assert_eq!(url, Some("https://gflow.com".to_string()));
    }
}
