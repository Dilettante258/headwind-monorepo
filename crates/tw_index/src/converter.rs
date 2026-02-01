use crate::plugin_map::get_plugin_properties;
use crate::value_map::{get_spacing_value, infer_value};
use headwind_core::Declaration;
use headwind_tw_parse::{Modifier, ParsedClass, ParsedValue};
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

    // Outline (valueless = 1px width)
    "outline" => ("outline-width", "1px"),

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
pub struct Converter;

impl Converter {
    pub fn new() -> Self {
        Self
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
            Some(ParsedValue::Standard(value)) => build_standard_declarations(parsed, value)
                .or_else(|| build_valueless_from_full_name(parsed, value))?,
            None => build_valueless_declarations(parsed)?,
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

/// 为标准值构建 CSS 声明
///
/// 例如：`p-4` → `padding: 1rem`
fn build_standard_declarations(parsed: &ParsedClass, value: &str) -> Option<Vec<Declaration>> {
    // 不在 plugin_map 中的复杂插件，走专门的分发逻辑
    if let Some(decls) = build_complex_standard(parsed, value) {
        return Some(decls);
    }

    let properties = get_plugin_properties(&parsed.plugin)?;
    let mut css_value = infer_value(&parsed.plugin, value)?;

    if parsed.negative {
        css_value = format!("-{}", css_value);
    }

    let declarations = properties
        .into_iter()
        .map(|property| Declaration::new(property, css_value.clone()))
        .collect();

    Some(declarations)
}

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

/// 处理复杂任意值插件
fn build_complex_arbitrary(parsed: &ParsedClass, raw_value: &str) -> Option<Vec<Declaration>> {
    match parsed.plugin.as_str() {
        // text-[#fff] → color
        "text" => {
            let value = if parsed.negative {
                format!("-{}", raw_value)
            } else {
                raw_value.to_string()
            };
            Some(vec![Declaration::new("color", value)])
        }
        _ => None,
    }
}

/// 处理复杂标准值插件（语义重载，不同值映射到不同 CSS 属性）
fn build_complex_standard(parsed: &ParsedClass, value: &str) -> Option<Vec<Declaration>> {
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
            | "7xl" | "8xl" | "9xl" => Some(vec![
                Declaration::new("font-size", format!("var(--text-{})", value)),
                Declaration::new("line-height", format!("var(--text-{}--line-height)", value)),
            ]),
            _ => {
                let css_value = infer_value(&parsed.plugin, value)?;
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
                None // fall through to standard path for colors
            }
        },

        // ── font: weight / family / stretch ──────────────────────
        "font" => match value {
            "sans" | "serif" | "mono" => Some(vec![Declaration::new(
                "font-family",
                format!("var(--font-{})", value),
            )]),
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

        // ── border: style / collapse ─────────────────────────────
        "border" => match value {
            "solid" | "dashed" | "dotted" | "double" | "hidden" | "none" => {
                Some(vec![Declaration::new("border-style", value)])
            }
            "collapse" | "separate" => {
                Some(vec![Declaration::new("border-collapse", value)])
            }
            _ => None, // fall through for width/color
        },

        // ── decoration: style / thickness ────────────────────────
        "decoration" => match value {
            "solid" | "dashed" | "dotted" | "double" | "wavy" => {
                Some(vec![Declaration::new("text-decoration-style", value)])
            }
            "auto" | "from-font" => {
                Some(vec![Declaration::new("text-decoration-thickness", value)])
            }
            _ => None,
        },

        // ── outline: style / hidden ──────────────────────────────
        "outline" => match value {
            "solid" | "dashed" | "dotted" | "double" | "none" => {
                Some(vec![Declaration::new("outline-style", value)])
            }
            "hidden" => Some(vec![
                Declaration::new("outline", "2px solid transparent"),
                Declaration::new("outline-offset", "2px"),
            ]),
            _ => None,
        },

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
        "blur" => Some(vec![Declaration::new(
            "filter",
            format!("blur(var(--blur-{}))", value),
        )]),

        // ── backdrop-blur: backdrop-filter with var() ────────────
        "backdrop-blur" => Some(vec![Declaration::new(
            "backdrop-filter",
            format!("blur(var(--blur-{}))", value),
        )]),

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
            "video" => Some(vec![Declaration::new(
                "aspect-ratio",
                "var(--aspect-video)",
            )]),
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

        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 工具函数
// ---------------------------------------------------------------------------

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
        assert_eq!(rule.declarations[0].value, "#3b82f6");
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
}
