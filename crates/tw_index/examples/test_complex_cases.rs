use headwind_tw_index::Converter;
use headwind_tw_parse::parse_class;

fn main() {
    println!("🧪 Testing Complex Tailwind CSS Cases\n");
    println!("{}\n", "=".repeat(80));

    // 创建基于规则的转换器
    let converter = Converter::new();
    println!("✅ 使用基于规则的转换器\n");

    // 复杂测试用例
    let test_cases = vec![
        // 1. 多重修饰符组合
        ("md:hover:focus:text-center", "多重修饰符（响应式 + 伪类）"),
        ("lg:dark:group-hover:bg-blue-500", "三重修饰符（响应式 + 状态 + 组）"),
        ("2xl:peer-focus:before:content-none", "复杂修饰符组合"),

        // 2. 复杂任意值
        ("w-[calc(100%-2rem)]", "calc() 函数"),
        ("bg-[url('/images/hero.jpg')]", "URL 任意值"),
        ("text-[clamp(1rem,2.5vw,2rem)]", "clamp() 函数"),
        ("grid-cols-[repeat(auto-fit,minmax(250px,1fr))]", "复杂 grid 值"),
        ("shadow-[0_35px_60px_-15px_rgba(0,0,0,0.3)]", "复杂阴影值"),

        // 3. 特殊字符和空格
        ("content-['Hello_World']", "content 带下划线"),
        ("bg-[rgb(255,0,0)]", "RGB 颜色"),
        ("bg-[rgba(255,0,0,0.5)]", "RGBA 颜色"),
        ("bg-[hsl(0,100%,50%)]", "HSL 颜色"),

        // 4. 负值 + 任意值
        ("-translate-x-[50px]", "负值 + 任意值"),
        ("-mt-[2.5rem]", "负 margin + 任意值"),
        ("-rotate-[45deg]", "负旋转 + 任意值"),

        // 5. Important + 修饰符
        ("hover:text-center!", "伪类 + important"),
        ("md:p-4!", "响应式 + important"),
        ("dark:bg-black!", "状态 + important"),

        // 6. Alpha 值
        ("bg-blue-500/50", "50% 不透明度"),
        ("bg-red-500/[0.75]", "任意不透明度"),
        ("text-gray-900/90", "90% 文本不透明度"),

        // 7. 复合插件名 + 任意值
        ("grid-cols-[1fr_2fr_1fr]", "复合插件 + 复杂值"),
        ("grid-rows-[auto_1fr_auto]", "grid rows 复杂值"),
        ("aspect-[16/9]", "宽高比"),

        // 8. 多属性插件 + 任意值
        ("px-[3.5rem]", "padding 左右"),
        ("py-[2.5rem]", "padding 上下"),
        ("mx-[auto]", "margin 左右 auto"),
        ("inset-x-[10%]", "左右定位"),
        ("inset-y-[5%]", "上下定位"),

        // 9. 长类名
        ("lg:hover:focus:disabled:opacity-50", "超长修饰符链"),
        ("2xl:dark:group-hover:peer-focus:ring-2", "四重修饰符"),

        // 10. 边缘情况
        ("w-[100%]", "百分比值"),
        ("h-[50vh]", "视口单位"),
        ("text-[14px]", "像素文本大小"),
        ("leading-[1.5]", "无单位行高"),
        ("tracking-[0.05em]", "em 单位字距"),

        // 11. 特殊 CSS 值
        ("w-[fit-content]", "fit-content"),
        ("w-[max-content]", "max-content"),
        ("w-[min-content]", "min-content"),
        ("flex-[1_1_0%]", "flex 简写"),

        // 12. 嵌套函数
        ("bg-[linear-gradient(to_right,#000,#fff)]", "渐变"),
        ("transform-[rotate(45deg)_scale(1.5)]", "多重变换"),
    ];

    let mut success_count = 0;
    let mut parse_errors = Vec::new();
    let mut convert_errors = Vec::new();

    for (class_name, description) in &test_cases {
        println!("📝 测试: {}", description);
        println!("   类名: {}", class_name);

        match parse_class(class_name) {
            Ok(parsed) => {
                println!("   ✅ 解析成功");
                println!("      插件: {}", parsed.plugin);
                println!("      修饰符数: {}", parsed.modifiers().len());
                if parsed.negative {
                    println!("      负值: true");
                }
                if parsed.important {
                    println!("      Important: true");
                }
                if let Some(ref value) = parsed.value {
                    println!("      值: {:?}", value);
                }
                if let Some(ref alpha) = parsed.alpha {
                    println!("      Alpha: {}", alpha);
                }

                // 尝试转换
                match converter.convert(&parsed) {
                    Some(rule) => {
                        println!("   ✅ 转换成功");
                        println!("      选择器: {}", rule.selector);
                        println!("      声明数: {}", rule.declarations.len());
                        for (i, decl) in rule.declarations.iter().enumerate() {
                            println!("      [{}.] {}: {}", i + 1, decl.property, decl.value);
                        }
                        success_count += 1;
                    }
                    None => {
                        println!("   ⚠️  转换失败（可能不在索引中或插件未映射）");
                        convert_errors.push(class_name.to_string());
                    }
                }
            }
            Err(e) => {
                println!("   ❌ 解析失败: {:?}", e);
                parse_errors.push(class_name.to_string());
            }
        }
        println!();
    }

    // 统计结果
    println!("{}", "=".repeat(80));
    println!("\n📊 测试结果统计:");
    println!("   总测试数: {}", test_cases.len());
    println!("   成功: {} ✅", success_count);
    println!("   解析失败: {} ❌", parse_errors.len());
    println!("   转换失败: {} ⚠️", convert_errors.len());
    println!(
        "   成功率: {:.1}%",
        (success_count as f64 / test_cases.len() as f64) * 100.0
    );

    if !parse_errors.is_empty() {
        println!("\n❌ 解析失败的类:");
        for class in &parse_errors {
            println!("   - {}", class);
        }
    }

    if !convert_errors.is_empty() {
        println!("\n⚠️  转换失败的类（已解析但无法转换）:");
        for class in &convert_errors {
            println!("   - {}", class);
        }
    }

    println!("\n{}", "=".repeat(80));

    // 返回状态码
    if parse_errors.is_empty() {
        println!("\n🎉 所有类都能成功解析！");
        if convert_errors.is_empty() {
            println!("🎉 所有类都能成功转换！");
        }
    } else {
        println!("\n⚠️  有些类解析失败，需要改进解析器");
        std::process::exit(1);
    }
}
