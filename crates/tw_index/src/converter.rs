use headwind_core::ColorMode;
use crate::plugin_map::get_plugin_properties;
use crate::theme_values;
use crate::value_map::{get_color_value, get_spacing_value, infer_value};
use headwind_core::Declaration;
use headwind_tw_parse::{CssVariableValue, Modifier, ParsedClass, ParsedValue};
use phf::phf_map;

/// CSS 规则，包含选择器和声明
#[derive(Debug, Clone, PartialEq)]
pub struct CssRule {
    /// 选择器（如 ".my-class:hover" 或 "@media (min-width: 640px)"）
    pub selector: String,
    /// CSS 声明列表
    pub declarations: Vec<Declaration>,
}

/// 无值类的静态映射：class name → (css property, css value)
static VALUELESS_MAP: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    // Display
    "block" => ("display", "block"),
    "inline-block" => ("display", "inline-block"),
    "inline" => ("display", "inline"),
    "flex" => ("display", "flex"),
    "inline-flex" => ("display", "inline-flex"),
    "grid" => ("display", "grid"),
    "inline-grid" => ("display", "inline-grid"),
    "hidden" => ("display", "none"),
    "contents" => ("display", "contents"),
    "table" => ("display", "table"),

    // Position
    "static" => ("position", "static"),
    "fixed" => ("position", "fixed"),
    "absolute" => ("position", "absolute"),
    "relative" => ("position", "relative"),
    "sticky" => ("position", "sticky"),

    // Overflow
    "overflow-auto" => ("overflow", "auto"),
    "overflow-hidden" => ("overflow", "hidden"),
    "overflow-visible" => ("overflow", "visible"),
    "overflow-scroll" => ("overflow", "scroll"),
    "overflow-clip" => ("overflow", "clip"),

    // Flex direction
    "flex-row" => ("flex-direction", "row"),
    "flex-row-reverse" => ("flex-direction", "row-reverse"),
    "flex-col" => ("flex-direction", "column"),
    "flex-col-reverse" => ("flex-direction", "column-reverse"),

    // Flex wrap
    "flex-wrap" => ("flex-wrap", "wrap"),
    "flex-wrap-reverse" => ("flex-wrap", "wrap-reverse"),
    "flex-nowrap" => ("flex-wrap", "nowrap"),

    // Items alignment
    "items-start" => ("align-items", "flex-start"),
    "items-end" => ("align-items", "flex-end"),
    "items-center" => ("align-items", "center"),
    "items-baseline" => ("align-items", "baseline"),
    "items-stretch" => ("align-items", "stretch"),

    // Pointer events
    "pointer-events-none" => ("pointer-events", "none"),
    "pointer-events-auto" => ("pointer-events", "auto"),

    // Cursor (basic)
    "cursor-auto" => ("cursor", "auto"),
    "cursor-default" => ("cursor", "default"),
    "cursor-pointer" => ("cursor", "pointer"),
    "cursor-wait" => ("cursor", "wait"),
    "cursor-text" => ("cursor", "text"),
    "cursor-move" => ("cursor", "move"),
    "cursor-not-allowed" => ("cursor", "not-allowed"),

    // Visibility
    "visible" => ("visibility", "visible"),
    "invisible" => ("visibility", "hidden"),
    "collapse" => ("visibility", "collapse"),

    // Text transform
    "uppercase" => ("text-transform", "uppercase"),
    "lowercase" => ("text-transform", "lowercase"),
    "capitalize" => ("text-transform", "capitalize"),
    "normal-case" => ("text-transform", "none"),

    // Font style
    "italic" => ("font-style", "italic"),
    "not-italic" => ("font-style", "normal"),

    // Text decoration line
    "underline" => ("text-decoration-line", "underline"),
    "overline" => ("text-decoration-line", "overline"),
    "line-through" => ("text-decoration-line", "line-through"),
    "no-underline" => ("text-decoration-line", "none"),

    // Font variant numeric
    "ordinal" => ("font-variant-numeric", "ordinal"),
    "diagonal-fractions" => ("font-variant-numeric", "diagonal-fractions"),
    "stacked-fractions" => ("font-variant-numeric", "stacked-fractions"),
    "lining-nums" => ("font-variant-numeric", "lining-nums"),
    "tabular-nums" => ("font-variant-numeric", "tabular-nums"),
    "oldstyle-nums" => ("font-variant-numeric", "oldstyle-nums"),
    "proportional-nums" => ("font-variant-numeric", "proportional-nums"),
    "slashed-zero" => ("font-variant-numeric", "slashed-zero"),
    "normal-nums" => ("font-variant-numeric", "normal"),

    // Isolation
    "isolate" => ("isolation", "isolate"),
    "isolation-auto" => ("isolation", "auto"),

    // Flex grow/shrink
    "grow" => ("flex-grow", "1"),
    "shrink" => ("flex-shrink", "1"),

    // Filters (valueless)
    "grayscale" => ("filter", "grayscale(100%)"),
    "invert" => ("filter", "invert(100%)"),
    "sepia" => ("filter", "sepia(100%)"),

    // Backdrop filters (valueless)
    "backdrop-grayscale" => ("backdrop-filter", "grayscale(100%)"),
    "backdrop-invert" => ("backdrop-filter", "invert(100%)"),
    "backdrop-sepia" => ("backdrop-filter", "sepia(100%)"),

    // Border (valueless = 1px width)
    "border" => ("border-width", "1px"),

    // Outline (valueless = 1px width)
    "outline" => ("outline-width", "1px"),

    // Ring (valueless = 1px width)
    "ring" => ("--tw-ring-shadow", "0 0 0 1px"),
    "inset-ring" => ("--tw-inset-ring-shadow", "inset 0 0 0 1px"),

    // Resize (valueless = both)
    "resize" => ("resize", "both"),

    // Box sizing
    "box-border" => ("box-sizing", "border-box"),
    "box-content" => ("box-sizing", "content-box"),
};

/// 响应式断点映射
static BREAKPOINT_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "sm" => "640px",
    "md" => "768px",
    "lg" => "1024px",
    "xl" => "1280px",
    "2xl" => "1536px",
};

/// 基于规则的 Tailwind 类转换器
///
/// 基于 plugin_map 和 value_map 进行转换，不依赖外部索引
pub struct Converter {
    /// true = 使用 var(--text-3xl)，false = 内联为 1.875rem
    use_variables: bool,
    /// 颜色输出模式（hex / oklch / hsl / var）
    color_mode: ColorMode,
    /// 是否使用 color-mix() 函数处理颜色透明度
    use_color_mix: bool,
}

impl Converter {
    pub fn new() -> Self {
        Self {
            use_variables: true,
            color_mode: ColorMode::default(),
            use_color_mix: false,
        }
    }

    /// 创建使用内联值的转换器（不依赖 Tailwind 主题变量）
    pub fn with_inline() -> Self {
        Self {
            use_variables: false,
            color_mode: ColorMode::default(),
            use_color_mix: false,
        }
    }

    /// 设置颜色输出模式（builder 模式）
    pub fn with_color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// 设置是否使用 color-mix() 函数处理颜色透明度（builder 模式）
    pub fn with_color_mix(mut self, enabled: bool) -> Self {
        self.use_color_mix = enabled;
        self
    }

    /// 将 Tailwind 类转换为 CSS 声明（仅声明，不含选择器）
    ///
    /// 适用于上下文模式，由调用者决定如何组织选择器。
    /// 复合插件（如 justify-items、gap-x）由解析器负责识别，
    /// 此处仅处理声明构建和无值类回退。
    pub fn to_declarations(&self, parsed: &ParsedClass) -> Option<Vec<Declaration>> {
        let declarations = match &parsed.value {
            Some(ParsedValue::Arbitrary(arb)) => {
                build_arbitrary_declarations(parsed, &arb.content)?
            }
            Some(ParsedValue::CssVariable(cv)) => {
                build_css_variable_declarations(parsed, cv)?
            }
            Some(ParsedValue::Standard(value)) => self
                .build_standard_declarations(parsed, value)
                .or_else(|| build_valueless_from_full_name(parsed, value))?,
            None => build_valueless_declarations(parsed)?,
        };

        // 为颜色属性应用 alpha 透明度（如 text-white/60 → color: #fff9）
        let declarations = if let Some(ref alpha) = parsed.alpha {
            apply_alpha_to_declarations(declarations, alpha, self.use_color_mix)
        } else {
            declarations
        };

        Some(apply_important(declarations, parsed.important))
    }

