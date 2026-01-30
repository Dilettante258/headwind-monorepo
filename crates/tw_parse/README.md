# headwind-tw-parse

Tailwind CSS class 名称解析器，支持完整的 Tailwind 语法。

## 功能特性

✅ **修饰符（Modifiers）**
- 响应式断点：`sm:`, `md:`, `lg:`, `xl:`, `2xl:`
- 伪类：`hover:`, `focus:`, `active:`, `disabled:` 等
- 伪元素：`before:`, `after:`, `placeholder:` 等
- 状态修饰符：`dark:`, `group-hover:`, `peer-focus:` 等
- 支持多层嵌套：`md:hover:dark:bg-blue-500`

✅ **任意值（Arbitrary Values）**
- 尺寸任意值：`w-[13px]`, `h-[calc(100vh-64px)]`
- 颜色任意值：`bg-[#ff0000]`, `text-[rgb(255,0,0)]`
- 嵌套括号：`grid-cols-[repeat(3,minmax(0,1fr))]`

✅ **其他特性**
- 负值：`-m-4`, `md:-top-1`
- 透明度修饰符：`bg-blue-500/50`, `text-black/75`
- 重要性标记：`p-4!`, `md:bg-red-500!`

## 使用示例

### 基本解析

```rust
use headwind_tw_parse::parse_class;

let parsed = parse_class("md:hover:bg-blue-500/50!").unwrap();

assert_eq!(parsed.modifiers.len(), 2);  // md, hover
assert_eq!(parsed.plugin, "bg");
assert_eq!(parsed.alpha, Some("50".to_string()));
assert!(parsed.important);
```

### 解析结果结构

```rust
pub struct ParsedClass {
    /// 修饰符列表（如 hover, md, dark）
    pub modifiers: Vec<Modifier>,

    /// 是否为负值（如 -m-4）
    pub negative: bool,

    /// 核心插件/命令（如 p, m, bg, text）
    pub plugin: String,

    /// 值部分
    pub value: Option<ParsedValue>,

    /// 透明度修饰符（如 /50）
    pub alpha: Option<String>,

    /// 重要性标记（!）
    pub important: bool,
}
```

### 修饰符分类

```rust
pub enum Modifier {
    /// 响应式断点（sm, md, lg, xl, 2xl）
    Responsive(String),

    /// 伪类（hover, focus, active 等）
    PseudoClass(String),

    /// 伪元素（before, after, placeholder 等）
    PseudoElement(String),

    /// 状态修饰符（dark, group-hover, peer-focus 等）
    State(String),

    /// 自定义修饰符
    Custom(String),
}
```

### 值类型

```rust
pub enum ParsedValue {
    /// 标准值（如 "4", "red-500", "lg"）
    Standard(String),

    /// 任意值（如 "[13px]", "[#ff0000]"）
    Arbitrary(ArbitraryValue),
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
| `w-[13px]` | `w` | `[13px]` | - | - |
| `bg-blue/50` | `bg` | `blue` | - | alpha=50 |
| `p-4!` | `p` | `4` | - | important=true |
| `grid-cols-[repeat(3,1fr)]` | `grid-cols` | `[repeat(3,1fr)]` | - | - |

## 设计说明

### 插件名称识别

解析器使用以下策略识别插件（plugin）和值（value）：

1. **优先查找 `-[` 模式**：如果存在 `-[`，则将其之前的部分作为 plugin
   - 例如：`grid-cols-[...]` → plugin=`grid-cols`，value=`[...]`

2. **默认在第一个 `-` 分割**：如果没有 `-[`，则在第一个 `-` 处分割
   - 例如：`p-4` → plugin=`p`，value=`4`
   - 例如：`bg-red-500` → plugin=`bg`，value=`red-500`

### 修饰符自动分类

解析器会自动将修饰符分类为：
- **响应式**：`sm`, `md`, `lg`, `xl`, `2xl`
- **伪类**：`hover`, `focus`, `active`, `visited` 等
- **伪元素**：`before`, `after`, `placeholder` 等
- **状态**：`dark`, `group-hover`, `peer-focus` 等

未识别的修饰符会被标记为 `Custom`。

## 限制和未来改进

### 当前限制

- 不验证 plugin 是否为有效的 Tailwind utility
- 不解析复杂的变体组合（如 `@`、`max-*` 等）
- 不处理 Tailwind 配置文件

### 未来改进

- [ ] 支持更多变体类型（`@media`, `@supports` 等）
- [ ] 与 Tailwind 配置文件集成
- [ ] 插件名称验证
- [ ] 性能优化（零拷贝解析）

## 测试

```bash
# 运行所有测试
cargo test -p headwind-tw-parse

# 查看测试覆盖
cargo test -p headwind-tw-parse -- --nocapture
```

## 参考

此解析器的设计参考了 [stailwc/tailwind-parse](https://github.com/arlyon/stailwc/tree/master/crates/tailwind-parse)。
