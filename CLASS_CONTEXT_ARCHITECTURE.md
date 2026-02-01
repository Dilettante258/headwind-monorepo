# ClassContext 架构实现

## 概述

ClassContext 是一个更简洁、更高效的架构，用于处理 Tailwind CSS 类的转换和 CSS 生成。它解决了之前实现中的复杂性问题，提供了更清晰的关注点分离。

## 核心思想

### 1. ParsedClass 作为"写操作"

将每个 `ParsedClass` 视为对 CSS 上下文的一次写操作，而不是独立的转换单元。这样可以：
- 简化转换逻辑
- 自动处理声明合并
- 避免区分单声明和多声明的复杂性

### 2. 按 raw_modifiers 分组优化

相同修饰符的类会被分组处理，减少重复的选择器生成：

```rust
// 示例：这三个类有相同的 raw_modifiers ("hover:")
"hover:p-4 hover:m-2 hover:text-center"

// 会生成一个选择器：
.my-class:hover {
  padding: 1rem;
  margin: 0.5rem;
  text-align: center;
}
```

### 3. 关注点分离

- **Converter**: 只负责生成 CSS 声明（`Declaration`）
- **ClassContext**: 负责收集声明、生成选择器、输出 CSS
- **Bundler**: 协调整个流程

## 核心类型

### ClassContext

```rust
pub struct ClassContext {
    pub class_name: String,
    groups: HashMap<String, (Vec<Modifier>, Vec<Declaration>)>,
}

impl ClassContext {
    /// 创建新的上下文
    pub fn new(class_name: String) -> Self

    /// 写入声明（相同 raw_modifiers 会自动合并）
    pub fn write(&mut self, raw_modifiers: &str, modifiers: Vec<Modifier>, declarations: Vec<Declaration>)

    /// 生成 CSS
    pub fn to_css(&self, indent: &str) -> String
}
```

### 数据流

```
Input: "p-4 hover:p-8 md:p-12"
  ↓
parse_classes()
  ↓
Vec<ParsedClass>
  ↓
按 raw_modifiers 分组
  ↓
对每个 ParsedClass:
  Converter::to_declarations() → Vec<Declaration>
  ↓
  ClassContext::write()
  ↓
ClassContext::to_css()
  ↓
Output: CSS 字符串
```

## API 使用

### 新 API（推荐）

```rust
use headwind_tw_index::Bundler;

let bundler = Bundler::new();

// 方法 1: 获取 ClassContext 对象
let context = bundler
    .bundle_to_context("my-class", "p-4 hover:p-8 md:p-12")
    .unwrap();
let css = context.to_css("  ");

// 方法 2: 直接获取 CSS 字符串
let css = bundler
    .bundle_to_css("my-class", "p-4 hover:p-8 md:p-12", "  ")
    .unwrap();
```

### 旧 API（向后兼容）

```rust
// 仍然支持，返回 RuleGroup
let group = bundler.bundle("p-4 hover:p-8").unwrap();
let css = bundler.generate_css("my-class", &group, "  ");
```

## 实现细节

### Converter 简化

添加了新方法 `to_declarations()`，只生成声明，不生成选择器：

```rust
impl Converter {
    /// 将 ParsedClass 转换为 CSS 声明（新架构）
    pub fn to_declarations(&self, parsed: &ParsedClass) -> Option<Vec<Declaration>> {
        let declarations = if let Some(value) = &parsed.value {
            if matches!(value, ParsedValue::Arbitrary(_)) {
                self.build_arbitrary_declarations(parsed)?
            } else {
                self.build_standard_declarations(parsed)?
            }
        } else {
            self.build_valueless_declarations(parsed)?
        };

        let declarations = if parsed.important {
            self.apply_important(declarations)
        } else {
            declarations
        };

        Some(declarations)
    }
}
```

### Bundler 实现

```rust
pub fn bundle_to_context(
    &self,
    class_name: &str,
    classes: &str,
) -> Result<ClassContext, String> {
    let mut context = ClassContext::new(class_name.to_string());

    // 一次性解析所有类名
    let parsed_list = parse_classes(classes)?;

    // 按 raw_modifiers 分组（优化）
    let mut grouped: HashMap<String, Vec<ParsedClass>> = HashMap::new();
    for parsed in parsed_list {
        grouped
            .entry(parsed.raw_modifiers.clone())
            .or_insert_with(Vec::new)
            .push(parsed);
    }

    // 处理每个分组：每个 ParsedClass 作为一个"写操作"
    for (raw_mods, classes) in grouped {
        for parsed in classes {
            if let Some(declarations) = self.converter.to_declarations(&parsed) {
                // 写入 context（相同 raw_modifiers 的声明会自动合并）
                context.write(&raw_mods, parsed.modifiers.clone(), declarations);
            }
        }
    }

    Ok(context)
}
```

## 优势

### 1. 代码更简洁

- Converter 不再需要处理选择器生成
- 不需要区分单声明和多声明插件
- 自动处理声明合并

### 2. 性能优化

通过 `raw_modifiers` 分组：
- 相同修饰符的类只生成一个选择器
- 减少重复的 CSS 输出
- 提高处理效率

### 3. 更好的关注点分离

```
Converter (转换逻辑)
    ↓
    只关心：plugin → CSS 声明

ClassContext (CSS 生成)
    ↓
    只关心：修饰符 → 选择器

Bundler (流程协调)
    ↓
    协调 Converter 和 ClassContext
```

### 4. 易于扩展

添加新的修饰符类型只需要：
1. 更新 `ClassContext::generate_selector_with_modifiers()`
2. 不需要修改 Converter

## 测试覆盖

新架构有完整的测试覆盖：

```bash
# ClassContext 测试
test context::tests::test_context_basic
test context::tests::test_context_merge_same_modifiers
test context::tests::test_context_with_modifiers

# Bundler 新 API 测试
test bundler::tests::test_bundle_to_context_basic
test bundler::tests::test_bundle_to_context_with_hover
test bundler::tests::test_bundle_to_context_with_responsive
test bundler::tests::test_bundle_to_context_grouping_optimization
test bundler::tests::test_bundle_to_css_convenience
test bundler::tests::test_bundle_to_context_complex
```

所有 106 个测试通过 ✅

## 示例输出

### 输入
```rust
bundler.bundle_to_css(
    "btn",
    "p-4 hover:p-8 md:p-12 hover:bg-blue-500 md:hover:p-16",
    "  "
)
```

### 输出
```css
.btn {
  padding: 1rem;
}

.btn:hover {
  padding: 2rem;
  background-color: rgb(59, 130, 246);
}

@media (min-width: 768px) {
  .btn {
    padding: 3rem;
  }

  .btn:hover {
    padding: 4rem;
  }
}
```

## 未来改进

1. **SWC 集成**: 将 ClassContext 与 SWC CSS 生成集成
2. **更多修饰符**: 支持更复杂的修饰符组合
3. **性能优化**: 进一步优化分组和合并算法
4. **增量更新**: 支持增量式的类添加和移除

## 总结

ClassContext 架构通过将 ParsedClass 视为"写操作"，实现了更简洁、更高效的 CSS 生成流程。它完美体现了关注点分离的原则，使得代码更易理解、维护和扩展。
