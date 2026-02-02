use crate::merge::merge_declarations;
use crate::naming::create_naming_strategy;
use crate::normalize::normalize_classes;
use crate::shorthand::optimize_shorthands;
use crate::types::{BundleRequest, BundleResult, Declaration, Diagnostic};

/// 主 bundle 函数
///
/// 将 Tailwind 类名列表转换为单个类名和对应的 CSS 声明
///
/// # 参数
///
/// * `request` - 包含类名列表和命名模式的请求
/// * `tw_index` - Tailwind 索引（需要从外部传入以保持解耦）
pub fn bundle<I>(request: BundleRequest, tw_index: &I) -> BundleResult
where
    I: TailwindIndexLookup,
{
    // 1. 规范化类名
    let normalized = normalize_classes(&request.classes);

    // 2. 查询 tw_index，获取 CSS 声明
    let mut declarations = Vec::new();
    let mut removed = Vec::new();
    let mut diagnostics = Vec::new();

    for class in &normalized {
        match tw_index.lookup(class) {
            Some(decls) => {
                declarations.extend(decls.to_vec());
            }
            None => {
                removed.push(class.clone());
                diagnostics.push(Diagnostic::warning(format!("Unknown class: {}", class)));
            }
        }
    }

    // 3. 合并 CSS 声明
    let merged = merge_declarations(declarations);

    // 4. 简写属性优化（如 padding-top/right/bottom/left → padding）
    let optimized = optimize_shorthands(merged);

    // 5. 生成类名
    let naming_strategy = create_naming_strategy(request.naming_mode);
    let new_class = naming_strategy.generate_name(&normalized);

    BundleResult {
        new_class,
        css_declarations: optimized,
        removed,
        diagnostics,
    }
}

/// TailwindIndex 的查询接口
///
/// 使用 trait 而不是具体类型，以便于测试和解耦
pub trait TailwindIndexLookup {
    fn lookup(&self, class: &str) -> Option<&[Declaration]>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NamingMode;
    use std::collections::HashMap;

    // 测试用的简单索引实现
    struct SimpleIndex {
        map: HashMap<String, Vec<Declaration>>,
    }

    impl SimpleIndex {
        fn new() -> Self {
            Self {
                map: HashMap::new(),
            }
        }

        fn insert(&mut self, class: String, decls: Vec<Declaration>) {
            self.map.insert(class, decls);
        }
    }

    impl TailwindIndexLookup for SimpleIndex {
        fn lookup(&self, class: &str) -> Option<&[Declaration]> {
            self.map.get(class).map(|v| v.as_slice())
        }
    }

    #[test]
    fn test_bundle_basic() {
        let mut index = SimpleIndex::new();
        index.insert(
            "p-4".to_string(),
            vec![Declaration::new("padding", "1rem")],
        );

        let request = BundleRequest {
            classes: vec!["p-4".to_string()],
            naming_mode: NamingMode::Hash,
        };

        let result = bundle(request, &index);

        assert_eq!(result.css_declarations.len(), 1);
        assert!(result.new_class.starts_with("c_"));
        assert!(result.diagnostics.is_empty());
        assert!(result.removed.is_empty());
    }

    #[test]
    fn test_bundle_multiple_classes() {
        let mut index = SimpleIndex::new();
        index.insert(
            "p-4".to_string(),
            vec![Declaration::new("padding", "1rem")],
        );
        index.insert(
            "m-2".to_string(),
            vec![Declaration::new("margin", "0.5rem")],
        );

        let request = BundleRequest {
            classes: vec!["p-4".to_string(), "m-2".to_string()],
            naming_mode: NamingMode::Readable,
        };

        let result = bundle(request, &index);

        assert_eq!(result.css_declarations.len(), 2);
        assert_eq!(result.new_class, "m2_p4"); // 规范化后排序
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_bundle_unknown_class() {
        let index = SimpleIndex::new();

        let request = BundleRequest {
            classes: vec!["unknown-class".to_string()],
            naming_mode: NamingMode::Hash,
        };

        let result = bundle(request, &index);

        assert_eq!(result.css_declarations.len(), 0);
        assert_eq!(result.removed.len(), 1);
        assert_eq!(result.diagnostics.len(), 1);
    }

    #[test]
    fn test_bundle_conflict_merge() {
        let mut index = SimpleIndex::new();
        // 两个类都定义了 padding，后者应该覆盖前者
        index.insert(
            "p-4".to_string(),
            vec![Declaration::new("padding", "1rem")],
        );
        index.insert(
            "p-8".to_string(),
            vec![Declaration::new("padding", "2rem")],
        );

        let request = BundleRequest {
            classes: vec!["p-4".to_string(), "p-8".to_string()],
            naming_mode: NamingMode::Hash,
        };

        let result = bundle(request, &index);

        // 合并后应该只有一个 padding 声明
        assert_eq!(result.css_declarations.len(), 1);
        assert_eq!(result.css_declarations[0].property, "padding");
        assert_eq!(result.css_declarations[0].value, "2rem"); // 后者覆盖
    }
}
