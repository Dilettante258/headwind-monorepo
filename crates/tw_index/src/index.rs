use headwind_core::Declaration;
use std::collections::HashMap;

/// Tailwind 类名索引
///
/// 提供从类名到 CSS 声明的映射
pub struct TailwindIndex {
    map: HashMap<String, Vec<Declaration>>,
}

impl TailwindIndex {
    /// 创建空索引
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// 插入类名和对应的 CSS 声明
    pub fn insert(&mut self, class: String, decls: Vec<Declaration>) {
        self.map.insert(class, decls);
    }

    /// 查询类名对应的 CSS 声明
    pub fn lookup(&self, class: &str) -> Option<&[Declaration]> {
        self.map.get(class).map(|v| v.as_slice())
    }

    /// 获取所有已知的类名
    pub fn classes(&self) -> Vec<&str> {
        self.map.keys().map(|s| s.as_str()).collect()
    }

    /// 获取索引中的类名数量
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 索引是否为空
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl Default for TailwindIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_insert_and_lookup() {
        let mut index = TailwindIndex::new();
        index.insert(
            "p-4".to_string(),
            vec![Declaration::new("padding", "1rem")],
        );

        let result = index.lookup("p-4");
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
        assert_eq!(result.unwrap()[0].property, "padding");
    }

    #[test]
    fn test_index_lookup_missing() {
        let index = TailwindIndex::new();
        let result = index.lookup("unknown-class");
        assert!(result.is_none());
    }

    #[test]
    fn test_index_len() {
        let mut index = TailwindIndex::new();
        assert_eq!(index.len(), 0);
        assert!(index.is_empty());

        index.insert(
            "p-4".to_string(),
            vec![Declaration::new("padding", "1rem")],
        );
        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
    }
}
