/// 演示批量解析优化
///
/// 展示新的 parse_classes 函数如何一次性处理多个类名

use headwind_tw_parse::{parse_class, parse_classes};

fn main() {
    println!("=== Tailwind 批量解析示例 ===\n");

    // 示例 1: 单个类解析（特例）
    println!("1. 单个类解析:");
    let single = parse_class("md:hover:p-4").unwrap();
    println!("   类名: md:hover:p-4");
    println!("   插件: {}", single.plugin);
    println!("   原始修饰符: {:?}", single.raw_modifiers);
    println!("   修饰符数量: {}", single.modifiers().len());
    println!();

    // 示例 2: 批量解析（优化后的主要方式）
    println!("2. 批量解析多个类:");
    let classes = "p-4 hover:bg-blue-500 md:text-center -m-2 w-[13px]";
    let parsed = parse_classes(classes).unwrap();

    println!("   输入: {}", classes);
    println!("   解析出 {} 个类:\n", parsed.len());

    for (i, p) in parsed.iter().enumerate() {
        println!("   [{}] {}", i + 1, p.plugin);
        println!("       原始修饰符: {:?}", p.raw_modifiers);
        println!("       修饰符: {:?}", p.modifiers());
        if let Some(value) = &p.value {
            println!("       值: {}", value);
        }
        if p.negative {
            println!("       负值: true");
        }
        println!();
    }

    // 示例 3: 性能优势展示
    println!("3. 性能优势:");
    let complex_classes = "md:hover:p-4 lg:focus:m-8 dark:bg-blue-500 \
                          xl:group-hover:text-white sm:border-2 \
                          2xl:opacity-50 md:w-full lg:h-screen";

    let parsed = parse_classes(complex_classes).unwrap();
    println!("   一次性解析 {} 个复杂类", parsed.len());
    println!("   所有类都包含原始修饰符信息");

    for p in &parsed {
        if !p.raw_modifiers.is_empty() {
            println!("   - {}: raw_modifiers = {:?}",
                     p.plugin, p.raw_modifiers);
        }
    }

    println!("\n=== 优化说明 ===");
    println!("1. parse_classes 一次性处理整个字符串");
    println!("2. 自动按空白分割并解析每个类");
    println!("3. raw_modifiers 字段保留原始修饰符字符串");
    println!("4. 减少了重复的字符串处理和内存分配");
}