    /// 将 Tailwind 类名转换为 CSS 规则（声明 + 选择器）
    pub fn convert(&self, parsed: &ParsedClass) -> Option<CssRule> {
        let declarations = self.to_declarations(parsed)?;
        let selector = build_selector(parsed);
        Some(CssRule { selector, declarations })
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 声明构建（纯函数，不依赖 Converter 状态）
// ---------------------------------------------------------------------------

/// 为任意值构建 CSS 声明
///
/// 例如：`w-[13px]` → `width: 13px`
fn build_arbitrary_declarations(parsed: &ParsedClass, raw_value: &str) -> Option<Vec<Declaration>> {
    // 不在 plugin_map 中的复杂插件，走专门的分发逻辑
    if let Some(decls) = build_complex_arbitrary(parsed, raw_value) {
        return Some(decls);
    }

    let properties = get_plugin_properties(&parsed.plugin)?;
    let declarations = properties
        .into_iter()
        .map(|property| {
            let value = if parsed.negative {
                format!("-{}", raw_value)
            } else {
                raw_value.to_string()
            };
            Declaration::new(property, value)
        })
        .collect();

    Some(declarations)
}

/// 为 CSS 自定义属性值构建声明
///
/// Tailwind v4 的 `-(...)` 语法：
/// - `bg-(--my-color)` → `background: var(--my-color)`
/// - `bg-(image:--my-bg)` → `background-image: var(--my-bg)`
/// - `bg-linear-(--custom)` → `background-image: linear-gradient(var(--tw-gradient-stops, var(--custom)))`
/// - `from-(--my-color)` → `--tw-gradient-from: var(--my-color)`
fn build_css_variable_declarations(
    parsed: &ParsedClass,
    cv: &CssVariableValue,
) -> Option<Vec<Declaration>> {
    let var_expr = format!("var({})", cv.property);

    // 有类型提示时，根据提示选择 CSS 属性
    if let Some(ref hint) = cv.type_hint {
        let property = match hint.as_str() {
            "image" => "background-image",
            "color" => "color",
            "font" => "font-family",
            "length" | "size" => "width", // 回退到 plugin_map
            _ => {
                // 未知类型提示，尝试直接作为属性名
                return Some(vec![Declaration::new(hint.as_str(), var_expr)]);
            }
        };
        // 对于有类型提示的情况，优先使用提示指定的属性
        // 但如果 plugin 自身有特定映射（如 bg → background），也参考 plugin
        let final_property = match (parsed.plugin.as_str(), hint.as_str()) {
            ("bg", "image") => "background-image",
            ("bg", "color") => "background-color",
            ("text", "color") => "color",
            ("text", "length") | ("text", "size") => "font-size",
            ("border", "length") | ("border", "size") => "border-width",
            _ => property,
        };
        return Some(vec![Declaration::new(final_property, var_expr)]);
    }

    // 无类型提示时，走专门的插件分发逻辑
    match parsed.plugin.as_str() {
        // 渐变系列
        "bg-linear" => Some(vec![Declaration::new(
            "background-image",
            format!("linear-gradient(var(--tw-gradient-stops, {}))", var_expr),
        )]),
        "bg-radial" => Some(vec![Declaration::new(
            "background-image",
            format!("radial-gradient(var(--tw-gradient-stops, {}))", var_expr),
        )]),
        // bg-conic CSS 变量直接作为 background-image（不包裹 conic-gradient）
        "bg-conic" => Some(vec![Declaration::new("background-image", var_expr)]),
        // 渐变色标
        "from" => Some(vec![Declaration::new("--tw-gradient-from", var_expr)]),
        "via" => Some(vec![Declaration::new("--tw-gradient-via", var_expr)]),
        "to" => Some(vec![Declaration::new("--tw-gradient-to", var_expr)]),
        // text 默认映射到 color
        "text" => Some(vec![Declaration::new("color", var_expr)]),
        // 颜色双语义插件：CSS 变量总是映射到颜色属性
        "border" => Some(vec![Declaration::new("border-color", var_expr)]),
        "outline" => Some(vec![Declaration::new("outline-color", var_expr)]),
        "decoration" => Some(vec![Declaration::new("text-decoration-color", var_expr)]),
        "stroke" => Some(vec![Declaration::new("stroke", var_expr)]),
        "shadow" => Some(vec![Declaration::new("--tw-shadow-color", var_expr)]),
        "inset-shadow" => Some(vec![Declaration::new("--tw-inset-shadow-color", var_expr)]),
        "ring" => Some(vec![Declaration::new("--tw-ring-shadow", format!("0 0 0 {}", var_expr))]),
        "inset-ring" => Some(vec![Declaration::new("--tw-inset-ring-shadow", format!("inset 0 0 0 {}", var_expr))]),
        // 通用：使用 plugin_map 查找 CSS 属性
        _ => {
            let properties = get_plugin_properties(&parsed.plugin)?;
            let declarations = properties
                .into_iter()
                .map(|property| Declaration::new(property, var_expr.clone()))
                .collect();
            Some(declarations)
        }
    }
}

// build_standard_declarations is now a method on Converter (see impl block below)

/// 为无值类构建声明
///
/// 例如：`flex` → `display: flex`
fn build_valueless_declarations(parsed: &ParsedClass) -> Option<Vec<Declaration>> {
    // Multi-declaration valueless classes
    match parsed.plugin.as_str() {
        "antialiased" => {
            return Some(vec![
                Declaration::new("-webkit-font-smoothing", "antialiased"),
                Declaration::new("-moz-osx-font-smoothing", "grayscale"),
            ])
        }
        _ => {}
    }

    let &(property, value) = VALUELESS_MAP.get(parsed.plugin.as_str())?;
    Some(vec![Declaration::new(property, value)])
}

/// 回退：将 plugin-value 作为完整类名查找 VALUELESS_MAP
///
/// 处理解析器无法区分"带值插件"和"多段无值类"的情况。
/// 例如：plugin=`overflow`, value=`auto` → 查找 `overflow-auto` → `overflow: auto`
///       plugin=`flex`, value=`row` → 查找 `flex-row` → `flex-direction: row`
fn build_valueless_from_full_name(parsed: &ParsedClass, value: &str) -> Option<Vec<Declaration>> {
    let full_name = format!("{}-{}", parsed.plugin, value);

    // Multi-declaration valueless classes
    match full_name.as_str() {
        "subpixel-antialiased" => {
            return Some(vec![
                Declaration::new("-webkit-font-smoothing", "auto"),
                Declaration::new("-moz-osx-font-smoothing", "auto"),
            ])
        }
        _ => {}
    }

    let &(property, css_value) = VALUELESS_MAP.get(full_name.as_str())?;
    Some(vec![Declaration::new(property, css_value)])
}

// ---------------------------------------------------------------------------
// 复杂插件分发（语义重载的插件，不适合放进静态 map）
// ---------------------------------------------------------------------------

/// 从字符串中提取方括号内的值
///
/// 例如：`"[45deg]"` → `Some("45deg")`，`"123"` → `None`
fn extract_bracket_value(s: &str) -> Option<&str> {
    s.strip_prefix('[').and_then(|s| s.strip_suffix(']'))
}

/// 判断任意值是否看起来像颜色值
///
/// 用于双语义插件（如 border）区分颜色和非颜色的任意值
fn looks_like_color_value(value: &str) -> bool {
    value.starts_with('#')
        || value.starts_with("rgb")
        || value.starts_with("hsl")
        || value.starts_with("oklch")
        || value.starts_with("oklab")
        || value.starts_with("color(")
}

/// 处理复杂任意值插件
fn build_complex_arbitrary(parsed: &ParsedClass, raw_value: &str) -> Option<Vec<Declaration>> {
    match parsed.plugin.as_str() {
        // text-[#fff] → color, text-[14px] → font-size
        "text" => {
            let value = if parsed.negative {
                format!("-{}", raw_value)
            } else {
                raw_value.to_string()
            };
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("color", value)])
            } else {
                Some(vec![Declaration::new("font-size", value)])
            }
        }
        // bg-linear-[<value>] → linear-gradient
        "bg-linear" => Some(vec![Declaration::new(
            "background-image",
            format!("linear-gradient(var(--tw-gradient-stops, {}))", raw_value),
        )]),
        // bg-conic-[<value>] → 直接作为 background-image（不包裹 conic-gradient）
        "bg-conic" => Some(vec![Declaration::new(
            "background-image",
            raw_value.to_string(),
        )]),
        // bg-radial-[<value>] → radial-gradient
        "bg-radial" => Some(vec![Declaration::new(
            "background-image",
            format!("radial-gradient(var(--tw-gradient-stops, {}))", raw_value),
        )]),
        // from-[<value>] → --tw-gradient-from
        "from" => Some(vec![Declaration::new(
            "--tw-gradient-from",
            raw_value.to_string(),
        )]),
        // via-[<value>] → --tw-gradient-via
        "via" => Some(vec![Declaration::new(
            "--tw-gradient-via",
            raw_value.to_string(),
        )]),
        // to-[<value>] → --tw-gradient-to
        "to" => Some(vec![Declaration::new(
            "--tw-gradient-to",
            raw_value.to_string(),
        )]),
        // border-[<color>] → border-color（仅颜色值，非颜色回退到 plugin_map 的 border-width）
        "border" => {
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("border-color", raw_value)])
            } else {
                None
            }
        }
        // outline-[<value>] → outline-color / outline-width
        "outline" => {
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("outline-color", raw_value)])
            } else {
                Some(vec![Declaration::new("outline-width", raw_value)])
            }
        }
        // decoration-[<value>] → text-decoration-color / text-decoration-thickness
        "decoration" => {
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("text-decoration-color", raw_value)])
            } else {
                Some(vec![Declaration::new("text-decoration-thickness", raw_value)])
            }
        }
        // stroke-[<value>] → stroke / stroke-width
        "stroke" => {
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("stroke", raw_value)])
            } else {
                Some(vec![Declaration::new("stroke-width", raw_value)])
            }
        }
        // shadow-[<color>] → --tw-shadow-color
        "shadow" => {
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("--tw-shadow-color", raw_value)])
            } else {
                None // fall through to plugin_map (box-shadow)
            }
        }
        // inset-shadow-[<color>] → --tw-inset-shadow-color, else box-shadow
        "inset-shadow" => {
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("--tw-inset-shadow-color", raw_value)])
            } else {
                Some(vec![Declaration::new("box-shadow", raw_value.to_string())])
            }
        }
        // ring-[<color>] → --tw-ring-color, ring-[<width>] → --tw-ring-shadow
        "ring" => {
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("--tw-ring-color", raw_value)])
            } else {
                Some(vec![Declaration::new("--tw-ring-shadow", format!("0 0 0 {}", raw_value))])
            }
        }
        // inset-ring-[<color>] → --tw-inset-ring-color, inset-ring-[<width>] → --tw-inset-ring-shadow
        "inset-ring" => {
            if looks_like_color_value(raw_value) {
                Some(vec![Declaration::new("--tw-inset-ring-color", raw_value)])
            } else {
                Some(vec![Declaration::new("--tw-inset-ring-shadow", format!("inset 0 0 0 {}", raw_value))])
            }
        }
        _ => None,
    }
}

