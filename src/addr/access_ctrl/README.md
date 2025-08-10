# NetAccessCtrl 配置说明文档

## 概述

`NetAccessCtrl` 是一个强大的网络访问控制模块，提供统一的地址重定向、认证管理、超时控制和代理配置功能。该模块支持多种网络协议（HTTP/HTTPS、Git），并提供了灵活的配置方式，适用于企业级网络环境管理。

## 核心组件

### 1. NetAccessCtrl (网络访问控制器)

`NetAccessCtrl` 是顶层控制器，管理多个 `Unit` 配置单元，提供统一的访问控制接口。

#### 核心功能
- **地址重定向**: 根据规则自动重定向网络请求
- **认证管理**: 为不同域名提供独立的认证配置
- **超时控制**: 为不同操作类型设置超时参数
- **代理配置**: 支持HTTP和SOCKS5代理配置
- **环境变量**: 支持动态配置和密钥管理

#### 结构定义
```rust
pub struct NetAccessCtrl {
    units: Vec<Unit>,     // 配置单元列表
    enable: bool,         // 启用/禁用开关
}
```

### 2. Unit (配置单元)

`Unit` 是基本的配置单元，包含一组重定向规则和相关的认证、超时、代理配置。

#### 结构定义
```rust
pub struct Unit {
    rules: Vec<Rule>,                    // 重定向规则列表
    auth: Option<AuthConfig>,           // 可选的认证配置
    timeout: Option<TimeoutConfig>,     // 可选的超时配置
    proxy: Option<ProxyConfig>,         // 可选的代理配置
}
```

#### 优先级处理
- Units 按顺序处理，第一个匹配的 Unit 生效
- 一旦匹配成功，后续 Unit 将被跳过

### 3. Rule (重定向规则)

`Rule` 定义了URL重定向规则，支持通配符匹配和环境变量。

#### 结构定义
```rust
pub struct Rule {
    pattern: String,    // 匹配模式（支持通配符*）
    target: String,     // 替换目标
}
```

#### 通配符匹配规则
- 使用 `*` 通配符匹配任意字符序列
- 匹配成功后，将通配符匹配的部分附加到目标URL
- 支持精确匹配（无通配符时）

#### 示例
```rust
let rule = Rule::new("https://github.com/*", "https://mirror.github.com/");
let result = rule.replace("https://github.com/user/repo/releases");
// 结果: "https://mirror.github.com/user/repo/releases"
```

### 4. AuthConfig (认证配置)

`AuthConfig` 提供HTTP基本认证信息，支持环境变量动态配置。

#### 结构定义
```rust
pub struct AuthConfig {
    username: String,  // 用户名
    password: String,  // 密码/令牌
}
```

#### 创建认证配置
```rust
let auth = AuthConfig::new("username", "password");
// 或使用环境变量
let auth = AuthConfig::new("${GITHUB_USER}", "${GITHUB_TOKEN}");
```

### 5. TimeoutConfig (超时配置)

`TimeoutConfig` 提供精细化的超时控制，适用于不同场景。

#### 结构定义
```rust
pub struct TimeoutConfig {
    connect_timeout: u64,  // 连接超时（秒）
    read_timeout: u64,     // 读写超时（秒）
    total_timeout: u64,    // 总操作超时（秒）
}
```

#### 预设配置
```rust
// 小文件下载（默认）
let timeout = TimeoutConfig::http_simple();

// 大文件下载
let timeout = TimeoutConfig::http_large_file();

// Git操作
let timeout = TimeoutConfig::git_operation();
```

### 6. ProxyConfig (代理配置)

`ProxyConfig` 提供HTTP和SOCKS5代理配置支持。

#### 结构定义
```rust
pub struct ProxyConfig {
    url: String,  // 代理URL
}
```

#### 创建代理配置
```rust
let proxy = ProxyConfig::new("http://proxy.example.com:8080");
// 或使用环境变量
let proxy = ProxyConfig::new("${PROXY_URL}");
```

### 7. UnitCtrl (单元控制器)

`UnitCtrl` 是运行时控制对象，为匹配的Unit提供认证、超时和代理配置。

#### 结构定义
```rust
pub struct UnitCtrl {
    auth: Option<AuthConfig>,
    timeout: Option<TimeoutConfig>,
    proxy: Option<ProxyConfig>,
}
```

