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

/// 颜色值映射
static COLOR_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    // 基础颜色
    "black" => "#000",
    "white" => "#fff",
    "transparent" => "transparent",
    "current" => "currentColor",

    // Gray
    "gray-50" => "#f9fafb",
    "gray-100" => "#f3f4f6",
    "gray-200" => "#e5e7eb",
    "gray-300" => "#d1d5db",
    "gray-400" => "#9ca3af",
    "gray-500" => "#6b7280",
    "gray-600" => "#4b5563",
    "gray-700" => "#374151",
    "gray-800" => "#1f2937",
    "gray-900" => "#111827",

    // Blue
    "blue-50" => "#eff6ff",
    "blue-100" => "#dbeafe",
    "blue-200" => "#bfdbfe",
    "blue-300" => "#93c5fd",
    "blue-400" => "#60a5fa",
    "blue-500" => "#3b82f6",
    "blue-600" => "#2563eb",
    "blue-700" => "#1d4ed8",
    "blue-800" => "#1e40af",
    "blue-900" => "#1e3a8a",

    // Red
    "red-50" => "#fef2f2",
    "red-100" => "#fee2e2",
    "red-200" => "#fecaca",
    "red-300" => "#fca5a5",
    "red-400" => "#f87171",
    "red-500" => "#ef4444",
    "red-600" => "#dc2626",
    "red-700" => "#b91c1c",
    "red-800" => "#991b1b",
    "red-900" => "#7f1d1d",

    // Green
    "green-50" => "#f0fdf4",
    "green-100" => "#dcfce7",
    "green-200" => "#bbf7d0",
    "green-300" => "#86efac",
    "green-400" => "#4ade80",
    "green-500" => "#22c55e",
    "green-600" => "#16a34a",
    "green-700" => "#15803d",
    "green-800" => "#166534",
    "green-900" => "#14532d",
};

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

/// 获取颜色值
pub fn get_color_value(key: &str) -> Option<&'static str> {
    COLOR_MAP.get(key).copied()
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

/// 根据插件类型推断值映射
pub fn infer_value(plugin: &str, value: &str) -> Option<String> {
    match plugin {
        // Spacing utilities
        "p" | "px" | "py" | "pt" | "pr" | "pb" | "pl" | "m" | "mx" | "my" | "mt" | "mr" | "mb"
        | "ml" | "gap" | "gap-x" | "gap-y" | "space-x" | "space-y" => get_spacing_value(value),

        // Width utilities
        "w" | "min-w" | "max-w" => {
            if value == "screen" {
                Some("100vw".to_string())
            } else {
                get_spacing_value(value)
            }
        }

        // Height utilities (screen = vh for height)
        "h" | "min-h" | "max-h" => {
            if value == "screen" {
                Some("100vh".to_string())
            } else {
                get_spacing_value(value)
            }
        }

        // Position utilities
        "top" | "right" | "bottom" | "left" | "inset" | "inset-x" | "inset-y" => {
            get_spacing_value(value)
        }

        // Background color
        "bg" => get_color_value(value)
            .map(|s| s.to_string())
            .or_else(|| get_spacing_value(value)),

        // Text color
        "text" => get_color_value(value).map(|s| s.to_string()),

        // Border (可能是颜色或宽度)
        "border" => {
            if let Some(color) = get_color_value(value) {
                Some(color.to_string())
            } else {
                get_spacing_value(value)
            }
        }

        // Opacity
        "opacity" | "bg-opacity" | "text-opacity" | "border-opacity" => get_opacity_value(value),

        // Border sub-directions
        "border-t" | "border-r" | "border-b" | "border-l" => get_spacing_value(value),

        // Border radius
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
                    "end-safe" => if is_justify { "safe flex-end" } else { "safe end" },
                    "evenly" => "space-evenly",
                    _ => value,
                }
                .to_string(),
            )
        }



        // Overflow (passthrough)
        "overflow-x" | "overflow-y" => Some(value.to_string()),

        // Object fit (passthrough)
        "object" => Some(value.to_string()),

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
        assert_eq!(get_color_value("black"), Some("#000"));
        assert_eq!(get_color_value("white"), Some("#fff"));
        assert_eq!(get_color_value("blue-500"), Some("#3b82f6"));
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
        assert_eq!(infer_value("p", "4"), Some("1rem".to_string()));
        assert_eq!(infer_value("w", "full"), Some("100%".to_string()));
        assert_eq!(infer_value("bg", "blue-500"), Some("#3b82f6".to_string()));
        assert_eq!(infer_value("opacity", "50"), Some("0.5".to_string()));
    }
}
