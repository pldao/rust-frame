# 快速开始 - 扫码登录功能

> 🎯 **3分钟完成本地测试** | 无需真实手机 | 使用模拟器即可

## 📋 前置要求

- ✅ Rust 已安装
- ✅ Docker 已安装（PostgreSQL）
- ✅ 浏览器（Chrome/Firefox/Safari）

---

## 🚀 3步快速测试

### 步骤1：启动后端服务（1分钟）

```bash
# 进入项目目录
cd /Users/alex/go/src/pldao/rust-frame

# 启动数据库（如果还没启动）
docker run -d --name rust-frame-db \
  -e POSTGRES_DB=rust_frame \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -p 5432:5432 \
  postgres:16-alpine

# 启动后端服务
cargo run -- --backend-port 8080
```

等待看到：`Starting server on 127.0.0.1:8080`

---

### 步骤2：打开Web端页面（30秒）

在浏览器中打开文件：
```
scaffold/examples/qr_login_websocket.html
```

或者使用简单HTTP服务器：
```bash
cd scaffold/examples
python3 -m http.server 8000
# 然后访问 http://localhost:8000/qr_login_websocket.html
```

点击 **"生成二维码"** 按钮，会看到：
- ✅ 二维码图片
- ✅ Session ID
- ✅ "等待扫码" 状态

---

### 步骤3：使用App模拟器确认登录（1分钟）

**打开新标签页**，在浏览器中打开：
```
scaffold/examples/app_simulator.html
```

在App模拟器页面：

1. **复制Session ID**
   - 回到Web端页面
   - 点击 **"📋 复制 Session ID"** 按钮
   - 或手动复制文本框中的session_id

2. **粘贴Session ID**
   - 在App模拟器的 "Session ID" 输入框中粘贴

3. **生成Token**（如果还没生成）
   - App模拟器会自动生成测试Token
   - 或点击 **"🎲 生成测试Token"** 按钮

4. **确认登录**
   - 点击 **"✅ 确认登录"** 按钮
   - 等待响应

---

## 🎉 观察结果

### Web端页面自动更新

确认登录成功后，Web端页面会**立即**更新为：
```
✅ 登录成功！
响应时间: XXXms（几乎实时！）
```

这是因为使用了 **WebSocket 实时推送**！

### 控制台日志

按 `F12` 打开浏览器控制台，可以看到详细的日志：
- 二维码生成
- WebSocket连接
- 消息接收
- 响应时间

---

## 🎯 完整流程图

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│  Web端页面   │         │   后端服务   │         │ App端模拟器  │
│             │         │             │         │             │
│ 点击生成QR   │────────▶│  生成session│         │             │
│ 显示二维码   │◀────────│  返回QR码   │         │             │
│ 连接WebSocket│────────▶│  等待确认   │         │             │
│             │         │             │◀────────│ 提交确认请求  │
│             │◀────────│  WebSocket  │         │             │
│  显示成功    │         │   推送状态   │         │             │
└─────────────┘         └─────────────┘         └─────────────┘
```

---

## 🔍 测试场景

### 场景1：正常登录流程 ✅

1. Web端生成二维码
2. 复制Session ID
3. App模拟器确认登录
4. Web端自动显示成功

### 场景2：二维码过期 ⏰

1. 生成二维码后等待5分钟
2. 尝试确认登录
3. 返回错误：`"code": 1301, "msg": "二维码已过期"`

### 场景3：重复确认 🚫

1. 第一次确认登录成功
2. 再次用同一session确认
3. 返回错误：`"code": 1203, "msg": "资源冲突"`

---

## ⚡ 一键测试脚本

如果你喜欢命令行，也可以使用：

```bash
# 生成token
python3 tests/local/generate_test_token.py

# 复制生成的token，然后执行：
SESSION_ID="粘贴session_id" \
  APP_TOKEN="粘贴token" \
  curl -X POST http://localhost:8080/v1/qr-login/confirm \
    -H "Content-Type: application/json" \
    -d "{\"session_id\":\"$SESSION_ID\",\"app_token\":\"$APP_TOKEN\"}"
```

---

## 📱 App模拟器功能

**自动生成测试Token**
- 包含 user_id, username, role
- 24小时有效期
- 开箱即用

**实时反馈**
- 请求状态
- 响应时间
- 错误信息

**调试友好**
- 浏览器控制台日志
- 详细的状态显示
- 错误提示

---

## 🐛 常见问题

### Q: 点击"生成二维码"没反应？

**A:** 检查后端是否启动：
```bash
curl http://localhost:8080/v1/qr-login/generate
```

### Q: WebSocket连接失败？

**A:**
1. 检查后端日志
2. 确认端口8080未被占用
3. 刷新页面重试

### Q: 确认登录没反应？

**A:**
1. 检查Session ID是否正确复制
2. 打开浏览器控制台(F12)查看错误
3. 确认后端日志中是否收到请求

---

## 📚 下一步

学习更多：
- 📖 [API文档](02_API_GUIDE.md)
- 📖 [完整功能说明](03_QR_LOGIN.md)
- 📖 [错误码说明](04_ERROR_CODES.md)
- 📖 [测试指南](06_TESTING.md)

---

**就这么简单！** 🎉

现在你已经完成了完整的QR登录测试流程！
