# headwind-tw-parse

Tailwind CSS class 名称解析器，支持完整的 Tailwind v3/v4 语法。

## 功能特性

**修饰符（Modifiers）**
- 响应式断点：`sm:`, `md:`, `lg:`, `xl:`, `2xl:`
- 伪类：`hover:`, `focus:`, `active:`, `disabled:` 等
- 伪元素：`before:`, `after:`, `placeholder:` 等
- 状态修饰符：`dark:`, `group-hover:`, `peer-focus:` 等
- 参数化修饰符：`has-[.foo]:`, `not-[.bar]:`, `aria-[checked]:`, `data-[state=open]:`
- 容器查询：`@md:`, `@[400px]:`
- 支持多层嵌套：`md:hover:dark:bg-blue-500`

**值语法**
- 标准值：`p-4`, `bg-red-500`, `text-lg`
- 任意值：`w-[13px]`, `bg-[#ff0000]`, `grid-cols-[repeat(3,1fr)]`
- CSS 变量（v4）：`bg-(--my-color)`, `text-(length:--size)`, `bg-linear-(--gradient)`
- 负值：`-m-4`, `md:-top-1`
- 透明度修饰符：`bg-blue-500/50`, `text-black/75`
- 重要性标记：`p-4!`, `md:bg-red-500!`

**复合插件**
- 自动识别多段插件：`justify-items`, `gap-x`, `border-t`, `translate-x`, `scroll-mt` 等
- 正确分割插件名和值：`justify-items-center` → plugin=`justify-items`, value=`center`

## 使用示例

```rust
use headwind_tw_parse::{parse_class, parse_classes};

// 单个类解析
let parsed = parse_class("md:hover:bg-blue-500/50!").unwrap();
assert_eq!(parsed.plugin, "bg");
assert_eq!(parsed.alpha, Some("50".to_string()));
assert!(parsed.important);

// 批量解析
let classes = parse_classes("p-4 m-2 hover:text-white");
assert_eq!(classes.len(), 3);
```

## 解析结果

```rust
pub struct ParsedClass {
    pub modifiers: Vec<Modifier>,   // 修饰符列表
    pub negative: bool,             // 是否负值
    pub plugin: String,             // 插件名（如 p, bg, text）
    pub value: Option<ParsedValue>, // 值部分
    pub alpha: Option<String>,      // 透明度修饰符
    pub important: bool,            // !important 标记
}

pub enum ParsedValue {
    Standard(String),               // 标准值：p-4 → "4"
    Arbitrary(ArbitraryValue),      // 任意值：w-[13px] → "13px"
    CssVariable(CssVariableValue),  // CSS 变量：bg-(--color) → "--color"
}
```

## 解析示例

| 输入 | Plugin | Value | 修饰符 | 其他 |
|------|--------|-------|--------|------|
| `p-4` | `p` | `4` | - | - |
| `bg-red-500` | `bg` | `red-500` | - | - |
| `hover:bg-blue` | `bg` | `blue` | `hover` | - |
| `md:hover:p-4` | `p` | `4` | `md`, `hover` | - |
| `-m-4` | `m` | `4` | - | negative=true |
| `w-[13px]` | `w` | `[13px]` | - | arbitrary |
| `bg-blue/50` | `bg` | `blue` | - | alpha=50 |
| `p-4!` | `p` | `4` | - | important=true |
| `bg-(--my-color)` | `bg` | `--my-color` | - | css variable |
| `justify-items-center` | `justify-items` | `center` | - | compound plugin |

## 测试

```bash
cargo test -p headwind-tw-parse
```

67 个单元测试 + 2 个集成测试（官方映射验证）。

## 参考

解析器设计参考了 [stailwc/tailwind-parse](https://github.com/arlyon/stailwc/tree/master/crates/tailwind-parse)。
