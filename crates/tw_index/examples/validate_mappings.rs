use headwind_tw_index::{load_from_official_json, Converter};
use headwind_tw_parse::parse_class;
use std::collections::HashMap;

fn main() {
    println!("🔍 Validating official Tailwind CSS mappings\n");
    println!("{}\n", "=".repeat(80));

    // 加载官方映射（用于验证）
    let json = include_str!("../../tw_index/fixtures/official-mappings.json");
    let index = load_from_official_json(json).expect("Failed to load mappings");

    // 使用基于规则的转换器
    let converter = Converter::new();

    println!("📚 Loaded {} official class mappings for validation\n", index.len());
    println!("🔧 Using rule-based converter (not index lookup)\n");

    // 统计信息
    let all_classes = index.classes();
    let mut stats = HashMap::new();
    let mut success = 0;
    let mut errors = Vec::new();

    // 验证每个类
    for class_name in &all_classes {
        match parse_class(class_name) {
            Ok(parsed) => {
                if converter.convert(&parsed).is_some() {
                    success += 1;
                    // 统计插件使用情况
                    *stats.entry(parsed.plugin.clone()).or_insert(0) += 1;
                } else {
                    errors.push(class_name);
                }
            }
            Err(_) => {
                errors.push(class_name);
            }
        }
    }

    // 打印验证结果
    println!("✅ Validation Results:");
    println!("   Total classes: {}", all_classes.len());
    println!("   Successfully validated: {}", success);
    println!("   Errors: {}", errors.len());
    println!("   Success rate: {:.1}%\n", (success as f64 / all_classes.len() as f64) * 100.0);

    // 打印插件统计（前 20 个最常用的）
    println!("📊 Top 20 Most Common Plugins:");
    let mut sorted_stats: Vec<_> = stats.iter().collect();
    sorted_stats.sort_by(|a, b| b.1.cmp(a.1));

    for (i, (plugin, count)) in sorted_stats.iter().take(20).enumerate() {
        println!("   {:2}. {:20} ({:3} classes)", i + 1, plugin, count);
    }

    // 展示一些示例
    println!("\n🎯 Example Validations:");
    let examples = [
        "absolute",
        "relative",
        "text-center",
        "-indent-px",
        "-translate-x-full",
        "antialiased",
        "text-left",
        "align-baseline",
    ];

    for class_name in &examples {
        if let Ok(parsed) = parse_class(class_name) {
            if let Some(rule) = converter.convert(&parsed) {
                println!("\n   ✓ {}", class_name);
                println!("     Selector: {}", rule.selector);
                for decl in &rule.declarations {
                    println!("     {}: {}", decl.property, decl.value);
                }
            }
        }
    }

    println!("\n{}", "=".repeat(80));

    if errors.is_empty() {
        println!("\n🎉 All mappings validated successfully!");
    } else {
        println!("\n⚠️  Found {} errors", errors.len());
        std::process::exit(1);
    }
}
