use headwind_tw_index::Bundler;

fn main() {
    println!("🎨 Tailwind CSS 类打包器示例\n");
    println!("{}\n", "=".repeat(80));

    // 创建打包器（基于规则系统，无需官方映射）
    let bundler = Bundler::new();
    println!("✅ 使用基于规则的转换器（plugin_map + value_map）\n");

    // 测试用例
    let test_cases = vec![
        (
            "simple",
            "text-center p-4",
            "基础类（无修饰符）",
        ),
        (
            "with-hover",
            "text-center hover:text-left p-4 hover:p-8",
            "带 hover 伪类",
        ),
        (
            "responsive",
            "text-center md:text-right lg:text-left",
            "响应式修饰符",
        ),
        (
            "complex",
            "text-center hover:text-left md:text-right p-4 md:p-8 lg:p-12 hover:bg-blue-500",
            "复杂组合（响应式 + 伪类）",
        ),
        (
            "dark-mode",
            "text-black dark:text-white",
            "暗色模式",
        ),
        (
            "before-after",
            "before:content-none after:content-none",
            "伪元素",
        ),
        (
            "group-hover",
            "text-center group-hover:text-left",
            "组状态",
        ),
        (
            "everything",
            "text-center hover:text-left focus:text-right md:text-left md:hover:text-right lg:text-right p-4 md:p-8 lg:p-12",
            "所有特性组合",
        ),
    ];

    for (class_name, classes, description) in test_cases {
        println!("📝 测试: {}", description);
        println!("   输入: {}", classes);
        println!("   类名: .{}\n", class_name);

        match bundler.bundle(classes) {
            Ok(group) => {
                let css = bundler.generate_css(class_name, &group, "  ");
                println!("   生成的 CSS:\n");

                // 添加缩进
                for line in css.lines() {
                    if !line.is_empty() {
                        println!("   {}", line);
                    } else {
                        println!();
                    }
                }
            }
            Err(e) => {
                println!("   ❌ 错误: {}", e);
            }
        }

        println!("\n{}\n", "-".repeat(80));
    }

    // 实际使用场景示例
    println!("🎯 实际使用场景示例\n");
    println!("{}\n", "=".repeat(80));

    let real_world_examples = vec![
        (
            "button",
            "text-center text-white p-4 rounded hover:opacity-80 active:opacity-60 disabled:opacity-50",
            "按钮样式",
        ),
        (
            "card",
            "p-6 rounded shadow hover:shadow-lg transition",
            "卡片样式",
        ),
        (
            "nav-link",
            "text-gray-700 hover:text-blue-500 dark:text-gray-300 dark:hover:text-blue-400",
            "导航链接",
        ),
        (
            "container",
            "w-full md:w-3/4 lg:w-1/2 mx-auto p-4 md:p-8",
            "响应式容器",
        ),
    ];

    for (class_name, classes, description) in real_world_examples {
        println!("📦 {}", description);
        println!("   Tailwind: {}", classes);
        println!();

        if let Ok(group) = bundler.bundle(classes) {
            let css = bundler.generate_css(class_name, &group, "  ");
            println!("   CSS:");
            for line in css.lines() {
                if !line.is_empty() {
                    println!("   {}", line);
                } else {
                    println!();
                }
            }
        }

        println!("\n{}\n", "-".repeat(80));
    }

    println!("✨ 完成！");
    println!("\n💡 提示:");
    println!("   - 基础类会合并到同一个选择器");
    println!("   - 伪类（hover、focus 等）会生成独立的选择器");
    println!("   - 响应式修饰符会生成 @media 查询");
    println!("   - 状态修饰符（dark、group-hover）会生成特殊选择器");
}
