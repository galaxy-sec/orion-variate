# RedirectService EnvEvalable 能力设计方案

## 背景

当前RedirectService的units（包含Rule和AuthConfig）不支持环境变量扩展，这限制了配置的灵活性。我们需要为RedirectService及其相关结构提供EnvEvalable能力，使其能够解析环境变量。

## 目标

为RedirectService的units提供完整的EnvEvalable能力，支持在以下字段中使用环境变量：
- Rule中的pattern和target字段
- AuthConfig中的username和password字段
- Unit中的rules和auth配置
- RedirectService中的units集合

## 设计方案

### 1. 实现EnvEvalable trait

为相关结构实现EnvEvalable trait，支持环境变量扩展：

#### 1.1 Rule结构实现

```rust
use crate::vars::{EnvDict, EnvEvalable, ValueType};
use wildmatch::WildMatch;

impl EnvEvalable<Rule> for Rule {
    fn env_eval(self, dict: &EnvDict) -> Rule {
        let pattern = self.pattern.env_eval(dict);
        Rule {
            matchs: WildMatch::new(&pattern),
            pattern,
            target: self.target.env_eval(dict),
        }
    }
}
```

#### 1.2 AuthConfig结构实现

```rust
impl EnvEvalable<AuthConfig> for AuthConfig {
    fn env_eval(self, dict: &EnvDict) -> AuthConfig {
        AuthConfig {
            username: self.username.env_eval(dict),
            password: self.password.env_eval(dict),
        }
    }
}
```

#### 1.3 Unit结构实现

```rust
impl EnvEvalable<Unit> for Unit {
    fn env_eval(self, dict: &EnvDict) -> Unit {
        Unit {
            rules: self.rules.into_iter().map(|rule| rule.env_eval(dict)).collect(),
            auth: self.auth.map(|auth| auth.env_eval(dict)),
            ..self
        }
    }
}
```

#### 1.4 RedirectService结构实现

```rust
impl EnvEvalable<RedirectService> for RedirectService {
    fn env_eval(self, dict: &EnvDict) -> RedirectService {
        RedirectService {
            units: self.units.into_iter().map(|unit| unit.env_eval(dict)).collect(),
            ..self
        }
    }
}
```



### 2. 使用示例

#### 2.1 YAML配置中使用环境变量

```yaml
redirect_service:
  enable: true
  units:
    - rules:
        - pattern: "https://github.com/${ORG}/*"
          target: "https://mirror.${DOMAIN}/github/${ORG}/"
      auth:
        username: "${GITHUB_USERNAME}"
        password: "${GITHUB_TOKEN}"
    - rules:
        - pattern: "https://gitlab.com/*"
          target: "https://mirror.${DOMAIN}/gitlab/"
```

#### 2.2 编程式使用

```rust
use crate::vars::EnvDict;
use crate::addr::redirect::{RedirectService, Unit, Rule, AuthConfig};

let mut env_dict = EnvDict::new();
env_dict.insert("ORG".to_string(), "myorg".to_string());
env_dict.insert("DOMAIN".to_string(), "example.com".to_string());
env_dict.insert("GITHUB_USERNAME".to_string(), "user".to_string());
env_dict.insert("GITHUB_TOKEN".to_string(), "token123".to_string());

let service = RedirectService::from_yaml(yaml_content);
let evaluated_service = service.env_eval(&env_dict);
```

### 3. 环境变量语法支持

支持以下环境变量语法：
- `${VAR}` - 使用环境变量VAR的值
- `${VAR:default}` - 使用环境变量VAR的值，如果不存在则使用默认值
- 支持嵌套在字符串中的任意位置

### 4. 测试策略

#### 4.1 单元测试

