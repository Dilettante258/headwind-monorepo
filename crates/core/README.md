# headwind-core

纯类型定义层，提供所有 crate 共享的数据结构。

## 导出类型

| 类型 | 说明 |
|------|------|
| `Declaration` | CSS 声明（property + value） |
| `BundleRequest` | 转换输入（类名列表 + 命名策略） |
| `BundleResult` | 转换输出（新类名、CSS 声明、诊断信息） |
| `NamingMode` | 命名策略：`Hash` / `Readable` / `CamelCase` / `Semantic` |
| `ColorMode` | 颜色输出模式：`Hex` / `Oklch` / `Hsl` / `Var` |
| `CssVariableMode` | CSS 变量模式：`Var`（引用）/ `Inline`（内联值） |
| `UnknownClassMode` | 未知类名处理：`Remove`（删除）/ `Preserve`（保留） |
| `Diagnostic` | 诊断信息（Warning / Error） |
| `DiagnosticLevel` | 诊断级别枚举 |

## 使用示例

```rust
use headwind_core::{Declaration, NamingMode, ColorMode};

let decl = Declaration::new("padding", "1rem");
assert_eq!(decl.property, "padding");
assert_eq!(decl.value, "1rem");
```

## 设计原则

- **零算法**：只包含类型定义和简单构造函数，不含任何业务逻辑
- **零外部依赖**：仅依赖 `serde` / `serde_json` 进行序列化
- **被所有 crate 共享**：`tw_parse`、`tw_index`、`transform`、`wasm` 均依赖此 crate

## 依赖

- `serde` + `serde_json` — JSON 序列化/反序列化
