# headwind-tw-index

HeadWind 的主要转换引擎，负责将 Tailwind 类名转换为 CSS。

## 架构

```
tw_index/src/
├── converter/          # Tailwind → CSS 转换器
│   ├── mod.rs          # Converter 结构体 + 公共 API
│   ├── standard.rs     # 标准值分发（text, bg, font, border 等 50+ 插件）
│   ├── arbitrary.rs    # 任意值 + CSS 变量声明构建
│   ├── valueless.rs    # 无值类静态映射（flex, hidden, relative 等）
│   ├── color.rs        # 颜色处理（alpha 透明度、color-mix）
│   └── selector.rs     # CSS 选择器构建 + 断点映射
├── bundler.rs          # 批量转换 + CSS 生成（修饰符分组、上下文模式）
├── context.rs          # CSS 类上下文（按修饰符分组声明）
├── css/                # CSS IR 和输出（基于 SWC CSS AST）
│   ├── ir.rs           # Stylesheet/Rule/Declaration IR
│   └── emit.rs         # CSS 字符串输出
├── bundle.rs           # 端到端转换（normalize → merge → naming）
├── naming.rs           # 类名生成策略（Hash/Readable/CamelCase）
├── shorthand.rs        # CSS 简写属性优化
├── normalize.rs        # 类名规范化（去重、排序）
├── merge.rs            # CSS 声明合并（冲突处理）
├── index.rs            # Tailwind 类名索引
├── loader.rs           # JSON 索引加载
├── plugin_map.rs       # 插件名 → CSS 属性映射
├── value_map.rs        # 值推断（spacing、color、opacity 等）
├── palette.rs          # Tailwind 完整调色板
├── theme_values.rs     # 主题值（text-size、blur-size、font-family）
└── variant.rs          # 修饰符 → CSS 选择器/at-rule 解析
```

## 核心功能

### 1. Converter — 基于规则的转换

无需外部索引，直接将 Tailwind 类名转换为 CSS 声明：

```rust
use headwind_tw_index::Converter;
use headwind_tw_parse::parse_class;

let converter = Converter::new();

// 标准值
let parsed = parse_class("p-4").unwrap();
let rule = converter.convert(&parsed).unwrap();
// → padding: 1rem

// 任意值
let parsed = parse_class("w-[13px]").unwrap();
let rule = converter.convert(&parsed).unwrap();
// → width: 13px

// 颜色 + 透明度
let parsed = parse_class("bg-blue-500/60").unwrap();
let decls = converter.to_declarations(&parsed).unwrap();
// → background: #3b82f699

// CSS 变量（v4）
let parsed = parse_class("bg-(--my-color)").unwrap();
let decls = converter.to_declarations(&parsed).unwrap();
// → background: var(--my-color)
```

### 2. Bundler — 批量转换 + CSS 生成

将多个 Tailwind 类转换并生成完整 CSS（含修饰符分组和简写优化）：

```rust
use headwind_tw_index::Bundler;
use headwind_core::NamingMode;

let bundler = Bundler::new();
let result = bundler.bundle_to_css(&["p-4", "m-2", "hover:text-white"]);
// 生成：
// .className { padding: 1rem; margin: 0.5rem; }
// @media (hover: hover) { .className:hover { color: #ffffff; } }
```

### 3. 颜色模式

```rust
use headwind_tw_index::Converter;
use headwind_core::ColorMode;

// Hex（默认）
Converter::new();                                    // #3b82f6

// OKLCH
Converter::new().with_color_mode(ColorMode::Oklch);  // oklch(0.623 0.214 259.815)

// HSL
Converter::new().with_color_mode(ColorMode::Hsl);    // hsl(217, 91%, 60%)

// CSS 变量
Converter::new().with_color_mode(ColorMode::Var);     // var(--color-blue-500)

// color-mix（统一透明度处理）
Converter::new().with_color_mix(true);                // color-mix(in oklab, #3b82f6 60%, transparent)
```

## 测试

```bash
cargo test -p headwind-tw-index
```

253 个单元测试 + 4 个集成测试 + 3 个官方映射验证测试。

## 依赖

- `headwind-core` — 共享类型定义
- `headwind-tw-parse` — Tailwind 类名解析器
- `blake3` — 快速内容哈希
- `indexmap` — 保持插入顺序的 Map
- `phf` — 编译期完美哈希映射
- `swc_css_*` — CSS AST 和代码生成