为每个结构创建单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vars::{EnvDict, ValueType};
    use wildmatch::WildMatch;

    #[test]
    fn test_rule_env_eval() {
        let mut dict = EnvDict::new();
        dict.insert("DOMAIN".to_string(), ValueType::String("example.com".to_string()));
        
        let rule = Rule {
            pattern: "https://${DOMAIN}/*".to_string(),
            target: "https://mirror.${DOMAIN}/".to_string(),
            matchs: WildMatch::new("https://example.com/*"),
        };
        let evaluated = rule.env_eval(&dict);
        
        assert_eq!(evaluated.pattern, "https://example.com/*");
        assert_eq!(evaluated.target, "https://mirror.example.com/");
    }

    #[test]
    fn test_auth_config_env_eval() {
        let mut dict = EnvDict::new();
        dict.insert("USER".to_string(), ValueType::String("admin".to_string()));
        dict.insert("PASS".to_string(), ValueType::String("secret123".to_string()));
        
        let auth = AuthConfig {
            username: "${USER}".to_string(),
            password: "${PASS}".to_string(),
        };
        let evaluated = auth.env_eval(&dict);
        
        assert_eq!(evaluated.username, "admin");
        assert_eq!(evaluated.password, "secret123");
    }

    #[test]
    fn test_unit_env_eval() {
        let mut dict = EnvDict::new();
        dict.insert("ORG".to_string(), ValueType::String("myorg".to_string()));
        
        let unit = Unit {
            rules: vec![Rule {
                pattern: "https://github.com/${ORG}/*".to_string(),
                target: "https://mirror.com/${ORG}/".to_string(),
                matchs: WildMatch::new("https://github.com/myorg/*"),
            }],
            auth: Some(AuthConfig {
                username: "user".to_string(),
                password: "${ORG}_token".to_string(),
            }),
            ..Default::default()
        };
        
        let evaluated = unit.env_eval(&dict);
        assert_eq!(evaluated.rules[0].pattern, "https://github.com/myorg/*");
        assert_eq!(evaluated.rules[0].target, "https://mirror.com/myorg/");
        assert_eq!(evaluated.auth.unwrap().password, "myorg_token");
    }
}
```

#### 4.2 集成测试

创建集成测试验证整个RedirectService的EnvEvalable功能：

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::vars::{EnvDict, ValueType};
    use wildmatch::WildMatch;

    #[test]
    fn test_redirect_service_env_eval() {
        let mut dict = EnvDict::new();
        dict.insert("MIRROR_DOMAIN".to_string(), ValueType::String("mirror.example.com".to_string()));
        dict.insert("GITHUB_TOKEN".to_string(), ValueType::String("gh_token_123".to_string()));

        let service = RedirectService {
            units: vec![Unit {
                rules: vec![Rule {
                    pattern: "https://github.com/*".to_string(),
                    target: "https://${MIRROR_DOMAIN}/github/".to_string(),
                    matchs: WildMatch::new("https://github.com/*"),
                }],
                auth: Some(AuthConfig {
                    username: "user".to_string(),
                    password: "${GITHUB_TOKEN}".to_string(),
                }),
                ..Default::default()
            }],
            enable: true,
        };

        let evaluated = service.env_eval(&dict);
        
        assert_eq!(evaluated.units[0].rules[0].target, "https://mirror.example.com/github/");
        assert_eq!(evaluated.units[0].auth.as_ref().unwrap().password, "gh_token_123");
    }
}
```

### 5. 实现步骤

1. **实现Rule的EnvEvalable** - 支持pattern和target字段的环境变量扩展
2. **实现AuthConfig的EnvEvalable** - 支持username和password字段的环境变量扩展
3. **实现Unit的EnvEvalable** - 支持rules和auth字段的环境变量扩展
4. **实现RedirectService的EnvEvalable** - 支持units集合的环境变量扩展
5. **添加单元测试** - 为每个结构添加对应的单元测试
6. **添加集成测试** - 验证整个RedirectService的EnvEvalable功能
7. **更新文档** - 在redirect-rules.md中添加环境变量使用的说明

### 6. 兼容性考虑

- 保持向后兼容性，EnvEvalable实现不会改变现有API
- 环境变量扩展是惰性的，只在调用env_eval时进行
- 支持空环境字典，此时返回原始值
- 错误处理：如果环境变量不存在，保持原始字符串不变

### 7. 性能优化

- 使用迭代器避免不必要的内存分配
- 只在需要时进行环境变量扩展
- 缓存WildMatch实例以避免重复编译模式

### 8. 错误处理

- 环境变量不存在时保持原始值
- 提供调试日志输出扩展后的值
- 在验证阶段检查扩展后的值是否有效

## 预期结果

完成此方案后，RedirectService将支持完整的环境变量扩展能力，用户可以在配置中灵活使用环境变量，提高配置的可移植性和安全性。

## 实现状态 ✅ 已完成

### 实现细节
- ✅ 为`AuthConfig`实现了`EnvEvalable` trait
- ✅ 为`Rule`实现了`EnvEvalable` trait  
- ✅ 为`Unit`实现了`EnvEvalable` trait
- ✅ 为`RedirectService`实现了`EnvEvalable` trait
- ✅ 添加了完整的测试用例覆盖
- ✅ 修复了所有编译错误和类型不匹配问题

### 测试结果
运行测试命令：`cargo test addr::redirect -- --nocapture`

**测试结果：27个测试全部通过**
- AuthConfig环境变量测试：2个测试通过
- Rule环境变量测试：2个测试通过  
- Unit环境变量测试：3个测试通过
- RedirectService环境变量测试：3个测试通过
- 原有功能测试：17个测试通过

### 使用示例
```yaml
# 支持环境变量的配置示例
units:
  - rules:
      - pattern: "${DOMAIN:example.com}/*"
        target: "https://${TARGET:backup.com}/${1}"
    auth:
      username: "${USERNAME}"
      password: "${PASSWORD:default_pass}"
```

RedirectService现在具备了完整的环境变量解析能力，可以在配置中使用`${VAR}`和`${VAR:default}`格式实现动态配置。