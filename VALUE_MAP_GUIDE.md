# 值映射系统使用指南

## 概述

值映射系统（value_map）允许 Bundler 在不依赖 official-mappings.json 的情况下，自动推断 Tailwind 类到 CSS 的转换。

## 工作原理

### 转换优先级

当转换一个 Tailwind 类时，系统按以下优先级处理：

1. **官方映射查找** - 从 official-mappings.json 查找（如果已加载）
2. **任意值** - 处理 `[...]` 语法，如 `w-[13px]`
3. **值映射推断** - 使用预定义的值映射，如 `p-4` → `1rem`
4. **插件默认值** - 某些插件的默认行为

### 架构

```
Tailwind 类: p-4
    ↓
解析器: { plugin: "p", value: "4" }
    ↓
插件映射: "p" → "padding"
    ↓
值映射: "4" → "1rem"
    ↓
CSS: padding: 1rem
```

## 支持的值映射

### 1. 间距值 (Spacing)

基于 Tailwind 默认配置的间距比例：

| Tailwind 值 | CSS 值 | 示例 |
|------------|--------|------|
| 0 | 0 | p-0 → padding: 0 |
| px | 1px | p-px → padding: 1px |
| 0.5 | 0.125rem | p-0.5 → padding: 0.125rem |
| 1 | 0.25rem | p-1 → padding: 0.25rem |
| 2 | 0.5rem | p-2 → padding: 0.5rem |
| 4 | 1rem | p-4 → padding: 1rem |
| 8 | 2rem | p-8 → padding: 2rem |
| ... | ... | ... |
| 96 | 24rem | p-96 → padding: 24rem |
| auto | auto | m-auto → margin: auto |

**适用插件**: p, px, py, pt, pr, pb, pl, m, mx, my, mt, mr, mb, ml, gap, space-x, space-y

### 2. 分数值 (Fractions)

用于宽度和高度：

| Tailwind 值 | CSS 值 | 示例 |
|------------|--------|------|
| full | 100% | w-full → width: 100% |
| 1/2 | 50% | w-1/2 → width: 50% |
| 1/3 | 33.333333% | w-1/3 → width: 33.333333% |
| 2/3 | 66.666667% | w-2/3 → width: 66.666667% |
| 1/4 | 25% | w-1/4 → width: 25% |
| 3/4 | 75% | w-3/4 → width: 75% |
| ... | ... | ... |

**适用插件**: w, h, min-w, max-w, min-h, max-h

### 3. 特殊尺寸值

| Tailwind 值 | CSS 值 | 适用于 | 示例 |
|------------|--------|--------|------|
| screen | 100vw | width | w-screen → width: 100vw |
| screen | 100vh | height | h-screen → height: 100vh |
| min | min-content | 尺寸 | w-min → width: min-content |
| max | max-content | 尺寸 | w-max → width: max-content |
| fit | fit-content | 尺寸 | w-fit → width: fit-content |

### 4. 颜色值

支持的颜色系列：

**基础颜色**:
- black → #000
- white → #fff
- transparent → transparent
- current → currentColor

**颜色阶梯** (50-900):
- gray-* (9 个阶梯)
- blue-* (9 个阶梯)
- red-* (9 个阶梯)
- green-* (9 个阶梯)

示例：
```
bg-blue-500 → background: #3b82f6
text-white → color: #fff
```

**适用插件**: bg, text, border (当值为颜色时)

### 5. 不透明度值

| Tailwind 值 | CSS 值 |
|------------|--------|
| 0 | 0 |
| 5 | 0.05 |
| 10 | 0.1 |
| 25 | 0.25 |
| 50 | 0.5 |
| 75 | 0.75 |
| 100 | 1 |

示例：
```
opacity-50 → opacity: 0.5
bg-opacity-75 → background-opacity: 0.75
```

**适用插件**: opacity, bg-opacity, text-opacity, border-opacity

### 6. 圆角值

| Tailwind 值 | CSS 值 |
|------------|--------|
| none | 0 |
| sm | 0.125rem |
| (默认) | 0.25rem |
| md | 0.375rem |
| lg | 0.5rem |
| xl | 0.75rem |
| 2xl | 1rem |
| 3xl | 1.5rem |
| full | 9999px |

示例：
```
rounded → border-radius: 0.25rem
rounded-lg → border-radius: 0.5rem
rounded-full → border-radius: 9999px
```

## 使用示例

