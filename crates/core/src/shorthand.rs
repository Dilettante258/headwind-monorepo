use crate::types::Declaration;
use std::collections::{HashMap, HashSet};

/// 简写属性类型
enum ShorthandKind {
    /// top, right, bottom, left (padding, margin, border-width, inset)
    Trbl,
    /// top-left, top-right, bottom-right, bottom-left (border-radius)
    FourCorner,
    /// 双值简写 (gap, overflow, overscroll-behavior)
    TwoValue,
}

/// 简写属性组定义
struct ShorthandGroup {
    /// 简写属性名（如 "padding"）
    shorthand: &'static str,
    /// 各个子属性名（按顺序排列）
    longhands: &'static [&'static str],
    /// 简写类型
    kind: ShorthandKind,
}

/// 所有支持的简写属性组
static SHORTHAND_GROUPS: &[ShorthandGroup] = &[
    ShorthandGroup {
        shorthand: "padding",
        longhands: &[
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ],
        kind: ShorthandKind::Trbl,
    },
    ShorthandGroup {
        shorthand: "margin",
        longhands: &[
            "margin-top",
            "margin-right",
            "margin-bottom",
            "margin-left",
        ],
        kind: ShorthandKind::Trbl,
    },
    ShorthandGroup {
        shorthand: "border-width",
        longhands: &[
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ],
        kind: ShorthandKind::Trbl,
    },
    ShorthandGroup {
        shorthand: "border-radius",
        longhands: &[
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-right-radius",
            "border-bottom-left-radius",
        ],
        kind: ShorthandKind::FourCorner,
    },
    ShorthandGroup {
        shorthand: "inset",
        longhands: &["top", "right", "bottom", "left"],
        kind: ShorthandKind::Trbl,
    },
    ShorthandGroup {
        shorthand: "gap",
        longhands: &["row-gap", "column-gap"],
        kind: ShorthandKind::TwoValue,
    },
    ShorthandGroup {
        shorthand: "overflow",
        longhands: &["overflow-x", "overflow-y"],
        kind: ShorthandKind::TwoValue,
    },
    ShorthandGroup {
        shorthand: "overscroll-behavior",
        longhands: &["overscroll-behavior-x", "overscroll-behavior-y"],
        kind: ShorthandKind::TwoValue,
    },
];

/// 将 CSS 子属性声明合并为简写属性
///
/// 仅当某个简写组的**所有**子属性都出现时才合并。
/// 合并后的简写声明放在第一个子属性的位置，其余子属性被移除。
/// 非简写声明保持原始顺序不变。
///
/// # 示例
///
/// ```
/// use headwind_core::types::Declaration;
/// use headwind_core::shorthand::optimize_shorthands;
///
/// let decls = vec![
///     Declaration::new("padding-top", "1rem"),
///     Declaration::new("padding-right", "1rem"),
///     Declaration::new("padding-bottom", "1rem"),
///     Declaration::new("padding-left", "1rem"),
/// ];
/// let result = optimize_shorthands(decls);
/// assert_eq!(result.len(), 1);
/// assert_eq!(result[0].property, "padding");
/// assert_eq!(result[0].value, "1rem");
/// ```
pub fn optimize_shorthands(decls: Vec<Declaration>) -> Vec<Declaration> {
    if decls.is_empty() {
        return decls;
    }

    // 1. 构建 property → value 映射
    let mut prop_map: HashMap<&str, &str> = HashMap::new();
    for decl in &decls {
        prop_map.insert(&decl.property, &decl.value);
    }

    // 2. 查找所有子属性齐全的简写组
    let mut matched_groups: Vec<&ShorthandGroup> = Vec::new();
    let mut consumed: HashSet<&str> = HashSet::new();

    for group in SHORTHAND_GROUPS {
        let all_present = group.longhands.iter().all(|lh| prop_map.contains_key(lh));
        if !all_present {
            continue;
        }

        // 检查 !important 一致性：全部有或全部没有才合并
        let values: Vec<&str> = group
            .longhands
            .iter()
            .map(|lh| *prop_map.get(lh).unwrap())
            .collect();

        let all_important = values.iter().all(|v| v.ends_with("!important"));
        let none_important = values.iter().all(|v| !v.ends_with("!important"));

        if !all_important && !none_important {
            // !important 不一致，跳过
            continue;
        }

        matched_groups.push(group);
        for lh in group.longhands {
            consumed.insert(lh);
        }
    }

    // 快速路径：无匹配组
    if matched_groups.is_empty() {
        return decls;
    }

    // 3. 为每个匹配组计算简写值
    let mut shorthand_values: HashMap<&str, String> = HashMap::new();

    for group in &matched_groups {
        let raw_values: Vec<&str> = group
            .longhands
            .iter()
            .map(|lh| *prop_map.get(lh).unwrap())
            .collect();

        let all_important = raw_values.iter().all(|v| v.ends_with("!important"));

        let values: Vec<&str> = if all_important {
            raw_values
                .iter()
                .map(|v| v.trim_end_matches("!important").trim())
                .collect()
        } else {
            raw_values
        };

        let compressed = match group.kind {
            ShorthandKind::Trbl | ShorthandKind::FourCorner => compress_trbl(&values),
            ShorthandKind::TwoValue => compress_two_value(&values),
        };

        let final_value = if all_important {
            format!("{} !important", compressed)
        } else {
            compressed
        };

        shorthand_values.insert(group.shorthand, final_value);
    }

    // 4. 重建声明列表
    let mut result: Vec<Declaration> = Vec::new();
    let mut emitted: HashSet<&str> = HashSet::new();

    for decl in &decls {
        if consumed.contains(decl.property.as_str()) {
            // 找到该属性所属的简写组
            if let Some(group) = matched_groups
                .iter()
                .find(|g| g.longhands.contains(&decl.property.as_str()))
            {
                if !emitted.contains(group.shorthand) {
                    result.push(Declaration::new(
                        group.shorthand,
                        shorthand_values.get(group.shorthand).unwrap().clone(),
                    ));
                    emitted.insert(group.shorthand);
                }
                // 跳过该子属性（无论是否为第一个）
            }
        } else {
            result.push(decl.clone());
        }
    }

    result
}

