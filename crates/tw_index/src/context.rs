use crate::variant::{
    self, parameterized_selector, pseudo_class_at_rule, pseudo_class_selector,
    pseudo_element_selector, responsive_at_rule, supports_at_rule, StateResolution,
};
use headwind_core::shorthand::optimize_shorthands;
use headwind_core::Declaration;
use headwind_tw_parse::{parse_modifiers_from_raw, Modifier};
use std::collections::HashMap;

/// CSS 类上下文 - 收集某个 CSS 类的所有声明
///
/// 按 raw_modifiers 分组，相同修饰符的声明会合并到一起
#[derive(Debug, Clone)]
pub struct ClassContext {
    /// 生成的 CSS 类名
    pub class_name: String,
    /// raw_modifiers -> declarations
    /// modifiers 在需要时从 raw_modifiers 解析
    groups: HashMap<String, Vec<Declaration>>,
}

impl ClassContext {
    pub fn new(class_name: String) -> Self {
        Self {
            class_name,
            groups: HashMap::new(),
        }
    }

    /// 写入声明到指定的修饰符组
    ///
    /// # 参数
    /// - `raw_modifiers`: 原始修饰符字符串（如 "md:hover:"）
    /// - `declarations`: CSS 声明列表
    ///
    /// modifiers 会在生成 CSS 时从 raw_modifiers 解析
    pub fn write(&mut self, raw_modifiers: &str, declarations: Vec<Declaration>) {
        self.groups
            .entry(raw_modifiers.to_string())
            .and_modify(|decls| decls.extend(declarations.clone()))
            .or_insert(declarations);
    }

    /// 生成 CSS 字符串
    pub fn to_css(&self, indent: &str) -> String {
        let mut css = String::new();

        // 1. 生成基础规则（无修饰符）
        if let Some(decls) = self.groups.get("") {
            if !decls.is_empty() {
                let decls = optimize_shorthands(decls.clone());
                css.push_str(&format!(".{} {{\n", self.class_name));
                for decl in &decls {
                    css.push_str(&format!("{}{}: {};\n", indent, decl.property, decl.value));
                }
                css.push_str("}\n");
            }
        }

        // 2. 生成带修饰符的规则
        let mut modifier_groups: Vec<_> = self
            .groups
            .iter()
            .filter(|(raw, _)| !raw.is_empty())
            .collect();

        // 按修饰符排序，保证输出稳定
        modifier_groups.sort_by_key(|(raw, _)| raw.as_str());

        for (raw_modifiers, decls) in modifier_groups {
            if decls.is_empty() {
                continue;
            }

            // 在需要时从 raw_modifiers 解析出 modifiers
            let modifiers = parse_modifiers_from_raw(raw_modifiers);

            // 简写属性优化
            let optimized = optimize_shorthands(decls.clone());

            // 根据修饰符类型生成选择器
            self.generate_selector_with_modifiers(&mut css, &modifiers, &optimized, indent);
        }

        css
    }

    /// 根据修饰符生成选择器
    fn generate_selector_with_modifiers(
        &self,
        css: &mut String,
        modifiers: &[Modifier],
        declarations: &[Declaration],
        indent: &str,
    ) {
        if modifiers.is_empty() {
            return;
        }

        // Collect at-rule wrappers and selector modifiers
        let mut at_rules: Vec<String> = Vec::new();
        let mut selector_mods: Vec<&Modifier> = Vec::new();

        for modifier in modifiers {
            match modifier {
                Modifier::Responsive(name) => {
                    // Container queries start with @
                    if let Some(container_name) = name.strip_prefix('@') {
                        if let Some(rule) = variant::container_at_rule(container_name) {
                            at_rules.push(rule);
                        }
                    } else if let Some(rule) = responsive_at_rule(name) {
                        at_rules.push(rule);
                    }
                }
                Modifier::PseudoClass(name) => {
                    // Some pseudo-classes need at-rule wrappers (hover → @media (hover: hover))
                    if let Some(at_rule) = pseudo_class_at_rule(name) {
                        at_rules.push(at_rule.to_string());
                    }
                    selector_mods.push(modifier);
                }
                Modifier::State(name) => {
                    // supports-[...] → @supports at-rule
                    if let Some(rule) = supports_at_rule(name) {
                        at_rules.push(rule);
                    } else if name == "starting" {
                        at_rules.push("@starting-style".to_string());
                    } else {
                        match variant::resolve_state(name, "") {
                            StateResolution::AtRule(rule) => at_rules.push(rule),
                            StateResolution::Selector(_) => selector_mods.push(modifier),
                        }
                    }
                }
                _ => selector_mods.push(modifier),
            }
        }

        // Build the selector
        let mut selector = format!(".{}", self.class_name);
        for modifier in &selector_mods {
            selector = self.apply_modifier(&selector, modifier);
        }

        if !at_rules.is_empty() {
            css.push('\n');
            // Open all at-rules with increasing indent (proper nesting)
            for (i, at_rule) in at_rules.iter().enumerate() {
                let prefix = indent.repeat(i);
                css.push_str(&format!("{}{} {{\n", prefix, at_rule));
            }
            let depth = at_rules.len();
            let sel_prefix = indent.repeat(depth);
            let decl_prefix = indent.repeat(depth + 1);

            css.push_str(&format!("{}{} {{\n", sel_prefix, selector));
            for decl in declarations {
                css.push_str(&format!(
                    "{}{}: {};\n",
                    decl_prefix, decl.property, decl.value
                ));
            }
            css.push_str(&format!("{}}}\n", sel_prefix));

            // Close at-rules in reverse order
            for i in (0..depth).rev() {
                let prefix = indent.repeat(i);
                css.push_str(&format!("{}}}\n", prefix));
            }
        } else {
            css.push('\n');
            css.push_str(&format!("{} {{\n", selector));
            for decl in declarations {
                css.push_str(&format!("{}{}: {};\n", indent, decl.property, decl.value));
            }
            css.push_str("}\n");
        }
    }

