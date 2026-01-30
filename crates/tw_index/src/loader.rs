use crate::index::TailwindIndex;
use headwind_core::Declaration;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ClassMapping {
    class: String,
    declarations: Vec<DeclarationJson>,
}

#[derive(Debug, Deserialize)]
struct DeclarationJson {
    property: String,
    value: String,
}

/// 从 JSON 字符串加载 Tailwind 索引
///
/// JSON 格式示例：
/// ```json
/// [
///   {
///     "class": "p-4",
///     "declarations": [
///       { "property": "padding", "value": "1rem" }
///     ]
///   }
/// ]
/// ```
pub fn load_from_json(json_str: &str) -> Result<TailwindIndex, serde_json::Error> {
    let mappings: Vec<ClassMapping> = serde_json::from_str(json_str)?;

    let mut index = TailwindIndex::new();

    for mapping in mappings {
        let declarations: Vec<Declaration> = mapping
            .declarations
            .into_iter()
            .map(|d| Declaration::new(d.property, d.value))
            .collect();

        index.insert(mapping.class, declarations);
    }

    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_json_basic() {
        let json = r#"[
            {
                "class": "p-4",
                "declarations": [
                    { "property": "padding", "value": "1rem" }
                ]
            },
            {
                "class": "m-2",
                "declarations": [
                    { "property": "margin", "value": "0.5rem" }
                ]
            }
        ]"#;

        let index = load_from_json(json).unwrap();

        assert_eq!(index.len(), 2);
        assert!(index.lookup("p-4").is_some());
        assert!(index.lookup("m-2").is_some());

        let p4_decls = index.lookup("p-4").unwrap();
        assert_eq!(p4_decls.len(), 1);
        assert_eq!(p4_decls[0].property, "padding");
        assert_eq!(p4_decls[0].value, "1rem");
    }

    #[test]
    fn test_load_from_json_multiple_declarations() {
        let json = r#"[
            {
                "class": "p-4",
                "declarations": [
                    { "property": "padding-top", "value": "1rem" },
                    { "property": "padding-bottom", "value": "1rem" }
                ]
            }
        ]"#;

        let index = load_from_json(json).unwrap();
        let decls = index.lookup("p-4").unwrap();
        assert_eq!(decls.len(), 2);
    }

    #[test]
    fn test_load_from_json_invalid() {
        let json = "invalid json";
        let result = load_from_json(json);
        assert!(result.is_err());
    }
}
