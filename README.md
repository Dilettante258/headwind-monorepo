# Headwind

![Icon](./assets/headwind.png)

**Atomic CSS to Semantic CSS compiler** built with Rust and SWC.

Headwind parses atomic utility classes from JSX/TSX/HTML source code, replaces them with generated semantic class names, and outputs corresponding CSS. It runs as a WASM module in the browser, a VS Code extension, or a Node.js transformer.

**Headwind** 是一个基于 Rust 和 SWC 构建的**原子化 CSS 到语义化 CSS 编译器**。

它从 JSX/TSX/HTML 源码中解析原子化工具类，替换为生成的语义化类名，并输出对应的 CSS。支持在浏览器中以 WASM 运行、作为 VS Code 扩展使用，或作为 Node.js 转换器集成。

---

## Features / 功能特性

- **Full Tailwind coverage** — 752/752 official utility mappings supported
  完整覆盖 Tailwind 官方映射 (752/752)

- **Multiple naming modes** — Hash (`c_a1b2c3`), Readable (`p4_m2`), CamelCase (`p4M2`)
  多种命名策略：哈希、可读、驼峰

- **CSS Modules support** — generates `styles.xxx` or `styles["xxx"]` with auto-injected imports
  支持 CSS Modules 模式，自动注入 import 语句

- **CSS variable modes** — use `var(--text-3xl)` references or inline concrete values (`1.875rem`), with auto-generated `:root` definitions
  CSS 变量模式：引用模式自动生成 `:root` 定义，内联模式直接输出具体值

- **JSX comment preservation** — `{/* comments */}` survive transformation
  JSX 注释在转换后完整保留

- **Unknown class handling** — preserve or remove unrecognized class names
  未知类名可配置保留或删除

- **Responsive & pseudo-class support** — `md:p-8`, `hover:bg-blue-700`, `dark:text-white`
  支持响应式、伪类、暗色模式等修饰符

- **Arbitrary values** — `bg-[#1da1f2]`, `p-[3.5rem]`, `grid-cols-[1fr_2fr]`
  支持任意值语法

- **Element tree generation** — output a structured text tree of all elements with their classes and `[ref=eN]` identifiers, ideal for passing to AI for secondary processing
  元素树生成：输出带 `[ref=eN]` 引用标识的结构化元素树文本，方便传给 AI 做二次处理

---

## Quick Start / 快速开始

### Prerequisites / 前置要求

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Install & Build / 安装与构建

```bash
# Clone the repository / 克隆仓库
git clone https://github.com/user/headwind.git
cd headwind

# Install JS dependencies / 安装 JS 依赖
pnpm install

# Run Rust tests / 运行 Rust 测试
cargo test --workspace

# Build WASM module / 构建 WASM 模块
pnpm build:wasm

# Start the playground / 启动 Playground
pnpm dev:playground
```

---

## Usage / 使用方式

### Rust API

```rust
use headwind_transform::{transform_jsx, TransformOptions};

let source = r#"
  export default function App() {
    return <div className="p-4 text-center hover:text-left">Hello</div>;
  }
"#;

let result = transform_jsx(source, "App.tsx", TransformOptions::default()).unwrap();

println!("{}", result.code);  // className replaced with generated name
println!("{}", result.css);   // .c_abc123 { padding: 1rem; text-align: center; }

// With element tree / 启用元素树
let result = transform_jsx(source, "App.tsx", TransformOptions {
    element_tree: true,
    ..Default::default()
}).unwrap();

if let Some(tree) = &result.element_tree {
    println!("{}", tree);
    // - div p-4 text-center hover:text-left "Hello" [ref=e1]
}
```

### WASM (Browser)

```typescript
import init, { transformJsx } from 'headwind-wasm';

await init();

const result = transformJsx(source, 'App.tsx', {
  namingMode: 'hash',
  outputMode: { type: 'global' },
  cssVariables: 'inline',
  unknownClasses: 'preserve',
});

console.log(result.code);
console.log(result.css);
console.log(result.classMap);
```

### Element Tree / 元素树

Enable `elementTree` to get a structured text representation of all elements, useful for passing to AI for secondary processing.

开启 `elementTree` 选项可获取所有元素的结构化文本表示，方便传给 AI 做二次处理。

```typescript
const result = transformJsx(source, 'App.tsx', {
  elementTree: true,
});

console.log(result.elementTree);
```

**Input / 输入:**

```jsx
<div className="w-full h-20 border">
  <h2 className="text-xl text-red-500">Title</h2>
  <p>some text</p>
  <div>
    <p className="text-lg text-blue-500">
      <span className="text-sm">inner</span>
    </p>
  </div>
</div>
```

**Output `elementTree` / 输出元素树:**

```
- div w-full h-20 border [ref=e1]
  - h2 text-xl text-red-500 "Title" [ref=e2]
  - p: some text [ref=e3]
  - div [ref=e4]
    - p text-lg text-blue-500 [ref=e5]
      - span text-sm "inner" [ref=e6]
```

Each node follows the format / 每个节点格式如下：

| Pattern / 模式 | Meaning / 含义 |
|---|---|
| `- tag classes [ref=eN]` | Element with Tailwind classes / 有 class 的元素 |
| `- tag: text [ref=eN]` | Element with text content only / 仅有文本的元素 |
| `- tag classes "text" [ref=eN]` | Element with both classes and text / 同时有 class 和文本 |

