# 📖 NetAccessCtrl 简约配置指南（非开发者版）

## 什么是 NetAccessCtrl？

NetAccessCtrl 是一个网络访问控制模块，可以在使用orino_variate 时自动将您的网络请求重定向到更快的镜像服务器，支持认证、超时设置和代理配置。它可以帮助您：

- 🚀 加速 GitHub、GitLab 等国外服务访问
- 🔐 安全管理认证信息
- ⏱️ 控制网络请求超时时间
- 🌐 配置代理服务器
- 📝 使用环境变量动态配置

## 快速开始

### 1. 创建配置文件

在您的项目根目录创建 `net-accessor_ctrl.yaml` 文件：

```yaml
# 基础配置示例
enable: true
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://mirror.ghproxy.com/"
    # 可选：添加认证信息
    auth:
      username: "your_username"
      password: "your_token"
```

### 2. 常用场景配置

#### GitHub 加速访问
```yaml
enable: true
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://mirror.ghproxy.com/"
      - pattern: "https://raw.githubusercontent.com/*"
        target: "https://raw.ghproxy.com/"
```

#### GitLab 镜像
```yaml
enable: true
units:
  - rules:
      - pattern: "https://gitlab.com/*"
        target: "https://gitlab-mirror.com/"
```

#### NPM 包管理器加速
```yaml
enable: true
units:
  - rules:
      - pattern: "https://registry.npmjs.org/*"
        target: "https://registry.npmmirror.com/"
```

### 3. 完整配置示例

```yaml
enable: true
units:
  # GitHub 配置
  - rules:
      - pattern: "https://github.com/*"
        target: "https://ghproxy.com/"
      - pattern: "https://raw.githubusercontent.com/*"
        target: "https://raw.ghproxy.com/"
    auth:
      username: "${GITHUB_USER}"
      password: "${GITHUB_TOKEN}"
    timeout:
      connect-timeout: 30
      read-timeout: 60
      total-timeout: 300
    proxy:
      url: "http://proxy.company.com:8080"

  # 其他服务配置
  - rules:
      - pattern: "https://api.example.com/*"
        target: "https://internal-api.example.com/"
```

## 配置参数说明

### 基本参数
- `enable`: `true` 或 `false`，是否启用网络访问控制
- `units`: 配置单元列表，每个单元包含重定向规则和配置

### 单元配置 (units)
每个 `unit` 包含：
- `rules`: 重定向规则列表
- `auth`: 可选的认证信息（用户名和密码）
- `timeout`: 可选的超时设置
- `proxy`: 可选的代理配置

### 规则配置 (rules)
每个 `rule` 包含：
- `pattern`: 要匹配的URL模式（支持 `*` 通配符）
- `target`: 重定向的目标地址

### 环境变量支持

您可以使用环境变量来动态配置，避免硬编码敏感信息：

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

环境变量语法：
- `${VARIABLE_NAME}`: 使用环境变量
- `${VARIABLE_NAME:default_value}`: 使用环境变量，如果不存在则使用默认值

## 使用方法

### 1. 设置环境变量（可选）

```bash
# Linux/Mac
export GITHUB_USER="your_username"
export GITHUB_TOKEN="your_token"
export PROXY_URL="http://proxy.company.com:8080"

# Windows
set GITHUB_USER=your_username
set GITHUB_TOKEN=your_token
set PROXY_URL=http://proxy.company.com:8080
```

### 2. 将配置文件放在正确位置

- 系统级配置：`/etc/net-access.yaml`
- 用户级配置：`~/.config/net-access.yaml`
- 项目级配置：`项目根目录/net-access.yaml`

### 3. 验证配置

配置完成后，您可以通过以下方式验证是否生效：

```bash
# 测试 GitHub 访问
curl -I "https://github.com/user/repo/releases"

# 查看是否重定向到镜像服务器
```

## 常见问题

### Q: 如何添加多个镜像服务器？
A: 在 `units` 中添加多个配置单元，系统会按顺序尝试：

```yaml
enable: true
units:
  # 第一个镜像
  - rules:
      - pattern: "https://github.com/*"
        target: "https://mirror1.github.com/"

  # 备用镜像
  - rules:
      - pattern: "https://github.com/*"
        target: "https://mirror2.github.com/"
```

