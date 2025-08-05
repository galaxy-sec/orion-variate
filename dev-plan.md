# 开发计划

## 模块说明
这是一个提供数据变量管理、模板处理、远程本地更新的RUST库


## 工作规则
- 工作任务结束，要把反馈写到此文档中。
- 对于设计型的任务，需要先写出设计方案文档，放置到tasks 目录下。
- 任务的方案，经过评审后，才能开始实现。

## 工作计划

### 分离 addr 模块  地址定义与地址的更新
[x] 提供 AddrAccessor 实现对地址的更新
[x] 提供 HttpAddrAccessor, 把 HttpAddr 的 update 逻辑 放置到 Accessor 里。分离 HttpAddr 中 redirect 也为实现逻辑。
    AddrAccessor 需要设计成为一个 Enum ，HttpAddrAccessor  是 AddrAccessor 的一个变体。
[x] 提供 Addr 里  redirect 规则 说明文档， 用于配置文件的编写



### 为RedirectService 的units 提供 EnvEvalable 能力
[x] 方案已经设计好，放到tasks/redirect-service-env-eval-design.md 中。
[ ] 执行设计方案

### 改进addr 的代码质量
[x]创建addr::constants模块管理常量
[x]添加配置验证方法
[]改进测试覆盖率和命名
[]添加详细的日志记录
[]重构错误类型，增加具体错误分类,需要先设计出方案，确认后再行动
[] 为所有公共API添加RustDoc文档