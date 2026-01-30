use headwind_core::{bundle::bundle, BundleRequest, NamingMode};
use headwind_tw_index::{load_from_json};

#[test]
fn test_end_to_end_with_json_index() {
    // 1. 准备 Tailwind 索引（从 JSON 加载）
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
        },
        {
            "class": "text-red-500",
            "declarations": [
                { "property": "color", "value": "rgb(239, 68, 68)" }
            ]
        }
    ]"#;

    let index = load_from_json(json).expect("Failed to load JSON index");

    // 2. 创建 bundle 请求
    let request = BundleRequest {
        classes: vec![
            "p-4".to_string(),
            "m-2".to_string(),
            "text-red-500".to_string(),
        ],
        naming_mode: NamingMode::Hash,
    };

    // 3. 执行 bundle
    let result = bundle(request, &index);

    // 4. 验证结果
    assert!(result.new_class.starts_with("c_"));
    assert_eq!(result.css_declarations.len(), 3);
    assert!(result.diagnostics.is_empty());
    assert!(result.removed.is_empty());

    // 验证 CSS 声明（按属性名排序）
    let props: Vec<&str> = result
        .css_declarations
        .iter()
        .map(|d| d.property.as_str())
        .collect();
    assert!(props.contains(&"padding"));
    assert!(props.contains(&"margin"));
    assert!(props.contains(&"color"));

    println!("Generated class: {}", result.new_class);
    println!("CSS declarations: {:?}", result.css_declarations);
}

#[test]
fn test_end_to_end_with_readable_naming() {
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

    let index = load_from_json(json).expect("Failed to load JSON index");

    let request = BundleRequest {
        classes: vec!["p-4".to_string(), "m-2".to_string()],
        naming_mode: NamingMode::Readable,
    };

    let result = bundle(request, &index);

    // Readable 命名应该生成可读的类名
    assert_eq!(result.new_class, "m2_p4");
    assert_eq!(result.css_declarations.len(), 2);
}

#[test]
fn test_end_to_end_with_css_output() {
    use headwind_css::{create_stylesheet, emit_css};

    let json = r#"[
        {
            "class": "p-4",
            "declarations": [
                { "property": "padding", "value": "1rem" }
            ]
        }
    ]"#;

    let index = load_from_json(json).expect("Failed to load JSON index");

    let request = BundleRequest {
        classes: vec!["p-4".to_string()],
        naming_mode: NamingMode::Hash,
    };

    let result = bundle(request, &index);

    // 生成 CSS
    let stylesheet = create_stylesheet(result.new_class.clone(), result.css_declarations);
    let css = emit_css(&stylesheet).expect("Failed to emit CSS");

    println!("Generated CSS:\n{}", css);

    // 验证 CSS 输出
    assert!(css.contains(&result.new_class));
    assert!(css.contains("padding"));
    assert!(css.contains("1rem"));
}

#[test]
fn test_end_to_end_with_duplicates_and_unknowns() {
    let json = r#"[
        {
            "class": "p-4",
            "declarations": [
                { "property": "padding", "value": "1rem" }
            ]
        }
    ]"#;

    let index = load_from_json(json).expect("Failed to load JSON index");

    let request = BundleRequest {
        classes: vec![
            "p-4".to_string(),
            "p-4".to_string(), // 重复
            "unknown-class".to_string(), // 未知
        ],
        naming_mode: NamingMode::Hash,
    };

    let result = bundle(request, &index);

    // 重复的类应该被去重
    assert_eq!(result.css_declarations.len(), 1);

    // 未知类应该在 removed 中
    assert_eq!(result.removed.len(), 1);
    assert_eq!(result.removed[0], "unknown-class");

    // 应该有一个警告
    assert_eq!(result.diagnostics.len(), 1);
}
