#!/bin/bash

# 稀疏克隆 tailwindcss.com，只获取 src/docs 目录（排除图片）

set -e

REPO_URL="https://github.com/tailwindlabs/tailwindcss.com.git"
TARGET_DIR="data/tailwindcss.com"

cd "$(dirname "$0")/.."

# 如果目录已存在，跳过
if [ -d "$TARGET_DIR/.git" ]; then
  echo "✓ Repository already exists at $TARGET_DIR"
  exit 0
fi

echo "🔄 Setting up sparse checkout for tailwindcss.com..."

# 清理可能存在的不完整目录
rm -rf "$TARGET_DIR"

# 创建目录
mkdir -p "$TARGET_DIR"
cd "$TARGET_DIR"

# 初始化 Git 仓库
git init
git remote add origin "$REPO_URL"

# 启用稀疏检出
git config core.sparseCheckout true

# 配置稀疏检出模式（cone mode 更高效）
git sparse-checkout set --cone

# 只检出 src/docs 目录，排除图片
echo "src/docs" >> .git/info/sparse-checkout

# 获取最新的主分支（浅克隆）
echo "📥 Fetching repository (this may take a minute)..."
git fetch --depth 1 origin master

# 检出代码
git checkout master

echo "✅ Sparse checkout completed!"
echo "📊 Directory size:"
du -sh .

echo ""
echo "💡 To update later, run:"
echo "   cd $TARGET_DIR && git pull"