### Q: 如何设置不同的超时时间？
A: 在 `timeout` 部分配置：

```yaml
timeout:
  connect-timeout: 30    # 连接超时（秒）
  read-timeout: 60       # 读取超时（秒）
  total-timeout: 300     # 总超时（秒）
```

### Q: 如何处理认证？
A: 在 `auth` 部分配置用户名和密码，推荐使用环境变量：

```yaml
auth:
  username: "${YOUR_USERNAME}"
  password: "${YOUR_PASSWORD}"
```

### Q: 配置不生效怎么办？
A: 检查以下几点：
1. 确保 `enable: true`
2. 检查配置文件路径是否正确
3. 验证 YAML 语法是否正确
4. 检查 URL 模式是否匹配

### Q: 如何配置代理？
A: 在 `proxy` 部分配置：

```yaml
proxy:
  url: "http://proxy.example.com:8080"
```

### Q: 支持哪些通配符？
A: 目前支持 `*` 通配符，可以匹配任意字符序列。例如：
- `https://github.com/*` 匹配所有 GitHub 地址
- `https://raw.githubusercontent.com/*` 匹配所有 GitHub 原始文件地址

## 配置示例合集

### 常用镜像服务

#### GitHub 全家桶加速
```yaml
enable: true
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://ghproxy.com/"
      - pattern: "https://raw.githubusercontent.com/*"
        target: "https://raw.ghproxy.com/"
      - pattern: "https://gist.github.com/*"
        target: "https://gist.ghproxy.com/"
```

#### Python 包管理器 (PyPI)
```yaml
enable: true
units:
  - rules:
      - pattern: "https://pypi.org/*"
        target: "https://pypi.doubanio.com/"
```

#### Docker 镜像加速
```yaml
enable: true
units:
  - rules:
      - pattern: "https://registry-1.docker.io/*"
        target: "https://dockerhub.azk8s.cn/"
```

#### RubyGems 加速
```yaml
enable: true
units:
  - rules:
      - pattern: "https://rubygems.org/*"
        target: "https://gems.ruby-china.com/"
```

### 企业内部配置

#### 内部服务映射
```yaml
enable: true
units:
  - rules:
      - pattern: "https://external-api.company.com/*"
        target: "https://internal-api.company.com/"
    auth:
      username: "${INTERNAL_API_USER}"
      password: "${INTERNAL_API_PASSWORD}"
    timeout:
      connect-timeout: 10
      read-timeout: 30
      total-timeout: 60
```

#### 多环境配置
```yaml
# 开发环境配置
enable: ${ENABLE_NET_ACCESS:true}
units:
  - rules:
      - pattern: "https://api.${ENV:dev}.company.com/*"
        target: "http://localhost:8080/"
    timeout:
      connect-timeout: 5
      read-timeout: 15
      total-timeout: 30
```

## 故障排除

### 检查配置文件语法

使用在线 YAML 验证工具检查配置文件语法：
1. 访问 https://www.yamllint.com/
2. 粘贴您的配置文件内容
3. 检查是否有语法错误

### 常见错误及解决方案

#### 1. 配置文件不生效
**症状**: 配置修改后没有效果
**解决方案**:
- 检查配置文件路径是否正确
- 确认 `enable: true`
- 重启应用程序
- 检查文件权限

#### 2. 环境变量未生效
**症状**: 环境变量没有正确替换
**解决方案**:
- 确认环境变量已正确设置
- 检查环境变量名称是否正确
- 使用 `echo $VARIABLE_NAME` 验证环境变量
- 重新启动终端或应用程序

#### 3. 网络连接超时
**症状**: 请求经常超时
**解决方案**:
- 增加 `timeout` 配置中的时间值
- 检查网络连接状态
- 尝试更换镜像服务器

#### 4. 认证失败
**症状**: 401 或 403 错误
**解决方案**:
- 检查用户名和密码是否正确
- 确认认证信息是否有权限访问目标服务
- 检查 token 是否已过期

### 调试技巧

#### 启用详细日志
如果应用程序支持日志，可以启用详细日志来查看重定向过程：
```bash
# 示例：启用调试日志
export RUST_LOG=debug
your_application
```

