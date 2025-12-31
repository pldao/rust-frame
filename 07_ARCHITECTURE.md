# API 架构设计

## 🏗 架构概览

本项目采用**版本化路由**设计，按照是否需要认证将API分为两个版本：

```
├─ v1/ ────────── 公开API（无需认证）
│  ├─ /ping
│  ├─ /auth/*
│  ├─ /code/*
│  ├─ /qr-login/*
│  └─ /ws/qr/{session_id}
│
└─ v2/ ────────── 需要认证的API
   ├─ /user/*
   ├─ /admin/*
   └─ ...（未来扩展）
```

---

## 📋 v1 API（公开接口）

### 基础路径
```
http://localhost:8080/v1
```

### 端点列表

| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/v1/ping` | GET | 健康检查 | ❌ |
| `/v1/auth/register` | POST | 用户注册 | ❌ |
| `/v1/auth/login` | POST | 用户登录 | ❌ |
| `/v1/code/send-email` | POST | 发送邮箱验证码 | ❌ |
| `/v1/code/send-phone` | POST | 发送手机验证码 | ❌ |
| `/v1/qr-login/generate` | POST | 生成二维码 | ❌ |
| `/v1/qr-login/status/{id}` | GET | 查询登录状态 | ❌ |
| `/v1/qr-login/confirm` | POST | 确认登录（App端） | ❌ |
| `/v1/ws/qr/{session_id}` | WebSocket | 实时推送状态 | ❌ |

### 特点

- ✅ **无需认证** - 直接访问
- ✅ **快速响应** - 不经过JWT验证
- ✅ **公开服务** - 用于注册、登录、验证码等

---

## 🔐 v2 API（需要认证）

### 基础路径
```
http://localhost:8080/v2
```

### 认证方式

所有v2接口都需要在请求头中携带JWT Token：

```http
Authorization: Bearer eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...
```

### 端点列表（示例）

| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/v2/user/profile` | GET | 获取用户信息 | ✅ |
| `/v2/user/update` | PUT | 更新用户信息 | ✅ |
| `/v2/admin/users` | GET | 管理员查看用户列表 | ✅ |
| `/v2/admin/logs` | GET | 查看操作日志 | ✅ |

### 特点

- 🔒 **需要JWT认证** - 所有请求必须携带有效Token
- 🛡 **权限控制** - 可根据用户角色进行细粒度控制
- 📊 **用户操作** - 需要用户身份的业务逻辑

---

## 🎯 设计优势

### 1. **清晰的边界**
```rust
// v1: 公开接口
web::scope("/v1")
    .wrap(Timed)  // 只有计时中间件
    .service(...)

// v2: 需要认证
web::scope("/v2")
    .wrap(Timed)
    .wrap(Auth)   // 增加认证中间件
    .service(...)
```

### 2. **易于维护**
- ✅ 不需要在Auth中间件中维护忽略列表
- ✅ 路由结构一目了然
- ✅ 新增接口时明确知道放在哪个版本

### 3. **性能优化**
- ✅ 公开接口不经过JWT验证，响应更快
- ✅ 减少不必要的中间件开销

### 4. **向后兼容**
- ✅ 可以同时保留v1和v2
- ✅ 未来可以添加v3、v4等新版本
- ✅ 方便API版本升级和迁移

---

## 📊 中间件执行顺序

### v1 请求流程
```
Request → CORS → Logger → Timed → Handler → Response
```

### v2 请求流程
```
Request → CORS → Logger → Timed → Auth → Handler → Response
                                      ↑
                                 JWT验证
```

---

## 🔄 扫码登录特殊说明

### 为什么扫码登录放在v1？

1. **生成二维码** - 任何人都可以生成，无需登录
2. **WebSocket连接** - 公开连接，等待状态推送
3. **查询状态** - 轮询状态，无需认证
4. **确认登录** - 虽然App端需要token，但在请求体中验证，不在中间件层

```javascript
// 前端直接访问v1接口
fetch('http://localhost:8080/v1/qr-login/generate')

// WebSocket连接
new WebSocket('ws://localhost:8080/v1/ws/qr/{session_id}')
```

---

## 📝 添加新接口示例

### 添加公开接口（v1）

```rust
// 1. 在 v1 scope 中添加
web::scope("/v1")
    .route("/new-public-api", web::get().to(handler))
```

### 添加需要认证的接口（v2）

```rust
// 1. 在 v2 scope 中添加
web::scope("/v2")
    .wrap(Auth)  // 自动认证
    .route("/new-protected-api", web::get().to(handler))
```

---

## 🧪 测试示例

### 测试v1接口（无需认证）

```bash
# 健康检查
curl http://localhost:8080/v1/ping

# 生成二维码
curl -X POST http://localhost:8080/v1/qr-login/generate \
  -H "Content-Type: application/json" \
  -d '{}'
```

### 测试v2接口（需要认证）

```bash
# 获取用户信息（需要Token）
curl http://localhost:8080/v2/user/profile \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 🚀 迁移指南

### 从旧版本迁移

**旧版本路径：**
```
/ping
/qr-login/generate
/ws/qr/{session_id}
```

**新版本路径：**
```
/v1/ping
/v1/qr-login/generate
/v1/ws/qr/{session_id}
```

### 前端代码更新

```javascript
// 旧版本
const API_BASE = 'http://localhost:8080';

// 新版本
const API_BASE = 'http://localhost:8080/v1';
```

---

## 📚 总结

| 特性 | v1 | v2 |
|------|----|----|
| **认证** | ❌ 不需要 | ✅ 需要JWT |
| **中间件** | Logger + Timed | Logger + Timed + Auth |
| **用途** | 公开服务 | 用户业务 |
| **性能** | 更快 | 需验证 |
| **示例** | 注册、登录、验证码、扫码 | 用户信息、管理功能 |

---

**版本：** 1.0.0  
**更新时间：** 2024-11-19  
**架构设计：** v1公开 + v2认证 双版本路由