### 不使用官方映射

```rust
use headwind_tw_index::{Bundler, Converter, TailwindIndex};

// 创建空索引（不加载官方映射）
let index = TailwindIndex::new();
let converter = Converter::new(&index);
let bundler = Bundler::new(converter);

// 打包类名
let classes = "p-4 bg-blue-500 hover:bg-blue-600 md:p-8";
let group = bundler.bundle(classes)?;
let css = bundler.generate_css("my-class", &group, "  ");
```

输出：
```css
.my-class {
  padding: 1rem;
  background: #3b82f6;
}

.my-class:hover {
  background: #2563eb;
}

@media (min-width: 768px) {
  .my-class {
    padding: 2rem;
  }
}
```

### 混合使用（推荐）

```rust
// 加载官方映射作为补充
let json = include_str!("../fixtures/official-mappings.json");
let index = load_from_official_json(json)?;
let converter = Converter::new(&index);
let bundler = Bundler::new(converter);
```

这样可以：
- ✅ 使用官方映射处理特殊类（如 `absolute`, `flex`）
- ✅ 使用值映射处理标准值（如 `p-4`, `bg-blue-500`）
- ✅ 使用任意值处理自定义值（如 `w-[13px]`）

## 支持的插件

### 间距类
- p, px, py, pt, pr, pb, pl
- m, mx, my, mt, mr, mb, ml
- gap, gap-x, gap-y
- space-x, space-y

### 尺寸类
- w, h
- min-w, min-h
- max-w, max-h

### 定位类
- top, right, bottom, left
- inset, inset-x, inset-y

### 颜色类
- bg (背景色)
- text (文本色)
- border (边框色)

### 其他类
- opacity
- rounded

## 当前限制

### 1. 分数值解析

⚠️ 当前解析器会将 `/` 视为 alpha 分隔符，因此 `w-3/4` 会被错误解析。

**解决方案**: 使用任意值语法
```
w-[75%] 而不是 w-3/4
```

### 2. 边框颜色

⚠️ `border-gray-300` 会被映射到 `border-width` 而不是 `border-color`。

**解决方案**:
- 使用官方映射（推荐）
- 或使用任意值: `border-[#d1d5db]`

### 3. 有限的颜色支持

当前只支持 4 个颜色系列（gray, blue, red, green）。

**解决方案**:
- 扩展 value_map.rs 添加更多颜色
- 使用任意值: `bg-[#ff6b6b]`
- 使用官方映射

### 4. 某些特殊类

某些特殊类（如 `shadow`, `transition`, `transform` 等）可能没有值映射。

**解决方案**:
- 使用官方映射（推荐）
- 或使用任意值

## 扩展值映射

### 添加新颜色

编辑 `value_map.rs`:

```rust
// 在 get_color_value 中添加
map.insert("purple-500", "#a855f7");
map.insert("pink-500", "#ec4899");
// ...
```

### 添加新值映射

```rust
// 在 infer_value 中添加新插件
"shadow" => match value {
    "sm" => Some("0 1px 2px 0 rgba(0, 0, 0, 0.05)".to_string()),
    "md" => Some("0 4px 6px -1px rgba(0, 0, 0, 0.1)".to_string()),
    _ => None,
},
```

## 性能考虑

- ✅ 所有映射使用 `OnceLock` 懒加载
- ✅ HashMap 查找 O(1) 时间复杂度
- ✅ 无运行时依赖

## 最佳实践

1. **混合使用**: 结合官方映射和值映射，获得最佳覆盖
2. **任意值**: 对于复杂或自定义值，使用 `[...]` 语法
3. **扩展映射**: 根据项目需求扩展值映射
4. **类型安全**: 利用 Rust 的类型系统避免错误

## 测试

运行示例：
```bash
# 不使用官方映射
cargo run -p headwind-tw-index --example bundle_without_index

# 使用官方映射
cargo run -p headwind-tw-index --example bundle_classes
```

运行测试：
```bash
cargo test -p headwind-tw-index value_map
```

## 总结

值映射系统提供了一个灵活的方式来处理 Tailwind 类的转换，无需依赖大型的映射文件。它特别适合：

- ✅ 需要轻量级转换器的场景
- ✅ 自定义 Tailwind 配置
- ✅ 与官方映射结合使用
- ✅ 快速原型开发

对于生产环境，建议混合使用官方映射和值映射，以获得最佳的类覆盖率。
