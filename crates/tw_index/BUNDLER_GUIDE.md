# Tailwind CSS 类打包器使用指南

## 概述

Bundler 模块可以将一串 Tailwind CSS 类名整理成一个 CSS 类，并自动按修饰符分组生成相应的选择器。

## 功能特点

✅ **自动分组**: 按修饰符类型自动分组（基础、伪类、响应式等）
✅ **智能合并**: 相同修饰符的声明自动合并
✅ **完整支持**: 支持所有 Tailwind 修饰符
✅ **格式美观**: 生成格式化的 CSS 代码

## 快速开始

### 基础用法

```rust
use headwind_tw_index::{load_from_official_json, Bundler, Converter};

// 1. 加载索引
let json = include_str!("../fixtures/official-mappings.json");
let index = load_from_official_json(json)?;

// 2. 创建打包器
let converter = Converter::new(&index);
let bundler = Bundler::new(converter);

// 3. 打包类名
let classes = "text-center hover:text-left md:text-right p-4";
let group = bundler.bundle(classes)?;

// 4. 生成 CSS
let css = bundler.generate_css("my-class", &group, "  ");
println!("{}", css);
```

输出：
```css
.my-class {
  text-align: center;
}

.my-class:hover {
  text-align: left;
}

@media (min-width: 768px) {
  .my-class {
    text-align: right;
  }
}
```

## 使用示例

### 1. 基础类（无修饰符）

**输入:**
```rust
let classes = "text-center p-4";
```

**输出:**
```css
.my-class {
  text-align: center;
  padding: 1rem;
}
```

### 2. 伪类修饰符

**输入:**
```rust
let classes = "text-center hover:text-left focus:text-right";
```

**输出:**
```css
.my-class {
  text-align: center;
}

.my-class:hover {
  text-align: left;
}

.my-class:focus {
  text-align: right;
}
```

### 3. 响应式修饰符

**输入:**
```rust
let classes = "text-center md:text-right lg:text-left";
```

**输出:**
```css
.my-class {
  text-align: center;
}

@media (min-width: 768px) {
  .my-class {
    text-align: right;
  }
}

@media (min-width: 1024px) {
  .my-class {
    text-align: left;
  }
}
```

### 4. 复杂组合

**输入:**
```rust
let classes = "text-center hover:text-left md:text-right md:hover:text-left p-4 md:p-8";
```

**输出:**
```css
.my-class {
  text-align: center;
  padding: 1rem;
}

.my-class:hover {
  text-align: left;
}

@media (min-width: 768px) {
  .my-class {
    text-align: right;
    padding: 2rem;
  }

  .my-class:hover {
    text-align: left;
  }
}
```

### 5. 暗色模式

**输入:**
```rust
let classes = "text-black dark:text-white";
```

**输出:**
```css
.my-class {
  color: black;
}

.dark .my-class {
  color: white;
}
```

### 6. 组状态

**输入:**
```rust
let classes = "text-center group-hover:text-left";
```

**输出:**
```css
.my-class {
  text-align: center;
}

.group:hover .my-class {
  text-align: left;
}
```

### 7. 伪元素

**输入:**
```rust
let classes = "before:content-none after:content-none";
```

**输出:**
```css
.my-class::before {
  content: none;
}

.my-class::after {
  content: none;
}
```

## 实际应用场景

### 按钮组件

**Tailwind 类:**
```
text-center px-4 py-2 rounded
hover:opacity-80
active:opacity-60
disabled:opacity-50
md:px-6 md:py-3
```

**生成的 CSS:**
```css
.button {
  text-align: center;
  padding-left: 1rem;
  padding-right: 1rem;
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
  border-radius: 0.25rem;
}

.button:hover {
  opacity: 0.8;
}

.button:active {
  opacity: 0.6;
}

.button:disabled {
  opacity: 0.5;
}

@media (min-width: 768px) {
  .button {
    padding-left: 1.5rem;
    padding-right: 1.5rem;
    padding-top: 0.75rem;
    padding-bottom: 0.75rem;
  }
}
```

### 导航链接

**Tailwind 类:**
```
text-gray-700
hover:text-blue-500
dark:text-gray-300
dark:hover:text-blue-400
```

**生成的 CSS:**
```css
.nav-link {
  color: rgb(55 65 81);
}

.nav-link:hover {
  color: rgb(59 130 246);
}

.dark .nav-link {
  color: rgb(209 213 219);
}

.dark .nav-link:hover {
  color: rgb(96 165 250);
}
```