## 配置文件格式

NetAccessCtrl 仅支持 YAML 配置格式，配置文件结构清晰且易于维护。

### 基本配置结构
```yaml
enable: true
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://mirror.github.com/"
    auth:
      username: "github_user"
      password: "github_token"
    timeout:
      connect-timeout: 30
      read-timeout: 60
      total-timeout: 300
    proxy:
      url: "http://proxy.example.com:8080"
```

### 多单元配置示例
```yaml
enable: true
units:
  # GitHub 配置单元
  - rules:
      - pattern: "https://github.com/*"
        target: "https://ghproxy.com/"
      - pattern: "https://raw.githubusercontent.com/*"
        target: "https://raw.ghproxy.com/"
    auth:
      username: "${GITHUB_USER:default_user}"
      password: "${GITHUB_TOKEN}"
    timeout:
      connect-timeout: 60
      read-timeout: 300
      total-timeout: 3600
  
  # GitLab 配置单元
  - rules:
      - pattern: "https://gitlab.com/*"
        target: "https://gitlab-mirror.com/"
    auth:
      username: "oauth2"
      password: "${GITLAB_TOKEN}"
  
  # NPM 配置单元
  - rules:
      - pattern: "https://registry.npmjs.org/*"
        target: "https://registry.npmmirror.com/"
    # 无需认证，使用默认超时
```

## 环境变量支持

### 环境变量语法

#### 基本语法
```
${VARIABLE_NAME}
```

#### 默认值语法
```
${VARIABLE_NAME:default_value}
```

#### 嵌套环境变量
```
${PREFIX_${DOMAIN}_SUFFIX}
```

### 环境变量配置示例

#### 配置文件
```yaml
enable: true
units:
  - rules:
      - pattern: "https://${GITHUB_DOMAIN:github.com}/*"
        target: "https://${MIRROR_DOMAIN:ghproxy.com}/"
    auth:
      username: "${GITHUB_USER}"
      password: "${GITHUB_TOKEN}"
    proxy:
      url: "${PROXY_URL:http://proxy.default:8080}"
```

#### 环境变量设置
```bash
export GITHUB_DOMAIN="github.com"
export MIRROR_DOMAIN="ghproxy.com"
export GITHUB_USER="myusername"
export GITHUB_TOKEN="ghp_xxxxxxxxxxxx"
export PROXY_URL="http://proxy.company.com:8080"
```

---

## 📖 简约配置指南

> 💡 **提示**: 如果您是系统管理员或普通用户，我们提供了专门的简约配置指南，请查看 [docs/net-access-ctrl-guide.md](../docs/net-access-ctrl-guide.md) 获取非开发者友好的配置说明。

该指南包含：
- 🚀 快速开始和基础配置
- 🔧 常见场景的现成配置模板
- ❓ 常见问题解答和故障排除
- 🛡️ 安全性和最佳实践建议

---

## 开发者使用示例

### 1. 基本重定向配置

#### 创建配置
```rust
use orion_variate::addr::access_ctrl::{NetAccessCtrl, Rule, AuthConfig, Unit};

// 创建重定向规则
let rules = vec![
    Rule::new("https://github.com/*", "https://mirror.github.com/"),
    Rule::new("https://raw.githubusercontent.com/*", "https://raw.fastgit.org/"),
];

// 创建认证配置
let auth = Some(AuthConfig::new("username", "password"));

// 创建配置单元
let unit = Unit::new(rules, auth, None);

// 创建网络访问控制器
let ctrl = NetAccessCtrl::new(vec![unit], true);
```

#### 使用控制器
```rust
// URL重定向
let original_url = "https://github.com/user/repo/releases";
let result = ctrl.redirect(original_url);

match result {
    RedirectResult::Direct(new_url, auth) => {
        println!("重定向到: {}", new_url);
        if let Some(auth) = auth {
            println!("使用认证: {}:{}", auth.username(), auth.password());
        }
    }
    RedirectResult::Origin(url) => {
        println!("保持原URL: {}", url);
    }
}
```

### 2. Git 仓库重定向

