# 更新日志 (CHANGELOG)

## [0.6.0] - 2024-12-19

### 新增功能

#### 🔄 重定向服务增强
- **环境变量支持**: 为RedirectService及其所有组件添加完整的EnvEvalable能力
  - 支持在Rule的pattern和target字段中使用环境变量
  - 支持在AuthConfig的username和password字段中使用环境变量
  - 支持环境变量默认值语法：`${VAR:default_value}`
  - 支持嵌套环境变量解析

#### 📋 配置格式优化
- **简化配置结构**: 移除冗余的`direct_serv`顶级包装，直接使用`enable`和`units`作为顶级字段
- **向后兼容**: 保持所有现有API不变，确保平滑升级
- **配置验证**: 新增配置格式验证功能，提供更友好的错误提示

#### 🔧 开发者体验改进
- **详细文档**: 新增完整的重定向规则配置文档 (`docs/redirect-rules.md`)
- **丰富示例**: 提供GitHub镜像、企业代理、最小配置等多种使用场景示例
- **测试覆盖**: 新增27个单元测试，确保环境变量功能的稳定性

### 技术变更

#### 代码重构
- **模块化改进**: 将地址相关常量迁移到`addr::constants`模块
- **错误处理**: 优化AddrError类型，提供更具体的错误分类
- **代码质量**: 改进内部实现，提高可维护性

#### API更新
- **新增方法**: 
  - `RedirectService::from_str()` - 从字符串直接加载配置
  - `RedirectService::from_file()` - 从配置文件加载（重命名自try_from）
- **保持兼容**: 所有现有API保持不变，无需修改现有代码

### 配置示例

#### 环境变量使用
```yaml
# 支持环境变量的配置
enable: true
units:
  - rules:
      - pattern: "https://${DOMAIN:github.com}/*"
        target: "https://${MIRROR:ghproxy.com}/"
    auth:
      username: "${GITHUB_USER}"
      password: "${GITHUB_TOKEN}"
```

#### 最小配置
```yaml
enable: true
units:
  - rules:
      - pattern: "https://github.com/*"
        target: "https://mirror.github.com/"
```

### 破坏性变更
- 无破坏性变更，所有现有配置继续兼容

### 测试
- ✅ 27个测试全部通过
- ✅ 环境变量扩展功能测试覆盖
- ✅ 向后兼容性验证
- ✅ 配置格式验证测试

### 文档
- 📖 新增《地址重定向规则配置文档》
- 📚 更新API参考文档
- 🎯 提供完整配置示例和使用指南

---

## [0.5.9] - 2024-12-15

### 新增功能
- 初始重定向服务实现
- 支持通配符匹配规则
- 支持基本认证配置
- 提供配置文件支持