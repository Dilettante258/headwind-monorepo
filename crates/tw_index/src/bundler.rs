use crate::context::ClassContext;
use crate::converter::Converter;
use crate::variant::{self, pseudo_class_selector, pseudo_element_selector, StateResolution};
use headwind_core::{ColorMode, Declaration};
use headwind_css::{create_stylesheet, emit_css};
use headwind_tw_parse::{parse_class, parse_classes, Modifier, ParsedClass};
use std::collections::{BTreeSet, HashMap};

/// CSS 规则组，按修饰符分组
#[derive(Debug, Clone)]
pub struct RuleGroup {
    /// 基础规则（无修饰符）
    pub base: Vec<Declaration>,
    /// 伪类规则（如 :hover, :focus）
    pub pseudo_classes: HashMap<String, Vec<Declaration>>,
    /// 伪元素规则（如 ::before, ::after）
    pub pseudo_elements: HashMap<String, Vec<Declaration>>,
    /// 响应式规则（如 @media）
    pub responsive: HashMap<String, Box<RuleGroup>>,
    /// 状态规则（如 .dark, .group-hover）
    pub states: HashMap<String, Box<RuleGroup>>,
}

impl RuleGroup {
    pub fn new() -> Self {
        Self {
            base: Vec::new(),
            pseudo_classes: HashMap::new(),
            pseudo_elements: HashMap::new(),
            responsive: HashMap::new(),
            states: HashMap::new(),
        }
    }

    /// 添加声明到对应的组
    fn add_declarations(&mut self, modifiers: &[Modifier], declarations: Vec<Declaration>) {
        if modifiers.is_empty() {
            // 无修饰符，添加到基础规则
            self.base.extend(declarations);
        } else {
            // 处理第一个修饰符
            match &modifiers[0] {
                Modifier::PseudoClass(name) => {
                    if modifiers.len() == 1 {
                        self.pseudo_classes
                            .entry(name.clone())
                            .or_insert_with(Vec::new)
                            .extend(declarations);
                    } else {
                        // 有更多修饰符，递归处理
                        self.pseudo_classes
                            .entry(name.clone())
                            .or_insert_with(Vec::new)
                            .extend(declarations);
                    }
                }
                Modifier::PseudoElement(name) => {
                    self.pseudo_elements
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .extend(declarations);
                }
                Modifier::Responsive(size) => {
                    let group = self
                        .responsive
                        .entry(size.clone())
                        .or_insert_with(|| Box::new(RuleGroup::new()));
                    group.add_declarations(&modifiers[1..], declarations);
                }
                Modifier::State(state) => {
                    let group = self
                        .states
                        .entry(state.clone())
                        .or_insert_with(|| Box::new(RuleGroup::new()));
                    group.add_declarations(&modifiers[1..], declarations);
                }
                Modifier::Custom(name) => {
                    self.pseudo_classes
                        .entry(name.clone())
                        .or_insert_with(Vec::new)
                        .extend(declarations);
                }
            }
        }
    }
}

impl Default for RuleGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Tailwind 类打包器
///
/// 将多个 Tailwind 类整理成一个 CSS 类，并按修饰符分组
pub struct Bundler {
    converter: Converter,
}

impl Bundler {
    pub fn new() -> Self {
        Self {
            converter: Converter::new(),
        }
    }

    /// 创建使用内联值的打包器（不依赖 Tailwind 主题变量）
    pub fn with_inline() -> Self {
        Self {
            converter: Converter::with_inline(),
        }
    }

    /// 设置颜色输出模式（builder 模式）
    pub fn with_color_mode(mut self, mode: ColorMode) -> Self {
        self.converter = self.converter.with_color_mode(mode);
        self
    }

    /// 设置是否使用 color-mix() 函数处理颜色透明度（builder 模式）
    pub fn with_color_mix(mut self, enabled: bool) -> Self {
        self.converter = self.converter.with_color_mix(enabled);
        self
    }

