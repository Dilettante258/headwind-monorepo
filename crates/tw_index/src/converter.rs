use crate::index::TailwindIndex;
use crate::plugin_map::get_plugin_properties;
use headwind_core::Declaration;
use headwind_tw_parse::{Modifier, ParsedClass, ParsedValue};

/// CSS 规则，包含选择器和声明
#[derive(Debug, Clone, PartialEq)]
pub struct CssRule {
    /// 选择器（如 ".my-class:hover" 或 "@media (min-width: 640px)"）
    pub selector: String,
    /// CSS 声明列表
    pub declarations: Vec<Declaration>,
}

/// 将解析后的 Tailwind 类转换为 CSS 规则
pub struct Converter<'a> {
    index: &'a TailwindIndex,
}

impl<'a> Converter<'a> {
    pub fn new(index: &'a TailwindIndex) -> Self {
        Self { index }
    }

    /// 将 Tailwind 类名转换为 CSS 规则
    ///
    /// 如果类名不存在于索引中且无法生成任意值 CSS，返回 None
    pub fn convert(&self, parsed: &ParsedClass) -> Option<CssRule> {
        // 先尝试从索引查找
        let base_class = self.build_base_class(parsed);
        let declarations = if let Some(base_declarations) = self.index.lookup(&base_class) {
            // 找到了，直接使用
            base_declarations.to_vec()
        } else if let Some(value) = &parsed.value {
            // 没找到，尝试处理任意值
            if matches!(value, ParsedValue::Arbitrary(_)) {
                self.build_arbitrary_declarations(parsed)?
            } else {
                // 不是任意值，且索引中没有，返回 None
                return None;
            }
        } else {
            // 没有值，且索引中没有，返回 None
            return None;
        };

        // 应用修饰符
        let selector = self.build_selector(parsed);
        let mut declarations = declarations;

        // 如果有 !important 标记，给所有声明添加
        if parsed.important {
            for decl in &mut declarations {
                if !decl.value.ends_with("!important") {
                    decl.value = format!("{} !important", decl.value);
                }
            }
        }

        Some(CssRule {
            selector,
            declarations,
        })
    }

    /// 为任意值构建 CSS 声明
    ///
    /// 例如：`w-[13px]` → `Declaration { property: "width", value: "13px" }`
    fn build_arbitrary_declarations(&self, parsed: &ParsedClass) -> Option<Vec<Declaration>> {
        let ParsedValue::Arbitrary(arbitrary_value) = parsed.value.as_ref()? else {
            return None;
        };

        // 获取插件对应的 CSS 属性
        let properties = get_plugin_properties(&parsed.plugin)?;

        // 为每个属性生成声明
        let declarations = properties
            .into_iter()
            .map(|property| Declaration::new(property, arbitrary_value.content.clone()))
            .collect();

        Some(declarations)
    }

    /// 构建基础类名（不包含修饰符和 !important）
    fn build_base_class(&self, parsed: &ParsedClass) -> String {
        let mut class = String::new();

        // 负号
        if parsed.negative {
            class.push('-');
        }

        // 插件名
        class.push_str(&parsed.plugin);

        // 值
        if let Some(value) = &parsed.value {
            class.push('-');
            class.push_str(&value.to_string());
        }

        // Alpha 值
        if let Some(alpha) = &parsed.alpha {
            class.push('/');
            class.push_str(alpha);
        }

        class
    }

    /// 构建 CSS 选择器，包含修饰符
    fn build_selector(&self, parsed: &ParsedClass) -> String {
        let class_name = self.build_base_class(parsed);

        // 如果没有修饰符，返回简单的类选择器
        if parsed.modifiers.is_empty() {
            return format!(".{}", class_name);
        }

        // 否则，构建带修饰符的选择器
        let mut selector = format!(".{}", class_name);

        for modifier in &parsed.modifiers {
            selector = self.apply_modifier(&selector, modifier);
        }

        selector
    }

    /// 应用单个修饰符到选择器
    fn apply_modifier(&self, selector: &str, modifier: &Modifier) -> String {
        match modifier {
            Modifier::PseudoClass(name) => {
                // 伪类：hover, focus, active 等
                format!("{}:{}", selector, name)
            }
            Modifier::PseudoElement(name) => {
                // 伪元素：before, after 等
                format!("{}::{}", selector, name)
            }
            Modifier::State(name) => {
                // 状态修饰符：dark, group-hover 等
                match name.as_str() {
                    "dark" => format!(".dark {}", selector),
                    name if name.starts_with("group-") => {
                        let pseudo = &name[6..]; // 移除 "group-" 前缀
                        format!(".group:{} {}", pseudo, selector)
                    }
                    name if name.starts_with("peer-") => {
                        let pseudo = &name[5..]; // 移除 "peer-" 前缀
                        format!(".peer:{} ~ {}", pseudo, selector)
                    }
                    _ => selector.to_string(),
                }
            }
            Modifier::Responsive(size) => {
                // 响应式修饰符：sm, md, lg 等
                let breakpoint = match size.as_str() {
                    "sm" => "640px",
                    "md" => "768px",
                    "lg" => "1024px",
                    "xl" => "1280px",
                    "2xl" => "1536px",
                    _ => "0px",
                };
                format!("@media (min-width: {}) {{ {} }}", breakpoint, selector)
            }
            Modifier::Custom(name) => {
                // 自定义修饰符，暂时当作伪类处理
                format!("{}:{}", selector, name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwind_tw_parse::parse_class;

    fn create_test_index() -> TailwindIndex {
        let mut index = TailwindIndex::new();
        index.insert(
            "p-4".to_string(),
            vec![Declaration::new("padding", "1rem")],
        );
        index.insert(
            "text-center".to_string(),
            vec![Declaration::new("text-align", "center")],
        );
        index.insert(
            "bg-blue-500".to_string(),
            vec![Declaration::new("background-color", "rgb(59 130 246)")],
        );
        index
    }

    #[test]
    fn test_convert_simple_class() {
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "padding");
        assert_eq!(rule.declarations[0].value, "1rem");
    }

    #[test]
    fn test_convert_with_pseudo_class() {
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("hover:p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4:hover");
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn test_convert_with_responsive() {
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("md:text-center").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert!(rule.selector.contains("@media"));
        assert!(rule.selector.contains("768px"));
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn test_convert_with_important() {
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("p-4!").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4");
        assert!(rule.declarations[0].value.contains("!important"));
    }

    #[test]
    fn test_convert_multiple_modifiers() {
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("md:hover:p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert!(rule.selector.contains("@media"));
        assert!(rule.selector.contains(":hover"));
    }

    #[test]
    fn test_convert_unknown_class() {
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("unknown-class").unwrap();
        let rule = converter.convert(&parsed);

        assert!(rule.is_none());
    }

    #[test]
    fn test_convert_arbitrary_value() {
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("w-[13px]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".w-[13px]");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "width");
        assert_eq!(rule.declarations[0].value, "13px");
    }

    #[test]
    fn test_convert_arbitrary_value_with_modifier() {
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("hover:w-[13px]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".w-[13px]:hover");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "width");
        assert_eq!(rule.declarations[0].value, "13px");
    }

    #[test]
    fn test_convert_arbitrary_value_multi_property() {
        let index = create_test_index();
        let converter = Converter::new(&index);

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
        let index = create_test_index();
        let converter = Converter::new(&index);

        let parsed = parse_class("text-[#1da1f2]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".text-[#1da1f2]");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "color");
        assert_eq!(rule.declarations[0].value, "#1da1f2");
    }
}