/// TRBL / 4-corner 值压缩
///
/// 输入: [top, right, bottom, left] 4个值
///
/// 规则：
/// - 全部相同:            "V"
/// - top==bottom, left==right: "V1 V2"
/// - left==right:          "V1 V2 V3"
/// - 全部不同:            "V1 V2 V3 V4"
fn compress_trbl(values: &[&str]) -> String {
    debug_assert_eq!(values.len(), 4);
    let (top, right, bottom, left) = (values[0], values[1], values[2], values[3]);

    if top == right && right == bottom && bottom == left {
        top.to_string()
    } else if top == bottom && left == right {
        format!("{} {}", top, right)
    } else if left == right {
        format!("{} {} {}", top, right, bottom)
    } else {
        format!("{} {} {} {}", top, right, bottom, left)
    }
}

/// 双值简写压缩
///
/// 输入: [first, second] 2个值
///
/// 规则：
/// - 相同: "V"
/// - 不同: "V1 V2"
fn compress_two_value(values: &[&str]) -> String {
    debug_assert_eq!(values.len(), 2);
    if values[0] == values[1] {
        values[0].to_string()
    } else {
        format!("{} {}", values[0], values[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== padding TRBL ==========

    #[test]
    fn test_padding_all_same() {
        let decls = vec![
            Declaration::new("padding-top", "1rem"),
            Declaration::new("padding-right", "1rem"),
            Declaration::new("padding-bottom", "1rem"),
            Declaration::new("padding-left", "1rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "1rem");
    }

    #[test]
    fn test_padding_two_value() {
        let decls = vec![
            Declaration::new("padding-top", "1rem"),
            Declaration::new("padding-right", "2rem"),
            Declaration::new("padding-bottom", "1rem"),
            Declaration::new("padding-left", "2rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "1rem 2rem");
    }

    #[test]
    fn test_padding_three_value() {
        let decls = vec![
            Declaration::new("padding-top", "1rem"),
            Declaration::new("padding-right", "2rem"),
            Declaration::new("padding-bottom", "3rem"),
            Declaration::new("padding-left", "2rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "1rem 2rem 3rem");
    }

    #[test]
    fn test_padding_four_value() {
        let decls = vec![
            Declaration::new("padding-top", "1rem"),
            Declaration::new("padding-right", "2rem"),
            Declaration::new("padding-bottom", "3rem"),
            Declaration::new("padding-left", "4rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "1rem 2rem 3rem 4rem");
    }

    // ========== margin ==========

    #[test]
    fn test_margin_auto_centering() {
        let decls = vec![
            Declaration::new("margin-top", "0"),
            Declaration::new("margin-right", "auto"),
            Declaration::new("margin-bottom", "0"),
            Declaration::new("margin-left", "auto"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "margin");
        assert_eq!(result[0].value, "0 auto");
    }

    // ========== partial group (不合并) ==========

    #[test]
    fn test_partial_padding_no_optimization() {
        let decls = vec![
            Declaration::new("padding-top", "1rem"),
            Declaration::new("padding-bottom", "1rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].property, "padding-top");
        assert_eq!(result[1].property, "padding-bottom");
    }

    // ========== border-radius ==========

    #[test]
    fn test_border_radius_all_same() {
        let decls = vec![
            Declaration::new("border-top-left-radius", "0.5rem"),
            Declaration::new("border-top-right-radius", "0.5rem"),
            Declaration::new("border-bottom-right-radius", "0.5rem"),
            Declaration::new("border-bottom-left-radius", "0.5rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "border-radius");
        assert_eq!(result[0].value, "0.5rem");
    }

    #[test]
    fn test_border_radius_two_value() {
        let decls = vec![
            Declaration::new("border-top-left-radius", "0.5rem"),
            Declaration::new("border-top-right-radius", "1rem"),
            Declaration::new("border-bottom-right-radius", "0.5rem"),
            Declaration::new("border-bottom-left-radius", "1rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "border-radius");
        assert_eq!(result[0].value, "0.5rem 1rem");
    }

    // ========== inset ==========

    #[test]
    fn test_inset_all_zero() {
        let decls = vec![
            Declaration::new("top", "0"),
            Declaration::new("right", "0"),
            Declaration::new("bottom", "0"),
            Declaration::new("left", "0"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "inset");
        assert_eq!(result[0].value, "0");
    }

    // ========== gap ==========

    #[test]
    fn test_gap_same() {
        let decls = vec![
            Declaration::new("row-gap", "1rem"),
            Declaration::new("column-gap", "1rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "gap");
        assert_eq!(result[0].value, "1rem");
    }

    #[test]
    fn test_gap_different() {
        let decls = vec![
            Declaration::new("row-gap", "1rem"),
            Declaration::new("column-gap", "2rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "gap");
        assert_eq!(result[0].value, "1rem 2rem");
    }

    // ========== overflow ==========

    #[test]
    fn test_overflow_same() {
        let decls = vec![
            Declaration::new("overflow-x", "hidden"),
            Declaration::new("overflow-y", "hidden"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "overflow");
        assert_eq!(result[0].value, "hidden");
    }

    #[test]
    fn test_overflow_different() {
        let decls = vec![
            Declaration::new("overflow-x", "hidden"),
            Declaration::new("overflow-y", "scroll"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "overflow");
        assert_eq!(result[0].value, "hidden scroll");
    }

    // ========== 顺序保持 ==========

    #[test]
    fn test_order_preservation_with_interleaved() {
        let decls = vec![
            Declaration::new("display", "flex"),
            Declaration::new("padding-top", "1rem"),
            Declaration::new("color", "red"),
            Declaration::new("padding-right", "1rem"),
            Declaration::new("padding-bottom", "1rem"),
            Declaration::new("padding-left", "1rem"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].property, "display");
        assert_eq!(result[1].property, "padding");
        assert_eq!(result[1].value, "1rem");
        assert_eq!(result[2].property, "color");
    }

    // ========== 多组同时合并 ==========

    #[test]
    fn test_multiple_groups() {
        let decls = vec![
            Declaration::new("padding-top", "1rem"),
            Declaration::new("padding-right", "1rem"),
            Declaration::new("padding-bottom", "1rem"),
            Declaration::new("padding-left", "1rem"),
            Declaration::new("margin-top", "0"),
            Declaration::new("margin-right", "auto"),
            Declaration::new("margin-bottom", "0"),
            Declaration::new("margin-left", "auto"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "1rem");
        assert_eq!(result[1].property, "margin");
        assert_eq!(result[1].value, "0 auto");
    }

    // ========== 无简写属性（原样返回） ==========

    #[test]
    fn test_no_longhands() {
        let decls = vec![
            Declaration::new("display", "flex"),
            Declaration::new("color", "red"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].property, "display");
        assert_eq!(result[1].property, "color");
    }

    // ========== 空输入 ==========

    #[test]
    fn test_empty_input() {
        let result = optimize_shorthands(vec![]);
        assert!(result.is_empty());
    }

    // ========== !important ==========

    #[test]
    fn test_important_all_consistent() {
        let decls = vec![
            Declaration::new("padding-top", "1rem !important"),
            Declaration::new("padding-right", "1rem !important"),
            Declaration::new("padding-bottom", "1rem !important"),
            Declaration::new("padding-left", "1rem !important"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "1rem !important");
    }

    #[test]
    fn test_important_mixed_no_optimization() {
        let decls = vec![
            Declaration::new("padding-top", "1rem !important"),
            Declaration::new("padding-right", "1rem"),
            Declaration::new("padding-bottom", "1rem !important"),
            Declaration::new("padding-left", "1rem"),
        ];
        let result = optimize_shorthands(decls);
        // !important 不一致，不合并
        assert_eq!(result.len(), 4);
    }

    // ========== var() 值 ==========

    #[test]
    fn test_var_values() {
        let decls = vec![
            Declaration::new("padding-top", "var(--spacing-4)"),
            Declaration::new("padding-right", "var(--spacing-4)"),
            Declaration::new("padding-bottom", "var(--spacing-4)"),
            Declaration::new("padding-left", "var(--spacing-4)"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "var(--spacing-4)");
    }

    // ========== border-width ==========

    #[test]
    fn test_border_width() {
        let decls = vec![
            Declaration::new("border-top-width", "1px"),
            Declaration::new("border-right-width", "2px"),
            Declaration::new("border-bottom-width", "1px"),
            Declaration::new("border-left-width", "2px"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "border-width");
        assert_eq!(result[0].value, "1px 2px");
    }

    // ========== overscroll-behavior ==========

    #[test]
    fn test_overscroll_behavior() {
        let decls = vec![
            Declaration::new("overscroll-behavior-x", "contain"),
            Declaration::new("overscroll-behavior-y", "contain"),
        ];
        let result = optimize_shorthands(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "overscroll-behavior");
        assert_eq!(result[0].value, "contain");
    }
}