### 响应式容器

**Tailwind 类:**
```
w-full
md:w-3/4
lg:w-1/2
mx-auto
p-4
md:p-8
```

**生成的 CSS:**
```css
.container {
  width: 100%;
  margin-left: auto;
  margin-right: auto;
  padding: 1rem;
}

@media (min-width: 768px) {
  .container {
    width: 75%;
    padding: 2rem;
  }
}

@media (min-width: 1024px) {
  .container {
    width: 50%;
  }
}
```

## API 参考

### `Bundler`

#### `new(converter: Converter) -> Self`

创建新的打包器实例。

#### `bundle(&self, classes: &str) -> Result<RuleGroup, String>`

将空格分隔的 Tailwind 类打包成规则组。

**参数:**
- `classes`: 空格分隔的类名字符串

**返回:**
- `Ok(RuleGroup)`: 打包后的规则组
- `Err(String)`: 错误信息

#### `generate_css(&self, class_name: &str, group: &RuleGroup, indent: &str) -> String`

将规则组生成为 CSS 字符串。

**参数:**
- `class_name`: 生成的 CSS 类名
- `group`: 规则组
- `indent`: 缩进字符串（通常为 `"  "` 或 `"\t"`）

**返回:**
- 格式化的 CSS 字符串

### `RuleGroup`

规则组结构，包含按修饰符分组的 CSS 声明：

- `base: Vec<Declaration>` - 基础规则（无修饰符）
- `pseudo_classes: HashMap<String, Vec<Declaration>>` - 伪类规则
- `pseudo_elements: HashMap<String, Vec<Declaration>>` - 伪元素规则
- `responsive: HashMap<String, Box<RuleGroup>>` - 响应式规则
- `states: HashMap<String, Box<RuleGroup>>` - 状态规则

## 支持的修饰符

### 伪类
- `hover:` - :hover
- `focus:` - :focus
- `active:` - :active
- `disabled:` - :disabled
- `visited:` - :visited
- 等等...

### 伪元素
- `before:` - ::before
- `after:` - ::after
- `placeholder:` - ::placeholder
- `selection:` - ::selection

### 响应式
- `sm:` - @media (min-width: 640px)
- `md:` - @media (min-width: 768px)
- `lg:` - @media (min-width: 1024px)
- `xl:` - @media (min-width: 1280px)
- `2xl:` - @media (min-width: 1536px)

### 状态
- `dark:` - .dark 父类
- `group-hover:` - .group:hover 父类
- `peer-focus:` - .peer:focus 兄弟元素

## 运行示例

```bash
# 运行打包示例
cargo run -p headwind-tw-index --example bundle_classes

# 运行测试
cargo test -p headwind-tw-index bundler
```

## 性能考虑

- ✅ **高效分组**: O(n) 时间复杂度，其中 n 为类名数量
- ✅ **内存优化**: 使用引用避免不必要的复制
- ✅ **懒加载**: 只在需要时生成 CSS 字符串

## 限制和注意事项

1. **依赖官方映射**: 只能转换索引中存在的类
2. **修饰符顺序**: 生成的 CSS 可能与原始类名顺序不同（按类型分组）
3. **声明合并**: 同一修饰符下的相同属性会被覆盖（后者优先）

## 常见问题

### Q: 为什么某些类没有生成 CSS？

A: 因为这些类不在 official-mappings.json 中。你可以：
1. 使用任意值语法（如 `w-[13px]`）
2. 扩展 official-mappings.json
3. 添加插件映射到 plugin_map.rs

### Q: 如何自定义缩进？

A: 使用 `generate_css` 的第三个参数：

```rust
// 使用 tab 缩进
let css = bundler.generate_css("my-class", &group, "\t");

// 使用 4 个空格缩进
let css = bundler.generate_css("my-class", &group, "    ");
```

### Q: 支持嵌套的响应式修饰符吗？

A: 是的！例如：

```rust
"md:hover:text-center" // 在 md 断点下的 hover 状态
```

生成：
```css
@media (min-width: 768px) {
  .my-class:hover {
    text-align: center;
  }
}
```

## 下一步

- 查看 [examples/bundle_classes.rs](examples/bundle_classes.rs) 获取更多示例
- 阅读 [README.md](README.md) 了解整体架构
- 运行测试了解详细行为：`cargo test -p headwind-tw-index bundler -- --nocapture`
