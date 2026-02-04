use serde::{Deserialize, Serialize};

/// 解析后的 Tailwind class 表示
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParsedClass {
    /// 原始修饰符字符串（如 "md:hover:"）
    /// 需要时可通过 parse_modifiers_from_raw() 解析成 Vec<Modifier>
    pub raw_modifiers: String,

    /// 是否为负值（如 -m-4）
    pub negative: bool,

    /// 核心插件/命令（如 p, m, bg, text）
    pub plugin: String,

    /// 值部分
    pub value: Option<ParsedValue>,

    /// 透明度修饰符（如 /50）
    pub alpha: Option<String>,

    /// 重要性标记（!）
    pub important: bool,
}

/// 修饰符类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modifier {
    /// 响应式断点（sm, md, lg, xl, 2xl）
    Responsive(String),

    /// 伪类（hover, focus, active, visited 等）
    PseudoClass(String),

    /// 伪元素（before, after, placeholder 等）
    PseudoElement(String),

    /// 状态修饰符（dark, group-hover, peer-focus 等）
    State(String),

    /// 自定义修饰符
    Custom(String),
}

/// 值类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParsedValue {
    /// 标准值（如 "4", "red-500", "lg"）
    Standard(String),

    /// 任意值（如 "[13px]", "[#ff0000]"）
    Arbitrary(ArbitraryValue),

    /// CSS 自定义属性引用（如 "(--my-color)", "(image:--my-bg)"）
    /// Tailwind v4 语法：`bg-(--my-color)` → `background: var(--my-color)`
    CssVariable(CssVariableValue),
}

/// 任意值表示
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArbitraryValue {
    /// 原始值（包含方括号）
    pub raw: String,

    /// 解析后的内容（去除方括号）
    pub content: String,
}

/// CSS 自定义属性值表示
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CssVariableValue {
    /// 原始值（包含圆括号，如 "(--my-color)" 或 "(image:--my-bg)"）
    pub raw: String,

    /// CSS 自定义属性名（如 "--my-color"）
    pub property: String,

    /// 可选的类型提示（如 "image" in "bg-(image:--my-bg)"）
    pub type_hint: Option<String>,
}

impl ParsedClass {
    /// 创建一个新的 ParsedClass
    pub fn new(plugin: String) -> Self {
        Self {
            raw_modifiers: String::new(),
            negative: false,
            plugin,
            value: None,
            alpha: None,
            important: false,
        }
    }

    /// 获取解析后的修饰符列表
    ///
    /// 这个方法会在需要时从 raw_modifiers 解析出 Modifier 列表
    pub fn modifiers(&self) -> Vec<Modifier> {
        parse_modifiers_from_raw(&self.raw_modifiers)
    }

    /// 添加修饰符
    pub fn with_modifier(mut self, modifier: Modifier) -> Self {
        // 更新 raw_modifiers
        if !self.raw_modifiers.is_empty() {
            self.raw_modifiers.push_str(&format!("{}:", modifier));
        } else {
            self.raw_modifiers = format!("{}:", modifier);
        }
        self
    }

    /// 设置值
    pub fn with_value(mut self, value: ParsedValue) -> Self {
        self.value = Some(value);
        self
    }

    /// 设置负值标记
    pub fn with_negative(mut self, negative: bool) -> Self {
        self.negative = negative;
        self
    }

    /// 设置透明度
    pub fn with_alpha(mut self, alpha: String) -> Self {
        self.alpha = Some(alpha);
        self
    }

    /// 设置重要性
    pub fn with_important(mut self, important: bool) -> Self {
        self.important = important;
        self
    }

    /// 获取规范化的 class 字符串（用于索引查找）
    pub fn to_normalized_string(&self) -> String {
        let mut result = String::new();

        // 添加修饰符（直接使用 raw_modifiers）
        result.push_str(&self.raw_modifiers);

        // 添加负值前缀
        if self.negative {
            result.push('-');
        }

        // 添加插件
        result.push_str(&self.plugin);

        // 添加值
        if let Some(value) = &self.value {
            result.push('-');
            result.push_str(&value.to_string());
        }

        // 添加透明度
        if let Some(alpha) = &self.alpha {
            result.push('/');
            result.push_str(alpha);
        }

        // 添加重要性
        if self.important {
            result.push('!');
        }

        result
    }
}

impl Modifier {
    /// 判断是否为响应式修饰符
    pub fn is_responsive(&self) -> bool {
        matches!(self, Modifier::Responsive(_))
    }

