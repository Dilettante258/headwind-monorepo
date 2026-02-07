use headwind_core::Declaration;
use swc_css_ast::{
    ClassSelector, ComplexSelector, ComplexSelectorChildren, ComponentValue,
    CompoundSelector, Declaration as SwcDeclaration, DeclarationName, Dimension,
    Ident, Length, Number, QualifiedRule, QualifiedRulePrelude, Rule, SelectorList,
    SimpleBlock, SubclassSelector, Stylesheet, Token, TokenAndSpan,
};
use swc_common::DUMMY_SP;

/// 解析 CSS 值字符串为 ComponentValue
fn parse_css_value(value_str: &str) -> ComponentValue {
    // 简单的值解析器，支持常见 CSS 值类型
    let trimmed = value_str.trim();

    // 尝试解析维度值 (如 "1rem", "10px", "0.5em")
    if let Some((num_str, unit)) = parse_dimension(trimmed) {
        if let Ok(num_val) = num_str.parse::<f64>() {
            return ComponentValue::Dimension(Box::new(Dimension::Length(Length {
                span: DUMMY_SP,
                value: Number {
                    span: DUMMY_SP,
                    value: num_val,
                    raw: Some(num_str.into()),
                },
                unit: Ident {
                    span: DUMMY_SP,
                    value: unit.into(),
                    raw: None,
                },
            })));
        }
    }

    // 默认作为标识符处理
    ComponentValue::Ident(Box::new(Ident {
        span: DUMMY_SP,
        value: trimmed.into(),
        raw: None,
    }))
}

/// 尝试解析维度值，返回 (数字部分, 单位部分)
fn parse_dimension(s: &str) -> Option<(&str, &str)> {
    // 支持的单位
    const UNITS: &[&str] = &[
        "px", "rem", "em", "vh", "vw", "vmin", "vmax", "%", "pt", "pc", "in", "cm", "mm", "ch",
        "ex", "s", "ms", "deg", "rad", "turn", "grad",
    ];

    for unit in UNITS {
        if let Some(num_part) = s.strip_suffix(unit) {
            // 验证数字部分是否有效
            if !num_part.is_empty()
                && (num_part.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')
                    || num_part == "0")
            {
                return Some((num_part, unit));
            }
        }
    }

    None
}

/// 从 headwind Declaration 创建 SWC CSS Declaration
pub fn create_swc_declaration(decl: &Declaration) -> SwcDeclaration {
    SwcDeclaration {
        span: DUMMY_SP,
        name: DeclarationName::Ident(Ident {
            span: DUMMY_SP,
            value: decl.property.clone().into(),
            raw: None,
        }),
        value: vec![parse_css_value(&decl.value)],
        important: None,
    }
}

/// 创建类选择器
pub fn create_class_selector(class_name: &str) -> ComplexSelector {
    let class_selector = ClassSelector {
        span: DUMMY_SP,
        text: Ident {
            span: DUMMY_SP,
            value: class_name.into(),
            raw: None,
        },
    };

    let compound_selector = CompoundSelector {
        span: DUMMY_SP,
        nesting_selector: None,
        type_selector: None,
        subclass_selectors: vec![SubclassSelector::Class(class_selector)],
    };

    ComplexSelector {
        span: DUMMY_SP,
        children: vec![ComplexSelectorChildren::CompoundSelector(compound_selector)],
    }
}

/// 从类名和声明列表创建 CSS 规则
pub fn create_qualified_rule(
    class_name: String,
    declarations: Vec<Declaration>,
) -> QualifiedRule {
    // 创建选择器列表
    let selector_list = SelectorList {
        span: DUMMY_SP,
        children: vec![create_class_selector(&class_name)],
    };

    // 创建声明块
    let mut block_children = Vec::new();
    for decl in declarations {
        block_children.push(ComponentValue::Declaration(Box::new(
            create_swc_declaration(&decl),
        )));
    }

    let block = SimpleBlock {
        span: DUMMY_SP,
        name: TokenAndSpan {
            span: DUMMY_SP,
            token: Token::LBrace,
        },
        value: block_children,
    };

    QualifiedRule {
        span: DUMMY_SP,
        prelude: QualifiedRulePrelude::SelectorList(selector_list),
        block,
    }
}

/// 从类名和声明列表创建样式表
pub fn create_stylesheet(class_name: String, declarations: Vec<Declaration>) -> Stylesheet {
    let rule = create_qualified_rule(class_name, declarations);

    Stylesheet {
        span: DUMMY_SP,
        rules: vec![Rule::QualifiedRule(Box::new(rule))],
    }
}

/// 合并多个规则到一个样式表
pub fn merge_stylesheets(stylesheets: Vec<Stylesheet>) -> Stylesheet {
    let mut all_rules = Vec::new();

    for stylesheet in stylesheets {
        all_rules.extend(stylesheet.rules);
    }

    Stylesheet {
        span: DUMMY_SP,
        rules: all_rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_swc_declaration() {
        let decl = Declaration::new("padding", "1rem");
        let swc_decl = create_swc_declaration(&decl);

        match &swc_decl.name {
            DeclarationName::Ident(ident) => {
                assert_eq!(ident.value.as_ref(), "padding");
            }
            _ => panic!("Expected Ident"),
        }
    }

    #[test]
    fn test_create_stylesheet() {
        let decls = vec![
            Declaration::new("padding", "1rem"),
            Declaration::new("margin", "0.5rem"),
        ];

        let stylesheet = create_stylesheet("test-class".to_string(), decls);
        assert_eq!(stylesheet.rules.len(), 1);
    }

    #[test]
    fn test_merge_stylesheets() {
        let stylesheet1 = create_stylesheet(
            "class1".to_string(),
            vec![Declaration::new("padding", "1rem")],
        );
        let stylesheet2 = create_stylesheet(
            "class2".to_string(),
            vec![Declaration::new("margin", "0.5rem")],
        );

        let merged = merge_stylesheets(vec![stylesheet1, stylesheet2]);
        assert_eq!(merged.rules.len(), 2);
    }
}
