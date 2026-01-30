use crate::types::NamingMode;

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
    ///
    /// 例如：
    /// - "p-4" → "p4"
    /// - "m-2" → "m2"
    /// - "text-red-500" → "text_red"
    fn extract_prefix(class: &str) -> String {
        // 移除连字符，限制长度
        let cleaned = class.replace('-', "");

        // 如果太长，只取前 8 个字符
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

        // 连接，用下划线分隔
        let combined = prefixes.join("_");

        // 限制总长度，避免过长
        if combined.len() > 32 {
            // 截断并添加 hash 后缀
            let truncated = &combined[..24];
            let hash = blake3::hash(combined.as_bytes());
            let hex = format!("{}", hash);
            format!("{}_{}", truncated, &hex[..6])
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
        NamingMode::Semantic => {
            // 未来实现 AI 命名
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
        // 应该被截断并添加 hash
        assert!(name.len() <= 32);
    }

    #[test]
    fn test_readable_naming_empty() {
        let naming = ReadableNaming;
        let classes: Vec<String> = vec![];

        let name = naming.generate_name(&classes);
        assert_eq!(name, "empty");
    }
}