#### 手动测试重定向
使用 `curl` 命令手动测试重定向是否工作：
```bash
# 测试重定向
curl -v "https://github.com/user/repo"

# 查看是否被重定向到镜像服务器
```

#### 检查配置加载
如果可能，查看应用程序启动时的日志，确认配置文件是否正确加载。

## 最佳实践

### 安全性建议

1. **使用环境变量**: 避免在配置文件中硬编码敏感信息
2. **设置文件权限**: 确保配置文件只有授权用户可读
   ```bash
   chmod 600 net-access.yaml
   ```
3. **定期更新认证信息**: 定期更换密码和访问令牌
4. **使用 HTTPS**: 确保所有目标地址使用 HTTPS 协议

### 性能优化建议

1. **规则排序**: 将最常用的规则放在前面
2. **合理设置超时**: 根据网络环境调整超时时间
3. **使用就近镜像**: 选择地理位置较近的镜像服务器
4. **避免过度重定向**: 不要配置过多的重定向层级

### 维护建议

1. **版本控制**: 将配置文件纳入版本控制（排除敏感信息）
2. **文档记录**: 记录配置文件的用途和变更历史
3. **定期测试**: 定期测试配置是否仍然有效
4. **备份配置**: 保留配置文件的备份

## 获取帮助

如果遇到问题，可以通过以下方式获取帮助：

### 检查清单
在寻求帮助前，请先检查：
- [ ] 配置文件语法是否正确
- [ ] 环境变量是否正确设置
- [ ] 网络连接是否正常
- [ ] 认证信息是否有效
- [ ] 目标服务器是否可访问

### 常见资源
- **YAML 语法验证**: https://www.yamllint.com/
- **环境变量设置指南**: 搜索 "环境变量设置 [您的操作系统]"
- **网络连接测试**: 使用 `ping` 和 `curl` 命令测试
- **镜像服务状态**: 查看镜像服务的官方状态页面

### 联系支持
如果以上方法都无法解决问题，请联系技术支持并提供以下信息：
1. 操作系统和版本
2. 配置文件内容（去除敏感信息）
3. 错误信息或日志
4. 重现问题的步骤

---

## 附录

### 配置文件模板

#### 基础模板
```yaml
# NetAccessCtrl 基础配置模板
enable: true
units:
  - rules:
      - pattern: "https://example.com/*"
        target: "https://mirror.example.com/"
```

#### 完整模板
```yaml
# NetAccessCtrl 完整配置模板
enable: true
units:
  - rules:
      - pattern: "https://service1.com/*"
        target: "https://mirror1.service1.com/"
      - pattern: "https://service2.com/*"
        target: "https://mirror2.service2.com/"
    auth:
      username: "${SERVICE1_USER}"
      password: "${SERVICE1_PASSWORD}"
    timeout:
      connect-timeout: 30
      read-timeout: 60
      total-timeout: 300
    proxy:
      url: "${PROXY_URL:http://proxy.default:8080}"

  - rules:
      - pattern: "https://another-service.com/*"
        target: "https://internal.another-service.com/"
    # 此单元无认证、超时和代理配置
```

### 常用镜像服务器列表

| 服务类型 | 原地址 | 推荐镜像地址 |
|----------|--------|--------------|
| GitHub | `https://github.com/*` | `https://ghproxy.com/` |
| GitHub Raw | `https://raw.githubusercontent.com/*` | `https://raw.ghproxy.com/` |
| PyPI | `https://pypi.org/*` | `https://pypi.doubanio.com/` |
| NPM | `https://registry.npmjs.org/*` | `https://registry.npmmirror.com/` |
| Docker Hub | `https://registry-1.docker.io/*` | `https://dockerhub.azk8s.cn/` |
| RubyGems | `https://rubygems.org/*` | `https://gems.ruby-china.com/` |

*注意：镜像服务地址可能会发生变化，请以最新信息为准。*

---

**快速开始总结**：
1. 创建 `net-access.yaml` 文件
2. 复制相应场景的配置示例
3. 设置环境变量（可选）
4. 放置配置文件到正确位置
5. 验证配置是否生效

祝您使用愉快！🎉
