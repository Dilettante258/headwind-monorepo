# Headwind 架构文档

## 项目概述

Headwind 是一个完整的 Tailwind CSS 工具链，包括解析、索引、转换和 CSS 生成功能。

## 核心架构

```
用户输入: "p-4 hover:bg-blue-500 md:text-center"
    ↓
┌─────────────────────────────────────────────────────────┐
│ 1. tw_parse (解析层)                                     │
│    - 手写递归下降解析器                                   │
│    - 零外部依赖                                          │
│    - 输出: ParsedClass                                   │
└─────────────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────────────┐
│ 2. tw_index (转换层)                                     │
│    a. Converter - 类名到声明的转换                       │
│       - 优先级: 官方映射 → 任意值 → 值映射推断           │
│       - 输出: CssRule (selector + declarations)         │
│                                                          │
│    b. Bundler - 多个类打包成规则组                       │
│       - 按修饰符分组 (base, pseudo, responsive, etc.)   │
│       - 输出: RuleGroup                                  │
└─────────────────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────────────────┐
│ 3. css (生成层) - 基于 SWC                               │
│    - create_stylesheet: 创建 SWC AST                     │
│    - emit_css: 生成 CSS 字符串                           │
│    - 输出: 格式化的 CSS 代码                             │
└─────────────────────────────────────────────────────────┘
    ↓
输出 CSS:
.my-class {
  padding: 1rem;
}
.my-class:hover {
  background: #3b82f6;
}
@media (min-width: 768px) {
  .my-class {
    text-align: center;
  }
}
```

## Crate 职责

### 1. tw_parse - Tailwind 类名解析

**职责**: 将 Tailwind 类名字符串解析为结构化的 AST

**特点**:
- ✅ 手写递归下降解析器
- ✅ 零运行时依赖
- ✅ 完整支持 Tailwind 语法
- ✅ 性能优化（O(n) 时间复杂度）

**输出**:
```rust
ParsedClass {
    modifiers: Vec<Modifier>,   // [Responsive("md"), PseudoClass("hover")]
    raw_modifiers: String,      // "md:hover:" - 原始修饰符字符串
    negative: bool,             // -m-4
    plugin: String,             // "bg"
    value: Option<ParsedValue>, // Standard("blue-500") | Arbitrary("13px")
    alpha: Option<String>,      // /50
    important: bool,            // !
}
```

**使用**:
```rust
// 批量解析（推荐 - 性能优化）
let parsed = parse_classes("p-4 hover:bg-blue-500 md:text-center")?;
// 返回 Vec<ParsedClass>

// 单个类解析（特例）
let parsed = parse_class("md:hover:bg-blue-500")?;
```

### 2. tw_index - 索引与转换

#### 2.1 TailwindIndex - 类名索引

**职责**: 存储和查询类名到 CSS 声明的映射

```rust
let index = load_from_official_json(json)?; // 加载 752 个官方类
let decls = index.lookup("absolute")?;      // 查询
```

#### 2.2 Converter - 单类转换

**职责**: 将单个 ParsedClass 转换为 CSS 规则

**转换优先级**:
1. **官方映射** (official-mappings.json) - 100% 准确
2. **任意值** ([...]) - w-[13px] → width: 13px
3. **值映射推断** - p-4 → padding: 1rem

```rust
let converter = Converter::new(&index);
let rule = converter.convert(&parsed)?;
// CssRule { selector: ".bg-blue-500", declarations: [...] }
```

#### 2.3 Bundler - 多类打包

**职责**: 将多个类整理成一个 CSS 类 + 各种选择器

**关键方法**:
- `bundle()` - 打包类名到 RuleGroup
- `generate_css()` - 生成 CSS（字符串拼接）
- `generate_css_with_swc()` - **使用 SWC 生成基础规则** ⬅️ 新增！
- `generate_css_hybrid()` - 混合方式：SWC + 字符串

