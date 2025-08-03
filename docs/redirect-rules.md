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
use orion_variate::addr::redirect::DirectServ;
use std::path::PathBuf;

// 从文件加载配置
let config_path = PathBuf::from("redirect.yml");
let direct_serv = DirectServ::try_from(&config_path)?;

// 应用重定向
let original_url = "https://github.com/user/repo.git";
let redirected_url = direct_serv.redirect(original_url);
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

## 调试和验证

### 验证配置文件

```bash
# 验证YAML格式
orion-variate validate --config redirect.yml

# 测试重定向规则
orion-variate redirect-test --url "https://github.com/user/repo.git"
```

### 调试输出

启用调试日志查看重定向过程：

```bash
RUST_LOG=debug orion-variate update
```

## 常见问题

### Q: 规则不生效怎么办？
- 检查 `enable: true` 是否设置
- 确认配置文件路径正确
- 检查YAML语法是否正确
- 使用调试模式查看匹配过程

### Q: 如何排除特定URL？
- 使用精确匹配覆盖通配符
- 将排除规则放在前面
- 使用空目标保持原URL

### Q: 性能如何？
- 规则按顺序匹配，建议将常用规则放在前面
- 通配符使用高效的WildMatch算法
- 建议规则数量控制在100条以内

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