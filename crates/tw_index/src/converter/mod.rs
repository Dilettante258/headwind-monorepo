use headwind_core::ColorMode;
use headwind_core::Declaration;
use headwind_tw_parse::{ParsedClass, ParsedValue};

mod arbitrary;
mod color;
mod selector;
mod standard;
mod valueless;

use arbitrary::{build_arbitrary_declarations, build_css_variable_declarations};
use color::{apply_alpha_to_declarations, apply_important};
use selector::build_selector;
use valueless::{build_valueless_declarations, build_valueless_from_full_name};

/// CSS 规则，包含选择器和声明
#[derive(Debug, Clone, PartialEq)]
pub struct CssRule {
    /// 选择器（如 ".my-class:hover" 或 "@media (min-width: 640px)"）
    pub selector: String,
    /// CSS 声明列表
    pub declarations: Vec<Declaration>,
}

/// 基于规则的 Tailwind 类转换器
///
/// 基于 plugin_map 和 value_map 进行转换，不依赖外部索引
pub struct Converter {
    /// true = 使用 var(--text-3xl)，false = 内联为 1.875rem
    pub(crate) use_variables: bool,
    /// 颜色输出模式（hex / oklch / hsl / var）
    pub(crate) color_mode: ColorMode,
    /// 是否使用 color-mix() 函数处理颜色透明度
    pub(crate) use_color_mix: bool,
}

impl Converter {
    pub fn new() -> Self {
        Self {
            use_variables: true,
            color_mode: ColorMode::default(),
            use_color_mix: false,
        }
    }

    /// 创建使用内联值的转换器（不依赖 Tailwind 主题变量）
    pub fn with_inline() -> Self {
        Self {
            use_variables: false,
            color_mode: ColorMode::default(),
            use_color_mix: false,
        }
    }

    /// 设置颜色输出模式（builder 模式）
    pub fn with_color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// 设置是否使用 color-mix() 函数处理颜色透明度（builder 模式）
    pub fn with_color_mix(mut self, enabled: bool) -> Self {
        self.use_color_mix = enabled;
        self
    }

    /// 将 Tailwind 类转换为 CSS 声明（仅声明，不含选择器）
    ///
    /// 适用于上下文模式，由调用者决定如何组织选择器。
    /// 复合插件（如 justify-items、gap-x）由解析器负责识别，
    /// 此处仅处理声明构建和无值类回退。
    pub fn to_declarations(&self, parsed: &ParsedClass) -> Option<Vec<Declaration>> {
        let declarations = match &parsed.value {
            Some(ParsedValue::Arbitrary(arb)) => {
                build_arbitrary_declarations(parsed, &arb.content)?
            }
            Some(ParsedValue::CssVariable(cv)) => {
                build_css_variable_declarations(parsed, cv)?
            }
            Some(ParsedValue::Standard(value)) => self
                .build_standard_declarations(parsed, value)
                .or_else(|| build_valueless_from_full_name(parsed, value))?,
            None => build_valueless_declarations(parsed)?,
        };

        // 为颜色属性应用 alpha 透明度（如 text-white/60 → color: #fff9）
        let declarations = if let Some(ref alpha) = parsed.alpha {
            apply_alpha_to_declarations(declarations, alpha, self.use_color_mix)
        } else {
            declarations
        };

        Some(apply_important(declarations, parsed.important))
    }

    /// 将 Tailwind 类名转换为 CSS 规则（声明 + 选择器）
    pub fn convert(&self, parsed: &ParsedClass) -> Option<CssRule> {
        let declarations = self.to_declarations(parsed)?;
        let selector = build_selector(parsed);
        Some(CssRule { selector, declarations })
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwind_tw_parse::parse_class;

    #[test]
    fn test_convert_standard_value() {
        let converter = Converter::new();

        let parsed = parse_class("p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "padding");
        assert_eq!(rule.declarations[0].value, "1rem");
    }

    #[test]
    fn test_convert_valueless_class() {
        let converter = Converter::new();

        let parsed = parse_class("flex").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".flex");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "display");
        assert_eq!(rule.declarations[0].value, "flex");
    }

