use crate::types::Declaration;
use indexmap::IndexMap;

/// 合并 CSS 声明
///
/// 功能：
/// - 处理 CSS 属性冲突（后者覆盖前者）
/// - 保持稳定输出顺序（使用 IndexMap）
pub fn merge_declarations(decls: Vec<Declaration>) -> Vec<Declaration> {
    let mut map: IndexMap<String, String> = IndexMap::new();

    for decl in decls {
        // 后者覆盖前者
        map.insert(decl.property, decl.value);
    }

    map.into_iter()
        .map(|(property, value)| Declaration { property, value })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_override() {
        let decls = vec![
            Declaration::new("padding", "1rem"),
            Declaration::new("padding", "2rem"),
        ];
        let result = merge_declarations(decls);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "2rem");
    }

    #[test]
    fn test_merge_no_conflict() {
        let decls = vec![
            Declaration::new("padding", "1rem"),
            Declaration::new("margin", "0.5rem"),
        ];
        let result = merge_declarations(decls);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[1].property, "margin");
    }

    #[test]
    fn test_merge_multiple_overrides() {
        let decls = vec![
            Declaration::new("padding", "1rem"),
            Declaration::new("margin", "0.5rem"),
            Declaration::new("padding", "2rem"),
            Declaration::new("margin", "1rem"),
        ];
        let result = merge_declarations(decls);
        assert_eq!(result.len(), 2);
        // IndexMap 保持首次插入的顺序
        assert_eq!(result[0].property, "padding");
        assert_eq!(result[0].value, "2rem");
        assert_eq!(result[1].property, "margin");
        assert_eq!(result[1].value, "1rem");
    }
}
