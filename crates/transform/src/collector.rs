use headwind_core::naming::{create_naming_strategy, NamingStrategy};
use headwind_core::NamingMode;
use headwind_tw_index::Bundler;
use indexmap::IndexMap;

/// 类名收集器 —— 收集源码中所有 Tailwind 类字符串，
/// 生成唯一类名，并产出对应的 CSS。
pub struct ClassCollector {
    bundler: Bundler,
    naming: Box<dyn NamingStrategy>,
    /// 原始类字符串 -> 生成的类名
    class_map: IndexMap<String, String>,
    /// 所有生成的 CSS 片段
    css_entries: Vec<String>,
    /// CSS 缩进
    indent: String,
}

impl ClassCollector {
    pub fn new(naming_mode: NamingMode) -> Self {
        let naming = create_naming_strategy(naming_mode);
        Self {
            bundler: Bundler::new(),
            naming,
            class_map: IndexMap::new(),
            css_entries: Vec::new(),
            indent: "  ".to_string(),
        }
    }

    /// 处理一组 Tailwind 类，返回生成的类名。
    /// 如果该类组合已处理过，直接返回缓存结果。
    pub fn process_classes(&mut self, classes: &str) -> String {
        let trimmed = classes.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // 缓存命中
        if let Some(name) = self.class_map.get(trimmed) {
            return name.clone();
        }

        // 生成类名
        let class_list: Vec<String> = trimmed.split_whitespace().map(|s| s.to_string()).collect();
        let new_name = self.naming.generate_name(&class_list);

        // 生成 CSS
        match self.bundler.bundle_to_css(&new_name, trimmed, &self.indent) {
            Ok(css) if !css.is_empty() => {
                self.css_entries.push(css);
            }
            _ => {}
        }

        self.class_map.insert(trimmed.to_string(), new_name.clone());
        new_name
    }

    /// 返回合并后的 CSS 输出
    pub fn combined_css(&self) -> String {
        self.css_entries.join("\n")
    }

    /// 返回类名映射表（原始 -> 生成）
    pub fn class_map(&self) -> &IndexMap<String, String> {
        &self.class_map
    }

    /// 消费 self，返回类名映射表
    pub fn into_class_map(self) -> IndexMap<String, String> {
        self.class_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_classes_basic() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let name = collector.process_classes("p-4 m-2");
        assert!(name.starts_with("c_"));
        assert!(!collector.combined_css().is_empty());
    }

    #[test]
    fn test_process_classes_caching() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let name1 = collector.process_classes("p-4 m-2");
        let name2 = collector.process_classes("p-4 m-2");
        assert_eq!(name1, name2);
        // CSS 应该只生成一次
        assert_eq!(collector.css_entries.len(), 1);
    }

    #[test]
    fn test_process_classes_different_inputs() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let name1 = collector.process_classes("p-4");
        let name2 = collector.process_classes("m-2");
        assert_ne!(name1, name2);
        assert_eq!(collector.css_entries.len(), 2);
    }

    #[test]
    fn test_process_empty_classes() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let name = collector.process_classes("  ");
        assert!(name.is_empty());
    }

    #[test]
    fn test_readable_naming() {
        let mut collector = ClassCollector::new(NamingMode::Readable);
        let name = collector.process_classes("p-4 m-2");
        assert_eq!(name, "p4_m2");
    }
}
