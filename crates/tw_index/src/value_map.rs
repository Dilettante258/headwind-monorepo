use crate::palette;
use headwind_core::ColorMode;
use phf::phf_map;

/// 间距关键字映射（非数字的特殊值）
///
/// 数字值（如 "4" → "1rem"）通过 `n * 0.25rem` 实时计算
static SPACING_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "px" => "1px",
    "auto" => "auto",

    // Fractions (分数)
    "full" => "100%",
    "1/2" => "50%",
    "1/3" => "33.333333%",
    "2/3" => "66.666667%",
    "1/4" => "25%",
    "2/4" => "50%",
    "3/4" => "75%",
    "1/5" => "20%",
    "2/5" => "40%",
    "3/5" => "60%",
    "4/5" => "80%",
    "1/6" => "16.666667%",
    "2/6" => "33.333333%",
    "3/6" => "50%",
    "4/6" => "66.666667%",
    "5/6" => "83.333333%",

    // Keywords
    "min" => "min-content",
    "max" => "max-content",
    "fit" => "fit-content",
};

// 颜色值通过 palette 模块提供，支持 22 色族 × 11 色阶 + 特殊颜色

/// 获取间距值
///
/// 优先查静态映射（关键字、分数），其次识别视口单位，最后尝试数字计算 `n * 0.25rem`
pub fn get_spacing_value(key: &str) -> Option<String> {
    // 1. 静态映射：关键字和分数
    if let Some(&v) = SPACING_MAP.get(key) {
        return Some(v.to_string());
    }

    // 2. 视口单位：svh → 100svh, dvw → 100dvw, etc.
    if is_viewport_unit(key) {
        return Some(format!("100{}", key));
    }

    // 3. 数字值：n * 0.25rem
    let n: f64 = key.parse().ok()?;
    if n < 0.0 {
        return None;
    }
    if n == 0.0 {
        return Some("0".to_string());
    }
    let rem = n * 0.25;
    Some(format!("{}rem", rem))
}

/// 判断是否为视口单位关键字(max,min现在无)
fn is_viewport_unit(key: &str) -> bool {
    matches!(
        key,
        "vw" | "vh" | "svw" | "svh" | "dvw" | "dvh" | "lvw" | "lvh"
    )
}

/// 获取颜色值（根据颜色模式输出对应格式）
pub fn get_color_value(key: &str, mode: ColorMode) -> Option<String> {
    palette::get_color(key, mode)
}

/// 获取不透明度值
///
/// 实时计算 `n / 100`，接受 0-100 的整数
pub fn get_opacity_value(key: &str) -> Option<String> {
    let n: u32 = key.parse().ok()?;
    if n > 100 {
        return None;
    }
    if n == 0 {
        return Some("0".to_string());
    }
    if n == 100 {
        return Some("1".to_string());
    }
    Some(format!("{}", n as f64 / 100.0))
}

/// 容器命名尺寸 → CSS 变量
fn get_container_size(key: &str) -> Option<String> {
    match key {
        "xs" | "sm" | "md" | "lg" | "xl" | "2xs" | "2xl" | "3xs" | "3xl" | "4xl" | "5xl"
        | "6xl" | "7xl" => Some(format!("var(--container-{})", key)),
        _ => None,
    }
}

