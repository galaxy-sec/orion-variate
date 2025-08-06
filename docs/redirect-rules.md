# 地址重定向规则配置文档

本文档详细说明了如何在配置文件中定义和使用地址重定向规则，用于优化网络访问和代理配置。

## 概述

`orion-variate` 提供了灵活的地址重定向机制，允许用户通过配置文件定义URL重写规则，实现：
- 镜像源替换（如GitHub → 国内镜像）
- 代理服务器配置
- 认证信息管理
- 多环境适配

## 配置结构

### 顶级字段

```yaml
# 重定向服务配置
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

## 完整配置示例

### 示例1：GitHub镜像配置

```yaml
# 配置文件：redirect.yml
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

### 示例3：最小配置

```yaml
enable: true
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://mirror.github.com/"
```

## API参考

### RedirectService
**方法：**
- `new(units: Vec<Unit>, enable: bool) -> Self` - 创建新的重定向服务
- `from_file(&PathBuf) -> Result<Self, AddrError>` - 从配置文件加载
- `from_str(&str) -> Result<Self, AddrError>` - 从字符串加载
- `redirect(&self, url: &str) -> RedirectResult` - 通用URL重定向
- `direct_http_addr(&self, origin: HttpResource) -> HttpResource` - HTTP资源重定向
- `direct_git_addr(&self, origin: GitRepository) -> GitRepository` - Git仓库重定向

文档已更新完成，移除了所有错误的direct_serv包装，直接使用enable和units作为顶级字段，符合最新代码实现。