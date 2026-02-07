use crate::plugin_map::get_plugin_properties;
use headwind_core::Declaration;
use headwind_tw_parse::{CssVariableValue, ParsedClass};

/// 为任意值构建 CSS 声明
///
/// 例如：`w-[13px]` → `width: 13px`
pub(super) fn build_arbitrary_declarations(parsed: &ParsedClass, raw_value: &str) -> Option<Vec<Declaration>> {
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
pub(super) fn build_css_variable_declarations(
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

/// 从字符串中提取方括号内的值
///
/// 例如：`"[45deg]"` → `Some("45deg")`，`"123"` → `None`
pub(super) fn extract_bracket_value(s: &str) -> Option<&str> {
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
