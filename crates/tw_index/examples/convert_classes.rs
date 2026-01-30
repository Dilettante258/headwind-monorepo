use headwind_tw_index::{load_from_official_json, Converter};
use headwind_tw_parse::parse_class;

fn main() {
    // 1. 加载官方映射
    println!("📚 Loading official Tailwind CSS mappings...\n");
    let json = include_str!("../../tw_index/fixtures/official-mappings.json");
    let index = load_from_official_json(json).expect("Failed to load mappings");
    println!("✓ Loaded {} class mappings\n", index.len());

    // 2. 创建转换器
    let converter = Converter::new(&index);

    // 3. 测试各种类名
    let test_cases = vec![
        // 简单类
        "absolute",
        "text-center",
        // 带修饰符
        "hover:text-center",
        "md:hover:text-center",
        // 任意值
        "w-[13px]",
        "px-[2rem]",
        "text-[#1da1f2]",
        // 任意值 + 修饰符
        "hover:w-[13px]",
        "md:px-[2rem]",
        // important
        "text-center!",
        "hover:text-center!",
        // 负值
        "-indent-px",
        // 变量
        "-translate-x-px",
    ];

    println!("🔄 Converting Tailwind classes to CSS...\n");
    println!("{}", "=".repeat(80));

    for class_name in test_cases {
        println!("\n📝 Input: {}", class_name);

        match parse_class(class_name) {
            Ok(parsed) => {
                println!("   Parsed: {:?}", parsed);

                match converter.convert(&parsed) {
                    Some(rule) => {
                        println!("   ✅ CSS:");
                        println!("      Selector: {}", rule.selector);
                        for decl in &rule.declarations {
                            println!("      {}: {}", decl.property, decl.value);
                        }
                    }
                    None => {
                        println!("   ⚠️  Not found in index (and not an arbitrary value)");
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Parse error: {:?}", e);
            }
        }
    }

    println!("\n{}", "=".repeat(80));
    println!("\n✨ Done!");
}