    #[test]
    fn test_convert_with_pseudo_class() {
        let converter = Converter::new();

        let parsed = parse_class("hover:p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4:hover");
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn test_convert_with_responsive() {
        let converter = Converter::new();

        let parsed = parse_class("md:text-center").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert!(rule.selector.contains("@media"));
        assert!(rule.selector.contains("768px"));
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn test_convert_with_important() {
        let converter = Converter::new();

        let parsed = parse_class("p-4!").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".p-4");
        assert!(rule.declarations[0].value.contains("!important"));
    }

    #[test]
    fn test_convert_multiple_modifiers() {
        let converter = Converter::new();

        let parsed = parse_class("md:hover:p-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert!(rule.selector.contains("@media"));
        assert!(rule.selector.contains(":hover"));
    }

    #[test]
    fn test_convert_arbitrary_value() {
        let converter = Converter::new();

        let parsed = parse_class("w-[13px]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".w-[13px]");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "width");
        assert_eq!(rule.declarations[0].value, "13px");
    }

    #[test]
    fn test_convert_arbitrary_value_with_modifier() {
        let converter = Converter::new();

        let parsed = parse_class("hover:w-[13px]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".w-[13px]:hover");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "width");
        assert_eq!(rule.declarations[0].value, "13px");
    }

    #[test]
    fn test_convert_arbitrary_value_multi_property() {
        let converter = Converter::new();

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
        let converter = Converter::new();

        let parsed = parse_class("text-[#1da1f2]").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".text-[#1da1f2]");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "color");
        assert_eq!(rule.declarations[0].value, "#1da1f2");
    }

    #[test]
    fn test_convert_color_value() {
        let converter = Converter::new();

        let parsed = parse_class("bg-blue-500").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".bg-blue-500");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "background");
        assert!(rule.declarations[0].value.starts_with('#'));
    }

    #[test]
    fn test_convert_negative_value() {
        let converter = Converter::new();

        let parsed = parse_class("-m-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.selector, ".-m-4");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "margin");
        assert_eq!(rule.declarations[0].value, "-1rem");
    }

    #[test]
    fn test_convert_valueless_fallback() {
        // overflow-auto: parser gives plugin="overflow", value="auto"
        // converter falls back to VALUELESS_MAP lookup of "overflow-auto"
        let converter = Converter::new();

        let parsed = parse_class("overflow-auto").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "overflow");
        assert_eq!(rule.declarations[0].value, "auto");
    }

