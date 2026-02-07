use headwind_core::{BundleRequest, NamingMode};
use headwind_tw_index::bundle::bundle;
use headwind_tw_index::css::{create_stylesheet, emit_css};
use headwind_tw_index::load_from_json;

#[test]
fn test_end_to_end_with_json_index() {
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

    let request = BundleRequest {
        classes: vec![
            "p-4".to_string(),
            "m-2".to_string(),
            "text-red-500".to_string(),
        ],
        naming_mode: NamingMode::Hash,
    };

    let result = bundle(request, &index);

    assert!(result.new_class.starts_with("c_"));
    assert_eq!(result.css_declarations.len(), 3);
    assert!(result.diagnostics.is_empty());
    assert!(result.removed.is_empty());

    let props: Vec<&str> = result
        .css_declarations
        .iter()
        .map(|d| d.property.as_str())
        .collect();
    assert!(props.contains(&"padding"));
    assert!(props.contains(&"margin"));
    assert!(props.contains(&"color"));
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

    assert_eq!(result.new_class, "m2_p4");
    assert_eq!(result.css_declarations.len(), 2);
}

#[test]
fn test_end_to_end_with_css_output() {
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

    let stylesheet = create_stylesheet(result.new_class.clone(), result.css_declarations);
    let css = emit_css(&stylesheet).expect("Failed to emit CSS");

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
            "p-4".to_string(),
            "unknown-class".to_string(),
        ],
        naming_mode: NamingMode::Hash,
    };

    let result = bundle(request, &index);

    assert_eq!(result.css_declarations.len(), 1);
    assert_eq!(result.removed.len(), 1);
    assert_eq!(result.removed[0], "unknown-class");
    assert_eq!(result.diagnostics.len(), 1);
}