    /// 将多个 Tailwind 类打包成一个规则组
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use headwind_tw_index::Bundler;
    /// let bundler = Bundler::new();
    /// let classes = "text-center hover:text-left md:text-right p-4";
    /// let group = bundler.bundle(classes).unwrap();
    /// ```
    pub fn bundle(&self, classes: &str) -> Result<RuleGroup, String> {
        let mut group = RuleGroup::new();

        // 一次性解析所有类名（优化：批量解析）
        let parsed_classes = parse_classes(classes).map_err(|e| format!("解析失败: {:?}", e))?;

        // 转换每个解析后的类
        for parsed in parsed_classes {
            if let Some(rule) = self.converter.convert(&parsed) {
                group.add_declarations(&parsed.modifiers(), rule.declarations);
            }
        }

        Ok(group)
    }

    /// 将规则组生成为 CSS 字符串
    ///
    /// # 参数
    ///
    /// - `class_name`: 生成的 CSS 类名
    /// - `group`: 规则组
    /// - `indent`: 缩进字符串（默认为 "  "）
    pub fn generate_css(
        &self,
        class_name: &str,
        group: &RuleGroup,
        indent: &str,
    ) -> String {
        let mut css = String::new();

        // 生成基础规则
        if !group.base.is_empty() {
            css.push_str(&format!(".{} {{\n", class_name));
            for decl in &group.base {
                css.push_str(&format!("{}{}: {};\n", indent, decl.property, decl.value));
            }
            css.push_str("}\n");
        }

        // 生成伪类规则
        for (pseudo, decls) in &group.pseudo_classes {
            if !decls.is_empty() {
                // Build selector: check parameterized, child, or standard
                let selector = if let Some(param_sel) = variant::parameterized_selector(pseudo) {
                    format!(".{}{}", class_name, param_sel)
                } else if pseudo == "*" {
                    format!(".{} > *", class_name)
                } else if pseudo == "**" {
                    format!(".{} *", class_name)
                } else {
                    let css_pseudo = pseudo_class_selector(pseudo);
                    format!(".{}:{}", class_name, css_pseudo)
                };

                // Check if this pseudo-class needs an at-rule wrapper
                if let Some(at_rule) = variant::pseudo_class_at_rule(pseudo) {
                    css.push('\n');
                    css.push_str(&format!("{} {{\n", at_rule));
                    css.push_str(&format!("{}{} {{\n", indent, selector));
                    for decl in decls {
                        css.push_str(&format!(
                            "{}{}{}: {};\n",
                            indent, indent, decl.property, decl.value
                        ));
                    }
                    css.push_str(&format!("{}}}\n", indent));
                    css.push_str("}\n");
                } else {
                    css.push('\n');
                    css.push_str(&format!("{} {{\n", selector));
                    for decl in decls {
                        css.push_str(&format!("{}{}: {};\n", indent, decl.property, decl.value));
                    }
                    css.push_str("}\n");
                }
            }
        }

        // 生成伪元素规则
        for (pseudo, decls) in &group.pseudo_elements {
            if !decls.is_empty() {
                let css_pseudo = pseudo_element_selector(pseudo);
                if pseudo == "marker" {
                    // marker targets both the element and its children
                    for sel in variant::marker_selectors(&format!(".{}", class_name)) {
                        css.push('\n');
                        css.push_str(&format!("{} {{\n", sel));
                        for decl in decls {
                            css.push_str(&format!(
                                "{}{}: {};\n",
                                indent, decl.property, decl.value
                            ));
                        }
                        css.push_str("}\n");
                    }
                } else {
                    css.push('\n');
                    css.push_str(&format!(".{}::{} {{\n", class_name, css_pseudo));
                    for decl in decls {
                        css.push_str(&format!(
                            "{}{}: {};\n",
                            indent, decl.property, decl.value
                        ));
                    }
                    css.push_str("}\n");
                }
            }
        }

        // 生成响应式规则
        for (size, nested_group) in &group.responsive {
            // Use variant resolver for breakpoints (v4 rem-based syntax)
            let at_rule = if let Some(container_name) = size.strip_prefix('@') {
                variant::container_at_rule(container_name)
            } else {
                variant::responsive_at_rule(size)
            };

            let at_rule = match at_rule {
                Some(rule) => rule,
                None => continue,
            };

            css.push('\n');
            css.push_str(&format!("{} {{\n", at_rule));

            // 基础规则
            if !nested_group.base.is_empty() {
                css.push_str(&format!("{}.{} {{\n", indent, class_name));
                for decl in &nested_group.base {
                    css.push_str(&format!(
                        "{}{}{}: {};\n",
                        indent, indent, decl.property, decl.value
                    ));
                }
                css.push_str(&format!("{}}}\n", indent));
            }

            // 伪类
            for (pseudo, decls) in &nested_group.pseudo_classes {
                if !decls.is_empty() {
                    let selector = if let Some(param_sel) = variant::parameterized_selector(pseudo)
                    {
                        format!(".{}{}", class_name, param_sel)
                    } else {
                        let css_pseudo = pseudo_class_selector(pseudo);
                        format!(".{}:{}", class_name, css_pseudo)
                    };

                    // Hover at-rule wrapping inside responsive
                    if let Some(hover_rule) = variant::pseudo_class_at_rule(pseudo) {
                        css.push('\n');
                        css.push_str(&format!("{}{} {{\n", indent, hover_rule));
                        css.push_str(&format!("{}{}{} {{\n", indent, indent, selector));
                        for decl in decls {
                            css.push_str(&format!(
                                "{}{}{}{}: {};\n",
                                indent, indent, indent, decl.property, decl.value
                            ));
                        }
                        css.push_str(&format!("{}{}}}\n", indent, indent));
                        css.push_str(&format!("{}}}\n", indent));
                    } else {
                        css.push('\n');
                        css.push_str(&format!("{}{} {{\n", indent, selector));
                        for decl in decls {
                            css.push_str(&format!(
                                "{}{}{}: {};\n",
                                indent, indent, decl.property, decl.value
                            ));
                        }
                        css.push_str(&format!("{}}}\n", indent));
                    }
                }
            }

            css.push_str("}\n");
        }

        // 生成状态规则
        for (state, nested_group) in &group.states {
            if nested_group.base.is_empty() {
                continue;
            }

            let class_sel = format!(".{}", class_name);

            // Check for supports-[...] → @supports at-rule
            if let Some(at_rule) = variant::supports_at_rule(state) {
                css.push('\n');
                css.push_str(&format!("{} {{\n", at_rule));
                css.push_str(&format!("{}{} {{\n", indent, class_sel));
                for decl in &nested_group.base {
                    css.push_str(&format!(
                        "{}{}{}: {};\n",
                        indent, indent, decl.property, decl.value
                    ));
                }
                css.push_str(&format!("{}}}\n", indent));
                css.push_str("}\n");
            } else if state == "starting" {
                css.push('\n');
                css.push_str("@starting-style {\n");
                css.push_str(&format!("{}{} {{\n", indent, class_sel));
                for decl in &nested_group.base {
                    css.push_str(&format!(
                        "{}{}{}: {};\n",
                        indent, indent, decl.property, decl.value
                    ));
                }
                css.push_str(&format!("{}}}\n", indent));
                css.push_str("}\n");
            } else {
                match variant::resolve_state(state, &class_sel) {
                    StateResolution::Selector(selector) => {
                        css.push('\n');
                        css.push_str(&format!("{} {{\n", selector));
                        for decl in &nested_group.base {
                            css.push_str(&format!(
                                "{}{}: {};\n",
                                indent, decl.property, decl.value
                            ));
                        }
                        css.push_str("}\n");
                    }
                    StateResolution::AtRule(rule) => {
                        css.push('\n');
                        css.push_str(&format!("{} {{\n", rule));
                        css.push_str(&format!("{}{} {{\n", indent, class_sel));
                        for decl in &nested_group.base {
                            css.push_str(&format!(
                                "{}{}{}: {};\n",
                                indent, indent, decl.property, decl.value
                            ));
                        }
                        css.push_str(&format!("{}}}\n", indent));
                        css.push_str("}\n");
                    }
                }
            }
        }

        css
    }

