# /me 接口测试指南

## 接口说明

`/me` 接口用于获取当前登录用户的信息，主要用于测试 JWT 认证是否正常工作。

- **路径**: `GET /v2/user/me`
- **认证**: 需要 JWT token（在 Authorization header 中）
- **响应**: 返回 token 中的用户信息

## 接口详情

### 请求

**Method**: `GET`

**URL**: `http://localhost:8080/v2/user/me`

**Headers**:
```
Authorization: Bearer YOUR_JWT_TOKEN
Content-Type: application/json
```

### 成功响应 (200 OK)

```json
{
  "code": 0,
  "message": "success",
  "path": "/v2/user/me",
  "data": {
    "user_id": "user123",
    "username": "alice",
    "role": "Admin",
    "exp": 1735689600
  }
}
```

**字段说明**:
- `user_id`: 用户唯一标识
- `username`: 用户名（或链上地址）
- `role`: 用户角色（Admin/User 等）
- `exp`: token 过期时间戳（Unix 时间）

### 错误响应

#### 1. Token 缺失 (401 Unauthorized)

```json
{
  "code": 1002,
  "msg": "Authorization header is missing or invalid"
}
```

**错误码**: 1002 (TokenMissing)

#### 2. Token 无效 (401 Unauthorized)

```json
{
  "code": 1003,
  "msg": "Invalid or expired token"
}
```

**错误码**: 1003 (TokenInvalid)

## 测试步骤

### 步骤 1: 生成 JWT Token

首先，你需要通过其他接口（如登录、注册、二维码登录）获取 JWT token。

例如，通过二维码登录：

```bash
# 1. 生成二维码
curl -X POST http://localhost:8080/v1/qr-login/generate

# 响应
{
  "code": 0,
  "message": "success",
  "data": {
    "session_id": "xxx",
    "expires_at": "xxx"
  }
}

# 2. 通过其他设备确认登录，获取 token
#（这部分需要使用 WebSocket 或其他接口完成）
```

### 步骤 2: 测试 /me 接口

使用获取到的 token 测试 `/me` 接口：

```bash
curl -X GET http://localhost:8080/v2/user/me \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### 步骤 3: 测试错误情况

#### 测试没有 token 的情况：

```bash
curl -X GET http://localhost:8080/v2/user/me
```

预期返回 401 错误，提示 token 缺失。

#### 测试无效 token 的情况：

```bash
curl -X GET http://localhost:8080/v2/user/me \
  -H "Authorization: Bearer invalid_token_12345"
```

预期返回 401 错误，提示 token 无效。

## 使用 Postman 测试

### 1. 创建新请求

- Method: `GET`
- URL: `http://localhost:8080/v2/user/me`

### 2. 添加 Authorization Header

在 Headers 标签页中添加：
```
Authorization: Bearer YOUR_JWT_TOKEN
```

### 3. 发送请求

点击 Send 按钮，查看响应。

## 使用代码测试

### Rust 示例

```rust
use reqwest::Client;

async fn get_me_info(token: &str) -> Result<serde_json::Value, reqwest::Error> {
    let client = Client::new();
    let response = client
        .get("http://localhost:8080/v2/user/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let json = response.json().await?;
    Ok(json)
}
```

### JavaScript/Node.js 示例

```javascript
const axios = require('axios');

async function getMeInfo(token) {
  try {
    const response = await axios.get('http://localhost:8080/v2/user/me', {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    });
    return response.data;
  } catch (error) {
    console.error('Error:', error.response.data);
  }
}
```

### Python 示例

```python
import requests

def get_me_info(token):
    url = 'http://localhost:8080/v2/user/me'
    headers = {
        'Authorization': f'Bearer {token}'
    }

    response = requests.get(url, headers=headers)
    return response.json()

# 使用示例
token = 'YOUR_JWT_TOKEN'
result = get_me_info(token)
print(result)
```

## JWT Token 自动续期

中间件会在 token 即将过期（剩余时间 < 1小时）时自动续签。续签后的新 token 会在响应的 `Authorization` header 中返回：

```
Authorization: Bearer NEW_JWT_TOKEN
```

客户端应该检查这个 header，如果存在则更新本地存储的 token。

## 相关代码

- 接口实现: `scaffold/src/backend/api/user/get_me.rs`
- 用户模块: `scaffold/src/backend/api/user/mod.rs`
- JWT 工具: `scaffold/src/backend/utils/jwt.rs`
- 认证中间件: `scaffold/src/backend/middleware/auth_middleware.rs`

## 常见问题

### Q: 为什么返回 401 错误？

A: 可能的原因：
1. 没有提供 Authorization header
2. Authorization header 格式错误（应该是 `Bearer <token>`）
3. Token 已经过期
4. Token 签名无效

### Q: 如何查看 token 中的内容？

A: 使用 `/me` 接口即可查看 token 中的用户信息。或者使用 JWT 调试工具（如 jwt.io）解码 token（注意：不要在生产环境中将敏感 token 输入到在线工具）。

### Q: Token 过期了怎么办？

A: 如果 token 还有至少 1 小时有效期，中间件会自动续签。检查响应的 `Authorization` header 获取新 token。如果已经过期，需要重新登录获取新 token。

## 安全建议

1. **HTTPS**: 在生产环境中务必使用 HTTPS 传输 token
2. **存储**: 客户端应安全存储 token（如使用 HttpOnly cookie 或安全存储）
3. **过期**: 设置合理的 token 过期时间
4. **刷新**: 实现完善的 token 刷新机制
5. **撤销**: 考虑实现 token 黑名单机制以支持主动撤销
