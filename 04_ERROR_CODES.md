# 错误码系统文档

本文档描述了项目中统一使用的错误码系统。

## 概述

项目使用统一的错误码系统，所有 API 响应都遵循标准的 JSON 格式。错误码按照类别进行组织，便于识别和管理。

## 错误响应格式

### 错误响应

所有错误响应都遵循以下格式：

```json
{
  "code": 1002,
  "msg": "token不能为空",
  "path": "/api/endpoint",
  "details": "可选的详细错误信息"
}
```

### 成功响应

成功响应格式：

```json
{
  "code": 0,
  "msg": "操作成功",
  "data": {
    // 响应数据
  }
}
```

## 错误码分类

### 成功 (0)

| 错误码 | 常量名 | 说明 |
|--------|--------|------|
| 0 | `Success` | 操作成功 |

### 客户端错误 (1000-1999)

#### 认证相关 (1001-1099)

| 错误码 | 常量名 | HTTP状态码 | 说明 |
|--------|--------|-----------|------|
| 1001 | `Unauthorized` | 401 | 未授权 |
| 1002 | `TokenMissing` | 401 | token不能为空 |
| 1003 | `TokenInvalid` | 401 | 无效的token |
| 1004 | `TokenExpired` | 401 | token已过期 |
| 1005 | `LoginFailed` | 401 | 登录失败 |
| 1006 | `PermissionDenied` | 401 | 权限不足 |

#### 请求相关 (1100-1199)

| 错误码 | 常量名 | HTTP状态码 | 说明 |
|--------|--------|-----------|------|
| 1100 | `BadRequest` | 400 | 错误的请求 |
| 1101 | `InvalidParams` | 400 | 无效的参数 |
| 1102 | `MissingRequiredField` | 400 | 缺少必填字段 |
| 1103 | `InvalidFormat` | 400 | 格式错误 |
| 1104 | `RateLimitExceeded` | 400 | 请求过于频繁，请稍后再试 |

#### 资源相关 (1200-1299)

| 错误码 | 常量名 | HTTP状态码 | 说明 |
|--------|--------|-----------|------|
| 1200 | `NotFound` | 404 | 资源不存在 |
| 1201 | `ResourceExpired` | 400 | 资源已过期 |
| 1202 | `ResourceAlreadyExists` | 400 | 资源已存在 |
| 1203 | `ResourceConflict` | 400 | 资源冲突 |

#### 二维码登录相关 (1300-1399)

| 错误码 | 常量名 | HTTP状态码 | 说明 |
|--------|--------|-----------|------|
| 1300 | `QRCodeNotFound` | 404 | 二维码不存在 |
| 1301 | `QRCodeExpired` | 400 | 二维码已过期 |
| 1302 | `QRCodePending` | 200 | 等待扫码 |
| 1303 | `QRCodeScanned` | 200 | 已扫码，等待确认 |
| 1304 | `QRCodeRejected` | 200 | 用户拒绝登录 |

#### 邮件相关 (1400-1499)

| 错误码 | 常量名 | HTTP状态码 | 说明 |
|--------|--------|-----------|------|
| 1400 | `EmailSendFailed` | 400 | 邮件发送失败 |
| 1401 | `EmailCodeInvalid` | 400 | 验证码错误 |
| 1402 | `EmailCodeExpired` | 400 | 验证码已过期 |
| 1403 | `EmailRateLimitExceeded` | 400 | 邮件发送过于频繁，请稍后再试 |

### 服务器错误 (2000-2999)

| 错误码 | 常量名 | HTTP状态码 | 说明 |
|--------|--------|-----------|------|
| 2000 | `InternalError` | 500 | 内部服务器错误 |
| 2001 | `DatabaseError` | 500 | 数据库错误 |
| 2002 | `NetworkError` | 500 | 网络错误 |
| 2003 | `ConfigurationError` | 500 | 配置错误 |
| 2004 | `ServiceUnavailable` | 500 | 服务暂时不可用 |

## 使用指南

### 基本用法

在 API 处理器中使用错误码：

```rust
use crate::backend::errors::{ErrorCode, error_response, SuccessResponse};

// 返回错误响应
let error_resp = error_response(
    ErrorCode::TokenInvalid,
    "无效的token"
);
return HttpResponse::Unauthorized().json(error_resp);

// 返回成功响应
HttpResponse::Ok().json(SuccessResponse::new(data))
```

### 自定义错误消息

```rust
let error_resp = error_response(
    ErrorCode::DatabaseError,
    format!("Failed to connect: {}", err)
);
```

### 包含路径信息的错误响应

```rust
use crate::backend::errors::error_response_with_path;

let error_resp = error_response_with_path(
    ErrorCode::TokenMissing,
    "token不能为空",
    req.path().to_string()
);
```

### 使用 AppError

```rust
use crate::backend::errors::AppError;

// 创建自定义错误
return Err(AppError::auth("用户未登录"));

// 数据库错误会自动转换
let user = users::Entity::find()
    .one(&db)
    .await?
    .ok_or_else(|| AppError::not_found("用户不存在"))?;
```

## 错误码设计原则

1. **分类明确**：按功能模块分类，便于管理
2. **易于扩展**：每个类别预留足够的错误码空间
3. **HTTP 映射**：每个错误码都有对应的 HTTP 状态码
4. **消息一致**：使用统一的中文错误消息

## 添加新错误码

如需添加新的错误码，请按以下步骤：

1. 在 `ErrorCode` 枚举中添加新的错误码变体
2. 在 `default_message()` 方法中添加对应的错误消息
3. 在 `http_status_code()` 方法中添加对应的 HTTP 状态码
4. 更新本文档

示例：

```rust
pub enum ErrorCode {
    // ... 现有错误码 ...

    /// 新增错误码
    NewError = 1500,
}

impl ErrorCode {
    pub fn default_message(&self) -> &str {
        match self {
            // ... 现有消息 ...

            ErrorCode::NewError => "新错误的描述",
        }
    }

    pub fn http_status_code(&self) -> u16 {
        match self {
            // ... 现有状态码映射 ...

            ErrorCode::NewError => 400,
        }
    }
}
```

## 迁移指南

### 从旧错误码迁移

旧的错误码系统：
- 只有错误码 `2` 用于所有认证错误
- 错误消息格式不统一

新的错误码系统：
- 分类明确的错误码（如 1002, 1003 等）
- 统一的 JSON 响应格式

迁移示例：

```rust
// 旧代码
return Err(error::ErrorUnauthorized(json!({
    "msg": "token不能为空",
    "code": 2,
    "path": path
})));

// 新代码
let error_resp = error_response_with_path(
    ErrorCode::TokenMissing,
    ErrorCode::TokenMissing.default_message(),
    path,
);
Err(error::ErrorUnauthorized(json!(error_resp)))
```

## 相关文件

- 错误定义: `scaffold/src/backend/errors.rs`
- 使用示例: `scaffold/src/backend/middleware/auth_middleware.rs`
- API 处理器: `scaffold/src/backend/api/`

## 维护说明

- 添加新错误码时，请确保更新本文档
- 错误码一经使用，不应修改其数值（避免破坏客户端兼容性）
- 如需废弃某个错误码，应保留其定义并标记为 deprecated