/// 根据插件类型推断值映射
pub fn infer_value(plugin: &str, value: &str, color_mode: ColorMode) -> Option<String> {
    match plugin {
        // ── Spacing ──────────────────────────────────────────────
        "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" | "m" | "mx" | "my" | "mt" | "mr"
        | "mb" | "ml" | "gap" | "gap-x" | "gap-y" | "space-x" | "space-y" => {
            get_spacing_value(value)
        }

        // ── Width ────────────────────────────────────────────────
        "w" | "min-w" | "max-w" => match value {
            "screen" => Some("100vw".to_string()),
            "none" => Some("none".to_string()),
            _ => get_container_size(value).or_else(|| get_spacing_value(value)),
        },

        // ── Height ───────────────────────────────────────────────
        "h" | "min-h" | "max-h" => match value {
            "screen" => Some("100vh".to_string()),
            "none" => Some("none".to_string()),
            "lh" => Some("1lh".to_string()),
            _ => get_spacing_value(value),
        },

        // ── Size (width + height) ────────────────────────────────
        "size" => match value {
            "auto" => Some("auto".to_string()),
            _ => get_spacing_value(value),
        },

        // ── Position ─────────────────────────────────────────────
        "top" | "right" | "bottom" | "left" | "inset" | "inset-x" | "inset-y" => {
            get_spacing_value(value)
        }

        // ── Background color (fall through for non-color) ────────
        "bg" => get_color_value(value, color_mode)
            .or_else(|| get_spacing_value(value)),

        // ── Text color ───────────────────────────────────────────
        "text" => get_color_value(value, color_mode),

        // ── Gradient color stops ────────────────────────────────
        "from" | "via" | "to" => get_color_value(value, color_mode),

        // ── Border (color or width) ──────────────────────────────
        "border" => {
            if let Some(color) = get_color_value(value, color_mode) {
                Some(color)
            } else {
                get_spacing_value(value)
            }
        }

        // ── Color-only plugins ───────────────────────────────────
        "accent" | "caret" | "fill" => get_color_value(value, color_mode),

        // ── Opacity ──────────────────────────────────────────────
        "opacity" | "bg-opacity" | "text-opacity" | "border-opacity" => get_opacity_value(value),

        // ── Border sub-directions ────────────────────────────────
        "border-t" | "border-r" | "border-b" | "border-l" => get_spacing_value(value),

        // ── Border radius ────────────────────────────────────────
        "rounded" | "rounded-t" | "rounded-r" | "rounded-b" | "rounded-l" => match value {
            "none" => Some("0".to_string()),
            "sm" => Some("0.125rem".to_string()),
            "" => Some("0.25rem".to_string()),
            "md" => Some("0.375rem".to_string()),
            "lg" => Some("0.5rem".to_string()),
            "xl" => Some("0.75rem".to_string()),
            "2xl" => Some("1rem".to_string()),
            "3xl" => Some("1.5rem".to_string()),
            "full" => Some("9999px".to_string()),
            _ => None,
        },

        // ── Layout alignment ─────────────────────────────────────
        "justify" | "justify-items" | "justify-self" | "place-content" | "place-items"
        | "place-self" => {
            let is_justify = plugin == "justify";
            Some(
                match value {
                    "start" if is_justify => "flex-start",
                    "end" if is_justify => "flex-end",
                    "around" => "space-around",
                    "between" => "space-between",
                    "center-safe" => "safe center",
                    "end-safe" => {
                        if is_justify {
                            "safe flex-end"
                        } else {
                            "safe end"
                        }
                    }
                    "evenly" => "space-evenly",
                    _ => value,
                }
                .to_string(),
            )
        }

        // ── Items & Self alignment ───────────────────────────────
        "items" | "self" => Some(
            match value {
                "start" => "flex-start",
                "end" => "flex-end",
                "baseline-last" => "last baseline",
                "center-safe" => "safe center",
                "end-safe" => "safe flex-end",
                _ => value,
            }
            .to_string(),
        ),

        // ── Vertical align (passthrough: text-bottom, text-top 保持连字符) ─
        "align" => Some(value.to_string()),

        // ── Overflow (passthrough) ───────────────────────────────
        "overflow-x" | "overflow-y" => Some(value.to_string()),

        // ── Cursor (passthrough) ─────────────────────────────────
        "cursor" => Some(value.to_string()),

        // ── Touch action (passthrough) ───────────────────────────
        "touch" => Some(value.to_string()),

        // ── White space (passthrough) ────────────────────────────
        "whitespace" => Some(value.to_string()),

        // ── Hyphens (passthrough) ────────────────────────────────
        "hyphens" => Some(value.to_string()),

        // ── Appearance (passthrough) ─────────────────────────────
        "appearance" => Some(value.to_string()),

        // ── Float ────────────────────────────────────────────────
        "float" => Some(
            match value {
                "start" => "inline-start",
                "end" => "inline-end",
                _ => value,
            }
            .to_string(),
        ),

        // ── Clear ────────────────────────────────────────────────
        "clear" => Some(
            match value {
                "start" => "inline-start",
                "end" => "inline-end",
                _ => value,
            }
            .to_string(),
        ),

        // ── Backface visibility (passthrough) ────────────────────
        "backface" => Some(value.to_string()),

        // ── Scroll behavior (passthrough) ────────────────────────
        "scroll" => Some(value.to_string()),

        // ── Overscroll behavior (passthrough) ────────────────────
        "overscroll" | "overscroll-x" | "overscroll-y" => Some(value.to_string()),

        // ── Color scheme ─────────────────────────────────────────
        "scheme" => Some(
            match value {
                "light-dark" => "light dark",
                "only-dark" => "only dark",
                "only-light" => "only light",
                _ => value,
            }
            .to_string(),
        ),

        // ── Flex basis ───────────────────────────────────────────
        "basis" => match value {
            "auto" => Some("auto".to_string()),
            "full" => Some("100%".to_string()),
            _ => get_container_size(value).or_else(|| get_spacing_value(value)),
        },

        // ── Columns ──────────────────────────────────────────────
        "columns" => match value {
            "auto" => Some("auto".to_string()),
            _ => get_container_size(value).or_else(|| value.parse::<u32>().ok().map(|n| n.to_string())),
        },

        // ── Grid template ────────────────────────────────────────
        "grid-cols" | "grid-rows" => match value {
            "none" | "subgrid" => Some(value.to_string()),
            _ => value
                .parse::<u32>()
                .ok()
                .map(|n| format!("repeat({}, minmax(0, 1fr))", n)),
        },

        // ── Grid auto flow ───────────────────────────────────────
        "grid-flow" => Some(
            match value {
                "col" => "column",
                "col-dense" => "column dense",
                "row-dense" => "row dense",
                _ => value,
            }
            .to_string(),
        ),

        // ── Grid auto columns/rows ──────────────────────────────
        "auto-cols" | "auto-rows" => Some(
            match value {
                "auto" => "auto".to_string(),
                "min" => "min-content".to_string(),
                "max" => "max-content".to_string(),
                "fr" => "minmax(0, 1fr)".to_string(),
                _ => return None,
            },
        ),

        // ── Grid column/row ──────────────────────────────────────
        "col" | "row" => match value {
            "auto" => Some("auto".to_string()),
            _ => None,
        },

        // ── Grid span ────────────────────────────────────────────
        "col-span" | "row-span" => match value {
            "full" => Some("1 / -1".to_string()),
            _ => value
                .parse::<u32>()
                .ok()
                .map(|n| format!("span {} / span {}", n, n)),
        },

        // ── Grid start/end ───────────────────────────────────────
        "col-start" | "col-end" | "row-start" | "row-end" => match value {
            "auto" => Some("auto".to_string()),
            _ => value.parse::<i32>().ok().map(|n| n.to_string()),
        },

        // ── Transform origin ─────────────────────────────────────
        "origin" => Some(value.replace('-', " ")),

        // ── Table layout (passthrough) ───────────────────────────
        "table" => Some(value.to_string()),

        // ── Caption side (passthrough) ───────────────────────────
        "caption" => Some(value.to_string()),

        // ── Transition timing function ───────────────────────────
        "ease" => match value {
            "linear" | "initial" => Some(value.to_string()),
            _ => Some(format!("var(--ease-{})", value)),
        },

        // ── Will change ──────────────────────────────────────────
        "will" => Some(
            match value {
                "change-auto" => "auto",
                "change-contents" => "contents",
                "change-scroll" => "scroll-position",
                "change-transform" => "transform",
                _ => return None,
            }
            .to_string(),
        ),

        // ── Transition behavior ──────────────────────────────────
        "transition" => Some(
            match value {
                "discrete" => "allow-discrete",
                _ => value,
            }
            .to_string(),
        ),

        // ── Break ────────────────────────────────────────────────
        "break-before" | "break-after" | "break-inside" => Some(value.to_string()),

        // ── Overflow wrap (passthrough) ──────────────────────────
        "wrap" => Some(value.to_string()),

        // ── User select (passthrough) ────────────────────────────
        "select" => Some(value.to_string()),

        // ── Resize ───────────────────────────────────────────────
        "resize" => Some(
            match value {
                "x" => "horizontal",
                "y" => "vertical",
                _ => value,
            }
            .to_string(),
        ),

        // ── Flex named values (仅限 shorthand，其余回退到 VALUELESS_MAP) ──
        "flex" => match value {
            "auto" | "none" => Some(value.to_string()),
            "initial" => Some("0 auto".to_string()),
            _ => None,
        },

        // ── Z-index ──────────────────────────────────────────────
        "z" => match value {
            "auto" => Some("auto".to_string()),
            _ => value.parse::<i32>().ok().map(|_| value.to_string()),
        },

        // ── Order ────────────────────────────────────────────────
        "order" => Some(
            match value {
                "first" => "-9999",
                "last" => "9999",
                "none" => "0",
                _ => return value.parse::<i32>().ok().map(|_| value.to_string()),
            }
            .to_string(),
        ),

        // ── Line height ──────────────────────────────────────────
        "leading" => match value {
            "none" => Some("1".to_string()),
            _ => Some(format!("var(--leading-{})", value)),
        },

        // ── Letter spacing ───────────────────────────────────────
        "tracking" => Some(format!("var(--tracking-{})", value)),

        // ── Duration ─────────────────────────────────────────────
        "duration" => match value {
            "initial" => Some("initial".to_string()),
            _ => value.parse::<u32>().ok().map(|n| format!("{}ms", n)),
        },

        // ── Text indent ──────────────────────────────────────────
        "indent" => get_spacing_value(value),

        // ── Flex grow/shrink (passthrough numeric) ───────────────
        "grow" | "shrink" => Some(value.to_string()),

        // ── Rotate ───────────────────────────────────────────────
        "rotate" => match value {
            "none" => Some("none".to_string()),
            _ => value.parse::<f64>().ok().map(|_| format!("{}deg", value)),
        },

        // ── Perspective ──────────────────────────────────────────
        "perspective" => match value {
            "none" => Some("none".to_string()),
            _ if value.starts_with("origin-") => None, // handled in build_complex_standard
            _ => Some(format!("var(--perspective-{})", value)),
        },

        // ── Field sizing ─────────────────────────────────────────
        "field" => Some(
            match value {
                "sizing-content" => "content",
                "sizing-fixed" => "fixed",
                _ => return None,
            }
            .to_string(),
        ),

        // ── Forced color adjust ──────────────────────────────────
        "forced" => Some(
            match value {
                "color-adjust-auto" => "auto",
                "color-adjust-none" => "none",
                _ => return None,
            }
            .to_string(),
        ),

        // ── Box decoration break (passthrough) ───────────────────
        "box-decoration" => Some(value.to_string()),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_values() {
        assert_eq!(get_spacing_value("4"), Some("1rem".to_string()));
        assert_eq!(get_spacing_value("8"), Some("2rem".to_string()));
        assert_eq!(get_spacing_value("px"), Some("1px".to_string()));
        assert_eq!(get_spacing_value("auto"), Some("auto".to_string()));
    }

    #[test]
    fn test_spacing_computed() {
        // 半值
        assert_eq!(get_spacing_value("0.5"), Some("0.125rem".to_string()));
        assert_eq!(get_spacing_value("1.5"), Some("0.375rem".to_string()));
        // 零
        assert_eq!(get_spacing_value("0"), Some("0".to_string()));
        // 大值
        assert_eq!(get_spacing_value("96"), Some("24rem".to_string()));
        // 负值不接受
        assert_eq!(get_spacing_value("-1"), None);
    }

    #[test]
    fn test_color_values() {
        assert_eq!(
            get_color_value("black", ColorMode::Hex),
            Some("#000000".into())
        );
        assert_eq!(
            get_color_value("white", ColorMode::Hex),
            Some("#ffffff".into())
        );
        // blue-500 oklch(0.623 0.214 259.815) → 接近 #3b82f6
        assert!(get_color_value("blue-500", ColorMode::Hex).is_some());
        // 新增颜色族
        assert!(get_color_value("orange-500", ColorMode::Hex).is_some());
        assert!(get_color_value("violet-500", ColorMode::Hex).is_some());
        assert!(get_color_value("slate-950", ColorMode::Hex).is_some());
    }

    #[test]
    fn test_opacity_values() {
        assert_eq!(get_opacity_value("50"), Some("0.5".to_string()));
        assert_eq!(get_opacity_value("100"), Some("1".to_string()));
        assert_eq!(get_opacity_value("0"), Some("0".to_string()));
        assert_eq!(get_opacity_value("75"), Some("0.75".to_string()));
    }

    #[test]
    fn test_opacity_computed() {
        // 任意 0-100 整数
        assert_eq!(get_opacity_value("33"), Some("0.33".to_string()));
        assert_eq!(get_opacity_value("5"), Some("0.05".to_string()));
        // 越界
        assert_eq!(get_opacity_value("101"), None);
        // 非数字
        assert_eq!(get_opacity_value("abc"), None);
    }

    #[test]
    fn test_spacing_viewport_units() {
        assert_eq!(get_spacing_value("svh"), Some("100svh".to_string()));
        assert_eq!(get_spacing_value("dvh"), Some("100dvh".to_string()));
        assert_eq!(get_spacing_value("lvh"), Some("100lvh".to_string()));
        assert_eq!(get_spacing_value("svw"), Some("100svw".to_string()));
        assert_eq!(get_spacing_value("dvw"), Some("100dvw".to_string()));
        assert_eq!(get_spacing_value("vw"), Some("100vw".to_string()));
        assert_eq!(get_spacing_value("vh"), Some("100vh".to_string()));
        // vmin/vmax 不在支持列表中
        assert_eq!(get_spacing_value("vmin"), None);
    }

    #[test]
    fn test_infer_value() {
        assert_eq!(infer_value("p", "4", ColorMode::Hex), Some("1rem".to_string()));
        assert_eq!(infer_value("w", "full", ColorMode::Hex), Some("100%".to_string()));
        assert!(infer_value("bg", "blue-500", ColorMode::Hex).is_some());
        assert_eq!(infer_value("opacity", "50", ColorMode::Hex), Some("0.5".to_string()));
        // oklch 模式
        assert_eq!(
            infer_value("text", "blue-500", ColorMode::Oklch),
            Some("oklch(0.623 0.214 259.815)".into())
        );
        // var 模式
        assert_eq!(
            infer_value("text", "blue-500", ColorMode::Var),
            Some("var(--color-blue-500)".into())
        );
    }
}