```rust
let bundler = Bundler::new(converter);
let group = bundler.bundle("p-4 hover:p-8 md:p-12")?;

// 方式 1: 纯字符串生成
let css = bundler.generate_css("my-class", &group, "  ");

// 方式 2: 使用 SWC（推荐）
let css = bundler.generate_css_with_swc("my-class", &group)?;
```

**RuleGroup 结构**:
```rust
RuleGroup {
    base: Vec<Declaration>,                           // 基础规则
    pseudo_classes: HashMap<String, Vec<Declaration>>, // hover, focus
    pseudo_elements: HashMap<String, Vec<Declaration>>,// before, after
    responsive: HashMap<String, Box<RuleGroup>>,      // sm, md, lg
    states: HashMap<String, Box<RuleGroup>>,          // dark, group-hover
}
```

### 3. css - CSS 生成（基于 SWC）

**职责**: 使用 SWC 生成标准的 CSS AST 和代码

**核心函数**:
```rust
// 创建 SWC Stylesheet
pub fn create_stylesheet(class_name: String, declarations: Vec<Declaration>) -> Stylesheet;

// 生成 CSS 字符串
pub fn emit_css(stylesheet: &Stylesheet) -> Result<String, std::fmt::Error>;
```

**使用**:
```rust
use headwind_css::{create_stylesheet, emit_css};

let stylesheet = create_stylesheet(
    "my-class".to_string(),
    vec![Declaration::new("padding", "1rem")],
);

let css = emit_css(&stylesheet)?;
// 输出: .my-class { padding: 1rem; }
```

**优势**:
- ✅ 使用 SWC 的标准 CSS 生成器
- ✅ 格式一致性保证
- ✅ 正确处理特殊字符和转义
- ✅ 可扩展到复杂的 CSS 结构

### 4. plugin_map - 插件映射

**职责**: 映射 Tailwind 插件名到 CSS 属性

```rust
get_plugin_properties("p")    // → ["padding"]
get_plugin_properties("px")   // → ["padding-left", "padding-right"]
get_plugin_properties("w")    // → ["width"]
```

**支持的插件**: 90+ (间距、尺寸、颜色、变换等)

### 5. value_map - 值映射

**职责**: 映射 Tailwind 值到 CSS 值

```rust
infer_value("p", "4")         // → Some("1rem")
infer_value("bg", "blue-500") // → Some("#3b82f6")
infer_value("opacity", "50")  // → Some("0.5")
```

**支持的值**:
- 间距: 0-96 (基于 Tailwind 默认配置)
- 颜色: gray, blue, red, green (50-900)
- 不透明度: 0-100
- 分数: 1/2, 1/3, 3/4 等

### 6. headwind-core - 核心类型

**职责**: 共享的类型定义

```rust
pub struct Declaration {
    pub property: String,  // "padding"
    pub value: String,     // "1rem"
}

pub struct BundleRequest {
    pub classes: Vec<String>,
    pub naming_mode: NamingMode,
}
```

## 数据流

### 完整流程示例

```rust
use headwind_tw_index::{
    load_from_official_json, Converter, Bundler,
};
use headwind_tw_parse::parse_class;
use headwind_css::emit_css;

// 1. 加载索引
let json = include_str!("official-mappings.json");
let index = load_from_official_json(json)?;

// 2. 创建转换器和打包器
let converter = Converter::new(&index);
let bundler = Bundler::new(converter);

// 3. 打包类名
let classes = "p-4 hover:bg-blue-500 md:text-center";
let group = bundler.bundle(classes)?;

// 4. 生成 CSS (使用 SWC)
let css = bundler.generate_css_with_swc("my-class", &group)?;

println!("{}", css);
```

输出:
```css
.my-class {
  padding: 1rem;
}
```

## 关键设计决策

### 1. 为什么手写解析器？

✅ **优势**:
- 零依赖，编译快
- 完全控制解析逻辑
- 针对 Tailwind 语法优化
- 更好的错误提示