    /// 使用 SWC 生成基础 CSS（仅基础规则，无修饰符）
    ///
    /// 这个方法使用 headwind-css crate 基于 SWC 生成 CSS
    /// 目前只支持基础规则，伪类和媒体查询仍使用字符串生成
    ///
    /// # 参数
    ///
    /// - `class_name`: CSS 类名
    /// - `group`: 规则组
    ///
    /// # 返回
    ///
    /// 使用 SWC 生成的 CSS 字符串（仅包含基础规则）
    pub fn generate_css_with_swc(
        &self,
        class_name: &str,
        group: &RuleGroup,
    ) -> Result<String, String> {
        if group.base.is_empty() {
            return Ok(String::new());
        }

        // 使用 SWC 生成基础规则
        let stylesheet = create_stylesheet(class_name.to_string(), group.base.clone());

        emit_css(&stylesheet).map_err(|e| format!("CSS 生成失败: {:?}", e))
    }

    /// 生成完整的 CSS（使用混合方式：SWC + 字符串）
    ///
    /// - 基础规则使用 SWC 生成
    /// - 其他规则（伪类、响应式等）使用字符串生成
    ///
    /// 这是一个过渡方案，未来可以完全迁移到 SWC
    pub fn generate_css_hybrid(
        &self,
        class_name: &str,
        group: &RuleGroup,
        indent: &str,
    ) -> Result<String, String> {
        let mut css = String::new();

        // 1. 使用 SWC 生成基础规则
        if !group.base.is_empty() {
            let base_css = self.generate_css_with_swc(class_name, group)?;
            css.push_str(&base_css);
        }

        // 2. 使用字符串生成其他规则（伪类、伪元素等）
        // 这部分保持不变，使用 generate_css 的逻辑

        // 伪类规则
        for (pseudo, decls) in &group.pseudo_classes {
            if !decls.is_empty() {
                let selector = if let Some(param_sel) = variant::parameterized_selector(pseudo) {
                    format!(".{}{}", class_name, param_sel)
                } else if pseudo == "*" {
                    format!(".{} > *", class_name)
                } else if pseudo == "**" {
                    format!(".{} *", class_name)
                } else {
                    let css_pseudo = pseudo_class_selector(pseudo);
                    format!(".{}:{}", class_name, css_pseudo)
                };

                if let Some(at_rule) = variant::pseudo_class_at_rule(pseudo) {
                    css.push('\n');
                    css.push_str(&format!("{} {{\n", at_rule));
                    css.push_str(&format!("{}{} {{\n", indent, selector));
                    for decl in decls {
                        css.push_str(&format!(
                            "{}{}{}: {};\n",
                            indent, indent, decl.property, decl.value
                        ));
                    }
                    css.push_str(&format!("{}}}\n", indent));
                    css.push_str("}\n");
                } else {
                    css.push('\n');
                    css.push_str(&format!("{} {{\n", selector));
                    for decl in decls {
                        css.push_str(&format!("{}{}: {};\n", indent, decl.property, decl.value));
                    }
                    css.push_str("}\n");
                }
            }
        }

        // 伪元素规则
        for (pseudo, decls) in &group.pseudo_elements {
            if !decls.is_empty() {
                let css_pseudo = pseudo_element_selector(pseudo);
                css.push('\n');
                css.push_str(&format!(".{}::{} {{\n", class_name, css_pseudo));
                for decl in decls {
                    css.push_str(&format!("{}{}: {};\n", indent, decl.property, decl.value));
                }
                css.push_str("}\n");
            }
        }

        // 响应式规则
        for (size, nested_group) in &group.responsive {
            let at_rule = if let Some(container_name) = size.strip_prefix('@') {
                variant::container_at_rule(container_name)
            } else {
                variant::responsive_at_rule(size)
            };

            let at_rule = match at_rule {
                Some(rule) => rule,
                None => continue,
            };

            css.push('\n');
            css.push_str(&format!("{} {{\n", at_rule));

            if !nested_group.base.is_empty() {
                css.push_str(&format!("{}.{} {{\n", indent, class_name));
                for decl in &nested_group.base {
                    css.push_str(&format!(
                        "{}{}{}: {};\n",
                        indent, indent, decl.property, decl.value
                    ));
                }
                css.push_str(&format!("{}}}\n", indent));
            }

            css.push_str("}\n");
        }

        Ok(css)
    }

