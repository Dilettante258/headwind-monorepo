use headwind_core::naming::{create_naming_strategy, NamingStrategy};
use headwind_core::{ColorMode, CssVariableMode, NamingMode, UnknownClassMode};
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
    /// CSS 变量模式
    css_variables: CssVariableMode,
    /// 未知类名处理模式
    unknown_class_mode: UnknownClassMode,
}

impl ClassCollector {
    pub fn new(
        naming_mode: NamingMode,
        css_variables: CssVariableMode,
        unknown_class_mode: UnknownClassMode,
        color_mode: ColorMode,
    ) -> Self {
        let naming = create_naming_strategy(naming_mode);
        let bundler = match css_variables {
            CssVariableMode::Var => Bundler::new(),
            CssVariableMode::Inline => Bundler::with_inline(),
        }
        .with_color_mode(color_mode);
        Self {
            bundler,
            naming,
            class_map: IndexMap::new(),
            css_entries: Vec::new(),
            indent: "  ".to_string(),
            css_variables,
            unknown_class_mode,
        }
    }

    /// 处理一组 Tailwind 类，返回生成的类名。
    /// 如果该类组合已处理过，直接返回缓存结果。
    ///
    /// Preserve 模式下，未识别的类名会保留在输出中：
    /// - 全部未识别 → 原样返回
    /// - 部分识别 → `"生成名 unknown1 unknown2"`
    pub fn process_classes(&mut self, classes: &str) -> String {
        let trimmed = classes.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // 缓存命中
        if let Some(name) = self.class_map.get(trimmed) {
            return name.clone();
        }

        if self.unknown_class_mode == UnknownClassMode::Preserve {
            // 分离已识别和未识别的类
            let mut recognized = Vec::new();
            let mut unrecognized = Vec::new();
            for class in trimmed.split_whitespace() {
                if self.bundler.is_recognized(class) {
                    recognized.push(class.to_string());
                } else {
                    unrecognized.push(class.to_string());
                }
            }

            // 全部未识别 → 原样返回
            if recognized.is_empty() {
                self.class_map.insert(trimmed.to_string(), trimmed.to_string());
                return trimmed.to_string();
            }

            // 仅从已识别的类生成名称和 CSS
            let recognized_str = recognized.join(" ");
            let new_name = self.naming.generate_name(&recognized);

            match self.bundler.bundle_to_css(&new_name, &recognized_str, &self.indent) {
                Ok(css) if !css.is_empty() => {
                    self.css_entries.push(css);
                }
                _ => {}
            }

            // 合并：生成名 + 未识别类
            let result = if unrecognized.is_empty() {
                new_name
            } else {
                format!("{} {}", new_name, unrecognized.join(" "))
            };

            self.class_map.insert(trimmed.to_string(), result.clone());
            result
        } else {
            // Remove 模式：原始行为
            let class_list: Vec<String> = trimmed.split_whitespace().map(|s| s.to_string()).collect();
            let new_name = self.naming.generate_name(&class_list);

            match self.bundler.bundle_to_css(&new_name, trimmed, &self.indent) {
                Ok(css) if !css.is_empty() => {
                    self.css_entries.push(css);
                }
                _ => {}
            }

            self.class_map.insert(trimmed.to_string(), new_name.clone());
            new_name
        }
    }

    /// 返回合并后的 CSS 输出
    ///
    /// Var 模式下自动在顶部插入 `:root { ... }` 主题变量定义。
    pub fn combined_css(&self) -> String {
        let css = self.css_entries.join("\n");
        if self.css_variables == CssVariableMode::Var && !css.is_empty() {
            let root = self.bundler.generate_root_css(&css);
            if root.is_empty() {
                css
            } else {
                format!("{}\n{}", root, css)
            }
        } else {
            css
        }
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
        let mut collector = ClassCollector::new(NamingMode::Hash, CssVariableMode::Var, UnknownClassMode::Remove, ColorMode::default());
        let name = collector.process_classes("p-4 m-2");
        assert!(name.starts_with("c_"));
        assert!(!collector.combined_css().is_empty());
    }

    #[test]
    fn test_process_classes_caching() {
        let mut collector = ClassCollector::new(NamingMode::Hash, CssVariableMode::Var, UnknownClassMode::Remove, ColorMode::default());
        let name1 = collector.process_classes("p-4 m-2");
        let name2 = collector.process_classes("p-4 m-2");
        assert_eq!(name1, name2);
        // CSS 应该只生成一次
        assert_eq!(collector.css_entries.len(), 1);
    }

    #[test]
    fn test_process_classes_different_inputs() {
        let mut collector = ClassCollector::new(NamingMode::Hash, CssVariableMode::Var, UnknownClassMode::Remove, ColorMode::default());
        let name1 = collector.process_classes("p-4");
        let name2 = collector.process_classes("m-2");
        assert_ne!(name1, name2);
        assert_eq!(collector.css_entries.len(), 2);
    }

    #[test]
    fn test_process_empty_classes() {
        let mut collector = ClassCollector::new(NamingMode::Hash, CssVariableMode::Var, UnknownClassMode::Remove, ColorMode::default());
        let name = collector.process_classes("  ");
        assert!(name.is_empty());
    }

    #[test]
    fn test_readable_naming() {
        let mut collector = ClassCollector::new(NamingMode::Readable, CssVariableMode::Var, UnknownClassMode::Remove, ColorMode::default());
        let name = collector.process_classes("p-4 m-2");
        assert_eq!(name, "p4_m2");
    }
}
