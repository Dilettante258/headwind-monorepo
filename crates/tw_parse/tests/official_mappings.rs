use headwind_tw_parse::parse_class;
use serde::Deserialize;

#[derive(Deserialize)]
struct OfficialMapping {
    class: String,
    #[allow(dead_code)]
    css: String,
    #[allow(dead_code)]
    source: String,
}

#[test]
fn test_parse_official_mappings() {
    // 读取官方映射文件
    let json = include_str!("../../tw_index/fixtures/official-mappings.json");
    let mappings: Vec<OfficialMapping> =
        serde_json::from_str(json).expect("Failed to parse JSON");

    println!("Testing {} official Tailwind classes", mappings.len());

    let mut failed = Vec::new();

    for mapping in &mappings {
        match parse_class(&mapping.class) {
            Ok(parsed) => {
                // 验证规范化后的字符串匹配原始 class
                let normalized = parsed.to_normalized_string();
                if normalized != mapping.class {
                    failed.push(format!(
                        "  ✗ {}: normalized to '{}' (expected '{}')",
                        mapping.class, normalized, mapping.class
                    ));
                }
            }
            Err(err) => {
                failed.push(format!("  ✗ {}: parse error - {}", mapping.class, err));
            }
        }
    }

    if !failed.is_empty() {
        eprintln!("\nFailed to parse {} classes:", failed.len());
        for msg in &failed {
            eprintln!("{}", msg);
        }
        panic!("{} classes failed to parse correctly", failed.len());
    }

    println!("✓ All {} classes parsed successfully", mappings.len());
}

#[test]
fn test_parse_specific_classes() {
    // 测试一些特定的常见 class
    let test_cases = vec![
        ("p-4", "p", Some("4")),
        ("m-2", "m", Some("2")),
        ("w-full", "w", Some("full")),
        ("bg-black", "bg", Some("black")),
        ("text-sm", "text", Some("sm")),
        ("flex", "flex", None),
        ("hidden", "hidden", None),
    ];

    for (class, expected_plugin, expected_value) in test_cases {
        let parsed = parse_class(class).unwrap_or_else(|_| panic!("Failed to parse: {}", class));

        assert_eq!(
            parsed.plugin, expected_plugin,
            "Plugin mismatch for '{}'",
            class
        );

        match (parsed.value.as_ref(), expected_value) {
            (Some(val), Some(exp)) => {
                assert_eq!(
                    val.to_string(),
                    exp,
                    "Value mismatch for '{}'",
                    class
                );
            }
            (None, None) => {}
            (got, expected) => {
                panic!(
                    "Value mismatch for '{}': got {:?}, expected {:?}",
                    class, got, expected
                );
            }
        }
    }
}
