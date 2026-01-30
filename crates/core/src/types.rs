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
    /// 基于内容 hash
    Hash,
    /// 调试友好（如 "p4_m2"）
    Readable,
    /// AI 命名（未来）
    Semantic,
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
