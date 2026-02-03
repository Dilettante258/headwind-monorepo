use phf::phf_map;

/// 插件名到单个 CSS 属性的映射
///
/// 用于处理任意值，如 `w-[13px]` → `width: 13px`
/// 使用 phf 在编译期生成完美哈希表，零运行时开销
static PLUGIN_PROPERTY_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    // Spacing (间距)
    "p" => "padding",
    "pt" => "padding-top",
    "pr" => "padding-right",
    "pb" => "padding-bottom",
    "pl" => "padding-left",
    "m" => "margin",
    "mt" => "margin-top",
    "mr" => "margin-right",
    "mb" => "margin-bottom",
    "ml" => "margin-left",

    // Sizing (尺寸)
    "w" => "width",
    "h" => "height",
    "min-w" => "min-width",
    "min-h" => "min-height",
    "max-w" => "max-width",
    "max-h" => "max-height",

    // Position (定位)
    "top" => "top",
    "right" => "right",
    "bottom" => "bottom",
    "left" => "left",
    "inset" => "inset",

    // Typography (排版)
    // 注意：text 不在此 map 中，因为它是语义重载的（color / font-size / text-align），
    // 由 converter 根据值类型做分发
    "font-size" => "font-size",
    "leading" => "line-height",
    "tracking" => "letter-spacing",

    // Background (背景)
    "bg" => "background",
    "bg-color" => "background-color",

    // Gradient color stops (渐变色)
    "from" => "--tw-gradient-from",
    "via" => "--tw-gradient-via",
    "to" => "--tw-gradient-to",

    // Border (边框)
    "border" => "border-width",
    "border-t" => "border-top-width",
    "border-r" => "border-right-width",
    "border-b" => "border-bottom-width",
    "border-l" => "border-left-width",
    "rounded" => "border-radius",

    // Flexbox & Grid
    "gap" => "gap",
    "gap-x" => "column-gap",
    "gap-y" => "row-gap",
    "grid-cols" => "grid-template-columns",
    "grid-rows" => "grid-template-rows",
    "col-span" => "grid-column",
    "row-span" => "grid-row",

    // Layout alignment (compound plugins)
    "justify" => "justify-content",
    "justify-items" => "justify-items",
    "justify-self" => "justify-self",
    "place-content" => "place-content",
    "place-items" => "place-items",
    "place-self" => "place-self",
    "align-content" => "align-content",
    "align-self" => "align-self",

    // Overflow (compound)
    "overflow-x" => "overflow-x",
    "overflow-y" => "overflow-y",

    // Object
    "object" => "object-fit",

    // Effects (效果)
    "opacity" => "opacity",
    "shadow" => "box-shadow",

    // Transform (变换)
    "translate" => "translate",
    "translate-x" => "translate",
    "translate-y" => "translate",
    "translate-z" => "translate",
    "rotate" => "rotate",
    "scale" => "scale",
    "scale-x" => "scale",
    "scale-y" => "scale",

    // Filters (滤镜)
    "blur" => "filter",
    "brightness" => "filter",
    "contrast" => "filter",
    "grayscale" => "filter",

    // Transitions & Animation (过渡和动画)
    "duration" => "transition-duration",
    "delay" => "transition-delay",

    // Typography extras
    "align" => "vertical-align",
    "indent" => "text-indent",
    "whitespace" => "white-space",
    "hyphens" => "hyphens",

    // Layout
    "float" => "float",
    "clear" => "clear",
    "columns" => "columns",
    "basis" => "flex-basis",

    // Color (颜色)
    "accent" => "accent-color",
    "caret" => "caret-color",
    "fill" => "fill",
    "stroke" => "stroke",

    // Appearance & Interaction
    "appearance" => "appearance",
    "touch" => "touch-action",
    "backface" => "backface-visibility",

    // Scroll & Overscroll
    "scroll" => "scroll-behavior",
    "overscroll" => "overscroll-behavior",
    "overscroll-x" => "overscroll-behavior-x",
    "overscroll-y" => "overscroll-behavior-y",

    // Theme
    "scheme" => "color-scheme",

    // Grid extras
    "auto-cols" => "grid-auto-columns",
    "auto-rows" => "grid-auto-rows",
    "grid-flow" => "grid-auto-flow",
    "col" => "grid-column",
    "col-start" => "grid-column-start",
    "col-end" => "grid-column-end",
    "row" => "grid-row",
    "row-start" => "grid-row-start",
    "row-end" => "grid-row-end",

    // Transform extras
    "origin" => "transform-origin",
    "perspective" => "perspective",
    "box-decoration" => "box-decoration-break",

    // Break
    "break-before" => "break-before",
    "break-after" => "break-after",
    "break-inside" => "break-inside",

    // Table
    "table" => "table-layout",
    "caption" => "caption-side",

    // Transitions extras
    "ease" => "transition-timing-function",
    "will" => "will-change",
    "transition" => "transition-behavior",

    // Other (其他)
    "z" => "z-index",
    "content" => "content",
    "aspect" => "aspect-ratio",
    "flex" => "flex",
    "grow" => "flex-grow",
    "shrink" => "flex-shrink",
    "transform" => "transform",
    "ring" => "box-shadow",
    "ring-offset" => "box-shadow",
    "order" => "order",
    "cursor" => "cursor",
    "pointer-events" => "pointer-events",
    "resize" => "resize",
    "select" => "user-select",
    "items" => "align-items",
    "self" => "align-self",
    "wrap" => "overflow-wrap",
    "field" => "field-sizing",
    "forced" => "forced-color-adjust",
};

/// 需要生成两个 CSS 声明的插件映射
///
/// 例如 `px-4` 需要同时设置 `padding-left` 和 `padding-right`
static MULTI_PROPERTY_MAP: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    "px" => ("padding-left", "padding-right"),
    "py" => ("padding-top", "padding-bottom"),
    "mx" => ("margin-left", "margin-right"),
    "my" => ("margin-top", "margin-bottom"),
    "inset-x" => ("left", "right"),
    "inset-y" => ("top", "bottom"),
    "rounded-t" => ("border-top-left-radius", "border-top-right-radius"),
    "rounded-r" => ("border-top-right-radius", "border-bottom-right-radius"),
    "rounded-b" => ("border-bottom-right-radius", "border-bottom-left-radius"),
    "rounded-l" => ("border-top-left-radius", "border-bottom-left-radius"),
    "size" => ("width", "height"),
};

/// 获取插件属性映射的引用
pub fn get_plugin_property_map() -> &'static phf::Map<&'static str, &'static str> {
    &PLUGIN_PROPERTY_MAP
}

/// 检查插件是否需要生成多个 CSS 声明
pub fn is_multi_declaration_plugin(plugin: &str) -> bool {
    MULTI_PROPERTY_MAP.contains_key(plugin)
}

/// 获取插件的所有 CSS 属性
///
/// 对于普通插件返回单个属性，对于 px/py/mx/my 等返回两个属性
pub fn get_plugin_properties(plugin: &str) -> Option<Vec<&'static str>> {
    if let Some(&(a, b)) = MULTI_PROPERTY_MAP.get(plugin) {
        Some(vec![a, b])
    } else {
        PLUGIN_PROPERTY_MAP.get(plugin).map(|&prop| vec![prop])
    }
}

/// 检查是否为已知插件（单属性或多属性）
pub fn is_known_plugin(plugin: &str) -> bool {
    PLUGIN_PROPERTY_MAP.contains_key(plugin) || MULTI_PROPERTY_MAP.contains_key(plugin)
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
