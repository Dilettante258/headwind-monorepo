use std::collections::BTreeSet;

/// 规范化类名列表
///
/// 功能：
/// 1. 合并所有输入，按空格拆分
/// 2. 去除空字符串
/// 3. 去重
/// 4. 排序（字典序，保证确定性）
pub fn normalize_classes(classes: &[String]) -> Vec<String> {
    let mut unique_classes = BTreeSet::new();

    for class_str in classes {
        for token in class_str.split_whitespace() {
            if !token.is_empty() {
                unique_classes.insert(token.to_string());
            }
        }
    }

    unique_classes.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_duplicates() {
        let input = vec!["p-4".to_string(), "p-4".to_string()];
        let result = normalize_classes(&input);
        assert_eq!(result, vec!["p-4"]);
    }

    #[test]
    fn test_normalize_sorting() {
        let input = vec!["m-2".to_string(), "p-4".to_string()];
        let result = normalize_classes(&input);
        assert_eq!(result, vec!["m-2", "p-4"]);
    }

    #[test]
    fn test_normalize_split_spaces() {
        let input = vec!["p-4  m-2".to_string()];
        let result = normalize_classes(&input);
        assert_eq!(result, vec!["m-2", "p-4"]);
    }

    #[test]
    fn test_normalize_empty_strings() {
        let input = vec!["".to_string(), "p-4".to_string(), "".to_string()];
        let result = normalize_classes(&input);
        assert_eq!(result, vec!["p-4"]);
    }

    #[test]
    fn test_normalize_complex() {
        let input = vec![
            "p-4 m-2".to_string(),
            "p-4".to_string(),
            "text-red-500".to_string(),
            "m-2".to_string(),
        ];
        let result = normalize_classes(&input);
        assert_eq!(result, vec!["m-2", "p-4", "text-red-500"]);
    }
}
