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
                css.push_str(&format!(".{} {{\n", self.class_name));
                for decl in decls {
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

            // 根据修饰符类型生成选择器
            self.generate_selector_with_modifiers(&mut css, &modifiers, decls, indent);
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

        // 分离响应式和其他修饰符
        let mut responsive_mods = Vec::new();
        let mut other_mods = Vec::new();

        for modifier in modifiers {
            if modifier.is_responsive() {
                responsive_mods.push(modifier);
            } else {
                other_mods.push(modifier);
            }
        }

        // 如果有响应式修饰符，生成 @media 查询
        if !responsive_mods.is_empty() {
            for responsive in responsive_mods {
                let breakpoint = self.get_breakpoint(responsive);
                css.push('\n');
                css.push_str(&format!("@media (min-width: {}) {{\n", breakpoint));

                // 基础选择器
                let mut selector = format!(".{}", self.class_name);

                // 添加其他修饰符（伪类、伪元素等）
                for modifier in &other_mods {
                    selector = self.apply_modifier(&selector, modifier);
                }

                css.push_str(&format!("{}{} {{\n", indent, selector));
                for decl in declarations {
                    css.push_str(&format!(
                        "{}{}{}: {};\n",
                        indent, indent, decl.property, decl.value
                    ));
                }
                css.push_str(&format!("{}}}\n", indent));
                css.push_str("}\n");
            }
        } else {
            // 没有响应式修饰符，直接生成选择器
            let mut selector = format!(".{}", self.class_name);

            for modifier in &other_mods {
                selector = self.apply_modifier(&selector, modifier);
            }

            css.push('\n');
            css.push_str(&format!("{} {{\n", selector));
            for decl in declarations {
                css.push_str(&format!("{}{}: {};\n", indent, decl.property, decl.value));
            }
            css.push_str("}\n");
        }
    }

    /// 应用单个修饰符到选择器
    fn apply_modifier(&self, selector: &str, modifier: &Modifier) -> String {
        match modifier {
            Modifier::PseudoClass(name) => {
                format!("{}:{}", selector, name)
            }
            Modifier::PseudoElement(name) => {
                format!("{}::{}", selector, name)
            }
            Modifier::State(name) => match name.as_str() {
                "dark" => format!(".dark {}", selector),
                name if name.starts_with("group-") => {
                    let pseudo = &name[6..];
                    format!(".group:{} {}", pseudo, selector)
                }
                name if name.starts_with("peer-") => {
                    let pseudo = &name[5..];
                    format!(".peer:{} ~ {}", pseudo, selector)
                }
                _ => selector.to_string(),
            },
            Modifier::Responsive(_) => {
                // 响应式修饰符在外层处理
                selector.to_string()
            }
            Modifier::Custom(name) => {
                format!("{}:{}", selector, name)
            }
        }
    }

    /// 获取响应式断点
    fn get_breakpoint(&self, modifier: &Modifier) -> &'static str {
        if let Modifier::Responsive(size) = modifier {
            match size.as_str() {
                "sm" => "640px",
                "md" => "768px",
                "lg" => "1024px",
                "xl" => "1280px",
                "2xl" => "1536px",
                _ => "0px",
            }
        } else {
            "0px"
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
    }
}