    /// Apply a single modifier to a selector, using the centralized variant resolver
    fn apply_modifier(&self, selector: &str, modifier: &Modifier) -> String {
        match modifier {
            Modifier::PseudoClass(name) => {
                // Parameterized pseudo-classes: has-[...], not-[...], aria-[...], data-[...], etc.
                if let Some(param_sel) = parameterized_selector(name) {
                    format!("{}{}", selector, param_sel)
                } else if name == "*" {
                    // Child selector: direct children
                    format!("{} > *", selector)
                } else if name == "**" {
                    // Descendant selector: all descendants
                    format!("{} *", selector)
                } else {
                    let css_pseudo = pseudo_class_selector(name);
                    format!("{}:{}", selector, css_pseudo)
                }
            }
            Modifier::PseudoElement(name) => {
                let css_pseudo = pseudo_element_selector(name);
                format!("{}::{}", selector, css_pseudo)
            }
            Modifier::State(name) => {
                match variant::resolve_state(name, selector) {
                    StateResolution::Selector(s) => s,
                    // AtRule states are handled in generate_selector_with_modifiers
                    StateResolution::AtRule(_) => selector.to_string(),
                }
            }
            Modifier::Responsive(_) => {
                // Handled at outer level
                selector.to_string()
            }
            Modifier::Custom(name) => {
                // Also check parameterized selector for custom modifiers
                if let Some(param_sel) = parameterized_selector(name) {
                    format!("{}{}", selector, param_sel)
                } else {
                    format!("{}:{}", selector, name)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_basic() {
        let mut ctx = ClassContext::new("my-class".to_string());

        // 写入基础声明（无修饰符）
        ctx.write("", vec![Declaration::new("padding", "1rem")]);

        let css = ctx.to_css("  ");
        assert!(css.contains(".my-class"));
        assert!(css.contains("padding: 1rem"));
    }

    #[test]
    fn test_context_with_modifiers() {
        let mut ctx = ClassContext::new("my-class".to_string());

        // 基础
        ctx.write("", vec![Declaration::new("padding", "1rem")]);

        // hover（modifiers 会从 "hover:" 自动解析）
        ctx.write("hover:", vec![Declaration::new("padding", "2rem")]);

        let css = ctx.to_css("  ");
        assert!(css.contains(".my-class {"));
        // hover is now wrapped in @media (hover: hover)
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(".my-class:hover {"));
    }

    #[test]
    fn test_context_merge_same_modifiers() {
        let mut ctx = ClassContext::new("my-class".to_string());

        // 两个 hover 类（相同的 raw_modifiers 会合并）
        ctx.write("hover:", vec![Declaration::new("padding", "1rem")]);

        ctx.write("hover:", vec![Declaration::new("margin", "0.5rem")]);

        let css = ctx.to_css("  ");

        // 应该只有一个 hover 选择器，包含两个声明
        assert_eq!(css.matches(".my-class:hover").count(), 1);
        assert!(css.contains("padding: 1rem"));
        assert!(css.contains("margin: 0.5rem"));
        // hover is wrapped in @media (hover: hover)
        assert!(css.contains("@media (hover: hover)"));
    }
}