Every element has a unique `[ref=eN]` identifier for easy reference in downstream AI prompts.

每个元素都有唯一的 `[ref=eN]` 标识，方便在后续 AI 提示中引用。

---

## Transform Options / 转换选项

| Option / 选项 | Values / 可选值 | Default / 默认值 | Description / 说明 |
|---|---|---|---|
| `namingMode` | `hash`, `readable`, `camelCase` | `hash` | Class name generation strategy / 类名生成策略 |
| `outputMode` | `{ type: 'global' }`, `{ type: 'cssModules', access: 'dot' \| 'bracket' }` | `global` | Output format / 输出格式 |
| `cssVariables` | `var`, `inline` | `var` | Use CSS variable references or inline values / 使用 CSS 变量引用或内联值 |
| `unknownClasses` | `remove`, `preserve` | `remove` | How to handle unrecognized classes / 未知类名处理方式 |
| `colorMode` | `hex`, `oklch`, `hsl`, `var` | `hex` | Color output format / 颜色输出格式 |
| `colorMix` | `true`, `false` | `false` | Use `color-mix()` for opacity / 使用 `color-mix()` 处理透明度 |
| `elementTree` | `true`, `false` | `false` | Generate element tree in result / 在结果中生成元素树 |

---

## Project Structure / 项目结构

```
headwind/
├── crates/                     # Rust crates
│   ├── core/                   # Core types: Declaration, NamingMode, etc.
│   │                            核心类型定义
│   ├── tw_parse/               # Tailwind class parser
│   │                            Tailwind 类名解析器
│   ├── tw_index/               # Converter, bundler, and value maps
│   │                            转换器、打包器、值映射表
│   ├── css/                    # CSS AST generation (SWC-based)
│   │                            CSS AST 生成
│   ├── transform/              # JSX/HTML transform pipeline
│   │                            JSX/HTML 转换流水线
│   ├── wasm/                   # WASM bindings (wasm-bindgen)
│   │                            WASM 绑定
│   └── swc_plugin/             # SWC plugin (experimental)
│                                SWC 插件（实验性）
├── apps/
│   ├── web-playground/         # Interactive playground (Vite + Solid.js)
│   │                            在线 Playground
│   ├── vscode-extension/       # VS Code extension
│   │                            VS Code 扩展
│   └── api/                    # Cloudflare Workers API
│                                Cloudflare Workers API
├── packages/
│   ├── transformer/            # Node.js transformer API
│   │                            Node.js 转换器 API
│   ├── swc-host/               # SWC core hosting for Node.js
│   │                            SWC 宿主层
│   ├── config/                 # Shared TypeScript/ESLint config
│   │                            共享配置
│   └── test-fixtures/          # Test utilities
│                                测试工具
└── tools/                      # Build scripts & Tailwind mapping extraction
                                 构建脚本与映射提取工具
```

### Data Flow / 数据流

```
Source Code          Rust Pipeline                    Output
───────────  ──────────────────────────────  ──────────────────

             ┌─────────┐   ┌───────────┐
  JSX/TSX ──>│tw_parse │──>│ converter │──> CSS Declarations
             └─────────┘   └───────────┘         │
                                                  v
             ┌───────────┐  ┌───────────┐   ┌──────────┐
             │ collector │<─│  bundler  │<──│ context  │
             └───────────┘  └───────────┘   └──────────┘
                  │                              │
                  v                              v
           Class Name Map              CSS Output (with :root)
                  │
                  v
          ┌──────────────┐
          │ jsx_visitor / │──> Transformed Code
          │ html parser  │
          └──────────────┘
```

---

## Development / 开发

```bash
# Run all Rust tests / 运行所有 Rust 测试
cargo test --workspace

# Run specific crate tests / 运行单个 crate 测试
cargo test -p headwind-transform

# Build WASM and run Node.js tests / 构建 WASM 并运行 Node 测试
pnpm build:wasm
node crates/wasm/tests/node_test.mjs

# Start playground in dev mode / 开发模式启动 Playground
pnpm dev:playground

# Full build via Turbo / 通过 Turbo 全量构建
pnpm build
```

---

## Example / 示例

**Input / 输入:**

```jsx
export default function App() {
  return (
    <div className="flex flex-col items-center p-8">
      <h1 className="text-3xl font-bold text-blue-600">
        Hello Headwind
      </h1>
      <button className="mt-6 px-6 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-700">
        Click me
      </button>
    </div>
  );
}
```

**Output Code / 输出代码:**

```jsx
export default function App() {
  return (
    <div className="c_a8f3e2">
      <h1 className="c_b4d1c7">
        Hello Headwind
      </h1>
      <button className="c_e9f2a1">
        Click me
      </button>
    </div>
  );
}
```

**Output CSS / 输出 CSS:**

```css
:root {
  --text-3xl: 1.875rem;
  --text-3xl--line-height: 2.25rem;
}

.c_a8f3e2 {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 2rem;
}

.c_b4d1c7 {
  font-size: var(--text-3xl);
  line-height: var(--text-3xl--line-height);
  font-weight: 700;
  color: rgb(37 99 235);
}

.c_e9f2a1 {
  margin-top: 1.5rem;
  padding: 0.5rem 1.5rem;
  background-color: rgb(59 130 246);
  color: rgb(255 255 255);
  border-radius: 0.5rem;
}

.c_e9f2a1:hover {
  background-color: rgb(29 78 216);
}
```

---

## License / 许可证

MIT
