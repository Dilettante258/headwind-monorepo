use serde::{Deserialize, Serialize};

/// 输入：Tailwind class 列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRequest {
    pub classes: Vec<String>,
    pub naming_mode: NamingMode,
}

/// 输出：转换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResult {
    /// 生成的类名
    pub new_class: String,
    /// CSS 声明
    pub css_declarations: Vec<Declaration>,
    /// 被移除的类
    pub removed: Vec<String>,
    /// 警告/错误
    pub diagnostics: Vec<Diagnostic>,
}

/// 命名策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamingMode {
    /// 基于内容 hash: `c_a1b2c3d4e5f6`
    Hash,
    /// 调试友好（下划线分隔）: `p4_m2`
    Readable,
    /// 驼峰式（适合 CSS Modules `styles.xxx`）: `p4M2`
    CamelCase,
    /// AI 命名（未来）
    Semantic,
}

/// CSS 变量模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CssVariableMode {
    /// 使用 CSS 变量引用: `font-size: var(--text-3xl)`
    /// 需要引入 Tailwind 的 @layer theme
    #[default]
    Var,
    /// 内联为具体值: `font-size: 1.875rem`
    /// 独立使用，不依赖 Tailwind 主题
    Inline,
}

/// 颜色输出模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorMode {
    /// Hex 颜色值：#3b82f6
    #[default]
    Hex,
    /// OKLCH 颜色空间：oklch(0.623 0.214 259.815)
    Oklch,
    /// HSL 颜色值：hsl(217, 91%, 60%)
    Hsl,
    /// CSS 自定义属性：var(--color-blue-500)
    Var,
}

/// 未知类名处理模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UnknownClassMode {
    /// 删除不可识别的类名（默认）
    #[default]
    Remove,
    /// 保留不可识别的类名（原样输出）
    Preserve,
}

/// CSS 声明
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Declaration {
    /// CSS 属性名（如 "padding"）
    pub property: String,
    /// CSS 属性值（如 "1rem"）
    pub value: String,
}

impl Declaration {
    pub fn new(property: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            property: property.into(),
            value: value.into(),
        }
    }
}

/// 诊断信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
}

impl Diagnostic {
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Warning,
    Error,
}
