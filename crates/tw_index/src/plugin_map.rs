use std::collections::HashMap;
use std::sync::OnceLock;

/// 插件名到 CSS 属性的映射
///
/// 用于处理任意值，如 `w-[13px]` → `width: 13px`
pub fn get_plugin_property_map() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

    MAP.get_or_init(|| {
        let mut map = HashMap::new();

        // Spacing (间距)
        map.insert("p", "padding");
        map.insert("pt", "padding-top");
        map.insert("pr", "padding-right");
        map.insert("pb", "padding-bottom");
        map.insert("pl", "padding-left");
        map.insert("px", "padding-left"); // 会生成两个声明
        map.insert("py", "padding-top"); // 会生成两个声明
        map.insert("m", "margin");
        map.insert("mt", "margin-top");
        map.insert("mr", "margin-right");
        map.insert("mb", "margin-bottom");
        map.insert("ml", "margin-left");
        map.insert("mx", "margin-left"); // 会生成两个声明
        map.insert("my", "margin-top"); // 会生成两个声明

        // Sizing (尺寸)
        map.insert("w", "width");
        map.insert("h", "height");
        map.insert("min-w", "min-width");
        map.insert("min-h", "min-height");
        map.insert("max-w", "max-width");
        map.insert("max-h", "max-height");

        // Position (定位)
        map.insert("top", "top");
        map.insert("right", "right");
        map.insert("bottom", "bottom");
        map.insert("left", "left");
        map.insert("inset", "inset");
        map.insert("inset-x", "left"); // 会生成两个声明
        map.insert("inset-y", "top"); // 会生成两个声明

        // Typography (排版)
        map.insert("text", "color"); // text-[color] 或 font-size
        map.insert("font-size", "font-size");
        map.insert("leading", "line-height");
        map.insert("tracking", "letter-spacing");

        // Background (背景)
        map.insert("bg", "background");
        map.insert("bg-color", "background-color");

        // Border (边框)
        map.insert("border", "border-width");
        map.insert("border-t", "border-top-width");
        map.insert("border-r", "border-right-width");
        map.insert("border-b", "border-bottom-width");
        map.insert("border-l", "border-left-width");
        map.insert("rounded", "border-radius");
        map.insert("rounded-t", "border-top-left-radius"); // 会生成两个声明
        map.insert("rounded-r", "border-top-right-radius"); // 会生成两个声明
        map.insert("rounded-b", "border-bottom-right-radius"); // 会生成两个声明
        map.insert("rounded-l", "border-top-left-radius"); // 会生成两个声明

        // Flexbox & Grid
        map.insert("gap", "gap");
        map.insert("gap-x", "column-gap");
        map.insert("gap-y", "row-gap");
        map.insert("grid-cols", "grid-template-columns");
        map.insert("grid-rows", "grid-template-rows");
        map.insert("col-span", "grid-column");
        map.insert("row-span", "grid-row");

        // Effects (效果)
        map.insert("opacity", "opacity");
        map.insert("shadow", "box-shadow");

        // Transform (变换)
        map.insert("translate", "translate");
        map.insert("translate-x", "translate");
        map.insert("translate-y", "translate");
        map.insert("translate-z", "translate");
        map.insert("rotate", "rotate");
        map.insert("scale", "scale");
        map.insert("scale-x", "scale");
        map.insert("scale-y", "scale");

        // Filters (滤镜)
        map.insert("blur", "filter");
        map.insert("brightness", "filter");
        map.insert("contrast", "filter");
        map.insert("grayscale", "filter");

        // Transitions & Animation (过渡和动画)
        map.insert("duration", "transition-duration");
        map.insert("delay", "transition-delay");

        // Other (其他)
        map.insert("z", "z-index");

        map
    })
}

/// 检查插件是否需要生成多个 CSS 声明
///
/// 例如 `px` 需要同时设置 `padding-left` 和 `padding-right`
pub fn is_multi_declaration_plugin(plugin: &str) -> bool {
    matches!(
        plugin,
        "px" | "py" | "mx" | "my" | "inset-x" | "inset-y" | "rounded-t" | "rounded-r"
            | "rounded-b" | "rounded-l"
    )
}

/// 获取插件的所有 CSS 属性
///
/// 对于普通插件返回单个属性，对于 px/py/mx/my 等返回两个属性
pub fn get_plugin_properties(plugin: &str) -> Option<Vec<&'static str>> {
    let map = get_plugin_property_map();

    match plugin {
        "px" => Some(vec!["padding-left", "padding-right"]),
        "py" => Some(vec!["padding-top", "padding-bottom"]),
        "mx" => Some(vec!["margin-left", "margin-right"]),
        "my" => Some(vec!["margin-top", "margin-bottom"]),
        "inset-x" => Some(vec!["left", "right"]),
        "inset-y" => Some(vec!["top", "bottom"]),
        "rounded-t" => Some(vec!["border-top-left-radius", "border-top-right-radius"]),
        "rounded-r" => Some(vec!["border-top-right-radius", "border-bottom-right-radius"]),
        "rounded-b" => Some(vec!["border-bottom-right-radius", "border-bottom-left-radius"]),
        "rounded-l" => Some(vec!["border-top-left-radius", "border-bottom-left-radius"]),
        _ => map.get(plugin).map(|&prop| vec![prop]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_map() {
        let map = get_plugin_property_map();
        assert_eq!(map.get("w"), Some(&"width"));
        assert_eq!(map.get("h"), Some(&"height"));
        assert_eq!(map.get("p"), Some(&"padding"));
    }

    #[test]
    fn test_multi_declaration_plugins() {
        assert!(is_multi_declaration_plugin("px"));
        assert!(is_multi_declaration_plugin("py"));
        assert!(!is_multi_declaration_plugin("p"));
        assert!(!is_multi_declaration_plugin("w"));
    }

    #[test]
    fn test_get_plugin_properties() {
        let props = get_plugin_properties("w").unwrap();
        assert_eq!(props, vec!["width"]);

        let props = get_plugin_properties("px").unwrap();
        assert_eq!(props, vec!["padding-left", "padding-right"]);

        let props = get_plugin_properties("py").unwrap();
        assert_eq!(props, vec!["padding-top", "padding-bottom"]);

        assert!(get_plugin_properties("unknown").is_none());
    }
}
