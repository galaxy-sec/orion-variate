# 地址重定向规则配置文档

本文档详细说明了如何在配置文件中定义和使用地址重定向规则，用于优化网络访问和代理配置。

## 概述

`orion-variate` 提供了灵活的地址重定向机制，允许用户通过配置文件定义URL重写规则，实现：
- 镜像源替换（如GitHub → 国内镜像）
- 代理服务器配置
- 认证信息管理
- 多环境适配

## 配置结构

### 顶级结构：DirectServ

```yaml
# 重定向服务配置
direct_serv:
  enable: true  # 是否启用重定向
  units:        # 重定向单元列表
    - ...
```

### 重定向单元：Unit

每个Unit包含一组规则和可选的认证信息：

```yaml
units:
  - rules:      # 规则列表
      - ...
    auth:       # 认证信息（可选）
      username: "user"
      password: "pass"
```

### 重定向规则：Rule

每个规则定义匹配模式和替换目标：

```yaml
rules:
  - pattern: "https://github.com/*"
    target: "https://ghproxy.com/"
  - pattern: "https://raw.githubusercontent.com/*"
    target: "https://raw.ghproxy.com/"
```

## 规则语法详解

### 通配符匹配

使用 `*` 作为通配符匹配任意字符：

| 模式 | 匹配示例 | 替换结果 |
|------|----------|----------|
| `https://github.com/*` | `https://github.com/user/repo` | `https://mirror.com/user/repo` |
| `https://*.npmjs.org/*` | `https://registry.npmjs.org/pkg` | `https://registry.npmmirror.com/pkg` |

### 精确匹配

不包含通配符的模式进行精确匹配：

| 模式 | 匹配示例 | 替换结果 |
|------|----------|----------|
| `https://example.com` | `https://example.com` | `https://proxy.com` |
| `https://example.com` | `https://example.com/path` | 不匹配 |

## 完整配置示例

### 示例1：GitHub镜像配置

```yaml
# 配置文件：redirect.yml
direct_serv:
  enable: true
  units:
    # GitHub 镜像配置
    - rules:
        - pattern: "https://github.com/*"
          target: "https://hub.fastgit.org/"
        - pattern: "https://raw.githubusercontent.com/*"
          target: "https://raw.fastgit.org/"
        - pattern: "https://gist.githubusercontent.com/*"
          target: "https://gist.fastgit.org/"
      auth:
        username: "mirror_user"
        password: "mirror_pass"
    
    # GitLab 镜像配置
    - rules:
        - pattern: "https://gitlab.com/*"
          target: "https://gitlab.cnpmjs.org/"
```

### 示例2：企业代理配置

```yaml
direct_serv:
  enable: true
  units:
    # 内部代理服务器
    - rules:
        - pattern: "https://registry.npmjs.org/*"
          target: "https://nexus.company.com/repository/npm-proxy/"
        - pattern: "https://pypi.org/simple/*"
          target: "https://nexus.company.com/repository/pypi-proxy/simple/"
        - pattern: "https://repo.maven.apache.org/*"
          target: "https://nexus.company.com/repository/maven-central/"
      auth:
        username: "${NEXUS_USER}"
        password: "${NEXUS_PASS}"
```

### 示例3：多环境配置

```yaml
direct_serv:
  enable: true
  units:
    # 开发环境
    - rules:
        - pattern: "https://dev-api.company.com/*"
          target: "http://localhost:3000/"
    
    # 测试环境
    - rules:
        - pattern: "https://test-api.company.com/*"
          target: "https://test-mirror.company.com/"
    
    # 生产环境
    - rules:
        - pattern: "https://api.company.com/*"
          target: "https://api.company.com/"
```

## 环境变量支持

配置文件中支持使用环境变量：

```yaml
units:
  - rules:
      - pattern: "https://private.repo.com/*"
        target: "https://proxy.repo.com/"
    auth:
      username: "${REPO_USERNAME}"    # 从环境变量读取
      password: "${REPO_PASSWORD}"  # 从环境变量读取
```

## 配置文件位置

重定向规则配置文件可以放置在以下位置：

1. **项目级配置**：`./redirect.yml`
2. **用户级配置**：`~/.config/orion-variate/redirect.yml`
3. **系统级配置**：`/etc/orion-variate/redirect.yml`

## 优先级规则

当多个规则匹配同一个URL时：

1. **精确匹配优先**：精确匹配的规则优先于通配符匹配
2. **最长匹配优先**：在通配符匹配中，最长匹配的规则优先
3. **顺序优先**：在同一Unit内，先定义的规则优先
4. **Unit顺序**：先定义的Unit优先

## 使用示例

### 在代码中使用

