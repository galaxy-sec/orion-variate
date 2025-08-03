# 开发计划

## 模块说明
这是一个提供数据变量管理、模板处理、远程本地更新的RUST库



## 工作计划

### 分离 addr 模块  地址定义与地址的更新
[x] 提供 AddrAccessor 实现对地址的更新
    [] 提供 HttpAddrAccessor, 把 HttpAddr 的 update 逻辑 放置到 Accessor 里。分离 HttpAddr 中 redirect 也为实现逻辑。
    AddrAccessor 需要设计成为一个 Enum ，HttpAddrAccessor  是 AddrAccessor 的一个变体。
[x] 提供 Addr 里 Redirector 中规则的文档

