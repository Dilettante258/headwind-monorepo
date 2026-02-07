use headwind_core::Declaration;
use headwind_tw_parse::ParsedClass;
use phf::phf_map;

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

/// 为无值类构建声明
///
/// 例如：`flex` → `display: flex`
pub(super) fn build_valueless_declarations(parsed: &ParsedClass) -> Option<Vec<Declaration>> {
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
pub(super) fn build_valueless_from_full_name(parsed: &ParsedClass, value: &str) -> Option<Vec<Declaration>> {
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
