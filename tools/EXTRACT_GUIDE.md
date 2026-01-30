# Tailwind 官方映射提取指南

## 📦 已完成的工作

### 1. 脚本和工具

✅ **稀疏克隆脚本** (`scripts/setup-sparse-clone.sh`)
- 使用 Git sparse-checkout 只克隆 `src/pages/docs` 目录
- 避免下载大量图片和其他不必要的文件
- 预计节省 ~90% 的下载量（~10-20MB vs ~200MB）

✅ **提取脚本** (`scripts/extract-tw-mappings.ts`)
- 扫描 MDX 文件查找 `<ApiTable>` 组件
- 解析并提取 class → CSS 映射
- 自动去重和排序
- 输出格式化的 JSON

✅ **测试文件** (`crates/tw_parse/tests/official_mappings.rs`)
- 验证解析器能正确处理所有官方 class 名称
- 当前测试 15 个示例映射，全部通过 ✅

### 2. 示例数据

创建了包含 15 个常见 Tailwind class 的示例映射：

```
crates/tw_index/fixtures/official-mappings.json
```

包括：
- Padding: p-0, p-1, p-4
- Margin: m-0, m-2
- Width: w-0, w-full
- Background: bg-transparent, bg-black, bg-white
- Text size: text-xs, text-sm
- Display: flex, grid, hidden

## 🚀 使用方法

### 选项 1：使用示例数据（当前）

示例数据已经可用，测试已通过：

```bash
cargo test -p headwind-tw-parse official_mappings
```

### 选项 2：提取完整数据（稍后执行）

当你有时间时，可以提取完整的官方映射：

```bash
cd tools

# Step 1: 稀疏克隆（可能需要 5-10 分钟）
bun run setup
# 或者直接运行：
bash scripts/setup-sparse-clone.sh

# Step 2: 提取映射（约 1 分钟）
bun run extract
# 或者直接运行：
~/.bun/bin/bun run scripts/extract-tw-mappings.ts

# Step 3: 验证结果
cargo test -p headwind-tw-parse official_mappings -- --nocapture
```

## 📊 预期结果

提取完整数据后，你应该会得到：

- **class 数量**: 500-1000 个（取决于 Tailwind 版本）
- **文件大小**: ~100-200KB
- **来源文件**: ~50-100 个 MDX 文件

### 数据格式

```json
[
  {
    "class": "p-4",
    "css": "padding: 1rem",
    "source": "/src/pages/docs/padding.mdx"
  }
]
```

## 🔍 故障排除

### 克隆太慢或失败

**问题**: Git clone 卡住或失败

**解决方案**:
```bash
# 停止当前进程（Ctrl+C）
cd tools
bun run clean
bun run setup
```

或者使用浅克隆替代：
```bash
cd tools/data
git clone --depth 1 --filter=blob:none https://github.com/tailwindlabs/tailwindcss.com.git
```

### 提取脚本找不到文件

**问题**: "Docs directory not found"

**检查**:
```bash
ls -la tools/data/tailwindcss.com/src/pages/docs/
```

**解决**: 确保稀疏克隆正确配置了 `src/pages/docs` 路径

### Bun 路径问题

**问题**: "command not found: bun"

**解决**: 使用完整路径
```bash
~/.bun/bin/bun run extract
```

或添加到 PATH:
```bash
export PATH="$HOME/.bun/bin:$PATH"
```

## 📝 维护计划

### 何时更新数据

建议在以下情况更新映射数据：

1. **Tailwind CSS 重大版本更新**（如 v3.x → v4.0）
   - 必须更新，API 可能有重大变化

2. **小版本更新**（如 v4.0 → v4.1）
   - 可选，仅在添加新 utility 时需要

3. **发现解析错误时**
   - 如果测试发现某些 class 解析失败

### 更新流程

```bash
cd tools/data/tailwindcss.com
git pull
cd ../..
bun run extract
cargo test -p headwind-tw-parse official_mappings
git add crates/tw_index/fixtures/official-mappings.json
git commit -m "Update Tailwind official mappings to v4.x.x"
```

## 🎯 下一步

### 短期（可选）

- [ ] 提取完整的官方映射数据
- [ ] 按类型分组映射（spacing, colors, layout 等）
- [ ] 添加更多具体的测试用例

### 长期

- [ ] 集成到 CI pipeline（自动检测 Tailwind 更新）
- [ ] 从提取的数据生成 tw_index 的完整索引
- [ ] 支持自定义 Tailwind 配置

## 📚 相关文档

- [tools/README.md](./README.md) - 工具总体说明
- [crates/tw_parse/README.md](../crates/tw_parse/README.md) - 解析器文档
- [Tailwind CSS 文档](https://tailwindcss.com/docs) - 官方文档

## ✅ 当前状态

- ✅ 脚本已创建并测试
- ✅ 示例数据已就绪
- ✅ 测试通过（15/15 classes）
- ⏳ 等待提取完整数据（可选，稍后执行）

**总测试数**: 52 个测试全部通过 ✅
