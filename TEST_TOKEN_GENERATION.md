# JWT Token 生成测试接口

## 概述

为了方便开发和测试，后端新增了两个测试接口，用于生成真实的 JWT token。这些 token 使用与生产环境相同的 EdDSA 算法签名，可以直接用于测试认证接口。

## 接口列表

### 1. 生成默认测试 Token（快速测试）

**接口**: `POST /v1/test/generate-token/default`

这是一个快速测试接口，使用预设的默认值生成 token。

**请求示例**:
```bash
curl -X POST http://localhost:8080/v1/test/generate-token/default \
  -H "Content-Type: application/json"
```

**响应示例**:
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9.eyJ1c2VyX2lkIjoidGVzdF91c2VyXzAwMSIsInVzZXJuYW1lIjoiYWxpY2UiLCJyb2xlIjoiQWRtaW4iLCJleHAiOjE3MzYxMjAwMDB9...",
    "user_id": "test_user_001",
    "username": "alice",
    "role": "Admin",
    "expires_at": "2025-01-05T12:00:00+00:00"
  }
}
```

**默认值**:
- User ID: `test_user_001`
- Username: `alice`
- Role: `Admin`
- 过期时间: 24小时后

---

### 2. 自定义测试 Token

**接口**: `POST /v1/test/generate-token`

允许自定义用户信息生成 token。

**请求参数**:
```json
{
  "user_id": "custom_user_123",
  "username": "bob",
  "role": "User"  // 可选: "Admin" 或 "User"
}
```

**请求示例**:
```bash
curl -X POST http://localhost:8080/v1/test/generate-token \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "custom_user_123",
    "username": "bob",
    "role": "User"
  }'
```

**响应示例**:
```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJFZERTQSJ9...",
    "user_id": "custom_user_123",
    "username": "bob",
    "role": "User",
    "expires_at": "2025-01-05T12:30:00+00:00"
  }
}
```

---

## 使用场景

### 1. 测试 /me 接口

```bash
# 1. 生成 token
TOKEN=$(curl -s -X POST http://localhost:8080/v1/test/generate-token/default | jq -r '.data.token')

# 2. 使用 token 测试 /me 接口
curl -X GET http://localhost:8080/v2/user/me \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json"
```

### 2. 在 App 模拟器中使用

打开 `scaffold/examples/app_simulator.html`，页面会自动调用后端接口生成真实的 token。

生成的 token 会自动填充到 "App Token" 输入框中，可以直接用于扫码登录测试。

### 3. 在 Postman 中测试

1. 先调用 `/v1/test/generate-token/default` 获取 token
2. 复制返回的 token
3. 在需要认证的接口（如 `/v2/user/me`）中添加 Authorization header：
   ```
   Authorization: Bearer <your_token>
   ```

### 4. 在 JavaScript 中使用

```javascript
// 生成 token
const response = await fetch('http://localhost:8080/v1/test/generate-token/default', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' }
});

const result = await response.json();
const token = result.data.token;

// 使用 token
const meResponse = await fetch('http://localhost:8080/v2/user/me', {
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  }
});

const userData = await meResponse.json();
console.log(userData);
```

---

## Token 验证测试

### 验证 Token 内容

使用 JWT 调试工具（如 jwt.io）可以查看 token 的内容：

**Header**:
```json
{
  "typ": "JWT",
  "alg": "EdDSA"
}
```

**Payload**:
```json
{
  "user_id": "test_user_001",
  "username": "alice",
  "role": "Admin",
  "exp": 1736120000
}
```

**Signature**: 使用 EdDSA 算法签名（后端配置的公钥/私钥对）

---

## 测试流程示例

### 完整的测试流程

```bash
# 1. 生成 Admin token
echo "=== 生成 Admin Token ==="
ADMIN_TOKEN=$(curl -s -X POST http://localhost:8080/v1/test/generate-token/default | jq -r '.data.token')
echo "Admin Token: ${ADMIN_TOKEN:0:50}..."

# 2. 测试 /me 接口
echo -e "\n=== 测试 /me 接口 ==="
curl -X GET http://localhost:8080/v2/user/me \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" | jq '.'

# 3. 生成普通 User token
echo -e "\n=== 生成 User Token ==="
USER_TOKEN=$(curl -s -X POST http://localhost:8080/v1/test/generate-token \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "username": "charlie",
    "role": "User"
  }' | jq -r '.data.token')
echo "User Token: ${USER_TOKEN:0:50}..."

# 4. 使用 User token 测试 /me 接口
echo -e "\n=== 使用 User Token 测试 /me 接口 ==="
curl -X GET http://localhost:8080/v2/user/me \
  -H "Authorization: Bearer $USER_TOKEN" \
  -H "Content-Type: application/json" | jq '.'
```

---

## 相关文件

- **后端接口实现**: `scaffold/src/backend/api/user/generate_test_token.rs`
- **App 模拟器**: `scaffold/examples/app_simulator.html`
- **JWT 工具**: `scaffold/src/backend/utils/jwt.rs`
- **/me 接口**: `scaffold/src/backend/api/user/get_me.rs`

---

## 安全注意事项

⚠️ **重要提示**:

1. **仅用于测试**: 这些接口仅用于开发和测试环境
2. **生产环境禁用**: 在生产环境中应该禁用或删除这些接口
3. **不要暴露**: 确保这些接口不能被公开访问
4. **Token 过期**: 生成的 token 会在 24 小时后过期
5. **密钥管理**: 测试环境使用的密钥应该与生产环境不同

---

## 常见问题

### Q: 为什么不使用前端生成的假 token？

A: 前端生成的 token 使用的是伪造的签名，无法通过后端的 EdDSA 签名验证。新的接口使用与生产环境相同的算法和密钥，生成的 token 可以正常验证。

### Q: Token 过期了怎么办？

A: Token 默认 24 小时后过期。你可以：
1. 重新调用生成接口获取新 token
2. 或者在接口代码中修改 `expires_at` 的值

### Q: 如何修改默认的用户信息？

A: 编辑 `generate_test_token.rs` 中的 `generate_default_test_token` 函数，修改默认值。

### Q: 可以生成不同角色的 token 吗？

A: 可以。在调用 `/v1/test/generate-token` 时，指定 `role` 参数为 `"Admin"` 或 `"User"`。

---

## 测试检查清单

- [x] 生成默认 Admin token
- [x] 生成自定义 User token
- [x] 使用 token 访问 /me 接口
- [x] 验证 token 内容正确
- [x] 测试 token 过期机制
- [x] 在 App 模拟器中使用生成的 token
- [x] 测试无效 token 的错误处理

---

## 快速开始

```bash
# 1. 启动后端服务
cargo run

# 2. 在另一个终端生成 token
curl -X POST http://localhost:8080/v1/test/generate-token/default | jq '.'

# 3. 复制返回的 token，测试 /me 接口
export TOKEN="粘贴你的token"
curl -X GET http://localhost:8080/v2/user/me \
  -H "Authorization: Bearer $TOKEN"

# 4. 或者打开浏览器测试
open scaffold/examples/qr_login_websocket.html
```

🎉 现在你可以轻松测试 JWT 认证功能了！