#### 配置文件 (git-redirect.yaml)
```yaml
enable: true
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://gitclone.com/github.com/"
    auth:
      username: "${GIT_USERNAME}"
      password: "${GIT_TOKEN}"
    timeout:
      connect-timeout: 120
      read-timeout: 180
      total-timeout: 1800
```

#### 代码使用
```rust
use std::path::PathBuf;
use orion_variate::addr::access_ctrl::NetAccessCtrl;

// 从配置文件加载
let config_path = PathBuf::from("git-redirect.yaml");
let ctrl = NetAccessCtrl::try_from(&config_path)?;

// Git仓库重定向
let original_repo = "https://github.com/user/project.git";
let git_repo = GitRepository::new(original_repo)?;

// 获取重定向后的仓库
let redirected_repo = ctrl.direct_git_addr(&git_repo);
if let Some(repo) = redirected_repo {
    println!("重定向到: {}", repo.repo());
    
    // 获取认证信息
    if let Some(auth) = ctrl.auth_git(&repo) {
        println!("使用认证: {}:{}", auth.username(), auth.password());
    }
}
```

### 3. HTTP 资源访问控制

#### 配置文件 (http-control.yaml)
```yaml
enable: true
units:
  - rules:
      - pattern: "https://api.example.com/*"
        target: "https://internal-api.example.com/"
    auth:
      username: "${API_USER}"
      password: "${API_KEY}"
    timeout:
      connect-timeout: 10
      read-timeout: 30
      total-timeout: 60
    proxy:
      url: "${PROXY_URL}"
```

#### 代码使用
```rust
use orion_variate::addr::HttpResource;

// 创建HTTP资源
let http_resource = HttpResource::new("https://api.example.com/v1/data")?;

// 应用访问控制
let controlled_resource = ctrl.direct_http_addr(&http_resource);

// 获取超时配置
if let Some(timeout) = ctrl.timeout_http(&controlled_resource) {
    println!("连接超时: {}秒", timeout.connect_timeout());
    println!("读取超时: {}秒", timeout.read_timeout());
    println!("总超时: {}秒", timeout.total_timeout());
}

// 获取代理配置
if let Some(proxy) = ctrl.proxy_http(&controlled_resource) {
    println!("使用代理: {}", proxy.url());
}
```

## 高级配置

### 1. 条件化配置

#### 基于环境的配置
```yaml
enable: ${ENABLE_REDIRECT:true}
units:
  - rules:
      - pattern: "https://${EXTERNAL_API_DOMAIN}/*"
        target: "https://internal-${ENV:dev}-api.company.com/"
    auth:
      username: "${API_USER_${ENV}}"
      password: "${API_KEY_${ENV}}"
    timeout:
      connect-timeout: ${CONNECT_TIMEOUT:30}
      read-timeout: ${READ_TIMEOUT:60}
      total-timeout: ${TOTAL_TIMEOUT:300}
```

#### 环境变量设置
```bash
# 开发环境
export ENV="dev"
export EXTERNAL_API_DOMAIN="api-dev.example.com"
export API_USER_dev="dev_user"
export API_KEY_dev="dev_key"
export CONNECT_TIMEOUT=10
export ENABLE_REDIRECT=true

# 生产环境
export ENV="prod"
export EXTERNAL_API_DOMAIN="api.example.com"
export API_USER_prod="prod_user"
export API_KEY_prod="prod_key"
export CONNECT_TIMEOUT=30
```

### 2. 复杂规则组合

#### 多规则单元
```yaml
enable: true
units:
  # GitHub 完整镜像配置
  - rules:
      - pattern: "https://github.com/*"
        target: "https://hub.fastgit.xyz/"
      - pattern: "https://raw.githubusercontent.com/*"
        target: "https://raw.fastgit.org/"
      - pattern: "https://gist.github.com/*"
        target: "https://gist.fastgit.org/"
    auth:
      username: "${GITHUB_USER}"
      password: "${GITHUB_TOKEN}"
    proxy:
      url: "${PROXY_URL}"
  
  # 企业内部服务
  - rules:
      - pattern: "https://external-service.com/*"
        target: "https://internal-service.company.com/"
    auth:
      username: "${INTERNAL_USER}"
      password: "${INTERNAL_PASSWORD}"
    timeout:
      connect-timeout: 5
      read-timeout: 30
      total-timeout: 120
```