    /// 使用 ClassContext 架构打包类（新架构）
    ///
    /// 这个方法使用更简洁的 ClassContext 架构：
    /// - 将 ParsedClass 按 raw_modifiers 分组（优化）
    /// - 每个 ParsedClass 作为一个"写操作"写入 context
    /// - Context 自动处理选择器生成和 CSS 输出
    ///
    /// # 参数
    ///
    /// - `class_name`: 生成的 CSS 类名
    /// - `classes`: 要打包的 Tailwind 类字符串
    ///
    /// # 返回
    ///
    /// 返回填充好的 ClassContext，可以调用其 to_css() 方法生成 CSS
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use headwind_tw_index::Bundler;
    /// let bundler = Bundler::new();
    /// let context = bundler.bundle_to_context("my-class", "p-4 hover:p-8 md:p-12").unwrap();
    /// let css = context.to_css("  ");
    /// println!("{}", css);
    /// ```
    pub fn bundle_to_context(
        &self,
        class_name: &str,
        classes: &str,
    ) -> Result<ClassContext, String> {
        let mut context = ClassContext::new(class_name.to_string());

        // 一次性解析所有类名
        let parsed_list =
            parse_classes(classes).map_err(|e| format!("解析失败: {:?}", e))?;

        // 按 raw_modifiers 分组（优化：相同修饰符的类会被合并处理）
        let mut grouped: HashMap<String, Vec<ParsedClass>> = HashMap::new();
        for parsed in parsed_list {
            grouped
                .entry(parsed.raw_modifiers.clone())
                .or_insert_with(Vec::new)
                .push(parsed);
        }

        // 处理每个分组：每个 ParsedClass 作为一个"写操作"
        for (raw_mods, classes) in grouped {
            for parsed in classes {
                // 转换为 CSS 声明
                if let Some(declarations) = self.converter.to_declarations(&parsed) {
                    // 写入 context（相同 raw_modifiers 的声明会自动合并）
                    // modifiers 会在生成 CSS 时从 raw_mods 自动解析
                    context.write(&raw_mods, declarations);
                }
            }
        }

        Ok(context)
    }