// build_complex_standard is now a method on Converter (see impl block below)

// ---------------------------------------------------------------------------
// Converter 方法扩展（需要访问 use_variables 的逻辑）
// ---------------------------------------------------------------------------

impl Converter {
    /// 为标准值构建 CSS 声明
    fn build_standard_declarations(&self, parsed: &ParsedClass, value: &str) -> Option<Vec<Declaration>> {
        if let Some(decls) = self.build_complex_standard(parsed, value) {
            return Some(decls);
        }

        let properties = get_plugin_properties(&parsed.plugin)?;
        let mut css_value = infer_value(&parsed.plugin, value, self.color_mode)?;

        if parsed.negative {
            css_value = format!("-{}", css_value);
        }

        let declarations = properties
            .into_iter()
            .map(|property| Declaration::new(property, css_value.clone()))
            .collect();

        Some(declarations)
    }

    /// 处理复杂标准值插件（语义重载，不同值映射到不同 CSS 属性）
    fn build_complex_standard(&self, parsed: &ParsedClass, value: &str) -> Option<Vec<Declaration>> {
        match parsed.plugin.as_str() {
            // ── text: text-align / text-wrap / font-size / color ─────
            "text" => match value {
                "left" | "center" | "right" | "justify" | "start" | "end" => {
                    Some(vec![Declaration::new("text-align", value.to_string())])
                }
                "nowrap" | "wrap" | "balance" | "pretty" => {
                    Some(vec![Declaration::new("text-wrap", value.to_string())])
                }
                "xs" | "sm" | "base" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "5xl" | "6xl"
                | "7xl" | "8xl" | "9xl" => {
                    let font_size = if self.use_variables {
                        format!("var(--text-{})", value)
                    } else {
                        theme_values::TEXT_SIZE.get(value)?.to_string()
                    };

                    // alpha 修饰符覆盖行高：text-base/6, text-base/[1.5rem], text-base/(--lh)
                    let line_height = if let Some(ref alpha) = parsed.alpha {
                        if alpha.starts_with('[') && alpha.ends_with(']') {
                            // 任意值：text-base/[1.5rem] → line-height: 1.5rem
                            let inner = &alpha[1..alpha.len() - 1];
                            inner.replace('_', " ")
                        } else if alpha.starts_with('(') && alpha.ends_with(')') {
                            // CSS 变量：text-base/(--lh) → line-height: var(--lh)
                            let inner = alpha.strip_prefix('(').and_then(|s| s.strip_suffix(')')).unwrap_or(alpha);
                            format!("var({})", inner)
                        } else if alpha.chars().all(|c| c.is_ascii_digit()) {
                            // 数字：text-base/6 → line-height: calc(var(--spacing) * 6)
                            format!("calc(var(--spacing) * {})", alpha)
                        } else {
                            alpha.clone()
                        }
                    } else if self.use_variables {
                        format!("var(--text-{}--line-height)", value)
                    } else {
                        theme_values::TEXT_LINE_HEIGHT.get(value)?.to_string()
                    };

                    Some(vec![
                        Declaration::new("font-size", font_size),
                        Declaration::new("line-height", line_height),
                    ])
                }
            _ => {
                let css_value = infer_value(&parsed.plugin, value, self.color_mode)?;
                Some(vec![Declaration::new("color", css_value)])
            }
        },

        // ── bg: size / position / clip / origin / blend / repeat / gradient / attachment ──
        "bg" => match value {
            // Background size
            "auto" | "contain" | "cover" => {
                Some(vec![Declaration::new("background-size", value)])
            }
            // Background attachment
            "fixed" | "local" | "scroll" => {
                Some(vec![Declaration::new("background-attachment", value)])
            }
            // Background repeat
            "repeat" | "no-repeat" | "repeat-x" | "repeat-y" => {
                Some(vec![Declaration::new("background-repeat", value)])
            }
            "repeat-round" => Some(vec![Declaration::new("background-repeat", "round")]),
            "repeat-space" => Some(vec![Declaration::new("background-repeat", "space")]),
            // Background position
            "top" | "bottom" | "left" | "right" | "center" => {
                Some(vec![Declaration::new("background-position", value)])
            }
            "top-left" => Some(vec![Declaration::new("background-position", "top left")]),
            "top-right" => Some(vec![Declaration::new("background-position", "top right")]),
            "bottom-left" => Some(vec![Declaration::new("background-position", "bottom left")]),
            "bottom-right" => {
                Some(vec![Declaration::new("background-position", "bottom right")])
            }
            // Background clip
            "clip-border" => Some(vec![Declaration::new("background-clip", "border-box")]),
            "clip-content" => Some(vec![Declaration::new("background-clip", "content-box")]),
            "clip-padding" => Some(vec![Declaration::new("background-clip", "padding-box")]),
            "clip-text" => Some(vec![Declaration::new("background-clip", "text")]),
            // Background origin
            "origin-border" => Some(vec![Declaration::new("background-origin", "border-box")]),
            "origin-content" => {
                Some(vec![Declaration::new("background-origin", "content-box")])
            }
            "origin-padding" => {
                Some(vec![Declaration::new("background-origin", "padding-box")])
            }
            // Gradient
            "none" => Some(vec![Declaration::new("background-image", "none")]),
            "radial" => Some(vec![Declaration::new(
                "background-image",
                "radial-gradient(in oklab, var(--tw-gradient-stops))",
            )]),
            "conic" => Some(vec![Declaration::new(
                "background-image",
                "conic-gradient(in oklab, var(--tw-gradient-stops))",
            )]),
            _ => {
                // blend-* → background-blend-mode
                if let Some(mode) = value.strip_prefix("blend-") {
                    return Some(vec![Declaration::new(
                        "background-blend-mode",
                        mode.to_string(),
                    )]);
                }
                // linear-to-* → background-image: linear-gradient(...)
                if let Some(dir) = value.strip_prefix("linear-to-") {
                    let direction = match dir {
                        "t" => "to top",
                        "b" => "to bottom",
                        "l" => "to left",
                        "r" => "to right",
                        "tl" => "to top left",
                        "tr" => "to top right",
                        "bl" => "to bottom left",
                        "br" => "to bottom right",
                        _ => return None,
                    };
                    return Some(vec![Declaration::new(
                        "background-image",
                        format!("linear-gradient({}, var(--tw-gradient-stops))", direction),
                    )]);
                }
                // linear-<angle> 或 linear-[<value>]
                if let Some(rest) = value.strip_prefix("linear-") {
                    if let Some(arb) = extract_bracket_value(rest) {
                        return Some(vec![Declaration::new(
                            "background-image",
                            format!("linear-gradient(var(--tw-gradient-stops, {}))", arb),
                        )]);
                    }
                    if let Ok(n) = rest.parse::<f64>() {
                        let deg = if parsed.negative {
                            format!("-{}deg", n)
                        } else {
                            format!("{}deg", n)
                        };
                        return Some(vec![Declaration::new(
                            "background-image",
                            format!(
                                "linear-gradient({} in oklab, var(--tw-gradient-stops))",
                                deg
                            ),
                        )]);
                    }
                }
                // conic-<angle> 或 conic-[<value>]
                if let Some(rest) = value.strip_prefix("conic-") {
                    // conic-[<value>] → 直接作为 background-image
                    if let Some(arb) = extract_bracket_value(rest) {
                        return Some(vec![Declaration::new(
                            "background-image",
                            arb.to_string(),
                        )]);
                    }
                    if let Ok(n) = rest.parse::<f64>() {
                        let deg = if parsed.negative {
                            format!("-{}deg", n)
                        } else {
                            format!("{}deg", n)
                        };
                        return Some(vec![Declaration::new(
                            "background-image",
                            format!(
                                "conic-gradient(from {} in oklab, var(--tw-gradient-stops))",
                                deg
                            ),
                        )]);
                    }
                }
                // radial-[<value>]
                if let Some(rest) = value.strip_prefix("radial-") {
                    if let Some(arb) = extract_bracket_value(rest) {
                        return Some(vec![Declaration::new(
                            "background-image",
                            format!("radial-gradient(var(--tw-gradient-stops, {}))", arb),
                        )]);
                    }
                }
                None // fall through to standard path for colors
            }
        },

        // ── font: weight / family / stretch ──────────────────────
        "font" => match value {
            "sans" | "serif" | "mono" => {
                if self.use_variables {
                    Some(vec![Declaration::new(
                        "font-family",
                        format!("var(--font-{})", value),
                    )])
                } else {
                    let family = theme_values::FONT_FAMILY.get(value)?;
                    Some(vec![Declaration::new("font-family", family.to_string())])
                }
            }
            "thin" => Some(vec![Declaration::new("font-weight", "100")]),
            "extralight" => Some(vec![Declaration::new("font-weight", "200")]),
            "light" => Some(vec![Declaration::new("font-weight", "300")]),
            "normal" => Some(vec![Declaration::new("font-weight", "400")]),
            "medium" => Some(vec![Declaration::new("font-weight", "500")]),
            "semibold" => Some(vec![Declaration::new("font-weight", "600")]),
            "bold" => Some(vec![Declaration::new("font-weight", "700")]),
            "extrabold" => Some(vec![Declaration::new("font-weight", "800")]),
            "black" => Some(vec![Declaration::new("font-weight", "900")]),
            _ => {
                if let Some(stretch) = value.strip_prefix("stretch-") {
                    return Some(vec![Declaration::new(
                        "font-stretch",
                        stretch.to_string(),
                    )]);
                }
                None
            }
        },

        // ── content: align-content vs content property ───────────
        "content" => match value {
            "none" => Some(vec![Declaration::new("content", "none")]),
            "start" => Some(vec![Declaration::new("align-content", "flex-start")]),
            "end" => Some(vec![Declaration::new("align-content", "flex-end")]),
            "around" => Some(vec![Declaration::new("align-content", "space-around")]),
            "between" => Some(vec![Declaration::new("align-content", "space-between")]),
            "evenly" => Some(vec![Declaration::new("align-content", "space-evenly")]),
            _ => Some(vec![Declaration::new("align-content", value.to_string())]),
        },

        // ── border: style / collapse / color ─────────────────────
        "border" => match value {
            "solid" | "dashed" | "dotted" | "double" | "hidden" | "none" => {
                Some(vec![Declaration::new("border-style", value)])
            }
            "collapse" | "separate" => {
                Some(vec![Declaration::new("border-collapse", value)])
            }
            _ => {
                if let Some(color) = get_color_value(value, self.color_mode) {
                    Some(vec![Declaration::new("border-color", color)])
                } else if let Ok(n) = value.parse::<f64>() {
                    // border-<number> → border-width: <number>px
                    let px = if parsed.negative { -n } else { n };
                    Some(vec![Declaration::new("border-width", format!("{}px", px))])
                } else {
                    None // fall through for width
                }
            }
        },

        // ── decoration: style / thickness / color ────────────────
        "decoration" => match value {
            "solid" | "dashed" | "dotted" | "double" | "wavy" => {
                Some(vec![Declaration::new("text-decoration-style", value)])
            }
            "auto" | "from-font" => {
                Some(vec![Declaration::new("text-decoration-thickness", value)])
            }
            _ => {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("text-decoration-color", color)])
            }
        },