```rust
use orion_variate::addr::redirect::RedirectService;
use std::path::PathBuf;

// 从文件加载配置
let config_path = PathBuf::from("redirect.yml");
let redirect_service = RedirectService::try_from(&config_path)?;

// 应用重定向
let original_url = "https://github.com/user/repo.git";
let redirected_url = redirect_service.redirect(original_url);

// 针对特定类型的地址进行重定向
use orion_variate::addr::{HttpResource, GitRepository};

// HTTP资源重定向
let http_addr = HttpResource::from("https://github.com/user/repo");
let redirected_http = redirect_service.direct_http_addr(http_addr);

// Git仓库重定向
let git_addr = GitRepository::from("https://github.com/user/repo.git");
let redirected_git = redirect_service.direct_git_addr(git_addr);
```

### 在配置中使用

```yaml
# orion-variate.yml
variables:
  - name: "source_url"
    value: "https://github.com/user/repo.git"
    redirect:
      file: "./redirect.yml"  # 指定重定向配置文件
```

### 编程式创建

```rust
use orion_variate::addr::redirect::{RedirectService, Rule, AuthConfig};

// 通过规则创建
let rule = Rule::new("https://github.com/*", "https://mirror.github.com/");
let auth = Some(AuthConfig::new("username", "password"));
let redirect_service = RedirectService::from_rule(rule, auth);

// 或者通过配置创建
let redirect_service = RedirectService::new(
    vec![
        Unit::new(
            vec![
                Rule::new("https://github.com/*", "https://mirror1.com/"),
                Rule::new("https://gitlab.com/*", "https://mirror2.com/"),
            ],
            Some(AuthConfig::new("user", "pass"))
        )
    ],
    true
);
```

## API参考

### 主要结构体

#### RedirectService
主重定向服务结构体，负责管理和应用重定向规则。

**方法：**
- `new(units: Vec<Unit>, enable: bool) -> Self` - 创建新的重定向服务
- `try_from(&PathBuf) -> Result<Self, AddrError>` - 从配置文件加载
- `redirect(&self, url: &str) -> RedirectResult` - 通用URL重定向
- `direct_http_addr(&self, origin: HttpResource) -> HttpResource` - HTTP资源重定向
- `direct_git_addr(&self, origin: GitRepository) -> GitRepository` - Git仓库重定向
- `from_rule(rule: Rule, auth: Option<AuthConfig>) -> Self` - 从单个规则创建

#### Rule
重定向规则，定义URL匹配和替换逻辑。

**方法：**
- `new(pattern: &str, target: &str) -> Self` - 创建新规则
- `replace(&self, input: &str) -> Option<String>` - 应用规则到URL

#### Unit
重定向单元，包含一组规则和可选认证信息。

**方法：**
- `new(rules: Vec<Rule>, auth: Option<AuthConfig>) -> Self` - 创建新单元
- `rules() -> &[Rule]` - 获取规则列表
- `auth() -> Option<&AuthConfig>` - 获取认证信息

#### AuthConfig
认证配置，支持用户名密码认证。

**方法：**
- `new(username: &str, password: &str) -> Self` - 创建认证配置
- `username() -> &str` - 获取用户名
- `password() -> &str` - 获取密码

## 调试和验证

### 验证配置文件

#### 1. 配置文件格式验证
```bash
# 验证YAML格式
orion-variate validate --config redirect.yml

# 测试重定向规则
orion-variate redirect-test --url "https://github.com/user/repo.git"
```

#### 2. 编程式验证
```rust
use orion_variate::addr::redirect::RedirectService;
use std::path::PathBuf;

fn test_redirect_rules() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = PathBuf::from("redirect.yml");
    let service = RedirectService::try_from(&config_path)?;
    
    // 测试单个URL
    let test_url = "https://github.com/user/repo.git";
    let result = service.redirect(test_url);
    println!("Original: {} -> Redirected: {:?}", test_url, result);
    
    // 测试HTTP资源
    use orion_variate::addr::HttpResource;
    let http_addr = HttpResource::from("https://github.com/user/repo");
    let redirected = service.direct_http_addr(http_addr);
    println!("HTTP redirect: {} -> {}", http_addr.url(), redirected.url());
    
    Ok(())
}
```

#### 3. 单元测试验证
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use orion_variate::addr::redirect::{RedirectService, Rule, AuthConfig};

    #[test]
    fn test_github_mirror() {
        let rule = Rule::new("https://github.com/*", "https://mirror.github.com/");
        let service = RedirectService::from_rule(rule, None);
        
        let result = service.redirect("https://github.com/user/repo.git");
        assert_eq!(result.path(), "https://mirror.github.com/user/repo.git");
    }

    #[test]
    fn test_no_match() {
        let rule = Rule::new("https://github.com/*", "https://mirror.github.com/");
        let service = RedirectService::from_rule(rule, None);
        
        let result = service.redirect("https://gitlab.com/user/repo.git");
        assert_eq!(result.path(), "https://gitlab.com/user/repo.git");
    }
}
```

### 调试输出

#### 1. 环境变量调试
启用调试日志查看重定向过程：

```bash
# 基本调试
RUST_LOG=debug orion-variate update

