# HeadWind Rust Crates

这是 HeadWind 项目的 Rust 核心实现，包含语义内核（Semantic Core）和相关模块。

## 架构

```
crates/
├── core/           # 核心转换逻辑
├── css/            # CSS IR 和输出（使用 swc_css）
├── tw_index/       # Tailwind 规则索引
├── tw_parse/       # Tailwind class 解析器
└── swc_plugin/     # SWC 插件（用于 JavaScript/TypeScript）
```

## 模块说明

### `headwind-core`

核心转换引擎，负责：
- **类名规范化**：去重、排序、拆分
- **CSS 声明合并**：处理属性冲突
- **命名策略**：Hash 或 Readable 模式生成类名
- **Bundle 逻辑**：端到端转换流程

**关键类型**：
- `BundleRequest`: 输入（类名列表 + 命名模式）
- `BundleResult`: 输出（新类名 + CSS 声明 + 诊断）
- `NamingMode`: Hash | Readable | Semantic

### `headwind-css`

CSS 中间表示（IR）和输出：
- **使用 swc_css**：基于 SWC 的官方 CSS AST (`swc_css_ast`, `swc_css_codegen`)
- **IR 结构**：`Stylesheet` → `Rule` → `Declaration`
- **稳定输出**：确保相同输入产生相同输出
- **值解析**：支持常见 CSS 维度值（rem, px, em 等）
- **格式化**：统一缩进、排序、换行

### `headwind-tw-parse`

Tailwind CSS class 解析器：
- **修饰符支持**：响应式、伪类、伪元素、状态修饰符
- **任意值**：支持 `[...]` 语法（如 `w-[13px]`, `bg-[#ff0000]`）
- **完整语法**：负值、透明度、重要性标记
- **结构化输出**：`ParsedClass` 包含修饰符、插件、值等信息
- **参考设计**：基于 [stailwc/tailwind-parse](https://github.com/arlyon/stailwc)

**关键类型**：
- `ParsedClass`: 解析后的 class 结构
- `Modifier`: 修饰符分类（Responsive, PseudoClass, State 等）
- `ParsedValue`: 标准值或任意值

### `headwind-tw-index`

Tailwind 类名索引：
- **JSON 加载**：从 JSON 文件加载类名 → CSS 映射
- **查询接口**：O(1) 查询类名对应的 CSS 声明
- **可扩展**：未来可支持从 Tailwind 配置动态生成

### `swc_plugin`

SWC 插件（WebAssembly）：
- 编译目标：`wasm32-wasip1`
- 用途：在 JavaScript/TypeScript 代码中转换类名

## 快速开始

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p headwind-core

# 运行集成测试
cargo test -p headwind-core --test integration
```

### 运行示例

```bash
cargo run --example basic_usage -p headwind-core
```

### 构建

```bash
# 开发构建
cargo build --workspace

# 发布构建
cargo build --workspace --release

# 构建 SWC 插件（wasm）
cargo build-wasip1 --release
```

## 使用示例

```rust
use headwind_core::{bundle::bundle, BundleRequest, NamingMode};
use headwind_tw_index::load_from_json;
use headwind_css::{emit_css, StyleSheet};

// 1. 加载 Tailwind 索引
let json = r#"[
    {
        "class": "p-4",
        "declarations": [
            { "property": "padding", "value": "1rem" }
        ]
    }
]"#;
let index = load_from_json(json).unwrap();

// 2. 创建转换请求
let request = BundleRequest {
    classes: vec!["p-4".to_string(), "m-2".to_string()],
    naming_mode: NamingMode::Hash,
};

// 3. 执行转换
let result = bundle(request, &index);

// 4. 生成 CSS
let stylesheet = StyleSheet::from_declarations(
    result.new_class,
    result.css_declarations,
);
let css = emit_css(&stylesheet);

println!("{}", css);
// 输出:
// .c_874b3c39f45d {
//   padding: 1rem;
// }
```

## 功能特性

### ✅ 已实现

- ✅ 类名规范化（去重、排序）
- ✅ CSS 声明合并（冲突处理）
- ✅ Hash 命名（稳定 hash）
- ✅ Readable 命名（可读前缀）
- ✅ JSON 索引加载
- ✅ CSS IR 和输出（基于 swc_css）
- ✅ **Tailwind class 解析器**（支持修饰符、任意值等）
- ✅ **修饰符支持**（响应式、伪类、伪元素、状态）
- ✅ **任意值支持**（`w-[13px]`, `bg-[#ff0000]` 等）
- ✅ 完整测试覆盖（**50 个测试**）
- ✅ 集成测试
- ✅ 示例代码

### 🚧 未来计划

- ⏳ 将 tw_parse 集成到 tw_index（支持复杂 class 查询）
- ⏳ 支持 @media 和 @layer 规则
- ⏳ AI 语义命名
- ⏳ 从 Tailwind CSS 文件解析索引
- ⏳ 支持更多变体类型（`@supports`, `max-*` 等）

## 测试统计

```
Total: 50 tests
├── headwind-core: 21 tests (17 unit + 4 integration)
├── headwind-css: 6 tests
├── headwind-tw-index: 6 tests
├── headwind-tw-parse: 16 tests
└── swc_plugin: 1 test
```

## 设计原则

1. **确定性**：相同输入永远产生相同输出
2. **可测试**：每个模块都有单元测试
3. **解耦**：模块之间通过 trait 交互
4. **可扩展**：支持未来添加新功能

## 依赖

```toml
indexmap = "2.0"    # 保持插入顺序
blake3 = "1.5"      # 快速 hash
serde = "1.0"       # JSON 序列化
```

## 性能

- Hash 计算：使用 `blake3`（快速）
- 查询：O(1)（HashMap）
- 排序：O(n log n)（BTreeSet）
- 合并：O(n)（IndexMap）

## 许可证

MIT
