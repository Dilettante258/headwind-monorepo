# HeadWind Tools

开发工具和脚本集合。

## 📁 目录结构

```
tools/
├── scripts/
│   ├── extract-tw-mappings.ts      # 从 MDX 提取 Tailwind 映射
│   └── setup-sparse-clone.sh       # 稀疏克隆 tailwindcss.com
├── data/
│   └── tailwindcss.com/            # 克隆的文档仓库（gitignore）
└── package.json
```

## 🚀 使用方法

### 1. 提取 Tailwind CSS 官方映射

这个工具从 Tailwind CSS 官方文档中提取 class → CSS 的映射关系，用于测试和验证。

#### 首次运行

```bash
cd tools

# Step 1: 克隆文档仓库（稀疏检出，只下载必要文件）
bun run setup

# Step 2: 提取映射数据
bun run extract
```

#### 更新数据

当 Tailwind CSS 有重大更新时：

```bash
cd tools

# 清理旧数据
bun run clean

# 重新克隆和提取
bun run setup
bun run extract
```

或者更新现有克隆：

```bash
cd tools/data/tailwindcss.com
git pull
cd ../..
bun run extract
```

### 2. 输出文件

提取的数据会保存到：

```
crates/tw_index/fixtures/official-mappings.json
```

这个文件会被提交到 Git，供测试使用。

## 📝 提取脚本详解

### extract-tw-mappings.ts

从 Tailwind CSS 文档的 MDX 文件中提取 `<ApiTable>` 组件的数据。

**工作原理：**

1. 扫描 `src/docs/**/*.mdx` 文件
2. 使用正则表达式匹配 `<ApiTable rows={[...]} />`
3. 解析数组数据（格式：`[["class-name", "css-property: value;"], ...]`）
4. 过滤掉占位符（如 `perspective-origin-[]`）
5. 去重并排序
6. 输出为 JSON

**示例输出：**

```json
[
  {
    "class": "p-4",
    "css": "padding: 1rem",
    "source": "/src/pages/docs/padding.mdx"
  },
  {
    "class": "m-2",
    "css": "margin: 0.5rem",
    "source": "/src/pages/docs/margin.mdx"
  }
]
```

## 🔧 稀疏克隆说明

为了避免下载整个仓库（包含大量图片和其他文件），我们使用 Git 稀疏检出：

**优点：**
- 只下载 `src/pages/docs` 目录
- 跳过图片和其他不必要的文件
- 大幅减少下载时间和磁盘占用

**配置：**

查看 `scripts/setup-sparse-clone.sh` 了解详细配置。

## 📊 预期数据量

- **完整仓库：** ~200MB
- **稀疏检出：** ~10-20MB
- **提取的 JSON：** ~100-200KB

## 🐛 故障排除

### 克隆失败

如果克隆失败或中断：

```bash
bun run clean
bun run setup
```

### 提取脚本报错

检查 tailwindcss.com 仓库结构是否变化：

```bash
ls -la tools/data/tailwindcss.com/src/pages/docs/
```

### Bun 未找到

确保 Bun 已安装并在 PATH 中：

```bash
bun --version
```

或使用绝对路径：

```bash
~/.bun/bin/bun run extract
```

## 📅 维护计划

建议每次 Tailwind CSS 发布重大版本时更新映射数据：

1. Tailwind CSS v3.x → v4.0：需要重新提取
2. 小版本更新：可选，视变化范围决定

## 🔗 相关资源

- [Tailwind CSS 文档仓库](https://github.com/tailwindlabs/tailwindcss.com)
- [Git 稀疏检出文档](https://git-scm.com/docs/git-sparse-checkout)
- [Bun 文档](https://bun.sh/docs)