        // ── outline: style / hidden / color / width ──────────────
        "outline" => match value {
            "solid" | "dashed" | "dotted" | "double" | "none" => {
                Some(vec![Declaration::new("outline-style", value)])
            }
            "hidden" => Some(vec![
                Declaration::new("outline", "2px solid transparent"),
                Declaration::new("outline-offset", "2px"),
            ]),
            _ => {
                if let Some(color) = get_color_value(value, self.color_mode) {
                    Some(vec![Declaration::new("outline-color", color)])
                } else if let Ok(n) = value.parse::<u32>() {
                    Some(vec![Declaration::new("outline-width", format!("{}px", n))])
                } else {
                    None
                }
            }
        },

        // ── stroke: color / width ────────────────────────────────
        "stroke" => {
            if let Some(color) = get_color_value(value, self.color_mode) {
                Some(vec![Declaration::new("stroke", color)])
            } else if let Ok(n) = value.parse::<u32>() {
                Some(vec![Declaration::new("stroke-width", n.to_string())])
            } else {
                None
            }
        }

        // ── shadow: named size / none / color ─────────────────────
        "shadow" => match value {
            "2xs" | "xs" | "sm" | "md" | "lg" | "xl" | "2xl" => {
                Some(vec![Declaration::new("box-shadow", format!("var(--shadow-{})", value))])
            }
            "none" => Some(vec![Declaration::new("box-shadow", "0 0 #0000")]),
            _ => {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("--tw-shadow-color", color)])
            }
        },

        // ── inset-shadow: named size / none / color ──────────────
        "inset-shadow" => match value {
            "2xs" | "xs" | "sm" => {
                Some(vec![Declaration::new("box-shadow", format!("var(--inset-shadow-{})", value))])
            }
            "none" => Some(vec![Declaration::new("box-shadow", "inset 0 0 #0000")]),
            _ => {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("--tw-inset-shadow-color", color)])
            }
        },

        // ── ring: number width / color ───────────────────────────
        "ring" => {
            if let Ok(n) = value.parse::<u32>() {
                Some(vec![Declaration::new("--tw-ring-shadow", format!("0 0 0 {}px", n))])
            } else {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("--tw-ring-color", color)])
            }
        }

        // ── inset-ring: number width / color ─────────────────────
        "inset-ring" => {
            if let Ok(n) = value.parse::<u32>() {
                Some(vec![Declaration::new("--tw-inset-ring-shadow", format!("inset 0 0 0 {}px", n))])
            } else {
                get_color_value(value, self.color_mode)
                    .map(|color| vec![Declaration::new("--tw-inset-ring-color", color)])
            }
        }

        // ── list: type / position / image ────────────────────────
        "list" => match value {
            "disc" | "decimal" | "none" => {
                Some(vec![Declaration::new("list-style-type", value)])
            }
            "inside" | "outside" => {
                Some(vec![Declaration::new("list-style-position", value)])
            }
            "image-none" => Some(vec![Declaration::new("list-style-image", "none")]),
            _ => None,
        },

        // ── object: fit vs position ──────────────────────────────
        "object" => match value {
            "contain" | "cover" | "fill" | "none" | "scale-down" => {
                Some(vec![Declaration::new("object-fit", value)])
            }
            _ => Some(vec![Declaration::new(
                "object-position",
                value.replace('-', " "),
            )]),
        },

        // ── mix: blend mode ──────────────────────────────────────
        "mix" => {
            if let Some(mode) = value.strip_prefix("blend-") {
                Some(vec![Declaration::new(
                    "mix-blend-mode",
                    mode.to_string(),
                )])
            } else {
                None
            }
        }

        // ── perspective: perspective vs perspective-origin ────────
        "perspective" => {
            if let Some(pos) = value.strip_prefix("origin-") {
                Some(vec![Declaration::new(
                    "perspective-origin",
                    pos.replace('-', " "),
                )])
            } else {
                None // fall through to standard path (infer_value handles named values)
            }
        }

        // ── snap: type / align / stop / strictness ───────────────
        "snap" => match value {
            "none" => Some(vec![Declaration::new("scroll-snap-type", "none")]),
            "x" => Some(vec![Declaration::new(
                "scroll-snap-type",
                "x var(--tw-scroll-snap-strictness)",
            )]),
            "y" => Some(vec![Declaration::new(
                "scroll-snap-type",
                "y var(--tw-scroll-snap-strictness)",
            )]),
            "both" => Some(vec![Declaration::new(
                "scroll-snap-type",
                "both var(--tw-scroll-snap-strictness)",
            )]),
            "start" | "end" | "center" => {
                Some(vec![Declaration::new("scroll-snap-align", value)])
            }
            "align-none" => Some(vec![Declaration::new("scroll-snap-align", "none")]),
            "normal" | "always" => {
                Some(vec![Declaration::new("scroll-snap-stop", value)])
            }
            "mandatory" | "proximity" => Some(vec![Declaration::new(
                "--tw-scroll-snap-strictness",
                value,
            )]),
            _ => None,
        },

        // ── mask: size / position / clip / origin / repeat / composite / mode / type ──
        "mask" => match value {
            "auto" | "contain" | "cover" => {
                Some(vec![Declaration::new("mask-size", value)])
            }
            "top" | "bottom" | "left" | "right" | "center" => {
                Some(vec![Declaration::new("mask-position", value)])
            }
            "top-left" => Some(vec![Declaration::new("mask-position", "top left")]),
            "top-right" => Some(vec![Declaration::new("mask-position", "top right")]),
            "bottom-left" => Some(vec![Declaration::new("mask-position", "bottom left")]),
            "bottom-right" => Some(vec![Declaration::new("mask-position", "bottom right")]),
            "repeat" | "no-repeat" | "repeat-x" | "repeat-y" => {
                Some(vec![Declaration::new("mask-repeat", value)])
            }
            "repeat-round" => Some(vec![Declaration::new("mask-repeat", "round")]),
            "repeat-space" => Some(vec![Declaration::new("mask-repeat", "space")]),
            "add" | "subtract" | "intersect" | "exclude" => {
                Some(vec![Declaration::new("mask-composite", value)])
            }
            "alpha" | "luminance" => {
                Some(vec![Declaration::new("mask-mode", value)])
            }
            "match" => Some(vec![Declaration::new("mask-mode", "match-source")]),
            "no-clip" => Some(vec![Declaration::new("mask-clip", "no-clip")]),
            _ => {
                if let Some(clip) = value.strip_prefix("clip-") {
                    let css = match clip {
                        "border" => "border-box",
                        "content" => "content-box",
                        "padding" => "padding-box",
                        "fill" => "fill-box",
                        "stroke" => "stroke-box",
                        "view" => "view-box",
                        _ => return None,
                    };
                    return Some(vec![Declaration::new("mask-clip", css)]);
                }
                if let Some(origin) = value.strip_prefix("origin-") {
                    let css = match origin {
                        "border" => "border-box",
                        "content" => "content-box",
                        "padding" => "padding-box",
                        "fill" => "fill-box",
                        "stroke" => "stroke-box",
                        "view" => "view-box",
                        _ => return None,
                    };
                    return Some(vec![Declaration::new("mask-origin", css)]);
                }
                if let Some(typ) = value.strip_prefix("type-") {
                    return Some(vec![Declaration::new("mask-type", typ.to_string())]);
                }
                None
            }
        },

        // ── translate: complex CSS variable composition ──────────
        "translate" | "translate-x" | "translate-y" | "translate-z" => {
            if value == "none" {
                return Some(vec![Declaration::new("translate", "none")]);
            }
            let css_val = get_spacing_value(value)?;
            let final_val = if parsed.negative {
                format!("-{}", css_val)
            } else {
                css_val
            };
            let result = match parsed.plugin.as_str() {
                "translate" => format!("{0} {0}", final_val),
                "translate-x" => format!("{} var(--tw-translate-y)", final_val),
                "translate-y" => format!("var(--tw-translate-x) {}", final_val),
                "translate-z" => {
                    format!("var(--tw-translate-x) var(--tw-translate-y) {}", final_val)
                }
                _ => unreachable!(),
            };
            Some(vec![Declaration::new("translate", result)])
        }

        // ── scale: named values ──────────────────────────────────
        "scale" => match value {
            "none" => Some(vec![Declaration::new("scale", "none")]),
            "3d" => Some(vec![Declaration::new(
                "scale",
                "var(--tw-scale-x) var(--tw-scale-y) var(--tw-scale-z)",
            )]),
            _ => None,
        },

        // ── transform: mode / style ──────────────────────────────
        "transform" => match value {
            "none" => Some(vec![Declaration::new("transform", "none")]),
            "gpu" => Some(vec![Declaration::new(
                "transform",
                "translateZ(0) var(--tw-rotate-x) var(--tw-rotate-y) var(--tw-rotate-z) var(--tw-skew-x) var(--tw-skew-y)",
            )]),
            "cpu" => Some(vec![Declaration::new(
                "transform",
                "var(--tw-rotate-x) var(--tw-rotate-y) var(--tw-rotate-z) var(--tw-skew-x) var(--tw-skew-y)",
            )]),
            "flat" => Some(vec![Declaration::new("transform-style", "flat")]),
            "3d" => Some(vec![Declaration::new("transform-style", "preserve-3d")]),
            _ => None,
        },

        // ── blur: filter with var() ──────────────────────────────
        "blur" => {
            if self.use_variables {
                Some(vec![Declaration::new(
                    "filter",
                    format!("blur(var(--blur-{}))", value),
                )])
            } else {
                let size = theme_values::BLUR_SIZE.get(value)?;
                Some(vec![Declaration::new(
                    "filter",
                    format!("blur({})", size),
                )])
            }
        }

        // ── backdrop-blur: backdrop-filter with var() ────────────
        "backdrop-blur" => {
            if self.use_variables {
                Some(vec![Declaration::new(
                    "backdrop-filter",
                    format!("blur(var(--blur-{}))", value),
                )])
            } else {
                let size = theme_values::BLUR_SIZE.get(value)?;
                Some(vec![Declaration::new(
                    "backdrop-filter",
                    format!("blur({})", size),
                )])
            }
        }

        // ── backdrop: filter-none ────────────────────────────────
        "backdrop" => match value {
            "filter-none" => Some(vec![Declaration::new("backdrop-filter", "none")]),
            _ => None,
        },

        // ── filter: none ─────────────────────────────────────────
        "filter" => match value {
            "none" => Some(vec![Declaration::new("filter", "none")]),
            _ => None,
        },

        // ── underline: offset ────────────────────────────────────
        "underline" => match value {
            "offset-auto" => Some(vec![Declaration::new("text-underline-offset", "auto")]),
            _ => None,
        },

        // ── line-clamp ───────────────────────────────────────────
        "line-clamp" => match value {
            "none" => Some(vec![
                Declaration::new("overflow", "visible"),
                Declaration::new("display", "block"),
                Declaration::new("-webkit-box-orient", "horizontal"),
                Declaration::new("-webkit-line-clamp", "unset"),
            ]),
            _ => None,
        },

        // ── break (word-break) ───────────────────────────────────
        "break" => match value {
            "all" => Some(vec![Declaration::new("word-break", "break-all")]),
            "keep" => Some(vec![Declaration::new("word-break", "keep-all")]),
            "normal" => Some(vec![Declaration::new("word-break", "normal")]),
            _ => None,
        },

        // ── aspect ratio ─────────────────────────────────────────
        "aspect" => match value {
            "auto" => Some(vec![Declaration::new("aspect-ratio", "auto")]),
            "square" => Some(vec![Declaration::new("aspect-ratio", "1 / 1")]),
            "video" => {
                if self.use_variables {
                    Some(vec![Declaration::new(
                        "aspect-ratio",
                        "var(--aspect-video)",
                    )])
                } else {
                    Some(vec![Declaration::new("aspect-ratio", "16 / 9")])
                }
            }
            _ => None,
        },

        // ── rotate ───────────────────────────────────────────────
        "rotate" => match value {
            "none" => Some(vec![Declaration::new("rotate", "none")]),
            _ => None,
        },

        // ── divide: border-style with child combinator ──────────
        "divide" => match value {
            "solid" | "dashed" | "dotted" | "double" | "hidden" | "none" => {
                Some(vec![Declaration::new("border-style", value)])
            }
            _ => None,
        },

        // ── leading: line-height ────────────────────────────────
        "leading" => match value {
            "none" => Some(vec![Declaration::new("line-height", "1")]),
            _ => {
                if let Ok(n) = value.parse::<u32>() {
                    Some(vec![Declaration::new(
                        "line-height",
                        format!("calc(var(--spacing) * {})", n),
                    )])
                } else {
                    None // fall through to standard path (infer_value handles named values)
                }
            }
        },

        // ── from / via / to: gradient color stops ────────────────
        "from" => {
            get_color_value(value, self.color_mode)
                .map(|color| vec![Declaration::new("--tw-gradient-from", color)])
        }
        "via" => {
            get_color_value(value, self.color_mode)
                .map(|color| vec![Declaration::new("--tw-gradient-via", color)])
        }
        "to" => {
            get_color_value(value, self.color_mode)
                .map(|color| vec![Declaration::new("--tw-gradient-to", color)])
        }

        _ => None,
    }
}
}

