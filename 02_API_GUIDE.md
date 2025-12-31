# API 接口文档

> **版本**: v1.0 | **基础路径**: `http://localhost:8080/v1`

## 📋 目录

1. [概述](#概述)
2. [接口列表](#接口列表)
3. [请求/响应格式](#请求响应格式)
4. [错误码](#错误码)
5. [接口详情](#接口详情)

---

## 概述

### 基础信息

- **Base URL**: `http://localhost:8080/v1`
- **Content-Type**: `application/json`
- **字符编码**: `UTF-8`

### WebSocket URL

- **WS Base**: `ws://localhost:8080/v1`
- **端点**: `/ws/qr/{session_id}`

---

## 接口列表

| 序号 | 接口 | 方法 | 说明 |
|------|------|------|------|
| 1 | `/qr-login/generate` | POST | 生成二维码 |
| 2 | `/qr-login/status/{session_id}` | GET | 查询登录状态 |
| 3 | `/qr-login/confirm` | POST | 确认登录 |
| 4 | `/ws/qr/{session_id}` | WS | WebSocket实时推送 |

---

## 请求/响应格式

### 标准成功响应

```json
{
  "code": 0,
  "msg": "操作成功",
  "data": { ... }
}
```

### 标准错误响应

```json
{
  "code": 1002,
  "msg": "错误描述",
  "path": "/api/endpoint"
}
```

---

## 错误码

详细错误码说明请参考：[04_ERROR_CODES.md](04_ERROR_CODES.md)

### 常用错误码

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 1002 | Token不能为空 |
| 1003 | 无效的Token |
| 1200 | 资源不存在 |
| 1300 | 二维码不存在 |
| 1301 | 二维码已过期 |

---

## 接口详情

### 1. 生成二维码

**接口**: `POST /qr-login/generate`

**请求参数**:
```json
{
  "client_info": "web-client-v1"  // 可选，客户端信息
}
```

**响应示例**:
```json
{
  "code": 0,
  "msg": "操作成功",
  "data": {
    "session_id": "abc-123-def-456",
    "qr_image": "data:image/png;base64,iVBORw0KGgo...",
    "qr_data": "{\"session_id\":\"abc-123\",\"action\":\"login\"}",
    "expires_in": 300
  }
}
```

**字段说明**:
- `session_id`: 会话ID，唯一标识本次登录
- `qr_image`: Base64编码的PNG图片
- `qr_data`: 二维码包含的JSON数据
- `expires_in`: 有效期（秒），默认300秒（5分钟）

---

### 2. 查询登录状态

**接口**: `GET /qr-login/status/{session_id}`

**路径参数**:
- `session_id`: 会话ID

**响应示例（pending）**:
```json
{
  "code": 0,
  "msg": "操作成功",
  "data": {
    "status": "pending",
    "web_token": null,
    "message": "Waiting for scan"
  }
}
```

**响应示例（confirmed）**:
```json
{
  "code": 0,
  "msg": "操作成功",
  "data": {
    "status": "confirmed",
    "web_token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
    "message": "Login successful"
  }
}
```

**状态值**:
- `pending`: 等待扫码
- `scanned`: 已扫码，等待确认
- `confirmed`: 登录成功
- `rejected`: 用户拒绝
- `expired`: 已过期

---

### 3. 确认登录

**接口**: `POST /qr-login/confirm`

**请求参数**:
```json
{
  "session_id": "abc-123-def-456",
  "app_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
```

**字段说明**:
- `session_id`: 会话ID
- `app_token`: App端的JWT token，必须包含有效的用户信息

**Token格式**:
```json
{
  "user_id": "user_123",
  "username": "testuser",
  "role": "user",
  "exp": 1735689600,
  "iat": 1735603200
}
```

**成功响应**:
```json
{
  "code": 0,
  "msg": "操作成功",
  "data": {
    "success": true,
    "message": "Login confirmed successfully"
  }
}
```

**错误响应**:
```json
{
  "code": 1300,
  "msg": "二维码不存在"
}
```

---

### 4. WebSocket实时推送

**接口**: `WS /ws/qr/{session_id}`

**连接**:
```javascript
const ws = new WebSocket('ws://localhost:8080/v1/ws/qr/abc-123-def-456');
```

**消息格式**:

**连接成功**:
```json
{
  "status": "connected",
  "message": "Waiting for confirmation"
}
```

**登录成功**:
```json
{
  "status": "confirmed",
  "message": "Login successful",
  "web_token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
```

**二维码过期**:
```json
{
  "status": "expired",
  "message": "QR code expired"
}
```

**用户拒绝**:
```json
{
  "status": "rejected",
  "message": "Login rejected by user"
}
```

---

## 使用示例

### JavaScript (Fetch)

```javascript
// 生成二维码
const response = await fetch('http://localhost:8080/v1/qr-login/generate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ client_info: 'web-client' })
});

const data = await response.json();
console.log(data.data.session_id);
```

### curl

```bash
# 生成二维码
curl -X POST http://localhost:8080/v1/qr-login/generate \
  -H "Content-Type: application/json" \
  -d '{"client_info":"test"}' | jq '.'

# 查询状态
curl http://localhost:8080/v1/qr-login/status/abc-123-def-456 | jq '.'

# 确认登录
curl -X POST http://localhost:8080/v1/qr-login/confirm \
  -H "Content-Type: application/json" \
  -d '{"session_id":"abc-123","app_token":"YOUR_TOKEN"}' | jq '.'
```

---

## WebSocket示例

### JavaScript

```javascript
const sessionId = 'abc-123-def-456';
const ws = new WebSocket(`ws://localhost:8080/v1/ws/qr/${sessionId}`);

ws.onopen = () => {
  console.log('✅ WebSocket已连接');
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('📩 收到消息:', data);

  if (data.status === 'confirmed') {
    console.log('✅ 登录成功!');
    console.log('Token:', data.web_token);
  }
};

ws.onerror = (error) => {
  console.error('❌ WebSocket错误:', error);
};

ws.onclose = () => {
  console.log('🔌 WebSocket已关闭');
};
```

---

## 注意事项

### Token验证

- App Token必须是有效的JWT
- Token必须包含user_id字段
- Token不能过期
- 签名必须正确

### Session管理

- Session默认有效期5分钟
- 过期后无法确认登录
- 确认后的Session无法再次使用
- Session ID使用UUID格式

### 限流建议

- 生成二维码：同一用户限制1次/秒
- 状态查询：同一Session限制10次/秒
- 确认登录：同一Session限制1次/秒

---

## 相关文档

- [04_ERROR_CODES.md](04_ERROR_CODES.md) - 完整错误码说明
- [03_QR_LOGIN.md](03_QR_LOGIN.md) - QR登录功能详解
- [05_WEBSOCKET.md](05_WEBSOCKET.md) - WebSocket使用指南

---

**版本**: v1.0
**最后更新**: 2025-12-31
