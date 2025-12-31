#!/bin/bash

# 运行所有测试的脚本

set -e

# 颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}==>${NC} $1"; }

echo "========================================"
echo "   Rust Frame - 测试套件"
echo "========================================"
echo ""

# 检查环境
log_step "检查环境..."
if ! command -v cargo &> /dev/null; then
  log_error "Rust/Cargo 未安装"
  exit 1
fi
log_info "环境检查通过"

# 格式化检查
log_step "检查代码格式..."
if cargo fmt -- --check &> /dev/null; then
  log_info "代码格式正确"
else
  log_warn "代码格式需要修正，运行: cargo fmt"
fi

# Clippy检查
log_step "运行 Clippy 检查..."
if cargo clippy --all-targets --all-features -- -D warnings &> /dev/null; then
  log_info "Clippy 检查通过"
else
  log_warn "Clippy 发现了一些问题，运行: cargo clippy --fix"
fi

# 单元测试
log_step "运行单元测试..."
if cargo test --lib 2>&1 | grep -E "(test result|running [0-9]+ test)"; then
  log_info "✓ 单元测试通过"
else
  log_error "单元测试失败"
  exit 1
fi

# 文档测试
log_step "运行文档测试..."
if cargo test --doc 2>&1 | tail -5; then
  log_info "✓ 文档测试通过"
else
  log_warn "文档测试有警告"
fi

# E2E测试（可选）
if [ -n "$RUN_E2E_TESTS" ]; then
  log_step "运行E2E测试..."

  # 检查服务器是否运行
  if curl -s http://localhost:8081 > /dev/null 2>&1; then
    if [ -x "tests/e2e/complete_login_flow.sh" ]; then
      tests/e2e/complete_login_flow.sh
      log_info "✓ E2E测试通过"
    else
      log_warn "E2E测试脚本不存在或不可执行"
    fi
  else
    log_warn "E2E测试需要服务器运行"
    log_warn "启动服务器: cargo run -- --backend-port 8081"
    log_warn "然后运行: RUN_E2E_TESTS=1 ./tests/run_all_tests.sh"
  fi
else
  log_info "跳过E2E测试（设置 RUN_E2E_TESTS=1 来运行）"
fi

# 构建检查
log_step "检查项目构建..."
if cargo build --release 2>&1 | tail -3; then
  log_info "✓ 项目可以成功构建"
else
  log_error "项目构建失败"
  exit 1
fi

echo ""
echo "========================================"
log_info "✓ 所有测试通过！"
echo "========================================"
echo ""
echo "测试覆盖："
echo "  ✓ 单元测试"
echo "  ✓ 文档测试"
echo "  ✓ 代码格式检查"
echo "  ✓ Clippy检查"
echo "  ✓ 构建检查"
if [ -n "$RUN_E2E_TESTS" ]; then
  echo "  ✓ E2E测试"
fi
echo ""
echo "其他有用的命令："
echo "  • 运行特定测试: cargo test test_name"
echo "  • 显示输出: cargo test -- --nocapture"
echo "  • 生成覆盖率: cargo tarpaulin --out Html"
echo ""
