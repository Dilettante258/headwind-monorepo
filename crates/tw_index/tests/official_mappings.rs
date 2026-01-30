use headwind_tw_index::{load_from_official_json, Converter};
use headwind_tw_parse::parse_class;

#[test]
fn test_load_official_mappings_fixture() {
    let json = include_str!("../fixtures/official-mappings.json");
    let index = load_from_official_json(json).expect("Failed to load official mappings");

    // 验证加载了正确数量的映射
    println!("Loaded {} class mappings", index.len());
    assert!(index.len() > 700, "Expected at least 700 mappings, got {}", index.len());

    // 测试一些已知的类
    let absolute = index.lookup("absolute");
    assert!(absolute.is_some(), "Should find 'absolute' class");
    let decls = absolute.unwrap();
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].property, "position");
    assert_eq!(decls[0].value, "absolute");

    // 测试负值类
    let indent = index.lookup("-indent-px");
    assert!(indent.is_some(), "Should find '-indent-px' class");
    let decls = indent.unwrap();
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].property, "text-indent");
    assert_eq!(decls[0].value, "-1px");

    // 测试带变量的类
    let translate_x = index.lookup("-translate-x-px");
    assert!(translate_x.is_some(), "Should find '-translate-x-px' class");
    let decls = translate_x.unwrap();
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].property, "translate");
    assert!(decls[0].value.contains("var(--tw-translate-y)"));
}

#[test]
fn test_official_mappings_coverage() {
    let json = include_str!("../fixtures/official-mappings.json");
    let index = load_from_official_json(json).expect("Failed to load official mappings");

    // 统计各类 CSS 属性的覆盖情况
    let all_classes = index.classes();

    // 检查是否包含一些已知存在的类
    let known_classes = [
        "absolute", "relative",
        "text-center", "text-left", "text-right",
        "antialiased",
    ];

    for class_name in &known_classes {
        assert!(
            all_classes.contains(class_name),
            "Should contain class: {}",
            class_name
        );
    }

    // 检查是否包含负值类
    assert!(all_classes.contains(&"-indent-px"));
    assert!(all_classes.contains(&"-translate-x-full"));
}

#[test]
fn test_validate_all_official_mappings() {
    let json = include_str!("../fixtures/official-mappings.json");
    let index = load_from_official_json(json).expect("Failed to load official mappings");
    let converter = Converter::new(&index);

    let all_classes = index.classes();

    let mut parse_errors = Vec::new();
    let mut convert_errors = Vec::new();
    let mut success_count = 0;

    println!("\n🔍 Validating {} official Tailwind classes...\n", all_classes.len());

    for class_name in &all_classes {
        // 尝试解析类名
        match parse_class(class_name) {
            Ok(parsed) => {
                // 尝试转换为 CSS
                match converter.convert(&parsed) {
                    Some(_rule) => {
                        success_count += 1;
                    }
                    None => {
                        convert_errors.push(format!("{} - parsed but failed to convert", class_name));
                    }
                }
            }
            Err(e) => {
                parse_errors.push(format!("{} - {:?}", class_name, e));
            }
        }
    }

    // 打印统计信息
    println!("✅ Successfully validated: {}/{}", success_count, all_classes.len());

    if !parse_errors.is_empty() {
        println!("\n❌ Parse errors ({}):", parse_errors.len());
        for error in parse_errors.iter().take(10) {
            println!("   {}", error);
        }
        if parse_errors.len() > 10 {
            println!("   ... and {} more", parse_errors.len() - 10);
        }
    }

    if !convert_errors.is_empty() {
        println!("\n⚠️  Convert errors ({}):", convert_errors.len());
        for error in convert_errors.iter().take(10) {
            println!("   {}", error);
        }
        if convert_errors.len() > 10 {
            println!("   ... and {} more", convert_errors.len() - 10);
        }
    }

    // 断言没有解析错误
    assert!(
        parse_errors.is_empty(),
        "Found {} parse errors in official mappings",
        parse_errors.len()
    );

    // 断言所有类都能成功转换
    assert!(
        convert_errors.is_empty(),
        "Found {} convert errors in official mappings",
        convert_errors.len()
    );

    println!("\n🎉 All {} official classes validated successfully!\n", all_classes.len());
}