### 3. 链式重定向

```yaml
enable: true
units:
  # 第一级：CDN加速
  - rules:
      - pattern: "https://github.com/*"
        target: "https://cdn.jsdelivr.net/gh/"
  
  # 第二级：国内镜像
  - rules:
      - pattern: "https://cdn.jsdelivr.net/*"
        target: "https://fastly.jsdelivr.net/"
  
  # 第三级：最终回退
  - rules:
      - pattern: "https://fastly.jsdelivr.net/*"
        target: "https://gcore.jsdelivr.net/"
```

## 最佳实践

### 1. 安全性最佳实践

#### 使用环境变量管理敏感信息
```yaml
# ✅ 好的做法
auth:
  username: "${API_USERNAME}"
  password: "${API_PASSWORD}"

# ❌ 避免的做法
auth:
  username: "hardcoded_username"
  password: "hardcoded_password"
```

#### 配置文件权限
```bash
# 设置配置文件权限为仅所有者可读写
chmod 600 net-access-config.yaml
```

### 2. 性能优化

#### 规则排序优化
```yaml
# ✅ 将最常用的规则放在前面
units:
  - rules:
      # 最常用的GitHub规则
      - pattern: "https://github.com/*"
        target: "https://mirror.github.com/"
  
  - rules:
      # 较少使用的GitLab规则
      - pattern: "https://gitlab.com/*"
        target: "https://mirror.gitlab.com/"
```

#### 超时配置优化
```yaml
# ✅ 根据场景选择合适的超时配置
units:
  - rules:
      - pattern: "https://api.fast-service.com/*"
        target: "https://internal-fast-api.com/"
    timeout:
      connect-timeout: 5   # 快速服务，短超时
      read-timeout: 15
      total-timeout: 30

  - rules:
      - pattern: "https://download.large-files.com/*"
        target: "https://mirror.download.com/"
    timeout:
      connect-timeout: 60   # 大文件下载，长超时
      read-timeout: 300
      total-timeout: 3600
```

### 3. 可维护性最佳实践

#### 分离环境配置
```yaml
# 开发环境: dev-config.yaml
enable: true
units:
  - rules:
      - pattern: "https://dev-api.example.com/*"
        target: "http://localhost:8080/"
    timeout:
      connect-timeout: 2
      read-timeout: 10
      total-timeout: 30
```

```yaml
# 生产环境: prod-config.yaml
enable: true
units:
  - rules:
      - pattern: "https://api.example.com/*"
        target: "https://internal-api.example.com/"
    timeout:
      connect-timeout: 30
      read-timeout: 60
      total-timeout: 300
    auth:
      username: "${PROD_API_USER}"
      password: "${PROD_API_KEY}"
```

#### 配置验证
```rust
use orion_variate::addr::access_ctrl::NetAccessCtrl;

fn validate_config(config_path: &Path) -> Result<(), String> {
    match NetAccessCtrl::try_from(&config_path.to_path_buf()) {
        Ok(ctrl) => {
            println!("✅ 配置验证成功");
            println!("📋 配置单元数: {}", ctrl.units().len());
            println!("🔧 启用状态: {}", ctrl.enable());
            Ok(())
        }
        Err(e) => {
            Err(format!("❌ 配置验证失败: {}", e))
        }
    }
}
```

## 错误处理

### 1. 常见错误类型

#### 配置文件错误
- **文件不存在**: 检查文件路径是否正确
- **格式错误**: 检查YAML语法是否正确
- **字段缺失**: 检查必需字段是否完整

#### 运行时错误
- **规则匹配失败**: 检查规则模式是否正确
- **认证失败**: 检查认证信息是否正确
- **网络超时**: 检查超时配置和网络连接

### 2. 错误处理示例