    /// 检查单个 Tailwind 类名是否可被识别并转换为 CSS
    pub fn is_recognized(&self, class: &str) -> bool {
        match parse_class(class) {
            Ok(parsed) => self.converter.to_declarations(&parsed).is_some(),
            Err(_) => false,
        }
    }

    /// 直接生成 CSS 字符串（使用 ClassContext 架构）
    ///
    /// 这是 bundle_to_context 的便捷版本，直接返回 CSS 字符串
    ///
    /// # 参数
    ///
    /// - `class_name`: 生成的 CSS 类名
    /// - `classes`: 要打包的 Tailwind 类字符串
    /// - `indent`: 缩进字符串（默认 "  "）
    ///
    /// # 示例
    ///
    /// ```no_run
    /// # use headwind_tw_index::Bundler;
    /// let bundler = Bundler::new();
    /// let css = bundler.bundle_to_css("my-class", "p-4 hover:p-8", "  ").unwrap();
    /// println!("{}", css);
    /// ```
    pub fn bundle_to_css(
        &self,
        class_name: &str,
        classes: &str,
        indent: &str,
    ) -> Result<String, String> {
        let context = self.bundle_to_context(class_name, classes)?;
        Ok(context.to_css(indent))
    }
}

// ---------------------------------------------------------------------------
// :root 主题变量生成
// ---------------------------------------------------------------------------

/// 从 CSS 中提取所有 var(--xxx) 引用的变量名
fn extract_var_references(css: &str) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    let mut search_from = 0;

    while let Some(pos) = css[search_from..].find("var(--") {
        let abs_start = search_from + pos + 4; // 指向 "--"
        if let Some(end) = css[abs_start..].find(')') {
            let var_name = &css[abs_start..abs_start + end]; // "--text-3xl"
            refs.insert(var_name.to_string());
            search_from = abs_start + end;
        } else {
            break;
        }
    }

    refs
}

/// 将已知主题变量名解析为内联值
fn resolve_theme_variable(var_name: &str) -> Option<String> {
    use crate::theme_values;

    // --text-{size}--line-height
    if let Some(size) = var_name.strip_prefix("--text-") {
        if let Some(lh_size) = size.strip_suffix("--line-height") {
            return theme_values::TEXT_LINE_HEIGHT.get(lh_size).map(|v| v.to_string());
        }
        return theme_values::TEXT_SIZE.get(size).map(|v| v.to_string());
    }

    // --font-{family}
    if let Some(family) = var_name.strip_prefix("--font-") {
        return theme_values::FONT_FAMILY.get(family).map(|v| v.to_string());
    }

    // --blur-{size}
    if let Some(size) = var_name.strip_prefix("--blur-") {
        return theme_values::BLUR_SIZE.get(size).map(|v| v.to_string());
    }

    // --aspect-video
    if var_name == "--aspect-video" {
        return Some("16 / 9".to_string());
    }

    None
}

