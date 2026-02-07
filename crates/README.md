# HeadWind Rust Crates

HeadWind 项目的 Rust 核心实现，负责 Tailwind CSS 类名解析、转换和 CSS 生成。

## 架构

```
crates/
├── core/           # 纯类型定义（共享数据结构）
├── tw_parse/       # Tailwind class 解析器
├── tw_index/       # 转换引擎（converter, bundler, css, naming 等）
├── transform/      # 源码变换（JSX/HTML class 替换）
└── wasm/           # WebAssembly 绑定（供 JS/TS 调用）
```

### 依赖关系

```
core ← tw_parse ← tw_index ← transform ← wasm
```

## 模块说明

### `headwind-core`

纯类型定义层，提供所有 crate 共享的数据结构：

- `Declaration` — CSS 声明（property + value）
- `BundleRequest` / `BundleResult` — 转换输入/输出
- `NamingMode` — 命名策略（Hash / Readable / CamelCase / Semantic）
- `ColorMode` — 颜色输出模式（Hex / Oklch / Hsl / Var）
- `CssVariableMode`, `UnknownClassMode`, `Diagnostic` 等

### `headwind-tw-parse`

Tailwind CSS class 解析器，将类名字符串解析为结构化表示：

- **修饰符**：响应式 (`md:`)、伪类 (`hover:`)、伪元素 (`before:`)、状态 (`dark:`)
- **任意值**：`w-[13px]`、`bg-[#ff0000]`、`grid-cols-[repeat(3,1fr)]`
- **CSS 变量**：`bg-(--my-color)`、`text-(length:--size)`
- **复合插件**：`justify-items`、`gap-x`、`border-t`、`translate-x` 等
- **其他**：负值 (`-m-4`)、透明度 (`/50`)、重要性 (`!`)

### `headwind-tw-index`

主要转换引擎，包含：

- **converter/** — 基于规则的 Tailwind → CSS 转换器（支持 50+ 插件）
- **bundler** — 批量转换 + CSS 生成（支持修饰符分组、简写优化）
- **css/** — CSS IR 和输出（基于 SWC CSS AST）
- **naming** — 类名生成策略（Hash / Readable / CamelCase）
- **bundle** — 端到端转换流程（normalize → merge → naming）
- **shorthand** — CSS 简写属性优化（padding、margin、border-radius 等）
- **index** — JSON 索引加载和查询
- **palette** — 完整 Tailwind 调色板（支持 Hex/Oklch/Hsl/Var 模式）
- **variant** — 修饰符 → CSS 选择器/at-rule 解析

### `headwind-transform`

源码变换层，使用 SWC 进行 AST 级别的 class 替换：

- **JSX 变换**：`className="p-4 m-2"` → `className="p4M2"` 或 `className={styles.p4M2}`
- **HTML 变换**：`class="p-4 m-2"` → `class="p4M2"`
- **输出模式**：Global（全局类名）/ CSS Modules（styles.xxx 引用）
- **元素树生成**：提取组件结构供 AI 语义命名使用
- **CSS 收集**：自动收集和去重所有生成的 CSS 规则

### `headwind-wasm`

WebAssembly 绑定，通过 `wasm-bindgen` 将 Rust 引擎暴露给 JavaScript/TypeScript：

- `transform_jsx(code, options)` — 变换 JSX/TSX 源码
- `transform_html(code, options)` — 变换 HTML 源码
- 类型安全的 JS ↔ Rust 选项映射

## 快速开始

### 运行测试

```bash
# 运行所有测试（389+ 个）
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p headwind-tw-index
cargo test -p headwind-tw-parse
cargo test -p headwind-transform
```

### 运行示例

```bash
cargo run --example basic_usage -p headwind-tw-index
```

### 构建

```bash
# 开发构建
cargo build --workspace

# 发布构建（启用 LTO）
cargo build --workspace --release

# WASM 构建
cd crates/wasm && wasm-pack build --target web
```

## 使用示例

```rust
use headwind_tw_index::{Bundler, Converter};
use headwind_tw_parse::parse_class;

// 1. 基于规则的转换（无需外部索引）
let converter = Converter::new();
let parsed = parse_class("hover:bg-blue-500/50").unwrap();
let rule = converter.convert(&parsed).unwrap();
// → selector: ".bg-blue-500/50:hover"
// → background: #3b82f680

// 2. 批量转换 + CSS 生成
let bundler = Bundler::new();
let result = bundler.bundle_to_css(&["p-4", "m-2", "hover:text-white"]);
// → 生成完整 CSS（含选择器、修饰符分组、简写优化）
```

## 设计原则

1. **确定性** — 相同输入永远产生相同输出
2. **可测试** — 每个模块都有单元测试和集成测试
3. **解耦** — 类型定义（core）、解析（tw_parse）、转换（tw_index）、变换（transform）分层清晰
4. **高性能** — blake3 哈希、PHF 静态映射、零运行时开销

## 依赖

| 依赖 | 用途 |
|------|------|
| `serde` + `serde_json` | JSON 序列化/反序列化 |
| `blake3` | 快速内容哈希（命名策略） |
| `indexmap` | 保持插入顺序的 Map |
| `phf` | 编译期完美哈希（静态映射） |
| `swc_core` | JavaScript/TypeScript AST 解析和代码生成 |
| `swc_css_*` | CSS AST 和代码生成 |
| `wasm-bindgen` | WebAssembly JS 绑定 |

## 许可证

MIT
