# Rust Frame - QR登录系统

> 🚀 基于 Actix-Web + SeaORM + PostgreSQL 的扫码登录实现

## ✨ 特性

- **后端生成二维码** - 无需前端依赖，直接返回Base64图片
- **WebSocket实时推送** - 登录状态即时更新，响应时间<100ms
- **完整测试套件** - 单元测试、集成测试、E2E测试
- **统一错误码系统** - 标准化的API响应格式
- **零依赖前端** - 打开HTML即可测试

---

## 📖 文档导航

| 文档 | 说明 |
|------|------|
| [01_QUICK_START.md](01_QUICK_START.md) | **⭐ 快速开始** - 3分钟完成本地测试 |
| [02_API_GUIDE.md](02_API_GUIDE.md) | API接口文档 |
| [03_QR_LOGIN.md](03_QR_LOGIN.md) | QR登录功能详解 |
| [04_ERROR_CODES.md](04_ERROR_CODES.md) | 错误码完整说明 |
| [05_WEBSOCKET.md](05_WEBSOCKET.md) | WebSocket使用指南 |
| [06_TESTING.md](06_TESTING.md) | 测试完整指南 |
| [07_ARCHITECTURE.md](07_ARCHITECTURE.md) | 系统架构设计 |

---

## 🚀 快速开始

### 方式1：使用模拟器（推荐）

```bash
# 1. 启动后端
cargo run -- --backend-port 8080

# 2. 打开Web端
# 在浏览器中打开: scaffold/examples/qr_login_websocket.html
# 点击"生成二维码"

# 3. 打开App模拟器
# 在新标签页打开: scaffold/examples/app_simulator.html
# 复制Session ID，点击"确认登录"

# 4. 观察Web端自动更新为"登录成功"✅
```

### 方式2：使用curl

```bash
# 生成token
python3 tests/local/generate_test_token.py

# 完整流程
SESSION_ID=$(curl -s -X POST http://localhost:8080/v1/qr-login/generate \
  -H "Content-Type: application/json" \
  -d '{"client_info":"test"}' | jq -r '.data.session_id')

curl -X POST http://localhost:8080/v1/qr-login/confirm \
  -H "Content-Type: application/json" \
  -d "{\"session_id\":\"$SESSION_ID\",\"app_token\":\"YOUR_TOKEN\"}"
```

详见：[01_QUICK_START.md](01_QUICK_START.md)

---

## 🏗️ 项目结构

```
rust-frame/
├── scaffold/
│   ├── src/
│   │   ├── backend/
│   │   │   ├── api/
│   │   │   │   ├── qr_login/      # QR登录API
│   │   │   │   └── code/          # 验证码API
│   │   │   ├── errors.rs          # 统一错误处理 ⭐
│   │   │   └── middleware/       # 中间件
│   │   └── models/               # 数据模型
│   └── examples/
│       ├── qr_login_websocket.html  # Web端页面
│       └── app_simulator.html        # App模拟器 ⭐
├── tests/
│   ├── local/                     # 本地测试工具 ⭐
│   ├── e2e/                       # E2E测试
│   └── integration/               # 集成测试
└── 01_QUICK_START.md              # 快速开始 ⭐
```

---

## 🎯 核心功能

### 1. 二维码登录流程

```
用户打开Web端 → 生成二维码 → WebSocket监听
                                  ↓
用户打开App端 → 扫描二维码 → 确认登录
                                  ↓
                WebSocket推送 → Web端收到token → 登录成功
```

**响应时间**: <100ms（WebSocket实时推送）

### 2. 错误处理

统一的错误响应格式：
```json
{
  "code": 1002,
  "msg": "token不能为空",
  "path": "/api/endpoint"
}
```

44个标准化错误码，详见：[04_ERROR_CODES.md](04_ERROR_CODES.md)

### 3. 测试覆盖

- ✅ 23个单元测试（全部通过）
- ✅ 集成测试框架
- ✅ E2E自动化测试
- ✅ 本地测试工具

---

## 🔧 技术栈

- **后端框架**: Actix-Web 4.0
- **数据库**: SeaORM + PostgreSQL
- **认证**: JWT (jsonwebtoken)
- **WebSocket**: actix-ws
- **图片处理**: qrcode + image
- **测试**: 单元测试 + 集成测试

---

## 📊 系统要求

- Rust 1.90.0+
- PostgreSQL 12+
- Docker & Docker Compose

---

## 🛠️ 开发

### 启动数据库

```bash
docker run -d \
  --name rust-frame-db \
  -e POSTGRES_DB=rust_frame \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  postgres:16-alpine
```

### 运行迁移

```bash
psql -h localhost -U postgres -d rust_frame \
  -f scaffold/migrations/001_create_qr_login_sessions.sql
```

### 启动服务

```bash
cargo run -- --backend-port 8080
```

### 运行测试

```bash
# 单元测试
cargo test

# 本地测试
./tests/local/local_test_flow.sh

# E2E测试
./tests/e2e/complete_login_flow.sh
```

---

## 📝 更新日志

### 2025-12-31

#### 新增功能
- ✅ 统一错误码系统（44个错误码）
- ✅ App端模拟器（app_simulator.html）
- ✅ 完整测试套件（23个单元测试）
- ✅ 本地测试工具

#### 文档整理
- 📖 重构文档结构（按序号编号）
- 📖 新增快速开始指南
- 📖 新增API文档
- 📖 删除冗余文档

详见：[CHANGES.md](CHANGES.md)

---

## 🔗 相关链接

- [Actix-Web文档](https://actix.rs/)
- [SeaORM文档](https://www.sea-ql.org/SeaORM/)
- [WebSocket文档](05_WEBSOCKET.md)

---

## 📄 License

MIT OR Apache-2.0

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

**快速开始?** → 查看 [01_QUICK_START.md](01_QUICK_START.md) ⭐