    /// 判断是否为伪类
    pub fn is_pseudo_class(&self) -> bool {
        matches!(self, Modifier::PseudoClass(_))
    }

    /// 从字符串推断修饰符类型
    pub fn from_str(s: &str) -> Self {
        // 响应式断点: sm, md, lg, xl, 2xl, max-sm, max-md, etc.
        // Also min-[...], max-[...]
        if matches!(s, "sm" | "md" | "lg" | "xl" | "2xl")
            || matches!(
                s,
                "max-sm" | "max-md" | "max-lg" | "max-xl" | "max-2xl"
            )
            || (s.starts_with("min-") && s.contains('['))
            || (s.starts_with("max-") && s.contains('['))
        {
            return Modifier::Responsive(s.to_string());
        }

        // Container queries: @3xs through @7xl, @max-*, @min-[...]
        if s.starts_with('@') {
            return Modifier::Responsive(s.to_string());
        }

        // supports-[...] → @supports at-rule (treated as state/at-rule)
        if s.starts_with("supports-") && s.contains('[') {
            return Modifier::State(s.to_string());
        }

        // Parameterized pseudo-classes: has-[...], not-[...], nth-[...],
        // nth-last-[...], nth-of-type-[...], nth-last-of-type-[...],
        // aria-[...], data-[...], in-[...]
        if (s.starts_with("has-")
            || s.starts_with("not-")
            || s.starts_with("nth-")
            || s.starts_with("in-")
            || s.starts_with("data-"))
            && s.contains('[')
        {
            return Modifier::PseudoClass(s.to_string());
        }

        // Named aria-* (no bracket): aria-busy, aria-checked, etc.
        if s.starts_with("aria-") {
            return Modifier::PseudoClass(s.to_string());
        }

        // 伪类
        if matches!(
            s,
            "hover"
                | "focus"
                | "active"
                | "visited"
                | "target"
                | "focus-within"
                | "focus-visible"
                | "disabled"
                | "enabled"
                | "checked"
                | "indeterminate"
                | "default"
                | "optional"
                | "required"
                | "valid"
                | "invalid"
                | "user-valid"
                | "user-invalid"
                | "in-range"
                | "out-of-range"
                | "placeholder-shown"
                | "autofill"
                | "read-only"
                | "empty"
                | "first"
                | "last"
                | "only"
                | "odd"
                | "even"
                | "first-of-type"
                | "last-of-type"
                | "only-of-type"
                | "open"
                | "inert"
        ) {
            return Modifier::PseudoClass(s.to_string());
        }

        // 伪元素
        if matches!(
            s,
            "before"
                | "after"
                | "placeholder"
                | "file"
                | "marker"
                | "selection"
                | "first-line"
                | "first-letter"
                | "backdrop"
                | "details-content"
        ) {
            return Modifier::PseudoElement(s.to_string());
        }

        // Child selectors
        if matches!(s, "*" | "**") {
            return Modifier::PseudoClass(s.to_string());
        }

        // 状态修饰符
        if s.starts_with("group-")
            || s.starts_with("peer-")
            || matches!(
                s,
                "dark"
                    | "light"
                    | "starting"
                    | "motion-safe"
                    | "motion-reduce"
                    | "contrast-more"
                    | "contrast-less"
                    | "portrait"
                    | "landscape"
                    | "print"
                    | "forced-colors"
                    | "inverted-colors"
                    | "pointer-fine"
                    | "pointer-coarse"
                    | "pointer-none"
                    | "any-pointer-fine"
                    | "any-pointer-coarse"
                    | "any-pointer-none"
                    | "noscript"
                    | "rtl"
                    | "ltr"
            )
        {
            return Modifier::State(s.to_string());
        }

        // 默认为自定义修饰符
        Modifier::Custom(s.to_string())
    }
}

impl std::fmt::Display for Modifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modifier::Responsive(s)
            | Modifier::PseudoClass(s)
            | Modifier::PseudoElement(s)
            | Modifier::State(s)
            | Modifier::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// 从 raw_modifiers 字符串解析出 Modifier 列表
///
/// # 示例
///
/// ```
/// use headwind_tw_parse::parse_modifiers_from_raw;
///
/// let modifiers = parse_modifiers_from_raw("hover:md:");
/// assert_eq!(modifiers.len(), 2);
/// ```
pub fn parse_modifiers_from_raw(raw: &str) -> Vec<Modifier> {
    if raw.is_empty() {
        return Vec::new();
    }

    // 按冒号分割，过滤空字符串
    raw.split(':')
        .filter(|s| !s.is_empty())
        .map(Modifier::from_str)
        .collect()
}