```rust
use orion_variate::addr::access_ctrl::NetAccessCtrl;
use orion_variate::addr::AddrError;

fn load_config_with_error_handling(config_path: &str) -> Result<NetAccessCtrl, String> {
    let path = PathBuf::from(config_path);
    
    match NetAccessCtrl::try_from(&path) {
        Ok(ctrl) => Ok(ctrl),
        Err(AddrError::Brief(msg)) => Err(format!("配置错误: {}", msg)),
        Err(AddrError::Uvs(reason)) => Err(format!("UVS错误: {}", reason)),
        Err(e) => Err(format!("未知错误: {:?}", e)),
    }
}

fn safe_redirect(ctrl: &NetAccessCtrl, url: &str) -> String {
    match ctrl.redirect(url) {
        RedirectResult::Direct(new_url, _) => {
            println!("✅ 成功重定向: {} -> {}", url, new_url);
            new_url
        }
        RedirectResult::Origin(original_url) => {
            println!("ℹ️  无匹配规则，保持原URL: {}", original_url);
            original_url.to_string()
        }
    }
}
```

### 3. 调试技巧

#### 启用详细日志
```rust
use log::LevelFilter;

fn setup_logging() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .target(env_logger::Target::Stdout)
        .init();
}
```

#### 配置验证工具
```bash
# 使用内置测试验证配置
cargo test access_ctrl::tests::test_serv_complex_yaml_structure

# 手动验证配置
cargo run -- validate-config --path config.yaml
```

## 迁移指南

### 1. 从旧版本迁移

#### v0.5.x 到 v0.6.x
```yaml
# 旧版本配置 (v0.5.x)
redirect_rules:
  - pattern: "https://github.com/*"
    target: "https://mirror.com/"
auth_configs:
  "https://github.com":
    username: "user"
    password: "pass"

# 新版本配置 (v0.6.x)
enable: true
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://mirror.com/"
    auth:
      username: "user"
      password: "pass"
```

### 2. 配置文件迁移工具

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct OldConfig {
    redirect_rules: Vec<OldRule>,
    auth_configs: std::collections::HashMap<String, OldAuth>,
}

#[derive(Deserialize)]
struct OldRule {
    pattern: String,
    target: String,
}

#[derive(Deserialize)]
struct OldAuth {
    username: String,
    password: String,
}

fn migrate_old_config(old_path: &Path, new_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let old_content = std::fs::read_to_string(old_path)?;
    let old_config: OldConfig = serde_yaml::from_str(&old_content)?;
    
    let new_config = NetAccessCtrl::new(
        old_config.redirect_rules.into_iter().map(|old_rule| {
            Unit::new(
                vec![Rule::new(old_rule.pattern, old_rule.target)],
                None, // 认证配置需要额外处理
                None,
            )
        }).collect(),
        true,
    );
    
    new_config.save_yml(new_path)?;
    Ok(())
}
```

### 3. 验证迁移结果

```rust
fn validate_migration(old_path: &Path, new_path: &Path) -> Result<(), String> {
    // 加载新旧配置
    let old_ctrl = load_old_config(old_path)?;
    let new_ctrl = NetAccessCtrl::try_from(&new_path.to_path_buf())
        .map_err(|e| format!("新配置加载失败: {}", e))?;
    
    // 测试相同的URL重定向结果
    let test_urls = vec![
        "https://github.com/user/repo",
        "https://gitlab.com/user/project",
        "https://example.com/api/data",
    ];
    
    for url in test_urls {
        let old_result = old_ctrl.redirect(url);
        let new_result = new_ctrl.redirect(url);
        
        match (old_result, new_result) {
            (RedirectResult::Origin(old_url), RedirectResult::Origin(new_url)) => {
                if old_url != new_url {
                    return Err(format!("URL {} 重定向结果不一致", url));
                }
            }
            (RedirectResult::Direct(old_url, _), RedirectResult::Direct(new_url, _)) => {
                if old_url != new_url {
                    return Err(format!("URL {} 重定向结果不一致", url));
                }
            }
            _ => {
                return Err(format!("URL {} 重定向类型不一致", url));
            }
        }
    }
    
    Ok(())
}
```

## 总结

`NetAccessCtrl` 提供了强大而灵活的网络访问控制功能，支持：

- ✅ 多种协议支持（HTTP/HTTPS、Git）
- ✅ 灵活的重定向规则配置
- ✅ 安全的认证信息管理
- ✅ 精细化的超时控制
- ✅ 多种代理配置选项
- ✅ 环境变量动态配置
- ✅ YAML 配置文件格式支持

通过合理配置，可以显著提升网络访问的效率、安全性和可靠性。建议在实际使用中结合具体业务场景，选择合适的配置策略。