# 详细调试（包括规则匹配过程）
RUST_LOG=orion_variate::addr::redirect=trace orion-variate update

# 输出到文件
RUST_LOG=debug orion-variate update 2> redirect-debug.log
```

#### 2. 程序化调试
```rust
use log::{debug, info};

fn debug_redirect_service(service: &RedirectService) {
    info!("Redirect service enabled: {}", service.enable());
    info!("Number of units: {}", service.units().len());
    
    for (i, unit) in service.units().iter().enumerate() {
        debug!("Unit {}: {} rules, auth: {}", 
               i, 
               unit.rules().len(),
               unit.auth().is_some());
        
        for (j, rule) in unit.rules().iter().enumerate() {
            debug!("  Rule {}: {} -> {}", j, rule.pattern(), rule.target());
        }
    }
}
```

## 常见问题

### Q: 规则不生效怎么办？
**排查步骤：**
1. 检查 `enable: true` 是否设置
2. 确认配置文件路径正确
3. 检查YAML语法是否正确（使用在线YAML验证器）
4. 使用调试模式查看匹配过程：`RUST_LOG=debug`
5. 确认文件权限可读
6. 检查环境变量是否正确解析

### Q: 如何排除特定URL？
**方法：**
- 使用精确匹配覆盖通配符
- 将排除规则放在前面
- 使用空目标保持原URL
- 创建专门的排除单元

**示例：**
```yaml
direct_serv:
  enable: true
  units:
    # 排除规则（优先匹配）
    - rules:
        - pattern: "https://github.com/exclude/*"
          target: "https://github.com/exclude/"  # 保持原URL
    # 通用规则
    - rules:
        - pattern: "https://github.com/*"
          target: "https://mirror.github.com/"
```

### Q: 性能如何优化？
**优化建议：**
- 规则按顺序匹配，将常用规则放在前面
- 通配符使用高效的WildMatch算法
- 建议规则数量控制在100条以内
- 避免复杂的嵌套通配符
- 使用精确匹配代替通配符（当可能时）

### Q: 如何支持多个镜像源？
**配置示例：**
```yaml
direct_serv:
  enable: true
  units:
    # GitHub镜像1
    - rules:
        - pattern: "https://github.com/*"
          target: "https://hub.fastgit.org/"
    # GitHub镜像2（备用）
    - rules:
        - pattern: "https://github.com/*"
          target: "https://ghproxy.com/"
    # GitLab镜像
    - rules:
        - pattern: "https://gitlab.com/*"
          target: "https://gitlab.cnpmjs.org/"
```

### Q: 环境变量不生效？
**检查：**
- 环境变量名是否正确（区分大小写）
- 默认值语法：`${VAR:-default}`
- 特殊字符需要转义
- 在shell中测试：`echo $MY_VAR`

### Q: 如何处理HTTPS证书问题？
**解决方案：**
- 使用正确的镜像源（支持HTTPS）
- 配置系统证书
- 使用HTTP镜像源（不推荐）
- 配置代理服务器

### Q: 如何调试规则匹配？
**调试工具：**
```bash
# 创建测试脚本
#!/bin/bash
export RUST_LOG=orion_variate::addr::redirect=trace
orion-variate redirect-test --url "$1"
```

### Q: 版本兼容性问题？
- **v0.5.0+**：基本重定向
- **v0.5.7+**：多Unit配置、环境变量
- **v0.6.0+**：计划支持正则表达式
- 升级时注意API变更（DirectServ -> RedirectService）

## 高级用法

### 条件重定向

结合环境变量实现条件重定向：

```yaml
direct_serv:
  enable: "${ENABLE_REDIRECT:-true}"  # 环境变量控制
  units:
    - rules:
        - pattern: "https://github.com/*"
          target: "${GITHUB_MIRROR:-https://github.com/}"
```

### 链式重定向

支持多个Unit的链式处理：

```yaml
direct_serv:
  enable: true
  units:
    # 第一步：域名替换
    - rules:
        - pattern: "https://github.com/*"
          target: "https://ghproxy.com/"
    
    # 第二步：路径调整
    - rules:
        - pattern: "https://ghproxy.com/*/releases/*"
          target: "https://ghproxy.com/mirror/\1/releases/\2"
```

## 版本兼容性

- **v0.5.0+**：支持基本重定向规则
- **v0.5.7+**：支持多Unit配置、环境变量
- **v0.6.0+**：计划支持正则表达式匹配

---

如需更多帮助，请参考项目文档或提交Issue。