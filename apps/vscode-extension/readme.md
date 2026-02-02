# Headwind for VS Code

**Atomic CSS to Semantic CSS compiler** — transform atomic utility classes into optimized semantic CSS directly in your editor.

**原子化 CSS 到语义化 CSS 编译器** — 在编辑器中将原子化工具类转换为优化的语义化 CSS。

Powered by Rust + WASM for near-instant transforms.
基于 Rust + WASM，转换几乎瞬间完成。

---

## Features / 功能

- **Control Panel** — Webview panel with live options, CSS preview, class map table, and action buttons
  控制面板 — 实时选项配置、CSS 预览、类名映射表、操作按钮

- **Diff Preview** — Side-by-side comparison of original vs transformed code using VS Code's built-in diff editor
  差异预览 — 使用 VS Code 内置 diff 编辑器对比转换前后代码

- **Apply Transform** — One-click apply: replaces source code and writes a companion CSS file (`.css` for global, `.module.css` for CSS Modules)
  一键应用 — 替换源码并生成配套 CSS 文件（全局模式 `.css`，CSS Modules 模式 `.module.css`）

- **Transform on Save** — Optionally auto-transform when saving supported files
  保存时自动转换（可选）

- **Smart File Tracking** — Switching to non-supported files (JSON, Markdown, etc.) retains the last valid file for operations
  智能文件追踪 — 切换到不支持的文件时保留上一个有效文件

- **Theme Aware** — Panel UI adapts to light, dark, and high-contrast themes automatically
  自动适配亮色、暗色和高对比度主题

---

## Commands / 命令

| Command / 命令 | Description / 说明 |
|---|---|
| `Headwind: Open Control Panel` | Open the Webview control panel / 打开控制面板 |
| `Headwind: Transform Current File` | Transform and open diff preview / 转换并打开差异预览 |
| `Headwind: Preview Transform (Diff)` | Side-by-side diff view / 并排差异对比 |
| `Headwind: Apply Transform` | Apply transform result and write CSS file / 应用转换结果并写入 CSS 文件 |

---

## Settings / 配置

| Setting / 配置项 | Type | Default | Description / 说明 |
|---|---|---|---|
| `headwind.namingMode` | `hash` \| `readable` \| `camelCase` | `hash` | Class name strategy / 类名生成策略 |
| `headwind.outputMode` | `global` \| `cssModules` | `global` | Output format / 输出格式 |
| `headwind.cssModulesAccess` | `dot` \| `bracket` | `dot` | CSS Modules access style / CSS Modules 访问方式 |
| `headwind.cssVariables` | `var` \| `inline` | `var` | CSS value mode / CSS 值模式 |
| `headwind.unknownClasses` | `remove` \| `preserve` | `preserve` | Unknown class handling / 未知类名处理 |
| `headwind.transformOnSave` | `boolean` | `false` | Auto-transform on save / 保存时自动转换 |
| `headwind.cssOutputPattern` | `string` | `[name].css` | CSS output filename pattern (global mode) / CSS 输出文件名模式（全局模式）|
| `headwind.include` | `string` | `**/*.{jsx,tsx,html}` | File glob for workspace transforms / 工作区转换文件匹配 |

---

## Supported Files / 支持的文件类型

`.jsx`, `.tsx`, `.js`, `.ts`, `.html`, `.htm`

---

## Getting Started / 快速开始

### Prerequisites / 前置要求

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Build & Run / 构建与运行

```bash
# Install dependencies / 安装依赖
pnpm install

# Build WASM for Node.js / 构建 Node.js WASM
pnpm build:wasm:node

# Build extension / 构建扩展
cd apps/vscode-extension && pnpm build
```

### Local Testing / 本地测试

1. Open the repo root in VS Code / 用 VS Code 打开仓库根目录
2. Press **F5** to launch the Extension Development Host / 按 F5 启动扩展开发宿主
3. Run command **Headwind: Open Control Panel** / 运行命令打开控制面板
4. Open a `.tsx` or `.jsx` file, click **Transform** / 打开一个 `.tsx` 或 `.jsx` 文件，点击转换

---

## Architecture / 架构

```
Extension Host                          Webview (Control Panel)
──────────────                          ───────────────────────
                                        Options dropdowns
  wasm.ts ── headwind-wasm (Rust)       Action buttons
       │                                CSS / Class Map / Code tabs
       v
  state.ts ── shared state ──────────── postMessage ↔ onMessage
       │
       ├── diffPreview.ts ── vscode.diff (before / after)
       ├── transformOnSave.ts ── onWillSaveTextDocument
       └── cssOutput.ts ── [name].css / [name].module.css
```

The transform engine is `headwind-wasm` compiled for Node.js (`wasm-pack --target nodejs`). At runtime, the WASM binary is loaded via `require()` from the `dist/` directory.

转换引擎为编译为 Node.js 目标的 `headwind-wasm`（`wasm-pack --target nodejs`）。运行时通过 `require()` 从 `dist/` 目录加载 WASM 二进制文件。

---

## License / 许可证

MIT