    #[test]
    fn test_convert_compound_plugin() {
        // justify-items-center: parser detects compound plugin "justify-items"
        let converter = Converter::new();

        let parsed = parse_class("justify-items-center").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "justify-items");
        assert_eq!(rule.declarations[0].value, "center");
    }

    #[test]
    fn test_convert_compound_gap_x() {
        let converter = Converter::new();

        let parsed = parse_class("gap-x-4").unwrap();
        let rule = converter.convert(&parsed).unwrap();

        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, "column-gap");
        assert_eq!(rule.declarations[0].value, "1rem");
    }

    // ── Gradient tests ──────────────────────────────────────────

    #[test]
    fn test_bg_linear_angle() {
        let converter = Converter::new();
        let parsed = parse_class("bg-linear-45").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(45deg in oklab, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_gradient_to_v3_compat() {
        let converter = Converter::new();
        // Tailwind v3 syntax: bg-gradient-to-b
        let parsed = parse_class("bg-gradient-to-b").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(to bottom, var(--tw-gradient-stops))"
        );

        // bg-gradient-to-tr
        let parsed = parse_class("bg-gradient-to-tr").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(
            decls[0].value,
            "linear-gradient(to top right, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_linear_negative_angle() {
        let converter = Converter::new();
        let parsed = parse_class("-bg-linear-45").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(-45deg in oklab, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_linear_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("bg-linear-[45deg]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(var(--tw-gradient-stops, 45deg))"
        );
    }

    #[test]
    fn test_bg_conic_angle() {
        let converter = Converter::new();
        let parsed = parse_class("bg-conic-90").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "conic-gradient(from 90deg in oklab, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_conic_arbitrary() {
        // bg-conic-[<value>] → background-image: <value> (raw, not wrapped)
        let converter = Converter::new();
        let parsed = parse_class("bg-conic-[from_45deg]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(decls[0].value, "from 45deg");
    }

    #[test]
    fn test_bg_conic_css_variable() {
        // bg-conic-(--my-gradient) → background-image: var(--my-gradient)
        let converter = Converter::new();
        let parsed = parse_class("bg-conic-(--my-gradient)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(decls[0].value, "var(--my-gradient)");
    }

    #[test]
    fn test_bg_conic_negative_angle() {
        // -bg-conic-90 → conic-gradient(from -90deg in oklab, ...)
        let converter = Converter::new();
        let parsed = parse_class("-bg-conic-90").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "conic-gradient(from -90deg in oklab, var(--tw-gradient-stops))"
        );
    }

    #[test]
    fn test_bg_radial_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("bg-radial-[circle]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "radial-gradient(var(--tw-gradient-stops, circle))"
        );
    }

    #[test]
    fn test_from_color() {
        let converter = Converter::new();
        let parsed = parse_class("from-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-from");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_from_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("from-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-from");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_via_color() {
        let converter = Converter::new();
        let parsed = parse_class("via-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-via");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_to_color() {
        let converter = Converter::new();
        let parsed = parse_class("to-green-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-to");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_to_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("to-[rgba(0,0,0,0.5)]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-to");
        assert_eq!(decls[0].value, "rgba(0,0,0,0.5)");
    }

    // ── 颜色模式测试 ──────────────────────────────────────────

    #[test]
    fn test_color_mode_oklch() {
        let converter = Converter::new().with_color_mode(ColorMode::Oklch);
        let parsed = parse_class("bg-blue-500").unwrap();
        let rule = converter.convert(&parsed).unwrap();
        assert_eq!(rule.declarations[0].value, "oklch(0.623 0.214 259.815)");
    }

    #[test]
    fn test_color_mode_var() {
        let converter = Converter::new().with_color_mode(ColorMode::Var);
        let parsed = parse_class("text-red-500").unwrap();
        let rule = converter.convert(&parsed).unwrap();
        assert_eq!(rule.declarations[0].property, "color");
        assert_eq!(rule.declarations[0].value, "var(--color-red-500)");
    }

    #[test]
    fn test_color_mode_hsl() {
        let converter = Converter::new().with_color_mode(ColorMode::Hsl);
        let parsed = parse_class("from-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert!(decls[0].value.starts_with("hsl("));
    }

    #[test]
    fn test_new_color_families() {
        let converter = Converter::new();
        // 新增的颜色族应该能正常转换
        for family in &["orange", "amber", "violet", "slate", "zinc", "rose", "emerald"] {
            let class = format!("bg-{}-500", family);
            let parsed = parse_class(&class).unwrap();
            let rule = converter.convert(&parsed);
            assert!(rule.is_some(), "Failed for: {}", class);
        }
    }

    // ── CSS 自定义属性 -(…) 语法测试 ──────────────────────────

    #[test]
    fn test_css_variable_bg() {
        let converter = Converter::new();
        let parsed = parse_class("bg-(--my-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background");
        assert_eq!(decls[0].value, "var(--my-color)");
    }

    #[test]
    fn test_css_variable_bg_with_type_hint_image() {
        let converter = Converter::new();
        let parsed = parse_class("bg-(image:--my-bg)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(decls[0].value, "var(--my-bg)");
    }

    #[test]
    fn test_css_variable_bg_with_type_hint_color() {
        let converter = Converter::new();
        let parsed = parse_class("bg-(color:--my-bg)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-color");
        assert_eq!(decls[0].value, "var(--my-bg)");
    }

    #[test]
    fn test_css_variable_bg_linear() {
        let converter = Converter::new();
        let parsed = parse_class("bg-linear-(--my-gradient)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "linear-gradient(var(--tw-gradient-stops, var(--my-gradient)))"
        );
    }

    #[test]
    fn test_css_variable_bg_radial() {
        let converter = Converter::new();
        let parsed = parse_class("bg-radial-(--my-gradient)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(
            decls[0].value,
            "radial-gradient(var(--tw-gradient-stops, var(--my-gradient)))"
        );
    }

    #[test]
    fn test_css_variable_bg_conic() {
        let converter = Converter::new();
        let parsed = parse_class("bg-conic-(--my-gradient)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background-image");
        assert_eq!(decls[0].value, "var(--my-gradient)");
    }

    #[test]
    fn test_css_variable_from() {
        let converter = Converter::new();
        let parsed = parse_class("from-(--start-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-from");
        assert_eq!(decls[0].value, "var(--start-color)");
    }

    #[test]
    fn test_css_variable_via() {
        let converter = Converter::new();
        let parsed = parse_class("via-(--mid-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-via");
        assert_eq!(decls[0].value, "var(--mid-color)");
    }

    #[test]
    fn test_css_variable_to() {
        let converter = Converter::new();
        let parsed = parse_class("to-(--end-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-gradient-to");
        assert_eq!(decls[0].value, "var(--end-color)");
    }

    #[test]
    fn test_css_variable_text() {
        let converter = Converter::new();
        let parsed = parse_class("text-(--my-text-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "var(--my-text-color)");
    }

    #[test]
    fn test_css_variable_padding() {
        let converter = Converter::new();
        let parsed = parse_class("p-(--spacing)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "padding");
        assert_eq!(decls[0].value, "var(--spacing)");
    }

    #[test]
    fn test_css_variable_width() {
        let converter = Converter::new();
        let parsed = parse_class("w-(--sidebar-width)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "width");
        assert_eq!(decls[0].value, "var(--sidebar-width)");
    }

    #[test]
    fn test_css_variable_with_important() {
        let converter = Converter::new();
        let parsed = parse_class("bg-(--my-color)!").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background");
        assert!(decls[0].value.contains("var(--my-color)"));
        assert!(decls[0].value.contains("!important"));
    }

    #[test]
    fn test_css_variable_gap_x() {
        let converter = Converter::new();
        let parsed = parse_class("gap-x-(--my-gap)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "column-gap");
        assert_eq!(decls[0].value, "var(--my-gap)");
    }

    // ── 颜色插件测试 ────────────────────────────────────────────

    // --- accent ---

    #[test]
    fn test_accent_standard_color() {
        let converter = Converter::new();
        let parsed = parse_class("accent-stone-100").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "accent-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_accent_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("accent-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "accent-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_accent_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("accent-(--my-accent)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "accent-color");
        assert_eq!(decls[0].value, "var(--my-accent)");
    }

    // --- caret ---

    #[test]
    fn test_caret_standard_color() {
        let converter = Converter::new();
        let parsed = parse_class("caret-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "caret-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_caret_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("caret-[#00ff00]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "caret-color");
        assert_eq!(decls[0].value, "#00ff00");
    }

    #[test]
    fn test_caret_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("caret-(--my-caret)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "caret-color");
        assert_eq!(decls[0].value, "var(--my-caret)");
    }

    // --- fill ---

    #[test]
    fn test_fill_standard_color() {
        let converter = Converter::new();
        let parsed = parse_class("fill-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "fill");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_fill_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("fill-[#0000ff]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "fill");
        assert_eq!(decls[0].value, "#0000ff");
    }

    #[test]
    fn test_fill_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("fill-(--icon-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "fill");
        assert_eq!(decls[0].value, "var(--icon-color)");
    }

    // --- stroke ---

    #[test]
    fn test_stroke_standard_color() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-green-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_stroke_width() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke-width");
        assert_eq!(decls[0].value, "2");
    }

    #[test]
    fn test_stroke_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_stroke_arbitrary_width() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-[3px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke-width");
        assert_eq!(decls[0].value, "3px");
    }

    #[test]
    fn test_stroke_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("stroke-(--stroke-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "stroke");
        assert_eq!(decls[0].value, "var(--stroke-color)");
    }

    // --- border color ---

    #[test]
    fn test_border_color() {
        let converter = Converter::new();
        let parsed = parse_class("border-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_border_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("border-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_border_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("border-(--border-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-color");
        assert_eq!(decls[0].value, "var(--border-color)");
    }

    // --- border width ---

    #[test]
    fn test_border_valueless() {
        // border → border-width: 1px
        let converter = Converter::new();
        let parsed = parse_class("border").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-width");
        assert_eq!(decls[0].value, "1px");
    }

    #[test]
    fn test_border_number() {
        // border-2 → border-width: 2px
        let converter = Converter::new();
        let parsed = parse_class("border-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-width");
        assert_eq!(decls[0].value, "2px");
    }

    #[test]
    fn test_border_arbitrary_width() {
        // border-[3px] → border-width: 3px
        let converter = Converter::new();
        let parsed = parse_class("border-[3px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-width");
        assert_eq!(decls[0].value, "3px");
    }

    #[test]
    fn test_border_css_variable_length() {
        // border-(length:--my-width) → border-width: var(--my-width)
        let converter = Converter::new();
        let parsed = parse_class("border-(length:--my-width)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "border-width");
        assert_eq!(decls[0].value, "var(--my-width)");
    }

    // --- outline color ---

    #[test]
    fn test_outline_color() {
        let converter = Converter::new();
        let parsed = parse_class("outline-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "outline-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_outline_width() {
        let converter = Converter::new();
        let parsed = parse_class("outline-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "outline-width");
        assert_eq!(decls[0].value, "2px");
    }

    #[test]
    fn test_outline_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("outline-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "outline-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_outline_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("outline-(--outline-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "outline-color");
        assert_eq!(decls[0].value, "var(--outline-color)");
    }

    // --- decoration color ---

    #[test]
    fn test_decoration_color() {
        let converter = Converter::new();
        let parsed = parse_class("decoration-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "text-decoration-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_decoration_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("decoration-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "text-decoration-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_decoration_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("decoration-(--deco-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "text-decoration-color");
        assert_eq!(decls[0].value, "var(--deco-color)");
    }

    // --- shadow color ---

    #[test]
    fn test_shadow_color() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-shadow-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_shadow_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-shadow-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_shadow_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-(--shadow-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-shadow-color");
        assert_eq!(decls[0].value, "var(--shadow-color)");
    }

    // --- ring color ---

    #[test]
    fn test_ring_color() {
        let converter = Converter::new();
        let parsed = parse_class("ring-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_ring_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("ring-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_ring_css_variable_width() {
        let converter = Converter::new();
        let parsed = parse_class("ring-(--ring-width)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-shadow");
        assert_eq!(decls[0].value, "0 0 0 var(--ring-width)");
    }

    // --- inset-shadow ---

    #[test]
    fn test_inset_shadow_css_variable_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-(--inset-shadow-color)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-shadow-color");
        assert_eq!(decls[0].value, "var(--inset-shadow-color)");
    }

    // --- inset-ring ---

    #[test]
    fn test_inset_ring_css_variable_width() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-(--ring-width)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-shadow");
        assert_eq!(decls[0].value, "inset 0 0 0 var(--ring-width)");
    }

    // ── shadow named sizes ───────────────────────────────────────

    #[test]
    fn test_shadow_sm() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-sm").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--shadow-sm)");
    }

    #[test]
    fn test_shadow_md() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-md").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--shadow-md)");
    }

    #[test]
    fn test_shadow_2xl() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-2xl").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--shadow-2xl)");
    }

    #[test]
    fn test_shadow_none() {
        let converter = Converter::new();
        let parsed = parse_class("shadow-none").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "0 0 #0000");
    }

    // ── inset-shadow named sizes ─────────────────────────────────

    #[test]
    fn test_inset_shadow_sm() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-sm").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--inset-shadow-sm)");
    }

    #[test]
    fn test_inset_shadow_2xs() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-2xs").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "var(--inset-shadow-2xs)");
    }

    #[test]
    fn test_inset_shadow_none() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-none").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "box-shadow");
        assert_eq!(decls[0].value, "inset 0 0 #0000");
    }

    #[test]
    fn test_inset_shadow_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-red-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-shadow-color");
        assert!(decls[0].value.starts_with('#'));
    }

    // ── ring width ───────────────────────────────────────────────

    #[test]
    fn test_ring_valueless() {
        let converter = Converter::new();
        let parsed = parse_class("ring").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-shadow");
        assert_eq!(decls[0].value, "0 0 0 1px");
    }

    #[test]
    fn test_ring_number() {
        let converter = Converter::new();
        let parsed = parse_class("ring-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-shadow");
        assert_eq!(decls[0].value, "0 0 0 2px");
    }

    #[test]
    fn test_ring_arbitrary_width() {
        let converter = Converter::new();
        let parsed = parse_class("ring-[3px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-ring-shadow");
        assert_eq!(decls[0].value, "0 0 0 3px");
    }

    // ── inset-ring width ─────────────────────────────────────────

    #[test]
    fn test_inset_ring_valueless() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-shadow");
        assert_eq!(decls[0].value, "inset 0 0 0 1px");
    }

    #[test]
    fn test_inset_ring_number() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-shadow");
        assert_eq!(decls[0].value, "inset 0 0 0 2px");
    }

    #[test]
    fn test_inset_ring_arbitrary_width() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-[3px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-shadow");
        assert_eq!(decls[0].value, "inset 0 0 0 3px");
    }

    #[test]
    fn test_inset_ring_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-blue-500").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-color");
        assert!(decls[0].value.starts_with('#'));
    }

    #[test]
    fn test_inset_ring_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-ring-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-ring-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    #[test]
    fn test_inset_shadow_arbitrary_color() {
        let converter = Converter::new();
        let parsed = parse_class("inset-shadow-[#ff0000]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "--tw-inset-shadow-color");
        assert_eq!(decls[0].value, "#ff0000");
    }

    // ── text: font-size vs color ─────────────────────────────────

    #[test]
    fn test_text_arbitrary_font_size() {
        let converter = Converter::new();
        let parsed = parse_class("text-[14px]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "14px");
    }

    #[test]
    fn test_text_arbitrary_font_size_rem() {
        let converter = Converter::new();
        let parsed = parse_class("text-[1.5rem]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "1.5rem");
    }

    #[test]
    fn test_text_arbitrary_color_hex() {
        let converter = Converter::new();
        let parsed = parse_class("text-[#1da1f2]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "#1da1f2");
    }

    #[test]
    fn test_text_arbitrary_color_rgb() {
        let converter = Converter::new();
        let parsed = parse_class("text-[rgb(255,0,0)]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "rgb(255,0,0)");
    }

    #[test]
    fn test_text_css_variable_length_hint() {
        let converter = Converter::new();
        let parsed = parse_class("text-(length:--my-size)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--my-size)");
    }

    // ── text-size/alpha: line-height overrides ───────────────────

    #[test]
    fn test_text_size_with_number_line_height() {
        let converter = Converter::new();
        let parsed = parse_class("text-base/6").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--text-base)");
        assert_eq!(decls[1].property, "line-height");
        assert_eq!(decls[1].value, "calc(var(--spacing) * 6)");
    }

    #[test]
    fn test_text_size_with_bracket_line_height() {
        let converter = Converter::new();
        let parsed = parse_class("text-base/[1.5rem]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--text-base)");
        assert_eq!(decls[1].property, "line-height");
        assert_eq!(decls[1].value, "1.5rem");
    }

    #[test]
    fn test_text_size_with_css_var_line_height() {
        let converter = Converter::new();
        let parsed = parse_class("text-base/(--my-lh)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--text-base)");
        assert_eq!(decls[1].property, "line-height");
        assert_eq!(decls[1].value, "var(--my-lh)");
    }

    // ── leading ──────────────────────────────────────────────────

    #[test]
    fn test_leading_none() {
        let converter = Converter::new();
        let parsed = parse_class("leading-none").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "line-height");
        assert_eq!(decls[0].value, "1");
    }

    #[test]
    fn test_leading_number() {
        let converter = Converter::new();
        let parsed = parse_class("leading-6").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "line-height");
        assert_eq!(decls[0].value, "calc(var(--spacing) * 6)");
    }

    #[test]
    fn test_leading_css_variable() {
        let converter = Converter::new();
        let parsed = parse_class("leading-(--my-lh)").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "line-height");
        assert_eq!(decls[0].value, "var(--my-lh)");
    }

    #[test]
    fn test_leading_arbitrary() {
        let converter = Converter::new();
        let parsed = parse_class("leading-[1.5rem]").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "line-height");
        assert_eq!(decls[0].value, "1.5rem");
    }

    // ── alpha / opacity ─────────────────────────────────────────

    #[test]
    fn test_alpha_hex_white_60() {
        // text-white/60 → color: #fff9 (short form: ff→f, ff→f, ff→f, 99→9)
        let converter = Converter::new();
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "#fff9");
    }

    #[test]
    fn test_alpha_hex_black_50() {
        // text-black/50 → color: #00000080 (50% = 0x80, digits 8/0 differ → long form)
        let converter = Converter::new();
        let parsed = parse_class("text-black/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "#00000080");
    }

    #[test]
    fn test_alpha_hex_color_long_form() {
        // bg-blue-500/60 → background: #3b82f699 (not shortable)
        let converter = Converter::new();
        let parsed = parse_class("bg-blue-500/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "background");
        assert!(decls[0].value.len() == 9); // #rrggbbaa
        assert!(decls[0].value.ends_with("99")); // 60% = 0x99
    }

    #[test]
    fn test_alpha_100_no_change() {
        // text-white/100 → no alpha applied
        let converter = Converter::new();
        let parsed = parse_class("text-white/100").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "#ffffff");
    }

    #[test]
    fn test_alpha_oklch_mode() {
        // text-white/60 in oklch → oklch(1 0 0 / 60%)
        let converter = Converter::new().with_color_mode(ColorMode::Oklch);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "oklch(1 0 0 / 60%)");
    }

    #[test]
    fn test_alpha_hsl_mode() {
        // text-white/60 in hsl → hsl(0, 0%, 100% / 60%)
        let converter = Converter::new().with_color_mode(ColorMode::Hsl);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "hsl(0, 0%, 100% / 60%)");
    }

    #[test]
    fn test_alpha_var_mode_no_color_mix() {
        // text-white/60 in var mode without color-mix → alpha not applied
        let converter = Converter::new().with_color_mode(ColorMode::Var);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(decls[0].value, "var(--color-white)"); // can't apply alpha
    }

    #[test]
    fn test_alpha_var_mode_with_color_mix() {
        // text-white/60 in var mode with color-mix → color-mix(in oklab, …)
        let converter = Converter::new()
            .with_color_mode(ColorMode::Var)
            .with_color_mix(true);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(
            decls[0].value,
            "color-mix(in oklab, var(--color-white) 60%, transparent)"
        );
    }

    #[test]
    fn test_alpha_color_mix_hex_mode() {
        // color-mix enabled even in hex mode → generates color-mix
        let converter = Converter::new()
            .with_color_mode(ColorMode::Hex)
            .with_color_mix(true);
        let parsed = parse_class("text-white/60").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "color");
        assert_eq!(
            decls[0].value,
            "color-mix(in oklab, #ffffff 60%, transparent)"
        );
    }

    #[test]
    fn test_alpha_does_not_apply_to_non_color() {
        // text-base/6 → alpha used for line-height, NOT applied to font-size
        let converter = Converter::new();
        let parsed = parse_class("text-base/6").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "font-size");
        assert_eq!(decls[0].value, "var(--text-base)"); // no alpha
        assert_eq!(decls[1].property, "line-height");
        assert_eq!(decls[1].value, "calc(var(--spacing) * 6)");
    }

    #[test]
    fn test_alpha_border_color() {
        // border-red-500/50 → border-color with alpha
        let converter = Converter::new();
        let parsed = parse_class("border-red-500/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "border-color");
        assert!(decls[0].value.ends_with("80")); // 50% = 0x80
    }

    #[test]
    fn test_alpha_transparent_unchanged() {
        // bg-transparent/50 → transparent stays as-is
        let converter = Converter::new();
        let parsed = parse_class("bg-transparent/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "background");
        assert_eq!(decls[0].value, "transparent");
    }

    #[test]
    fn test_alpha_shadow_color() {
        // shadow-red-500/50 → --tw-shadow-color with alpha
        let converter = Converter::new();
        let parsed = parse_class("shadow-red-500/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "--tw-shadow-color");
        assert!(decls[0].value.ends_with("80")); // 50% = 0x80
    }

    #[test]
    fn test_alpha_ring_color() {
        // ring-blue-500/50 → --tw-ring-color with alpha
        let converter = Converter::new();
        let parsed = parse_class("ring-blue-500/50").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls[0].property, "--tw-ring-color");
        assert!(decls[0].value.ends_with("80")); // 50% = 0x80
    }

    // ── space-x / space-y ──────────────────────────────────────────

    #[test]
    fn test_space_x_0() {
        let converter = Converter::new();
        let parsed = parse_class("space-x-0").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "column-gap");
        assert_eq!(decls[0].value, "0");
    }

    #[test]
    fn test_space_x_2() {
        let converter = Converter::new();
        let parsed = parse_class("space-x-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "column-gap");
        assert_eq!(decls[0].value, "0.5rem");
    }

    #[test]
    fn test_space_y_4() {
        let converter = Converter::new();
        let parsed = parse_class("space-y-4").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "row-gap");
        assert_eq!(decls[0].value, "1rem");
    }

    // ── scroll padding / margin ────────────────────────────────────

    #[test]
    fn test_scroll_pr_6() {
        let converter = Converter::new();
        let parsed = parse_class("scroll-pr-6").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "scroll-padding-right");
        assert_eq!(decls[0].value, "1.5rem");
    }

    #[test]
    fn test_scroll_mt_2() {
        let converter = Converter::new();
        let parsed = parse_class("scroll-mt-2").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "scroll-margin-top");
        assert_eq!(decls[0].value, "0.5rem");
    }

    #[test]
    fn test_scroll_px_4() {
        let converter = Converter::new();
        let parsed = parse_class("scroll-px-4").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "scroll-padding-left");
        assert_eq!(decls[0].value, "1rem");
        assert_eq!(decls[1].property, "scroll-padding-right");
        assert_eq!(decls[1].value, "1rem");
    }

    #[test]
    fn test_scroll_my_8() {
        let converter = Converter::new();
        let parsed = parse_class("scroll-my-8").unwrap();
        let decls = converter.to_declarations(&parsed).unwrap();
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "scroll-margin-top");
        assert_eq!(decls[0].value, "2rem");
        assert_eq!(decls[1].property, "scroll-margin-bottom");
        assert_eq!(decls[1].value, "2rem");
    }
}