impl ParsedValue {
    /// 判断是否为任意值
    pub fn is_arbitrary(&self) -> bool {
        matches!(self, ParsedValue::Arbitrary(_))
    }

    /// 判断是否为 CSS 自定义属性引用
    pub fn is_css_variable(&self) -> bool {
        matches!(self, ParsedValue::CssVariable(_))
    }
}

impl std::fmt::Display for ParsedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedValue::Standard(s) => write!(f, "{}", s),
            ParsedValue::Arbitrary(arb) => write!(f, "{}", arb.raw),
            ParsedValue::CssVariable(cv) => write!(f, "{}", cv.raw),
        }
    }
}

impl CssVariableValue {
    /// 创建新的 CSS 自定义属性值
    ///
    /// 从原始圆括号字符串中解析出属性名和可选类型提示。
    /// 例如：`"(--my-color)"` → property: `"--my-color"`, type_hint: None
    ///       `"(image:--my-bg)"` → property: `"--my-bg"`, type_hint: Some("image")
    pub fn new(raw: String) -> Self {
        let inner = raw
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&raw);

        let (type_hint, property) = if let Some(colon_pos) = inner.find(':') {
            let hint = &inner[..colon_pos];
            let prop = &inner[colon_pos + 1..];
            // 只有当冒号前的部分不以 -- 开头时才视为类型提示
            // 否则整个值就是属性名（如 --my-color 本身不含类型提示）
            if !hint.starts_with("--") {
                (Some(hint.to_string()), prop.to_string())
            } else {
                (None, inner.to_string())
            }
        } else {
            (None, inner.to_string())
        };

        Self {
            raw,
            property,
            type_hint,
        }
    }
}

impl ArbitraryValue {
    /// 创建新的任意值
    ///
    /// 按照 Tailwind 规范，任意值中的 `_` 会转换为空格，
    /// `\_` 会保留为字面下划线。
    pub fn new(raw: String) -> Self {
        let stripped = raw
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .unwrap_or(&raw);

        let content = convert_underscores(stripped);

        Self { raw, content }
    }
}

/// 将任意值中的下划线转换为空格
///
/// 规则：
/// - `_` → 空格
/// - `\_` → 字面下划线 `_`
fn convert_underscores(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'_') {
            // \_ → 字面下划线
            chars.next();
            result.push('_');
        } else if ch == '_' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_classification() {
        assert!(Modifier::from_str("md").is_responsive());
        assert!(Modifier::from_str("hover").is_pseudo_class());
        assert_eq!(
            Modifier::from_str("dark"),
            Modifier::State("dark".to_string())
        );
    }

    #[test]
    fn test_parsed_class_normalization() {
        let class = ParsedClass::new("p".to_string())
            .with_modifier(Modifier::Responsive("md".to_string()))
            .with_modifier(Modifier::PseudoClass("hover".to_string()))
            .with_value(ParsedValue::Standard("4".to_string()));

        assert_eq!(class.to_normalized_string(), "md:hover:p-4");
    }

    #[test]
    fn test_arbitrary_value() {
        let arb = ArbitraryValue::new("[13px]".to_string());
        assert_eq!(arb.content, "13px");
        assert_eq!(arb.raw, "[13px]");
    }

    #[test]
    fn test_arbitrary_value_underscore_to_space() {
        // _ → 空格
        let arb = ArbitraryValue::new("[0_1px_2px_red]".to_string());
        assert_eq!(arb.content, "0 1px 2px red");
    }

    #[test]
    fn test_arbitrary_value_escaped_underscore() {
        // \_ → 字面下划线
        let arb = ArbitraryValue::new("[content\\_value]".to_string());
        assert_eq!(arb.content, "content_value");
    }

    #[test]
    fn test_arbitrary_value_mixed_underscores() {
        // 混合: _ 转空格, \_ 保留
        let arb = ArbitraryValue::new("[a_b\\_c_d]".to_string());
        assert_eq!(arb.content, "a b_c d");
    }

    #[test]
    fn test_arbitrary_value_no_underscores() {
        let arb = ArbitraryValue::new("[#ff0000]".to_string());
        assert_eq!(arb.content, "#ff0000");
    }
}