impl Bundler {
    /// 从 CSS 中提取用到的主题变量引用，生成 :root 定义块。
    ///
    /// 只处理已知主题变量（--text-*, --font-*, --blur-*, --aspect-video），
    /// 内部 --tw-* 变量自动排除。
    pub fn generate_root_css(&self, css: &str) -> String {
        let var_refs = extract_var_references(css);

        let mut definitions: Vec<(String, String)> = Vec::new();
        for var_name in &var_refs {
            if let Some(value) = resolve_theme_variable(var_name) {
                definitions.push((var_name.clone(), value));
            }
        }

        if definitions.is_empty() {
            return String::new();
        }

        let mut root_css = ":root {\n".to_string();
        for (name, value) in &definitions {
            root_css.push_str(&format!("  {}: {};\n", name, value));
        }
        root_css.push('}');

        root_css
    }
}

impl Default for Bundler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_basic() {
        let bundler = Bundler::new();

        let group = bundler.bundle("text-center p-4").unwrap();

        assert_eq!(group.base.len(), 2);
        assert!(group.pseudo_classes.is_empty());
        assert!(group.responsive.is_empty());
    }

    #[test]
    fn test_bundle_with_hover() {
        let bundler = Bundler::new();

        let group = bundler.bundle("text-center hover:text-left").unwrap();

        assert_eq!(group.base.len(), 1);
        assert_eq!(group.pseudo_classes.len(), 1);
        assert!(group.pseudo_classes.contains_key("hover"));
    }

    #[test]
    fn test_bundle_with_responsive() {
        let bundler = Bundler::new();

        let group = bundler.bundle("text-center md:text-right").unwrap();

        assert_eq!(group.base.len(), 1);
        assert_eq!(group.responsive.len(), 1);
        assert!(group.responsive.contains_key("md"));
    }

    #[test]
    fn test_generate_css_basic() {
        let bundler = Bundler::new();

        let group = bundler.bundle("text-center p-4").unwrap();
        let css = bundler.generate_css("my-class", &group, "  ");

        assert!(css.contains(".my-class {"));
        assert!(css.contains("text-align: center;"));
        assert!(css.contains("padding: 1rem;"));
    }

    #[test]
    fn test_generate_css_with_hover() {
        let bundler = Bundler::new();

        let group = bundler.bundle("text-center hover:text-left").unwrap();
        let css = bundler.generate_css("my-class", &group, "  ");

        assert!(css.contains(".my-class {"));
        assert!(css.contains("text-align: center;"));
        // hover is now wrapped in @media (hover: hover)
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(".my-class:hover {"));
        assert!(css.contains("text-align: left;"));
    }

    #[test]
    fn test_generate_css_with_responsive() {
        let bundler = Bundler::new();

        let group = bundler.bundle("text-center md:text-right").unwrap();
        let css = bundler.generate_css("my-class", &group, "  ");

        assert!(css.contains(".my-class {"));
        assert!(css.contains("text-align: center;"));
        assert!(css.contains("@media (width >= 48rem)"));
        assert!(css.contains("text-align: right;"));
    }

    #[test]
    fn test_generate_css_with_swc() {
        let bundler = Bundler::new();

        let group = bundler.bundle("text-center p-4").unwrap();
        let css = bundler.generate_css_with_swc("my-class", &group).unwrap();

        println!("\nSWC generated CSS:\n{}", css);

        // 验证包含类名和声明
        assert!(css.contains("my-class"));
        assert!(css.contains("text-align"));
        assert!(css.contains("center"));
        assert!(css.contains("padding"));
        assert!(css.contains("1rem"));
    }

    #[test]
    fn test_generate_css_hybrid() {
        let bundler = Bundler::new();

        let group = bundler
            .bundle("text-center hover:text-left p-4")
            .unwrap();
        let css = bundler.generate_css_hybrid("my-class", &group, "  ").unwrap();

        println!("\nHybrid generated CSS:\n{}", css);

        // 验证基础规则（SWC 生成）
        assert!(css.contains("text-align"));
        assert!(css.contains("padding"));

        // 验证伪类规则（字符串生成, wrapped in @media (hover: hover))
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(":hover"));
    }

    #[test]
    fn test_complex_bundle() {
        let bundler = Bundler::new();

        let group = bundler
            .bundle("text-center hover:text-left md:text-right p-4 hover:p-8")
            .unwrap();
        let css = bundler.generate_css("my-class", &group, "  ");

        println!("\n{}", css);

        // 检查基础规则
        assert!(css.contains("text-align: center;"));
        assert!(css.contains("padding: 1rem;"));

        // 检查 hover 规则 (wrapped in @media (hover: hover))
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(".my-class:hover {"));
        assert!(css.contains("text-align: left;"));
        assert!(css.contains("padding: 2rem;"));

        // 检查响应式规则
        assert!(css.contains("@media (width >= 48rem)"));
        assert!(css.contains("text-align: right;"));
    }

    // === 新架构测试（ClassContext） ===

    #[test]
    fn test_bundle_to_context_basic() {
        let bundler = Bundler::new();

        let context = bundler
            .bundle_to_context("my-class", "text-center p-4")
            .unwrap();

        assert_eq!(context.class_name, "my-class");

        let css = context.to_css("  ");
        assert!(css.contains(".my-class {"));
        assert!(css.contains("text-align: center;"));
        assert!(css.contains("padding: 1rem;"));
    }

    #[test]
    fn test_bundle_to_context_with_hover() {
        let bundler = Bundler::new();

        let context = bundler
            .bundle_to_context("my-class", "text-center hover:text-left")
            .unwrap();

        let css = context.to_css("  ");

        println!("\nGenerated CSS:\n{}", css);

        assert!(css.contains(".my-class {"));
        assert!(css.contains("text-align: center;"));
        // hover is now wrapped in @media (hover: hover)
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(".my-class:hover {"));
        assert!(css.contains("text-align: left;"));
    }

    #[test]
    fn test_bundle_to_context_with_responsive() {
        let bundler = Bundler::new();

        let context = bundler
            .bundle_to_context("my-class", "p-4 md:p-8")
            .unwrap();

        let css = context.to_css("  ");

        println!("\nGenerated CSS:\n{}", css);

        assert!(css.contains(".my-class {"));
        assert!(css.contains("padding: 1rem;"));
        assert!(css.contains("@media (width >= 48rem)"));
        assert!(css.contains("padding: 2rem;"));
    }

    #[test]
    fn test_bundle_to_context_grouping_optimization() {
        let bundler = Bundler::new();

        // 这些类有相同的 raw_modifiers，应该被分组处理
        let context = bundler
            .bundle_to_context("my-class", "hover:p-4 hover:m-2 hover:text-center")
            .unwrap();

        let css = context.to_css("  ");

        println!("\nGenerated CSS:\n{}", css);

        // 应该生成一个 hover 规则包含所有三个声明
        assert!(css.contains(".my-class:hover {"));
        assert!(css.contains("padding: 1rem;"));
        assert!(css.contains("margin: 0.5rem;"));
        assert!(css.contains("text-align: center;"));
    }

    #[test]
    fn test_bundle_to_css_convenience() {
        let bundler = Bundler::new();

        let css = bundler
            .bundle_to_css("my-class", "p-4 hover:p-8", "  ")
            .unwrap();

        println!("\nGenerated CSS:\n{}", css);

        assert!(css.contains(".my-class {"));
        assert!(css.contains("padding: 1rem;"));
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(".my-class:hover {"));
        assert!(css.contains("padding: 2rem;"));
    }

    #[test]
    fn test_bundle_to_context_complex() {
        let bundler = Bundler::new();

        let context = bundler
            .bundle_to_context(
                "my-class",
                "text-center hover:text-left md:text-right p-4 hover:p-8 md:p-12",
            )
            .unwrap();

        let css = context.to_css("  ");

        println!("\nGenerated CSS:\n{}", css);

        // 基础规则
        assert!(css.contains("text-align: center;"));
        assert!(css.contains("padding: 1rem;"));

        // hover 规则 (wrapped in @media (hover: hover))
        assert!(css.contains("@media (hover: hover)"));
        assert!(css.contains(".my-class:hover {"));
        assert!(css.contains("text-align: left;"));
        assert!(css.contains("padding: 2rem;"));

        // 响应式规则
        assert!(css.contains("@media (width >= 48rem)"));
        assert!(css.contains("text-align: right;"));
        assert!(css.contains("padding: 3rem;"));
    }
}
