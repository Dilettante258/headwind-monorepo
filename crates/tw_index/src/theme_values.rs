//! Tailwind v4 默认主题变量的内联值映射
//!
//! 当 `CssVariableMode::Inline` 时，使用这些值替代 `var(--xxx)` 引用。
//! 仅包含 converter.rs 中实际使用的主题变量，不包含 `--tw-*` 内部状态变量。

use phf::phf_map;

/// `--text-{size}` → font-size 值
pub static TEXT_SIZE: phf::Map<&'static str, &'static str> = phf_map! {
    "xs" => "0.75rem",
    "sm" => "0.875rem",
    "base" => "1rem",
    "lg" => "1.125rem",
    "xl" => "1.25rem",
    "2xl" => "1.5rem",
    "3xl" => "1.875rem",
    "4xl" => "2.25rem",
    "5xl" => "3rem",
    "6xl" => "3.75rem",
    "7xl" => "4.5rem",
    "8xl" => "6rem",
    "9xl" => "8rem",
};

/// `--text-{size}--line-height` → line-height 值
pub static TEXT_LINE_HEIGHT: phf::Map<&'static str, &'static str> = phf_map! {
    "xs" => "calc(1 / 0.75)",
    "sm" => "calc(1.25 / 0.875)",
    "base" => "calc(1.5 / 1)",
    "lg" => "calc(1.75 / 1.125)",
    "xl" => "calc(1.75 / 1.25)",
    "2xl" => "calc(2 / 1.5)",
    "3xl" => "calc(2.25 / 1.875)",
    "4xl" => "calc(2.5 / 2.25)",
    "5xl" => "1",
    "6xl" => "1",
    "7xl" => "1",
    "8xl" => "1",
    "9xl" => "1",
};

/// `--font-{family}` → font-family 值
pub static FONT_FAMILY: phf::Map<&'static str, &'static str> = phf_map! {
    "sans" => "ui-sans-serif, system-ui, sans-serif, \"Apple Color Emoji\", \"Segoe UI Emoji\"",
    "serif" => "ui-serif, Georgia, Cambria, \"Times New Roman\", Times, serif",
    "mono" => "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, \"Liberation Mono\", monospace",
};

/// `--blur-{size}` → blur 像素值
pub static BLUR_SIZE: phf::Map<&'static str, &'static str> = phf_map! {
    "none" => "0",
    "sm" => "4px",
    "DEFAULT" => "8px",
    "md" => "12px",
    "lg" => "16px",
    "xl" => "24px",
    "2xl" => "40px",
    "3xl" => "64px",
};
