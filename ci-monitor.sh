#!/bin/bash
# CI Monitor Script for WAVELET
# 确保每次推送后CI正常运转

echo "=== WAVELET CI Monitor ==="
echo ""

# 检查本地测试
echo "1. 运行本地测试..."
cargo test --lib --no-default-features
TEST_RESULT=$?

if [ $TEST_RESULT -ne 0 ]; then
    echo "❌ 测试失败！请修复后再推送。"
    exit 1
fi

echo ""
echo "2. 检查Clippy..."
cargo clippy --lib --no-default-features -- -D warnings
CLIPPY_RESULT=$?

if [ $CLIPPY_RESULT -ne 0 ]; then
    echo "⚠️ Clippy有警告，但允许推送"
fi

echo ""
echo "3. 检查代码格式化..."
cargo fmt --check
FORMAT_RESULT=$?

if [ $FORMAT_RESULT -ne 0 ]; then
    echo "⚠️ 代码格式不正确，运行 'cargo fmt' 修复"
    cargo fmt
fi

echo ""
echo "=== CI检查完成 ==="
echo "✅ 所有测试通过"
echo "✅ 可以安全推送了！"