// ---------------------------------------------------------------------------
// 工具函数
// ---------------------------------------------------------------------------

/// 判断 CSS 属性是否为颜色属性
fn is_color_property(property: &str) -> bool {
    matches!(
        property,
        "color"
            | "background"
            | "background-color"
            | "border-color"
            | "outline-color"
            | "text-decoration-color"
            | "stroke"
            | "fill"
            | "accent-color"
            | "caret-color"
            | "--tw-shadow-color"
            | "--tw-inset-shadow-color"
            | "--tw-ring-color"
            | "--tw-inset-ring-color"
            | "--tw-gradient-from"
            | "--tw-gradient-via"
            | "--tw-gradient-to"
    )
}

/// 将透明度百分比转为 hex 字节（0-255），返回 2 位 hex 字符串
fn alpha_percent_to_hex(percent: f64) -> String {
    let byte = (percent / 100.0 * 255.0).round() as u8;
    format!("{:02x}", byte)
}

/// 为 hex 颜色添加 alpha 通道
///
/// 支持短格式优化：当 #rrggbb 每对字符相同且 alpha hex 也相同时，
/// 输出 4 位短格式（如 #ffffff + 60% → #fff9）
fn apply_alpha_to_hex(hex: &str, alpha_pct: f64) -> String {
    let body = hex.strip_prefix('#').unwrap_or(hex);
    let alpha_hex = alpha_percent_to_hex(alpha_pct);
    let ab = alpha_hex.as_bytes();

    if body.len() == 6 {
        let b = body.as_bytes();
        // 可以缩短：每对字符相同 + alpha 两位相同
        if b[0] == b[1] && b[2] == b[3] && b[4] == b[5] && ab[0] == ab[1] {
            return format!("#{}{}{}{}", b[0] as char, b[2] as char, b[4] as char, ab[0] as char);
        }
    }

    format!("#{}{}", body, alpha_hex)
}

/// 为颜色值应用 alpha 透明度
///
/// 根据值的格式选择不同的策略：
/// - hex: #rrggbb → #rrggbbaa（支持短格式优化）
/// - oklch/hsl/rgb: 在闭合括号前插入 `/ N%`
/// - var(): 无法直接应用 alpha，需要 color-mix（此函数跳过）
/// - transparent/currentColor: 跳过
fn apply_alpha_to_color(value: &str, alpha: &str, use_color_mix: bool) -> String {
    let alpha_pct: f64 = match alpha.parse() {
        Ok(n) => n,
        Err(_) => return value.to_string(),
    };

    // 100% = 完全不透明 → 不修改
    if (alpha_pct - 100.0).abs() < f64::EPSILON {
        return value.to_string();
    }

    // transparent / currentColor 无法应用 alpha
    if value == "transparent" || value == "currentColor" {
        return value.to_string();
    }

    // color-mix 模式：所有颜色值统一使用 color-mix
    if use_color_mix {
        return format!(
            "color-mix(in oklab, {} {}%, transparent)",
            value, alpha_pct as u32
        );
    }

    if value.starts_with('#') {
        apply_alpha_to_hex(value, alpha_pct)
    } else if value.starts_with("var(") {
        // var() → 无法直接应用 alpha（需要 color-mix）
        value.to_string()
    } else if let Some(pos) = value.rfind(')') {
        // oklch(...) / hsl(...) / rgb(...) → 插入 / N%
        format!("{} / {}%)", &value[..pos], alpha_pct as u32)
    } else {
        value.to_string()
    }
}