❌ **劣势**:
- 需要手动维护
- 相比 nom/pest 代码更多

**结论**: 对于明确的语法，手写解析器是最佳选择

### 2. 为什么使用 SWC？

✅ **优势**:
- 标准的 CSS AST
- 正确的格式化
- 处理特殊字符和转义
- 可扩展性强

❌ **劣势**:
- API 复杂
- 编译依赖较重

**结论**: 基础规则使用 SWC，复杂规则暂时用字符串（混合方式）

### 3. 为什么三层转换优先级？

1. **官方映射** - 最准确，覆盖特殊类
2. **任意值** - 灵活性，用户自定义
3. **值映射** - 轻量级，常用值推断

**优势**: 既保证准确性，又提供灵活性和轻量级方案

### 4. 为什么分离 tw_parse 和 tw_index？

- **tw_parse**: 纯解析，零依赖，可独立使用
- **tw_index**: 依赖映射和配置，业务逻辑

**优势**: 清晰的职责分离，更好的可测试性

## 性能特点

| 组件 | 时间复杂度 | 空间复杂度 | 备注 |
|------|-----------|-----------|------|
| tw_parse | O(n) | O(1) | n = 类名长度 |
| parse_classes | O(n) | O(k) | 批量解析，k = 类数量 |
| TailwindIndex | O(1) | O(m) | m = 映射数量 |
| Converter | O(1) | O(1) | HashMap 查找 |
| Bundler | O(k) | O(k) | k = 类名数量 |
| CSS 生成 (SWC) | O(d) | O(d) | d = 声明数量 |

**总体**: O(k) 线性复杂度，k 为输入类名数量

**性能优化**:
- ✅ 批量解析：`parse_classes` 一次性处理多个类，减少重复字符串操作
- ✅ 原始修饰符缓存：`raw_modifiers` 字段避免重复拼接
- ✅ 单遍扫描：解析器对每个字符只访问一次
- ✅ 零拷贝设计：尽可能使用字符串切片而非复制

## 测试覆盖

| Crate | 单元测试 | 集成测试 | 总计 |
|-------|---------|---------|------|
| tw_parse | 16 | 2 | 18 |
| tw_index | 33 | 4 | 37 |
| css | 6 | 0 | 6 |
| core | 17 | 4 | 21 |
| value_map | 4 | 0 | 4 |
| plugin_map | 3 | 0 | 3 |
| **总计** | **79** | **10** | **89** |

**覆盖率**:
- ✅ 100% 官方映射验证 (752/752)
- ✅ 81% 复杂用例覆盖 (34/42)
- ✅ 所有核心功能测试通过

## 未来扩展

### 短期（已规划）

1. ✅ 值映射系统 - 已完成
2. ✅ SWC 集成 - 已完成（基础规则）
3. ⬜ 完整 SWC 支持 - 包括伪类、媒体查询
4. ⬜ Alpha 值支持 - bg-blue-500/50

### 中期

1. ⬜ 更多颜色支持 - 扩展 value_map
2. ⬜ 自定义主题配置
3. ⬜ JIT 模式支持
4. ⬜ 插件系统

### 长期

1. ⬜ VS Code 扩展
2. ⬜ CLI 工具
3. ⬜ 实时预览
4. ⬜ 性能分析工具

## 总结

Headwind 采用清晰的三层架构：
1. **解析层** (tw_parse) - 手写解析器，高性能
2. **转换层** (tw_index) - 灵活的多级转换系统
3. **生成层** (css) - 基于 SWC 的标准 CSS 生成

**核心优势**:
- ✅ 模块化设计，职责清晰
- ✅ 高性能，零运行时依赖（解析层）
- ✅ 灵活的转换系统（官方映射 + 值推断 + 任意值）
- ✅ 使用 SWC 保证 CSS 质量
- ✅ 全面的测试覆盖

**适用场景**:
- Tailwind 静态分析工具
- CSS-in-JS 转换
- 代码优化和检查
- 组件库开发
