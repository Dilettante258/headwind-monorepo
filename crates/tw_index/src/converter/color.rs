use headwind_core::Declaration;

/// 判断 CSS 属性是否为颜色属性
fn is_color_property(property: &str) -> bool {
    matches!(
        property,
        "color"
            | "background"
            | "background-color"
            | "border-color"
            | "outline-color"
            | "text-decoration-color"
            | "stroke"
            | "fill"
            | "accent-color"
            | "caret-color"
            | "--tw-shadow-color"
            | "--tw-inset-shadow-color"
            | "--tw-ring-color"
            | "--tw-inset-ring-color"
            | "--tw-gradient-from"
            | "--tw-gradient-via"
            | "--tw-gradient-to"
    )
}

/// 将透明度百分比转为 hex 字节（0-255），返回 2 位 hex 字符串
fn alpha_percent_to_hex(percent: f64) -> String {
    let byte = (percent / 100.0 * 255.0).round() as u8;
    format!("{:02x}", byte)
}

/// 为 hex 颜色添加 alpha 通道
///
/// 支持短格式优化：当 #rrggbb 每对字符相同且 alpha hex 也相同时，
/// 输出 4 位短格式（如 #ffffff + 60% → #fff9）
fn apply_alpha_to_hex(hex: &str, alpha_pct: f64) -> String {
    let body = hex.strip_prefix('#').unwrap_or(hex);
    let alpha_hex = alpha_percent_to_hex(alpha_pct);
    let ab = alpha_hex.as_bytes();

    if body.len() == 6 {
        let b = body.as_bytes();
        // 可以缩短：每对字符相同 + alpha 两位相同
        if b[0] == b[1] && b[2] == b[3] && b[4] == b[5] && ab[0] == ab[1] {
            return format!("#{}{}{}{}", b[0] as char, b[2] as char, b[4] as char, ab[0] as char);
        }
    }

    format!("#{}{}", body, alpha_hex)
}

/// 为颜色值应用 alpha 透明度
///
/// 根据值的格式选择不同的策略：
/// - hex: #rrggbb → #rrggbbaa（支持短格式优化）
/// - oklch/hsl/rgb: 在闭合括号前插入 `/ N%`
/// - var(): 无法直接应用 alpha，需要 color-mix（此函数跳过）
/// - transparent/currentColor: 跳过
fn apply_alpha_to_color(value: &str, alpha: &str, use_color_mix: bool) -> String {
    let alpha_pct: f64 = match alpha.parse() {
        Ok(n) => n,
        Err(_) => return value.to_string(),
    };

    // 100% = 完全不透明 → 不修改
    if (alpha_pct - 100.0).abs() < f64::EPSILON {
        return value.to_string();
    }

    // transparent / currentColor 无法应用 alpha
    if value == "transparent" || value == "currentColor" {
        return value.to_string();
    }

    // color-mix 模式：所有颜色值统一使用 color-mix
    if use_color_mix {
        return format!(
            "color-mix(in oklab, {} {}%, transparent)",
            value, alpha_pct as u32
        );
    }

    if value.starts_with('#') {
        apply_alpha_to_hex(value, alpha_pct)
    } else if value.starts_with("var(") {
        // var() → 无法直接应用 alpha（需要 color-mix）
        value.to_string()
    } else if let Some(pos) = value.rfind(')') {
        // oklch(...) / hsl(...) / rgb(...) → 插入 / N%
        format!("{} / {}%)", &value[..pos], alpha_pct as u32)
    } else {
        value.to_string()
    }
}

/// 为声明列表中的颜色属性应用 alpha 透明度
pub(super) fn apply_alpha_to_declarations(
    declarations: Vec<Declaration>,
    alpha: &str,
    use_color_mix: bool,
) -> Vec<Declaration> {
    declarations
        .into_iter()
        .map(|mut decl| {
            if is_color_property(&decl.property) {
                decl.value = apply_alpha_to_color(&decl.value, alpha, use_color_mix);
            }
            decl
        })
        .collect()
}

/// 应用 !important 标记
pub(super) fn apply_important(declarations: Vec<Declaration>, important: bool) -> Vec<Declaration> {
    if !important {
        return declarations;
    }
    declarations
        .into_iter()
        .map(|mut decl| {
            if !decl.value.ends_with("!important") {
                decl.value = format!("{} !important", decl.value);
            }
            decl
        })
        .collect()
}
