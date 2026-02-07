use headwind_tw_parse::{Modifier, ParsedClass};
use phf::phf_map;

/// 响应式断点映射
static BREAKPOINT_MAP: phf::Map<&'static str, &'static str> = phf_map! {
    "sm" => "640px",
    "md" => "768px",
    "lg" => "1024px",
    "xl" => "1280px",
    "2xl" => "1536px",
};

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
pub(super) fn build_selector(parsed: &ParsedClass) -> String {
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
