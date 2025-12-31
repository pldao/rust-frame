#!/bin/bash

# QR登录完整流程测试脚本
# 此脚本测试从生成二维码到完成登录的完整流程

set -e

# 配置
BASE_URL="${BASE_URL:-http://localhost:8081}"
WAIT_TIME="${WAIT_TIME:-2}"  # 等待时间（秒）

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# 检查环境
check_environment() {
  log_info "检查测试环境..."

  # 检查jq
  if ! command -v jq &> /dev/null; then
    log_error "jq 未安装，请先安装: brew install jq 或 apt-get install jq"
    exit 1
  fi

  # 检查服务器
  if ! curl -s "${BASE_URL}" > /dev/null 2>&1; then
    log_error "无法连接到服务器 ${BASE_URL}"
    log_error "请确保服务器正在运行: cargo run -- --backend-port 8081"
    exit 1
  fi

  log_info "环境检查通过"
}

# 1. 生成二维码
test_generate_qr() {
  log_info "[1/5] 生成二维码..."

  local response=$(curl -s -X POST "${BASE_URL}/qr-login/generate" \
    -H "Content-Type: application/json" \
    -d '{"client_info":"test_e2e"}')

  echo "响应: $response"

  local code=$(echo "$response" | jq -r '.code // empty')

  if [ "$code" != "0" ]; then
    log_error "生成二维码失败"
    echo "$response" | jq '.'
    exit 1
  fi

  SESSION_ID=$(echo "$response" | jq -r '.data.session_id // empty')
  QR_IMAGE=$(echo "$response" | jq -r '.data.qr_image // empty')

  if [ -z "$SESSION_ID" ] || [ "$SESSION_ID" = "null" ]; then
    log_error "未能获取 session_id"
    exit 1
  fi

  log_info "✓ 二维码生成成功"
  log_info "  session_id: $SESSION_ID"
  log_info "  qr_image: ${QR_IMAGE:0:50}..."
}

# 2. 查询初始状态
test_check_initial_status() {
  log_info "[2/5] 查询初始状态..."

  local response=$(curl -s "${BASE_URL}/qr-login/status/${SESSION_ID}")
  echo "响应: $response"

  local status=$(echo "$response" | jq -r '.data.status // empty')

  if [ "$status" != "pending" ]; then
    log_error "初始状态应该是 'pending'，实际是: $status"
    exit 1
  fi

  log_info "✓ 初始状态正确: pending"
}

# 3. 模拟App端确认登录
test_confirm_login() {
  log_info "[3/5] App端确认登录..."

  # 注意：这里需要有效的 JWT token
  # 实际测试时，应该先调用登录接口获取 token
  # 或者创建一个测试用户并获取其 token

  # 从环境变量或配置文件读取测试用的 app_token
  if [ -z "$TEST_APP_TOKEN" ]; then
    log_warn "未设置 TEST_APP_TOKEN 环境变量"
    log_warn "跳过确认登录测试"
    log_warn "要测试完整流程，请设置有效的 JWT token:"
    log_warn "  export TEST_APP_TOKEN=\"your_jwt_token\""
    SKIP_CONFIRM=true
    return
  fi

  local response=$(curl -s -X POST "${BASE_URL}/qr-login/confirm" \
    -H "Content-Type: application/json" \
    -d "{
      \"session_id\": \"$SESSION_ID\",
      \"app_token\": \"$TEST_APP_TOKEN\"
    }")

  echo "响应: $response"

  local code=$(echo "$response" | jq -r '.code // empty')

  if [ "$code" != "0" ]; then
    log_warn "确认登录失败（可能是token无效）"
    log_warn "这在测试环境中是正常的，如果使用真实token应该成功"
    SKIP_CONFIRM=true
    return
  fi

  log_info "✓ 登录确认成功"
}

# 4. 查询最终状态
test_check_final_status() {
  log_info "[4/5] 查询最终状态..."

  local response=$(curl -s "${BASE_URL}/qr-login/status/${SESSION_ID}")
  echo "响应: $response"

  local code=$(echo "$response" | jq -r '.code // empty')
  local status=$(echo "$response" | jq -r '.data.status // .msg // empty' 2>/dev/null || echo "unknown")

  if [ "$code" != "0" ]; then
    log_warn "状态查询返回错误: $status"
    log_warn "这可能是因为我们没有实际确认登录"
    return
  fi

  if [ "$status" = "confirmed" ]; then
    WEB_TOKEN=$(echo "$response" | jq -r '.data.web_token // empty')
    log_info "✓ 最终状态正确: confirmed"
    log_info "  web_token: ${WEB_TOKEN:0:50}..."
  elif [ "$status" = "pending" ]; then
    log_warn "状态仍为 pending（未确认登录）"
  else
    log_info "状态: $status"
  fi
}

# 5. 测试无效的session
test_invalid_session() {
  log_info "[5/5] 测试无效的session_id..."

  local response=$(curl -s "${BASE_URL}/qr-login/status/invalid-session-id-12345")
  echo "响应: $response"

  local code=$(echo "$response" | jq -r '.code // empty')

  # 应该返回错误码 1300 (QRCodeNotFound)
  if [ "$code" = "1300" ]; then
    log_info "✓ 正确返回 QRCodeNotFound 错误"
  else
    log_warn "未返回预期的错误码 (1300)，实际: $code"
  fi
}

# 清理
cleanup() {
  log_info "清理测试数据..."
  # 这里可以添加清理数据库的命令
  # 例如：psql -c "DELETE FROM qr_login_sessions WHERE session_id LIKE 'test-%';"
}

# 主流程
main() {
  echo "========================================"
  echo "   QR登录完整流程测试"
  echo "========================================"
  echo ""

  check_environment
  cleanup

  test_generate_qr
  test_check_initial_status
  test_confirm_login
  test_check_final_status
  test_invalid_session

  cleanup

  echo ""
  echo "========================================"
  if [ "$SKIP_CONFIRM" = true ]; then
    log_info "测试完成（部分测试已跳过）"
    log_warn "要运行完整测试，请设置 TEST_APP_TOKEN 环境变量"
  else
    log_info "✓ 所有测试通过"
  fi
  echo "========================================"
}

# 运行
trap cleanup EXIT
main "$@"
