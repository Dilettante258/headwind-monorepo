use headwind_core::NamingMode;

/// 命名策略 trait
pub trait NamingStrategy {
    fn generate_name(&self, classes: &[String]) -> String;
}

/// Hash 命名策略：基于类名内容生成稳定 hash
pub struct HashNaming;

impl NamingStrategy for HashNaming {
    fn generate_name(&self, classes: &[String]) -> String {
        // 将所有类名连接，用空格分隔（因为已经规范化过）
        let input = classes.join(" ");

        // 使用 blake3 计算 hash
        let hash = blake3::hash(input.as_bytes());

        // 取前 6 个字节的十六进制表示
        let hex = format!("{}", hash);
        let short_hash = &hex[..12];

        format!("c_{}", short_hash)
    }
}

/// Readable 命名策略：组合类名前缀生成可读名称
pub struct ReadableNaming;

impl ReadableNaming {
    /// 从 Tailwind 类名中提取可读前缀
    fn extract_prefix(class: &str) -> String {
        let cleaned = class.replace('-', "");

        if cleaned.len() > 8 {
            cleaned[..8].to_string()
        } else {
            cleaned
        }
    }
}

impl NamingStrategy for ReadableNaming {
    fn generate_name(&self, classes: &[String]) -> String {
        if classes.is_empty() {
            return "empty".to_string();
        }

        let prefixes: Vec<String> = classes.iter().map(|c| Self::extract_prefix(c)).collect();

        let combined = prefixes.join("_");

        if combined.len() > 32 {
            let truncated = &combined[..24];
            let hash = blake3::hash(combined.as_bytes());
            let hex = format!("{}", hash);
            format!("{}_{}", truncated, &hex[..6])
        } else {
            combined
        }
    }
}

/// CamelCase 命名策略：生成驼峰式类名，适合 CSS Modules 的 `styles.xxx` 访问
pub struct CamelCaseNaming;

impl CamelCaseNaming {
    /// 将单个 Tailwind 类转换为 camelCase 片段
    fn class_to_camel(class: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;

        for ch in class.chars() {
            if ch == '-' || ch == ':' {
                capitalize_next = true;
            } else if capitalize_next {
                result.extend(ch.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }

        result
    }
}

impl NamingStrategy for CamelCaseNaming {
    fn generate_name(&self, classes: &[String]) -> String {
        if classes.is_empty() {
            return "empty".to_string();
        }

        let mut combined = String::new();

        for (i, class) in classes.iter().enumerate() {
            let camel = Self::class_to_camel(class);
            if i == 0 {
                combined.push_str(&camel);
            } else {
                let mut chars = camel.chars();
                if let Some(first) = chars.next() {
                    combined.extend(first.to_uppercase());
                    combined.push_str(chars.as_str());
                }
            }
        }

        if combined.len() > 32 {
            let truncated = &combined[..24];
            let hash = blake3::hash(combined.as_bytes());
            let hex = format!("{}", hash);
            format!("{}{}", truncated, &hex[..6])
        } else {
            combined
        }
    }
}

/// 根据 NamingMode 创建对应的策略
pub fn create_naming_strategy(mode: NamingMode) -> Box<dyn NamingStrategy> {
    match mode {
        NamingMode::Hash => Box::new(HashNaming),
        NamingMode::Readable => Box::new(ReadableNaming),
        NamingMode::CamelCase => Box::new(CamelCaseNaming),
        NamingMode::Semantic => {
            unimplemented!("Semantic naming not yet implemented")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_naming_stability() {
        let naming = HashNaming;
        let classes = vec!["p-4".to_string(), "m-2".to_string()];

        let name1 = naming.generate_name(&classes);
        let name2 = naming.generate_name(&classes);

        assert_eq!(name1, name2, "Hash naming should be stable");
        assert!(name1.starts_with("c_"), "Hash name should start with c_");
        assert_eq!(name1.len(), 14, "Hash name should be c_ + 12 chars");
    }

    #[test]
    fn test_hash_naming_different_inputs() {
        let naming = HashNaming;
        let classes1 = vec!["p-4".to_string(), "m-2".to_string()];
        let classes2 = vec!["p-8".to_string(), "m-4".to_string()];

        let name1 = naming.generate_name(&classes1);
        let name2 = naming.generate_name(&classes2);

        assert_ne!(
            name1, name2,
            "Different inputs should produce different hashes"
        );
    }

    #[test]
    fn test_readable_naming() {
        let naming = ReadableNaming;
        let classes = vec!["p-4".to_string(), "m-2".to_string()];

        let name = naming.generate_name(&classes);
        assert_eq!(name, "p4_m2");
    }

    #[test]
    fn test_readable_naming_long_classes() {
        let naming = ReadableNaming;
        let classes = vec![
            "p-4".to_string(),
            "m-2".to_string(),
            "text-red-500".to_string(),
            "bg-blue-600".to_string(),
            "border-gray-300".to_string(),
        ];

        let name = naming.generate_name(&classes);
        assert!(name.len() <= 32);
    }

    #[test]
    fn test_readable_naming_empty() {
        let naming = ReadableNaming;
        let classes: Vec<String> = vec![];

        let name = naming.generate_name(&classes);
        assert_eq!(name, "empty");
    }

    #[test]
    fn test_camel_case_naming_basic() {
        let naming = CamelCaseNaming;
        let classes = vec!["p-4".to_string(), "m-2".to_string()];

        let name = naming.generate_name(&classes);
        assert_eq!(name, "p4M2");
    }

    #[test]
    fn test_camel_case_naming_complex() {
        let naming = CamelCaseNaming;
        let classes = vec!["text-center".to_string(), "bg-blue-500".to_string()];

        let name = naming.generate_name(&classes);
        assert_eq!(name, "textCenterBgBlue500");
    }

    #[test]
    fn test_camel_case_naming_with_modifiers() {
        let naming = CamelCaseNaming;
        let classes = vec![
            "text-center".to_string(),
            "hover:text-left".to_string(),
        ];

        let name = naming.generate_name(&classes);
        assert_eq!(name, "textCenterHoverTextLeft");
    }

    #[test]
    fn test_camel_case_naming_single() {
        let naming = CamelCaseNaming;
        let classes = vec!["flex".to_string()];

        let name = naming.generate_name(&classes);
        assert_eq!(name, "flex");
    }

    #[test]
    fn test_camel_case_naming_long() {
        let naming = CamelCaseNaming;
        let classes = vec![
            "bg-blue-500".to_string(),
            "text-white".to_string(),
            "hover:bg-blue-700".to_string(),
            "font-bold".to_string(),
            "px-4".to_string(),
            "py-2".to_string(),
            "rounded".to_string(),
        ];

        let name = naming.generate_name(&classes);
        assert!(name.len() <= 32);
    }

    #[test]
    fn test_camel_case_naming_empty() {
        let naming = CamelCaseNaming;
        let classes: Vec<String> = vec![];

        let name = naming.generate_name(&classes);
        assert_eq!(name, "empty");
    }
}
