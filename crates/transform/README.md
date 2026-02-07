# headwind-transform

源码变换层，使用 SWC 进行 AST 级别的 Tailwind 类名替换和 CSS 生成。

## 功能特性

### JSX/TSX 变换

```rust
use headwind_transform::{transform_jsx, TransformOptions};

let source = r#"
export default function App() {
    return <div className="p-4 text-center hover:text-left">Hello</div>;
}
"#;

let result = transform_jsx(source, "App.tsx", TransformOptions::default()).unwrap();
println!("Code:\n{}", result.code);   // className 已替换为生成的类名
println!("CSS:\n{}", result.css);     // 对应的 CSS 规则
```

### HTML 变换

```rust
use headwind_transform::{transform_html, TransformOptions};

let html = r#"<div class="p-4 text-center">Hello</div>"#;
let result = transform_html(html, TransformOptions::default()).unwrap();
```

### 输出模式

**Global 模式**（默认）：直接替换为类名字符串
```
className="p-4 m-2" → className="c_abc123"
```

**CSS Modules 模式**：替换为 `styles.xxx` 引用，自动注入 import
```
className="p-4 m-2" → className={styles.p4M2}
// 文件头部自动注入: import styles from './App.module.css'
```

### 配置选项

| 选项 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `naming_mode` | `NamingMode` | `Hash` | 命名策略（Hash / Readable / CamelCase / Semantic） |
| `output_mode` | `OutputMode` | `Global` | 输出模式（Global / CssModules） |
| `css_variables` | `CssVariableMode` | `Var` | CSS 变量处理方式 |
| `unknown_classes` | `UnknownClassMode` | `Remove` | 未知类名处理 |
| `color_mode` | `ColorMode` | `Hex` | 颜色输出格式 |
| `color_mix` | `bool` | `false` | 使用 color-mix() 处理透明度 |
| `element_tree` | `bool` | `false` | 生成元素树（供 AI 语义命名） |

### 元素树生成

开启 `element_tree` 后，输出结果包含结构化的组件树文本，每个元素附带 `[ref=eN]` 引用标识：

```text
## App
- div w-full h-20 border [ref=e1]
  - h2 text-xl text-red-500 "Title" [ref=e2]
  - p: some text [ref=e3]
```

## 架构

```
transform/src/
├── lib.rs           # 公共 API（transform_jsx, transform_html）
├── collector.rs     # CSS 收集器（调用 tw_index 进行转换和命名）
├── jsx_visitor.rs   # SWC AST 访问器（className/class 属性替换）
├── html.rs          # HTML 正则替换
└── element_tree.rs  # JSX/HTML 元素树构建
```

## 测试

```bash
cargo test -p headwind-transform
```

30+ 个单元测试，覆盖 JSX/HTML 变换、CSS Modules、多种命名模式组合。

## 依赖

- `headwind-core` — 共享类型定义
- `headwind-tw-index` — 转换引擎（Converter, Bundler, naming）
- `swc_core` — JavaScript/TypeScript AST 解析和代码生成
- `indexmap` — 保持插入顺序的 Map
- `blake3` — 内容哈希
