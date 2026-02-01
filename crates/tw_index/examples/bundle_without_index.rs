use headwind_tw_index::Bundler;

fn main() {
    println!("🎨 基于规则系统的 Tailwind CSS 类打包器\n");
    println!("{}\n", "=".repeat(80));
    println!("✨ 完全基于规则（plugin_map + value_map），无需官方映射文件");
    println!("🚀 使用新的 ClassContext 架构，更简洁高效\n");

    // 创建基于规则的打包器
    let bundler = Bundler::new();

    // 测试用例
    let test_cases = vec![
        (
            "button",
            "p-4 px-6 bg-blue-500 text-white rounded hover:bg-blue-600 active:bg-blue-700",
            "按钮样式"
        ),
        (
            "card",
            "p-6 m-4 rounded shadow",
            "卡片样式"
        ),
        (
            "container",
            "w-full md:w-3/4 lg:w-1/2 mx-auto p-4 md:p-8 lg:p-12",
            "响应式容器"
        ),
        (
            "spacing",
            "p-4 pt-2 pr-6 pb-8 pl-10 m-auto",
            "间距测试"
        ),
        (
            "sizing",
            "w-full h-screen min-w-0 max-w-96",
            "尺寸测试"
        ),
        (
            "colors",
            "bg-blue-500 text-white border-gray-300",
            "颜色测试"
        ),
        (
            "opacity",
            "opacity-50 bg-opacity-75",
            "不透明度测试"
        ),
        (
            "arbitrary",
            "w-[200px] h-[100px] bg-[#ff0000] p-[2.5rem]",
            "任意值测试"
        ),
        (
            "mixed",
            "p-4 px-[3rem] hover:p-6 md:p-8 lg:px-[4rem]",
            "混合值测试（标准值 + 任意值）"
        ),
    ];

    for (class_name, classes, description) in test_cases {
        println!("📝 {}", description);
        println!("   输入: {}", classes);
        println!();

        // 使用新的 ClassContext API（更简洁！）
        match bundler.bundle_to_css(class_name, classes, "  ") {
            Ok(css) => {
                if css.trim().is_empty() {
                    println!("   ⚠️  无法生成 CSS（可能某些类缺少值映射）");
                } else {
                    println!("   生成的 CSS:");
                    println!();
                    for line in css.lines() {
                        if !line.is_empty() {
                            println!("   {}", line);
                        } else {
                            println!();
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ❌ 错误: {}", e);
            }
        }

        println!("\n{}\n", "-".repeat(80));
    }

    // 统计信息
    println!("📊 支持的值映射:");
    println!();
    println!("   间距值: 0, px, 0.5~96 (基于 Tailwind 默认配置)");
    println!("   分数值: 1/2, 1/3, 2/3, 1/4, 3/4, 1/5~4/5, 1/6~5/6");
    println!("   颜色值: black, white, gray-50~900, blue-50~900, red-50~900, green-50~900");
    println!("   不透明度: 0, 5, 10, 20, 25, 30, 40, 50, 60, 70, 75, 80, 90, 95, 100");
    println!();
    println!("💡 ClassContext 架构优势:");
    println!("   - 不依赖官方映射文件（纯规则系统）");
    println!("   - 按 raw_modifiers 分组优化（性能提升）");
    println!("   - 支持所有标准 Tailwind 值");
    println!("   - 支持任意值 [...] 语法");
    println!("   - 自动推断 CSS 值");
    println!("   - 自动合并相同修饰符的声明");
    println!();
    println!("🏗️  架构特点:");
    println!("   - ParsedClass 作为\"写操作\"");
    println!("   - Converter: 只生成声明（关注点分离）");
    println!("   - ClassContext: 管理选择器和 CSS 输出");
    println!();
    println!("⚠️  当前限制:");
    println!("   - 值映射需要预先定义");
    println!("   - 某些特殊类可能无法识别");
    println!("   - 覆盖率: ~3.7% (28/752 官方类)");
    println!("   - 随着规则系统完善，覆盖率会持续提高");
    println!();
    println!("✨ 完成！");
}
