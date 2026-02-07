/// 基本使用示例：展示如何使用 headwind 进行 Tailwind 类名转换
///
/// 运行示例：
/// ```bash
/// cargo run --example basic_usage -p headwind-tw-index
/// ```

use headwind_core::{BundleRequest, NamingMode};
use headwind_tw_index::bundle::bundle;
use headwind_tw_index::css::{create_stylesheet, emit_css};
use headwind_tw_index::load_from_json;

fn main() {
    println!("=== HeadWind 基本使用示例 ===\n");

    // 1. 准备 Tailwind 索引（从 JSON 加载）
    let tailwind_json = r#"[
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
        },
        {
            "class": "bg-blue-600",
            "declarations": [
                { "property": "background-color", "value": "rgb(37, 99, 235)" }
            ]
        }
    ]"#;

    let index = load_from_json(tailwind_json).expect("Failed to load Tailwind index");
    println!("✓ 加载 Tailwind 索引：{} 个类", index.len());

    // 2. 示例 1：使用 Hash 命名
    println!("\n--- 示例 1: Hash 命名 ---");
    let request = BundleRequest {
        classes: vec![
            "p-4".to_string(),
            "m-2".to_string(),
            "text-red-500".to_string(),
        ],
        naming_mode: NamingMode::Hash,
    };

    let result = bundle(request, &index);
    println!("输入类名: p-4 m-2 text-red-500");
    println!("生成类名: {}", result.new_class);
    println!("CSS 声明数: {}", result.css_declarations.len());

    let stylesheet = create_stylesheet(
        result.new_class.clone(),
        result.css_declarations.clone(),
    );
    let css = emit_css(&stylesheet).expect("Failed to emit CSS");
    println!("生成的 CSS:\n{}", css);

    // 3. 示例 2：使用 Readable 命名
    println!("--- 示例 2: Readable 命名 ---");
    let request = BundleRequest {
        classes: vec!["p-4".to_string(), "m-2".to_string()],
        naming_mode: NamingMode::Readable,
    };

    let result = bundle(request, &index);
    println!("输入类名: p-4 m-2");
    println!("生成类名: {} (可读形式)", result.new_class);

    // 4. 示例 3：处理重复和未知类
    println!("\n--- 示例 3: 处理重复和未知类 ---");
    let request = BundleRequest {
        classes: vec![
            "p-4".to_string(),
            "p-4".to_string(),
            "unknown-class".to_string(),
            "m-2".to_string(),
        ],
        naming_mode: NamingMode::Hash,
    };

    let result = bundle(request, &index);
    println!("输入类名: p-4 p-4 unknown-class m-2");
    println!("生成类名: {}", result.new_class);
    println!("有效 CSS 声明: {}", result.css_declarations.len());
    println!("移除的类: {:?}", result.removed);
    println!("诊断信息: {} 条", result.diagnostics.len());
    for diag in &result.diagnostics {
        println!("  - {:?}: {}", diag.level, diag.message);
    }

    // 5. 示例 4：CSS 冲突处理
    println!("\n--- 示例 4: CSS 冲突处理 ---");

    let conflicting_json = r#"[
        {
            "class": "p-4",
            "declarations": [
                { "property": "padding", "value": "1rem" }
            ]
        },
        {
            "class": "p-8",
            "declarations": [
                { "property": "padding", "value": "2rem" }
            ]
        }
    ]"#;

    let index2 = load_from_json(conflicting_json).unwrap();

    let request = BundleRequest {
        classes: vec!["p-4".to_string(), "p-8".to_string()],
        naming_mode: NamingMode::Hash,
    };

    let result = bundle(request, &index2);
    println!("输入类名: p-4 p-8 (都设置 padding)");
    println!("生成类名: {}", result.new_class);
    println!("CSS 声明数: {} (冲突后合并)", result.css_declarations.len());
    println!("最终 padding 值: {}", result.css_declarations[0].value);

    println!("\n=== 示例完成 ===");
}
