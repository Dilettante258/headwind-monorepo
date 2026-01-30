# headwind-tw-index

Tailwind CSS 类名索引和转换器。

## 功能

### 1. 索引加载

从不同格式的 JSON 文件加载 Tailwind 类名到 CSS 的映射：

```rust
use headwind_tw_index::{load_from_official_json, TailwindIndex};

// 从官方映射格式加载（来自 tailwindcss.com 文档）
let json = r#"[
  {
    "class": "absolute",
    "css": "position: absolute",
    "source": "/src/docs/position.mdx"
  }
]"#;

let index = load_from_official_json(json)?;
println!("Loaded {} classes", index.len());

// 查询类名
if let Some(decls) = index.lookup("absolute") {
    for decl in decls {
        println!("{}: {}", decl.property, decl.value);
    }
}
```

### 2. CSS 声明解析

自动解析 CSS 字符串：

```rust
// 单个声明
"position: absolute" → Declaration { property: "position", value: "absolute" }

// 多个声明
"padding: 1rem; margin: 2rem" → [
    Declaration { property: "padding", value: "1rem" },
    Declaration { property: "margin", value: "2rem" }
]
```

### 3. 类名转换

将解析后的 Tailwind 类名转换为 CSS 规则：

```rust
use headwind_tw_index::Converter;
use headwind_tw_parse::parse_class;

let converter = Converter::new(&index);

// 简单类
let parsed = parse_class("text-center")?;
let rule = converter.convert(&parsed)?;
// → Selector: .text-center
//   text-align: center

// 带修饰符
let parsed = parse_class("hover:text-center")?;
let rule = converter.convert(&parsed)?;
// → Selector: .text-center:hover
//   text-align: center

// 响应式
let parsed = parse_class("md:text-center")?;
let rule = converter.convert(&parsed)?;
// → Selector: @media (min-width: 768px) { .text-center }
//   text-align: center
```

### 4. 任意值支持

支持 Tailwind 的任意值语法：

```rust
// 自定义宽度
let parsed = parse_class("w-[13px]")?;
let rule = converter.convert(&parsed)?;
// → width: 13px

// 自定义颜色
let parsed = parse_class("text-[#1da1f2]")?;
let rule = converter.convert(&parsed)?;
// → color: #1da1f2

// 多属性插件（px 同时设置 left 和 right）
let parsed = parse_class("px-[2rem]")?;
let rule = converter.convert(&parsed)?;
// → padding-left: 2rem
//   padding-right: 2rem
```

### 5. 修饰符支持

支持各种 Tailwind 修饰符：

- **伪类**: `hover:`, `focus:`, `active:`, `disabled:`, etc.
- **伪元素**: `before:`, `after:`, `placeholder:`, etc.
- **响应式**: `sm:`, `md:`, `lg:`, `xl:`, `2xl:`
- **状态**: `dark:`, `group-hover:`, `peer-focus:`, etc.

可以组合使用：

```rust
let parsed = parse_class("md:hover:text-center")?;
// → @media (min-width: 768px) { .text-center }:hover
```

### 6. 其他特性

- **负值**: `-indent-px`, `-translate-x-full`
- **Important**: `text-center!` → `text-align: center !important`
- **Alpha 值**: `bg-blue-500/50` → 50% 不透明度
- **CSS 变量**: 正确处理 Tailwind 的 CSS 变量（如 `var(--tw-translate-y)`）

## 测试

项目包含全面的测试：

```bash
# 运行所有测试
cargo test -p headwind-tw-index

# 运行转换示例
cargo run -p headwind-tw-index --example convert_classes

# 运行验证示例（验证所有 752 个官方类）
cargo run -p headwind-tw-index --example validate_mappings
```

测试覆盖：
- ✅ 索引加载和查询（3 个测试）
- ✅ CSS 声明解析（5 个测试）
- ✅ 类名转换（11 个测试）
- ✅ 任意值处理（5 个测试）
- ✅ 修饰符处理（6 个测试）
- ✅ 插件映射（3 个测试）
- ✅ 官方映射验证（3 个集成测试）

**验证结果：**
- ✅ **752/752 官方类 100% 验证通过**
- ✅ 所有类都能正确解析
- ✅ 所有类都能正确转换为 CSS
- ✅ 支持 50+ 不同的 Tailwind 插件

## 官方映射数据

项目包含从 Tailwind CSS 官网提取的 **752 个**官方类映射，存储在：

```
fixtures/official-mappings.json
```

这些映射来自 [tailwindcss.com](https://tailwindcss.com) 官方文档，包含：
- 布局类（position, display, flex, grid 等）
- 间距类（padding, margin）
- 尺寸类（width, height）
- 排版类（text-align, font, line-height 等）
- 背景和边框类
- 效果和滤镜类
- 变换类（translate, rotate, scale 等）

## 架构

```
tw_index/
├── src/
│   ├── index.rs        # 索引数据结构和查询
│   ├── loader.rs       # JSON 加载和 CSS 解析
│   ├── converter.rs    # 类名到 CSS 的转换
│   ├── plugin_map.rs   # 插件名到 CSS 属性的映射
│   └── lib.rs
├── tests/
│   └── official_mappings.rs  # 集成测试
├── fixtures/
│   └── official-mappings.json  # 官方映射数据
└── examples/
    └── convert_classes.rs      # 使用示例
```

## 依赖

- `headwind-core`: 核心类型定义（Declaration, etc.）
- `headwind-tw-parse`: Tailwind 类名解析器
- `serde`, `serde_json`: JSON 序列化

## 使用场景

1. **静态分析工具**: 分析项目中使用的 Tailwind 类
2. **CSS 生成器**: 将 Tailwind 类转换为原生 CSS
3. **IDE 插件**: 提供自动补全和悬浮提示
4. **优化工具**: 分析和优化 Tailwind 使用情况
5. **测试工具**: 验证 Tailwind 类的正确性
