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

/// 官方映射格式（来自 official-mappings.json）
#[derive(Debug, Deserialize)]
struct OfficialMapping {
    class: String,
    css: String,
    #[allow(dead_code)]
    source: Option<String>,
}

/// 解析 CSS 声明字符串，如 "text-indent: -1px" 或 "padding: 1rem; margin: 2rem"
///
/// 返回解析出的 Declaration 列表
fn parse_css_declarations(css: &str) -> Vec<Declaration> {
    let mut declarations = Vec::new();

    // 按分号分割多个声明
    for decl_str in css.split(';') {
        let decl_str = decl_str.trim();
        if decl_str.is_empty() {
            continue;
        }

        // 查找冒号分隔属性和值
        if let Some(colon_pos) = decl_str.find(':') {
            let property = decl_str[..colon_pos].trim();
            let value = decl_str[colon_pos + 1..].trim();

            if !property.is_empty() && !value.is_empty() {
                declarations.push(Declaration::new(property, value));
            }
        }
    }

    declarations
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

/// 从官方映射 JSON 字符串加载 Tailwind 索引
///
/// JSON 格式示例（来自 official-mappings.json）：
/// ```json
/// [
///   {
///     "class": "p-4",
///     "css": "padding: 1rem",
///     "source": "/src/docs/padding.mdx"
///   }
/// ]
/// ```
pub fn load_from_official_json(json_str: &str) -> Result<TailwindIndex, serde_json::Error> {
    let mappings: Vec<OfficialMapping> = serde_json::from_str(json_str)?;

    let mut index = TailwindIndex::new();

    for mapping in mappings {
        let declarations = parse_css_declarations(&mapping.css);

        // 只添加非空的映射
        if !declarations.is_empty() {
            index.insert(mapping.class, declarations);
        }
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

    #[test]
    fn test_parse_css_declarations_single() {
        let css = "text-indent: -1px";
        let decls = parse_css_declarations(css);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "text-indent");
        assert_eq!(decls[0].value, "-1px");
    }

    #[test]
    fn test_parse_css_declarations_multiple() {
        let css = "padding: 1rem; margin: 2rem";
        let decls = parse_css_declarations(css);
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "padding");
        assert_eq!(decls[0].value, "1rem");
        assert_eq!(decls[1].property, "margin");
        assert_eq!(decls[1].value, "2rem");
    }

    #[test]
    fn test_parse_css_declarations_with_trailing_semicolon() {
        let css = "position: absolute;";
        let decls = parse_css_declarations(css);
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, "position");
        assert_eq!(decls[0].value, "absolute");
    }

    #[test]
    fn test_parse_css_declarations_with_whitespace() {
        let css = "  padding-top  :  1rem  ;  padding-bottom  :  2rem  ";
        let decls = parse_css_declarations(css);
        assert_eq!(decls.len(), 2);
        assert_eq!(decls[0].property, "padding-top");
        assert_eq!(decls[0].value, "1rem");
        assert_eq!(decls[1].property, "padding-bottom");
        assert_eq!(decls[1].value, "2rem");
    }

    #[test]
    fn test_load_from_official_json() {
        let json = r#"[
            {
                "class": "absolute",
                "css": "position: absolute",
                "source": "/src/docs/position.mdx"
            },
            {
                "class": "-indent-px",
                "css": "text-indent: -1px",
                "source": "/src/docs/text-indent.mdx"
            }
        ]"#;

        let index = load_from_official_json(json).unwrap();

        assert_eq!(index.len(), 2);

        let absolute_decls = index.lookup("absolute").unwrap();
        assert_eq!(absolute_decls.len(), 1);
        assert_eq!(absolute_decls[0].property, "position");
        assert_eq!(absolute_decls[0].value, "absolute");

        let indent_decls = index.lookup("-indent-px").unwrap();
        assert_eq!(indent_decls.len(), 1);
        assert_eq!(indent_decls[0].property, "text-indent");
        assert_eq!(indent_decls[0].value, "-1px");
    }
}
