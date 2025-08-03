# Redirector 规则文档

## 概述

Redirector 模块提供了地址重定向和认证管理功能，用于在地址更新过程中对URL进行转换和认证处理。

## 核心组件

### 1. Rule（重定向规则）

`Rule` 结构体定义了URL重定向规则，支持通配符匹配。

#### 结构定义
```rust
pub struct Rule {
    pattern: String,    // 匹配模式（支持通配符*）
    target: String,      // 替换目标
}
```

#### 创建规则
```rust
use crate::addr::redirect::Rule;

// 创建重定向规则
let rule = Rule::new(
    "https://github.com/galaxy-sec/galaxy-flow*",
    "https://gflow.com"
);

// 应用规则
let new_url = rule.replace("https://github.com/galaxy-sec/galaxy-flow/releases/download/v0.8.5/file.tar.gz");
// 结果: "https://gflow.com/releases/download/v0.8.5/file.tar.gz"
```

#### 通配符规则
- 支持 `*` 通配符匹配任意字符
- 匹配成功后，将匹配部分附加到目标URL
- 精确匹配时直接替换整个字符串

#### 示例规则
| 模式 | 目标 | 输入 | 输出 |
|------|------|------|------|
| `https://github.com/user/repo*` | `https://mirror.com` | `https://github.com/user/repo/releases` | `https://mirror.com/releases` |
| `http://old-site.com/*` | `https://new-site.com` | `http://old-site.com/path/file.txt` | `https://new-site.com/path/file.txt` |
| `https://gitlab.com/*` | `https://gitlab-mirror.com` | `https://gitlab.com/user/project` | `https://gitlab-mirror.com/user/project` |

### 2. Auth（认证配置）

`Auth` 结构体用于存储HTTP基本认证信息。

#### 结构定义
```rust
pub struct Auth {
    username: String,  // 用户名
    password: String,  // 密码
}
```

#### 创建认证
```rust
use crate::addr::redirect::Auth;

// 创建认证配置
let auth = Auth::new("username", "password");

// 获取认证信息
let username = auth.username();  // "username"
let password = auth.password();  // "password"
```

### 3. 与AddrAccessor集成

`AddrAccessor` 可以与Redirector规则结合使用，实现地址转换和认证管理。

#### 使用示例
```rust
use crate::addr::{AddrAccessor, redirect::{Rule, Auth}};

// 创建地址访问器
let mut accessor = AddrAccessor::from_str("https://github.com/user/repo.git");

// 应用重定向规则
let rule = Rule::new("https://github.com/*", "https://mirror.com/");
// 在实际应用中，规则会由配置系统加载

// 设置认证信息
let auth = Auth::new("token", "your-access-token");
```

## 配置文件格式

### YAML配置示例
```yaml
redirect_rules:
  - pattern: "https://github.com/galaxy-sec/*"
    target: "https://internal-mirror.com/galaxy-sec/"
  - pattern: "https://gitlab.com/*"
    target: "https://gitlab-mirror.com/"

auth_configs:
  "https://github.com":
    username: "oauth2"
    password: "your-token"
  "https://gitlab.com":
    username: "oauth2"
    password: "your-gitlab-token"
```

### JSON配置示例
```json
{
  "redirect_rules": [
    {
      "pattern": "https://github.com/galaxy-sec/*",
      "target": "https://internal-mirror.com/galaxy-sec/"
    },
    {
      "pattern": "https://gitlab.com/*",
      "target": "https://gitlab-mirror.com/"
    }
  ],
  "auth_configs": {
    "https://github.com": {
      "username": "oauth2",
      "password": "your-token"
    }
  }
}
```

## 最佳实践

1. **规则优先级**：先定义具体规则，再定义通用规则
2. **认证安全**：使用环境变量存储敏感认证信息
3. **模式匹配**：使用精确的通配符模式避免意外匹配
4. **测试验证**：在生产环境前测试重定向规则的正确性

## 错误处理

- 规则不匹配时返回None，不影响原始地址
- 认证信息缺失时使用默认认证或无认证
- 配置错误时会记录警告日志，但不会影响程序运行