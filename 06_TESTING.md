# 测试指南

> 完整的测试指南，包括单元测试、集成测试、E2E测试和本地测试工具

## 📋 目录

1. [快速测试](#快速测试)
2. [单元测试](#单元测试)
3. [本地测试工具](#本地测试工具)
4. [测试场景](#测试场景)
5. [故障排查](#故障排查)

---

## 快速测试

### 方式1：使用浏览器测试（最简单）⭐

```bash
# 1. 启动后端
cargo run -- --backend-port 8080

# 2. 打开两个浏览器标签
# 标签1: scaffold/examples/qr_login_websocket.html (Web端)
# 标签2: scaffold/examples/app_simulator.html (App模拟器)

# 3. 在Web端点击"生成二维码"
# 4. 复制Session ID到App模拟器
# 5. 在App模拟器点击"确认登录"
# 6. 观察Web端自动更新为"登录成功"
```

### 方式2：使用curl测试

```bash
# 生成二维码
curl -X POST http://localhost:8080/v1/qr-login/generate \
  -H "Content-Type: application/json" \
  -d '{"client_info":"test"}' | jq '.data.session_id'

# 保存session_id
export SESSION_ID="返回的session_id"

# 查询状态
curl http://localhost:8080/v1/qr-login/status/$SESSION_ID | jq '.'

# 生成token
python3 tests/local/generate_test_token.py

# 确认登录
curl -X POST http://localhost:8080/v1/qr-login/confirm \
  -H "Content-Type: application/json" \
  -d "{\"session_id\":\"$SESSION_ID\",\"app_token\":\"YOUR_TOKEN\"}"
```

---

## 单元测试

### 运行单元测试

```bash
# 运行所有测试
cargo test

# 运行特定模块
cargo test backend::errors

# 显示输出
cargo test -- --nocapture

# 生成覆盖率
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### 测试覆盖

- ✅ 错误码系统（23个测试，全部通过）
- ✅ 错误响应格式
- ✅ 分页响应
- ✅ 批量操作
- ✅ WebSocket错误处理

**运行结果**：
```
running 23 tests
test result: ok. 23 passed; 0 failed
```

---

## 本地测试工具

### 1. 测试脚本

**位置**: `tests/local/local_test_flow.sh`

**功能**：
- 自动生成JWT token
- 自动生成二维码
- 模拟确认登录
- 显示完整流程

**运行**：
```bash
chmod +x tests/local/local_test_flow.sh
./tests/local/local_test_flow.sh
```

### 2. Token生成器

**位置**: `tests/local/generate_test_token.py`

**功能**：
- 生成测试JWT token
- 支持自定义参数
- 解码token内容

**运行**：
```bash
# 默认配置
python3 tests/local/generate_test_token.py

# 自定义用户
python3 tests/local/generate_test_token.py --user-id test_123

# 解码显示
python3 tests/local/generate_test_token.py --decode
```

---

## 测试场景

### 场景1：正常登录流程 ✅

**步骤**：
1. Web端生成二维码
2. App端获取Session ID
3. App端确认登录
4. Web端收到WebSocket推送

**预期结果**：
```json
{
  "status": "confirmed",
  "web_token": "eyJ0eXAiOiJKV1Q...",
  "message": "Login successful"
}
```

### 场景2：二维码过期 ⏰

**步骤**：
1. 生成二维码
2. 等待5分钟过期
3. 尝试确认登录

**预期结果**：
```json
{
  "code": 1301,
  "msg": "二维码已过期"
}
```

### 场景3：无效Session ID

**步骤**：
1. 使用不存在的Session ID
2. 尝试确认登录

**预期结果**：
```json
{
  "code": 1300,
  "msg": "二维码不存在"
}
```

### 场景4：重复确认

**步骤**：
1. 第一次确认成功
2. 再次确认登录

**预期结果**：
```json
{
  "code": 1203,
  "msg": "资源冲突"
}
```

---

## 性能测试

### 使用Apache Bench

```bash
# 安装ab
brew install httpd  # macOS
apt-get install apache2-utils  # Ubuntu

# 测试生成二维码接口
ab -n 1000 -c 10 \
  -H "Content-Type: application/json" \
  -p post_data.json \
  http://localhost:8080/v1/qr-login/generate
```

### 使用wrk

```bash
# 安装wrk
git clone https://github.com/wg/wrk.git
cd wrk && make

# 测试状态查询
wrk -t4 -c100 -d30s \
  http://localhost:8080/v1/qr-login/status/test-session
```

---

## 故障排查

### 问题1：Token验证失败

**错误**: `code: 1003, msg: 无效的token`

**解决**：
1. 检查JWT_SECRET配置
2. 确认token未过期
3. 验证token格式正确

### 问题2：WebSocket连接失败

**错误**: WebSocket连接错误

**解决**：
1. 检查后端是否启动
2. 确认端口8080未被占用
3. 查看浏览器控制台错误

### 问题3：数据库连接失败

**错误**: Database connection failed

**解决**：
```bash
# 检查数据库
docker ps | grep rust-frame-db

# 重启数据库
docker restart rust-frame-db
```

---

## CI/CD

### GitHub Actions示例

创建 `.github/workflows/test.yml`：

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_DB: test_db
          POSTGRES_USER: test_user
          POSTGRES_PASSWORD: test_password
        ports:
          - 5432:5432

    steps:
    - uses: actions/checkout@v3

    - name: Run tests
      run: cargo test --lib

    - name: Generate coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml
```

---

## 相关文档

- [01_QUICK_START.md](01_QUICK_START.md) - 快速开始
- [02_API_GUIDE.md](02_API_GUIDE.md) - API文档
- [04_ERROR_CODES.md](04_ERROR_CODES.md) - 错误码说明

---

**测试是最好的文档！** 🎉