/// 为声明列表中的颜色属性应用 alpha 透明度
fn apply_alpha_to_declarations(
    declarations: Vec<Declaration>,
    alpha: &str,
    use_color_mix: bool,
) -> Vec<Declaration> {
    declarations
        .into_iter()
        .map(|mut decl| {
            if is_color_property(&decl.property) {
                decl.value = apply_alpha_to_color(&decl.value, alpha, use_color_mix);
            }
            decl
        })
        .collect()
}

/// 应用 !important 标记
fn apply_important(declarations: Vec<Declaration>, important: bool) -> Vec<Declaration> {
    if !important {
        return declarations;
    }
    declarations
        .into_iter()
        .map(|mut decl| {
            if !decl.value.ends_with("!important") {
                decl.value = format!("{} !important", decl.value);
            }
            decl
        })
        .collect()
}

/// 构建基础类名（不包含修饰符）
fn build_base_class(parsed: &ParsedClass) -> String {
    let mut class = String::new();

    if parsed.negative {
        class.push('-');
    }

    class.push_str(&parsed.plugin);

    if let Some(value) = &parsed.value {
        class.push('-');
        class.push_str(&value.to_string());
    }

    if let Some(alpha) = &parsed.alpha {
        class.push('/');
        class.push_str(alpha);
    }

    class
}

/// 构建 CSS 选择器，包含修饰符
fn build_selector(parsed: &ParsedClass) -> String {
    let class_name = build_base_class(parsed);
    let mut selector = format!(".{}", class_name);

    for modifier in &parsed.modifiers() {
        selector = apply_modifier(&selector, modifier);
    }

    selector
}

