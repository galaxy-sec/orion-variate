# 代码覆盖率服务迁移指南

## 🚀 问题背景

由于Codecov服务存在的限速问题，我们提供了多种替代方案来解决代码覆盖率上传问题。

## 📊 建议的替代方案

### 方案1: Coveralls（推荐）
- **优点**: 与Codecov功能相似，使用简单
- **设置**: 已在`.github/workflows/ci.yml`中配置
- **激活**: 需要配置GitHub密钥（见下方说明）

### 方案2: GitHub Artifacts（完全无外部依赖）
- **优点**: 不依赖任何外部服务，100% GitHub原生
- **局限**: 无在线覆盖率显示，仅提供下载
- **查看**: Actions运行结果中下载lcov文件

### 方案3: Code Climate
- **优点**: 提供代码质量和覆盖率双重报告
- **设置**: 需要额外的环境变量配置

## 🔧 配置步骤

### 使用Coveralls（推荐）

1. 访问 [Coveralls官网](https://coveralls.io)
2. 使用GitHub账号登录
3. 导入你的仓库
4. 复制仓库的token
5. 在GitHub仓库设置中添加密钥：
   - 名称：`COVERALLS_REPO_TOKEN`
   - 值：从Coveralls复制的token

### 使用Code Climate

1. 访问 [Code Climate](https://codeclimate.com)
2. 导入你的仓库
3. 获取`CC_TEST_REPORTER_ID`
4. 在GitHub仓库设置中添加密钥：
   - 名称：`CC_TEST_REPORTER_ID`
   - 值：从Code Climate获取的ID

### 使用GitHub Artifacts（最稳定）

无需额外配置，已经内置在CI中。覆盖率报告会作为构建工件保存30天。

## 🔄 如何切换方案

修改`.github/workflows/ci.yml`文件的最后部分，选择你需要的方案：

```yaml
# 取消注释你想要的方案，注释掉其他方案

# 方案1: Coveralls（需要配置token）
- name: Upload to Coveralls
  uses: coverallsapp/github-action@v2
  with:
    file: lcov.info
    format: lcov
    github-token: ${{ secrets.GITHUB_TOKEN }}

# 方案2: GitHub Artifacts（无需配置）
- name: Upload to GitHub Artifacts
  uses: actions/upload-artifact@v4
  with:
    name: coverage-data
    path: lcov.info
    retention-days: 30

# 方案3: 仅生成本地报告
- name: Generate HTML report
  run: |
    cargo llvm-cov --all-features --workspace --html --output-dir coverage-report
- name: Upload HTML report
  uses: actions/upload-artifact@v4
  with:
    name: coverage-html
    path: coverage-report/
    retention-days: 30
```

## 📱 覆盖率徽章

如果使用Coveralls，更新README.md中的徽章：

```markdown
[![Coverage Status](https://coveralls.io/repos/github/{user}/{repo}/badge.svg?branch=main)](https://coveralls.io/github/{user}/{repo}?branch=main)
```

## 🎯 下一步

1. **立即生效**: GitHub Artifacts已经可用
2. **推荐配置**: 设置Coveralls以获得更好的体验
3. **监控**: 观察1-2周，确保新方案稳定运行

## 📞 遇到问题？

如果在配置过程中遇到问题：
- 检查GitHub仓库的Secrets设置
- 查看Actions的运行日志
- 参考各服务商的官方文档

## 🌟 最佳实践

对于大多数项目，推荐按以下优先级选择：
1. **Coveralls** - 功能完整，界面友好
2. **GitHub Artifacts** - 稳定可靠，零故障
3. **仅本地模式** - 最大化稳定性