/// 应用单个修饰符到选择器
fn apply_modifier(selector: &str, modifier: &Modifier) -> String {
    match modifier {
        Modifier::PseudoClass(name) => format!("{}:{}", selector, name),
        Modifier::PseudoElement(name) => format!("{}::{}", selector, name),
        Modifier::State(name) => match name.as_str() {
            "dark" => format!(".dark {}", selector),
            name if name.starts_with("group-") => {
                format!(".group:{} {}", &name[6..], selector)
            }
            name if name.starts_with("peer-") => {
                format!(".peer:{} ~ {}", &name[5..], selector)
            }
            _ => selector.to_string(),
        },
        Modifier::Responsive(size) => {
            let breakpoint = BREAKPOINT_MAP.get(size.as_str()).copied().unwrap_or("0px");
            format!("@media (min-width: {}) {{ {} }}", breakpoint, selector)
        }
        Modifier::Custom(name) => format!("{}:{}", selector, name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwind_tw_parse::parse_class;

    #[test]
    fn test_convert_standard_value() {
        let converter = Converter::new();

        let parsed = parse_class("p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "padding");
        assert_eq!(rule.declarations[0].value, "1rem");
    }

    #[test]
    fn test_convert_valueless_class() {
        let converter = Converter::new();

        let parsed = parse_class("flex").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".flex");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "display");
        assert_eq!(rule.declarations[0].value, "flex");
    }

    #[test]
    fn test_convert_with_pseudo_class() {
        let converter = Converter::new();

        let parsed = parse_class("hover:p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4:hover");
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn test_convert_with_responsive() {
        let converter = Converter::new();

        let parsed = parse_class("md:text-center").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert!(rule.selector.contains("@media"));
        assert!(rule.selector.contains("768px"));
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn test_convert_with_important() {
        let converter = Converter::new();

        let parsed = parse_class("p-4!").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4");
        assert!(rule.declarations[0].value.contains("!important"));
    }

    #[test]
    fn test_convert_multiple_modifiers() {
        let converter = Converter::new();

        let parsed = parse_class("md:hover:p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert!(rule.selector.contains("@media"));
        assert!(rule.selector.contains(":hover"));
    }

    #[test]
    fn test_convert_arbitrary_value() {
        let converter = Converter::new();

        let parsed = parse_class("w-[13px]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".w-[13px]");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "width");
        assert_eq!(rule.declarations[0].value, "13px");
    }

    #[test]
    fn test_convert_arbitrary_value_with_modifier() {
        let converter = Converter::new();

        let parsed = parse_class("hover:w-[13px]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".w-[13px]:hover");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "width");
        assert_eq!(rule.declarations[0].value, "13px");
    }

    #[test]
    fn test_convert_arbitrary_value_multi_property() {
        let converter = Converter::new();

        let parsed = parse_class("px-[2rem]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".px-[2rem]");
        assert_eq!(rule.declarations.len(), 2);
        assert_eq!(rule.declarations[0].property, "padding-left");
        assert_eq!(rule.declarations[0].value, "2rem");
        assert_eq!(rule.declarations[1].property, "padding-right");
        assert_eq!(rule.declarations[1].value, "2rem");
    }

    #[test]
    fn test_convert_arbitrary_value_with_color() {
        let converter = Converter::new();

        let parsed = parse_class("text-[#1da1f2]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".text-[#1da1f2]");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "color");
        assert_eq!(rule.declarations[0].value, "#1da1f2");
    }

    #[test]
    fn test_convert_color_value() {
        let converter = Converter::new();

        let parsed = parse_class("bg-blue-500").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".bg-blue-500");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "background");
        assert!(rule.declarations[0].value.starts_with('#'));
    }

    #[test]
    fn test_convert_negative_value() {
        let converter = Converter::new();

        let parsed = parse_class("-m-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".-m-4");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "margin");
        assert_eq!(rule.declarations[0].value, "-1rem");
    }

    #[test]
    fn test_convert_valueless_fallback() {
        // overflow-auto: parser gives plugin="overflow", value="auto"
        // converter falls back to VALUELESS_MAP lookup of "overflow-auto"
        let converter = Converter::new();

        let parsed = parse_class("overflow-auto").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "overflow");
        assert_eq!(rule.declarations[0].value, "auto");
    }

    #[test]
    fn test_convert_compound_plugin() {
        // justify-items-center: parser detects compound plugin "justify-items"
        let converter = Converter::new();

        let parsed = parse_class("justify-items-center").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "justify-items");
        assert_eq!(rule.declarations[0].value, "center");
    }

    #[test]
    fn test_convert_compound_gap_x() {
        let converter = Converter::new();

        let parsed = parse_class("gap-x-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "column-gap");
        assert_eq!(rule.declarations[0].value, "1rem");
    }

    // ── Gradient tests ──────────────────────────────────────────

    #[test]
    fn test_bg_linear_angle() {
        let converter = Converter::new();
        let parsed = parse_class("bg-linear-45").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(45deg in oklab, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_linear_negative_angle() {
        let converter = Converter::new();
        let parsed = parse_class("-bg-linear-45").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(-45deg in oklab, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_linear_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("bg-linear-[45deg]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(var(--tw-gradient-stops, 45deg))"
        );
    }

    #[test]
    fn test_bg_conic_angle() {
        let converter = Converter::new();
        let parsed = parse_class("bg-conic-90").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "conic-gradient(from 90deg in oklab, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_conic_arbitrary() {
        // bg-conic-[<value>] → background-image: <value> (raw, not wrapped)
        let converter = Converter::new();
        let parsed = parse_class("bg-conic-[from_45deg]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(decls[0].value, "from 45deg");
    }

    #[test]
    fn test_bg_conic_css_variable() {
        // bg-conic-(--my-gradient) → background-image: var(--my-gradient)
        let converter = Converter::new();
        let parsed = parse_class("bg-conic-(--my-gradient)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(decls[0].value, "var(--my-gradient)");
    }

    #[test]
    fn test_bg_conic_negative_angle() {
        // -bg-conic-90 → conic-gradient(from -90deg in oklab, ...)
        let converter = Converter::new();
        let parsed = parse_class("-bg-conic-90").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "conic-gradient(from -90deg in oklab, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_radial_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("bg-radial-[circle]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "radial-gradient(var(--tw-gradient-stops, circle))"
        );
    }

    #[test]
    fn test_from_color() {
        let converter = Converter::new();
        let parsed = parse_class("from-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-from");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_from_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("from-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-from");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_via_color() {
        let converter = Converter::new();
        let parsed = parse_class("via-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-via");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_to_color() {
        let converter = Converter::new();
        let parsed = parse_class("to-green-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-to");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_to_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("to-[rgba(0,0,0,0.5)]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-to");
        assert_eq!(decls[0].value, "rgba(0,0,0,0.5)");
    }

    // ── 颜色模式测试 ──────────────────────────────────────────

    #[test]
    fn test_color_mode_oklch() {
        let converter = Converter::new().with_color_mode(ColorMode::Oklch);
        let parsed = parse_class("bg-blue-500").unwrap();
        let rule = converter.convert(&parsed).unwrap();
        assert_eq!(rule.declarations[0].value, "oklch(0.623 0.214 259.815)");
    }

    #[test]
    fn test_color_mode_var() {
        let converter = Converter::new().with_color_mode(ColorMode::Var);
        let parsed = parse_class("text-red-500").unwrap();
        let rule = converter.convert(&parsed).unwrap();
        assert_eq!(rule.declarations[0].property, "color");
        assert_eq!(rule.declarations[0].value, "var(--color-red-500)");
    }

    #[test]
    fn test_color_mode_hsl() {
        let converter = Converter::new().with_color_mode(ColorMode::Hsl);
        let parsed = parse_class("from-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert!(decls[0].value.starts_with("hsl("));
    }

    #[test]
    fn test_new_color_families() {
        let converter = Converter::new();
        // 新增的颜色族应该能正常转换
        for family in &["orange", "amber", "violet", "slate", "zinc", "rose", "emerald"] {
            let class = format!("bg-{}-500", family);
            let parsed = parse_class(&class).unwrap();
            let rule = converter.convert(&parsed);
            assert!(rule.is_some(), "Failed for: {}", class);
        }
    }

    // ── CSS 自定义属性 -(…) 语法测试 ──────────────────────────

    #[test]
    fn test_css_variable_bg() {
        let converter = Converter::new();
        let parsed = parse_class("bg-(--my-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background");
        assert_eq!(decls[0].value, "var(--my-color)");
    }

    #[test]
    fn test_css_variable_bg_with_type_hint_image() {
        let converter = Converter::new();
        let parsed = parse_class("bg-(image:--my-bg)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(decls[0].value, "var(--my-bg)");
    }

    #[test]
    fn test_css_variable_bg_with_type_hint_color() {
        let converter = Converter::new();
        let parsed = parse_class("bg-(color:--my-bg)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-color");
        assert_eq!(decls[0].value, "var(--my-bg)");
    }

    #[test]
    fn test_css_variable_bg_linear() {
        let converter = Converter::new();
        let parsed = parse_class("bg-linear-(--my-gradient)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(var(--tw-gradient-stops, var(--my-gradient)))"
        );
    }

    #[test]
    fn test_css_variable_bg_radial() {
        let converter = Converter::new();
        let parsed = parse_class("bg-radial-(--my-gradient)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "radial-gradient(var(--tw-gradient-stops, var(--my-gradient)))"
        );
    }

    #[test]
    fn test_css_variable_bg_conic() {
        let converter = Converter::new();
        let parsed = parse_class("bg-conic-(--my-gradient)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(decls[0].value, "var(--my-gradient)");
    }

    #[test]
    fn test_css_variable_from() {
        let converter = Converter::new();
        let parsed = parse_class("from-(--start-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-from");
        assert_eq!(decls[0].value, "var(--start-color)");
    }

    #[test]
    fn test_css_variable_via() {
        let converter = Converter::new();
        let parsed = parse_class("via-(--mid-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-via");
        assert_eq!(decls[0].value, "var(--mid-color)");
    }

    #[test]
    fn test_css_variable_to() {
        let converter = Converter::new();
        let parsed = parse_class("to-(--end-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-to");
        assert_eq!(decls[0].value, "var(--end-color)");
    }

    #[test]
    fn test_css_variable_text() {
        let converter = Converter::new();
        let parsed = parse_class("text-(--my-text-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "var(--my-text-color)");
    }

    #[test]
    fn test_css_variable_padding() {
        let converter = Converter::new();
        let parsed = parse_class("p-(--spacing)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "padding");
        assert_eq!(decls[0].value, "var(--spacing)");
    }

    #[test]
    fn test_css_variable_width() {
        let converter = Converter::new();
        let parsed = parse_class("w-(--sidebar-width)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "width");
        assert_eq!(decls[0].value, "var(--sidebar-width)");
    }

    #[test]
    fn test_css_variable_with_important() {
        let converter = Converter::new();
        let parsed = parse_class("bg-(--my-color)!").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background");
        assert!(decls[0].value.contains("var(--my-color)"));
        assert!(decls[0].value.contains("!important"));
    }

    #[test]
    fn test_css_variable_gap_x() {
        let converter = Converter::new();
        let parsed = parse_class("gap-x-(--my-gap)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "column-gap");
        assert_eq!(decls[0].value, "var(--my-gap)");
    }

    // ── 颜色插件测试 ────────────────────────────────────────────

    // --- accent ---

    #[test]
    fn test_accent_standard_color() {
        let converter = Converter::new();
        let parsed = parse_class("accent-stone-100").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "accent-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_accent_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("accent-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "accent-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_accent_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("accent-(--my-accent)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "accent-color");
        assert_eq!(decls[0].value, "var(--my-accent)");
    }

    // --- caret ---

    #[test]
    fn test_caret_standard_color() {
        let converter = Converter::new();
        let parsed = parse_class("caret-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "caret-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_caret_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("caret-[#00ff00]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "caret-color");
        assert_eq!(decls[0].value, "#00ff00");
    }

    #[test]
    fn test_caret_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("caret-(--my-caret)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "caret-color");
        assert_eq!(decls[0].value, "var(--my-caret)");
    }

    // --- fill ---

    #[test]
    fn test_fill_standard_color() {
        let converter = Converter::new();
        let parsed = parse_class("fill-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "fill");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_fill_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("fill-[#0000ff]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "fill");
        assert_eq!(decls[0].value, "#0000ff");
    }

    #[test]
    fn test_fill_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("fill-(--icon-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "fill");
        assert_eq!(decls[0].value, "var(--icon-color)");
    }

    // --- stroke ---

    #[test]
    fn test_stroke_standard_color() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-green-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_stroke_width() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke-width");
        assert_eq!(decls[0].value, "2");
    }

    #[test]
    fn test_stroke_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_stroke_arbitrary_width() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-[3px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke-width");
        assert_eq!(decls[0].value, "3px");
    }

    #[test]
    fn test_stroke_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-(--stroke-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke");
        assert_eq!(decls[0].value, "var(--stroke-color)");
    }

    // --- border color ---

    #[test]
    fn test_border_color() {
        let converter = Converter::new();
        let parsed = parse_class("border-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_border_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("border-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_border_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("border-(--border-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-color");
        assert_eq!(decls[0].value, "var(--border-color)");
    }

    // --- border width ---

    #[test]
    fn test_border_valueless() {
        // border → border-width: 1px
        let converter = Converter::new();
        let parsed = parse_class("border").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-width");
        assert_eq!(decls[0].value, "1px");
    }

    #[test]
    fn test_border_number() {
        // border-2 → border-width: 2px
        let converter = Converter::new();
        let parsed = parse_class("border-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-width");
        assert_eq!(decls[0].value, "2px");
    }

    #[test]
    fn test_border_arbitrary_width() {
        // border-[3px] → border-width: 3px
        let converter = Converter::new();
        let parsed = parse_class("border-[3px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-width");
        assert_eq!(decls[0].value, "3px");
    }

    #[test]
    fn test_border_css_variable_length() {
        // border-(length:--my-width) → border-width: var(--my-width)
        let converter = Converter::new();
        let parsed = parse_class("border-(length:--my-width)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-width");
        assert_eq!(decls[0].value, "var(--my-width)");
    }

    // --- outline color ---

    #[test]
    fn test_outline_color() {
        let converter = Converter::new();
        let parsed = parse_class("outline-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "outline-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_outline_width() {
        let converter = Converter::new();
        let parsed = parse_class("outline-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "outline-width");
        assert_eq!(decls[0].value, "2px");
    }

    #[test]
    fn test_outline_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("outline-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "outline-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_outline_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("outline-(--outline-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "outline-color");
        assert_eq!(decls[0].value, "var(--outline-color)");
    }

    // --- decoration color ---

    #[test]
    fn test_decoration_color() {
        let converter = Converter::new();
        let parsed = parse_class("decoration-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "text-decoration-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_decoration_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("decoration-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "text-decoration-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_decoration_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("decoration-(--deco-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "text-decoration-color");
        assert_eq!(decls[0].value, "var(--deco-color)");
    }

    // --- shadow color ---

    #[test]
    fn test_shadow_color() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-shadow-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_shadow_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-shadow-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_shadow_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-(--shadow-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-shadow-color");
        assert_eq!(decls[0].value, "var(--shadow-color)");
    }

    // --- ring color ---

    #[test]
    fn test_ring_color() {
        let converter = Converter::new();
        let parsed = parse_class("ring-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_ring_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("ring-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_ring_css_variable_width() {
        let converter = Converter::new();
        let parsed = parse_class("ring-(--ring-width)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-shadow");
        assert_eq!(decls[0].value, "0 0 0 var(--ring-width)");
    }

    // --- inset-shadow ---

    #[test]
    fn test_inset_shadow_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-(--inset-shadow-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-shadow-color");
        assert_eq!(decls[0].value, "var(--inset-shadow-color)");
    }

    // --- inset-ring ---

    #[test]
    fn test_inset_ring_css_variable_width() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-(--ring-width)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-shadow");
        assert_eq!(decls[0].value, "inset 0 0 0 var(--ring-width)");
    }

    // ── shadow named sizes ───────────────────────────────────────

    #[test]
    fn test_shadow_sm() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-sm").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--shadow-sm)");
    }

    #[test]
    fn test_shadow_md() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-md").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--shadow-md)");
    }

    #[test]
    fn test_shadow_2xl() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-2xl").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--shadow-2xl)");
    }

    #[test]
    fn test_shadow_none() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-none").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "0 0 #0000");
    }

    // ── inset-shadow named sizes ─────────────────────────────────

    #[test]
    fn test_inset_shadow_sm() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-sm").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--inset-shadow-sm)");
    }

    #[test]
    fn test_inset_shadow_2xs() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-2xs").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--inset-shadow-2xs)");
    }

    #[test]
    fn test_inset_shadow_none() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-none").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "inset 0 0 #0000");
    }

    #[test]
    fn test_inset_shadow_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-shadow-color");
        assert!(decls[0].value.starts_with('#'));
    }

    // ── ring width ───────────────────────────────────────────────

    #[test]
    fn test_ring_valueless() {
        let converter = Converter::new();
        let parsed = parse_class("ring").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-shadow");
        assert_eq!(decls[0].value, "0 0 0 1px");
    }

    #[test]
    fn test_ring_number() {
        let converter = Converter::new();
        let parsed = parse_class("ring-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-shadow");
        assert_eq!(decls[0].value, "0 0 0 2px");
    }

    #[test]
    fn test_ring_arbitrary_width() {
        let converter = Converter::new();
        let parsed = parse_class("ring-[3px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-shadow");
        assert_eq!(decls[0].value, "0 0 0 3px");
    }

    // ── inset-ring width ─────────────────────────────────────────

    #[test]
    fn test_inset_ring_valueless() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-shadow");
        assert_eq!(decls[0].value, "inset 0 0 0 1px");
    }

    #[test]
    fn test_inset_ring_number() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-shadow");
        assert_eq!(decls[0].value, "inset 0 0 0 2px");
    }

    #[test]
    fn test_inset_ring_arbitrary_width() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-[3px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-shadow");
        assert_eq!(decls[0].value, "inset 0 0 0 3px");
    }

    #[test]
    fn test_inset_ring_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_inset_ring_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_inset_shadow_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-shadow-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    // ── text: font-size vs color ─────────────────────────────────

    #[test]
    fn test_text_arbitrary_font_size() {
        let converter = Converter::new();
        let parsed = parse_class("text-[14px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "14px");
    }

    #[test]
    fn test_text_arbitrary_font_size_rem() {
        let converter = Converter::new();
        let parsed = parse_class("text-[1.5rem]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "1.5rem");
    }

    #[test]
    fn test_text_arbitrary_color_hex() {
        let converter = Converter::new();
        let parsed = parse_class("text-[#1da1f2]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "#1da1f2");
    }

    #[test]
    fn test_text_arbitrary_color_rgb() {
        let converter = Converter::new();
        let parsed = parse_class("text-[rgb(255,0,0)]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "rgb(255,0,0)");
    }

    #[test]
    fn test_text_css_variable_length_hint() {
        let converter = Converter::new();
        let parsed = parse_class("text-(length:--my-size)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--my-size)");
    }

    // ── text-size/alpha: line-height overrides ───────────────────

    #[test]
    fn test_text_size_with_number_line_height() {
        let converter = Converter::new();
        let parsed = parse_class("text-base/6").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--text-base)");
        assert_eq!(decls[1].property, "line-height");
        assert_eq!(decls[1].value, "calc(var(--spacing) * 6)");
    }

    #[test]
    fn test_text_size_with_bracket_line_height() {
        let converter = Converter::new();
        let parsed = parse_class("text-base/[1.5rem]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--text-base)");
        assert_eq!(decls[1].property, "line-height");
        assert_eq!(decls[1].value, "1.5rem");
    }

    #[test]
    fn test_text_size_with_css_var_line_height() {
        let converter = Converter::new();
        let parsed = parse_class("text-base/(--my-lh)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--text-base)");
        assert_eq!(decls[1].property, "line-height");
        assert_eq!(decls[1].value, "var(--my-lh)");
    }

    // ── leading ──────────────────────────────────────────────────

    #[test]
    fn test_leading_none() {
        let converter = Converter::new();
        let parsed = parse_class("leading-none").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "line-height");
        assert_eq!(decls[0].value, "1");
    }

    #[test]
    fn test_leading_number() {
        let converter = Converter::new();
        let parsed = parse_class("leading-6").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "line-height");
        assert_eq!(decls[0].value, "calc(var(--spacing) * 6)");
    }

    #[test]
    fn test_leading_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("leading-(--my-lh)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "line-height");
        assert_eq!(decls[0].value, "var(--my-lh)");
    }

    #[test]
    fn test_leading_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("leading-[1.5rem]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "line-height");
        assert_eq!(decls[0].value, "1.5rem");
    }

    // ── alpha / opacity ─────────────────────────────────────────

    #[test]
    fn test_alpha_hex_white_60() {
        // text-white/60 → color: #fff9 (short form: ff→f, ff→f, ff→f, 99→9)
        let converter = Converter::new();
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "#fff9");
    }

    #[test]
    fn test_alpha_hex_black_50() {
        // text-black/50 → color: #00000080 (50% = 0x80, digits 8/0 differ → long form)
        let converter = Converter::new();
        let parsed = parse_class("text-black/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "#00000080");
    }

    #[test]
    fn test_alpha_hex_color_long_form() {
        // bg-blue-500/60 → background: #3b82f699 (not shortable)
        let converter = Converter::new();
        let parsed = parse_class("bg-blue-500/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background");
        assert!(decls[0].value.len() == 9); // #rrggbbaa
        assert!(decls[0].value.ends_with("99")); // 60% = 0x99
    }

    #[test]
    fn test_alpha_100_no_change() {
        // text-white/100 → no alpha applied
        let converter = Converter::new();
        let parsed = parse_class("text-white/100").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "#ffffff");
    }

    #[test]
    fn test_alpha_oklch_mode() {
        // text-white/60 in oklch → oklch(1 0 0 / 60%)
        let converter = Converter::new().with_color_mode(ColorMode::Oklch);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "oklch(1 0 0 / 60%)");
    }

    #[test]
    fn test_alpha_hsl_mode() {
        // text-white/60 in hsl → hsl(0, 0%, 100% / 60%)
        let converter = Converter::new().with_color_mode(ColorMode::Hsl);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "hsl(0, 0%, 100% / 60%)");
    }

    #[test]
    fn test_alpha_var_mode_no_color_mix() {
        // text-white/60 in var mode without color-mix → alpha not applied
        let converter = Converter::new().with_color_mode(ColorMode::Var);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "var(--color-white)"); // can't apply alpha
    }

    #[test]
    fn test_alpha_var_mode_with_color_mix() {
        // text-white/60 in var mode with color-mix → color-mix(in oklab, …)
        let converter = Converter::new()
            .with_color_mode(ColorMode::Var)
            .with_color_mix(true);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(
            decls[0].value,
            "color-mix(in oklab, var(--color-white) 60%, transparent)"
        );
    }

    #[test]
    fn test_alpha_color_mix_hex_mode() {
        // color-mix enabled even in hex mode → generates color-mix
        let converter = Converter::new()
            .with_color_mode(ColorMode::Hex)
            .with_color_mix(true);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(
            decls[0].value,
            "color-mix(in oklab, #ffffff 60%, transparent)"
        );
    }

    #[test]
    fn test_alpha_does_not_apply_to_non_color() {
        // text-base/6 → alpha used for line-height, NOT applied to font-size
        let converter = Converter::new();
        let parsed = parse_class("text-base/6").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--text-base)"); // no alpha
        assert_eq!(decls[1].property, "line-height");
        assert_eq!(decls[1].value, "calc(var(--spacing) * 6)");
    }

    #[test]
    fn test_alpha_border_color() {
        // border-red-500/50 → border-color with alpha
        let converter = Converter::new();
        let parsed = parse_class("border-red-500/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "border-color");
        assert!(decls[0].value.ends_with("80")); // 50% = 0x80
    }

    #[test]
    fn test_alpha_transparent_unchanged() {
        // bg-transparent/50 → transparent stays as-is
        let converter = Converter::new();
        let parsed = parse_class("bg-transparent/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "background");
        assert_eq!(decls[0].value, "transparent");
    }

    #[test]
    fn test_alpha_shadow_color() {
        // shadow-red-500/50 → --tw-shadow-color with alpha
        let converter = Converter::new();
        let parsed = parse_class("shadow-red-500/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "--tw-shadow-color");
        assert!(decls[0].value.ends_with("80")); // 50% = 0x80
    }

    #[test]
    fn test_alpha_ring_color() {
        // ring-blue-500/50 → --tw-ring-color with alpha
        let converter = Converter::new();
        let parsed = parse_class("ring-blue-500/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "--tw-ring-color");
        assert!(decls[0].value.ends_with("80")); // 50% = 0x80
    }

    // ── space-x / space-y ──────────────────────────────────────────

    #[test]
    fn test_space_x_0() {
        let converter = Converter::new();
        let parsed = parse_class("space-x-0").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "column-gap");
        assert_eq!(decls[0].value, "0");
    }

    #[test]
    fn test_space_x_2() {
        let converter = Converter::new();
        let parsed = parse_class("space-x-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "column-gap");
        assert_eq!(decls[0].value, "0.5rem");
    }

    #[test]
    fn test_space_y_4() {
        let converter = Converter::new();
        let parsed = parse_class("space-y-4").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "row-gap");
        assert_eq!(decls[0].value, "1rem");
    }

    // ── scroll padding / margin ────────────────────────────────────

    #[test]
    fn test_scroll_pr_6() {
        let converter = Converter::new();
        let parsed = parse_class("scroll-pr-6").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "scroll-padding-right");
        assert_eq!(decls[0].value, "1.5rem");
    }

    #[test]
    fn test_scroll_mt_2() {
        let converter = Converter::new();
        let parsed = parse_class("scroll-mt-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "scroll-margin-top");
        assert_eq!(decls[0].value, "0.5rem");
    }

    #[test]
    fn test_scroll_px_4() {
        let converter = Converter::new();
        let parsed = parse_class("scroll-px-4").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "scroll-padding-left");
        assert_eq!(decls[0].value, "1rem");
        assert_eq!(decls[1].property, "scroll-padding-right");
        assert_eq!(decls[1].value, "1rem");
    }

    #[test]
    fn test_scroll_my_8() {
        let converter = Converter::new();
        let parsed = parse_class("scroll-my-8").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "scroll-margin-top");
        assert_eq!(decls[0].value, "2rem");
        assert_eq!(decls[1].property, "scroll-margin-bottom");
        assert_eq!(decls[1].value, "2rem");
    }
}
