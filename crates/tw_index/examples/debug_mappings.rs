use headwind_tw_index::{load_from_official_json, Converter};
use headwind_tw_parse::parse_class;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 可选过滤：cargo run --example debug_mappings -- text
    let filter = args.get(1).map(|s| s.as_str());

    let json = include_str!("../../tw_index/fixtures/official-mappings.json");
    let index = load_from_official_json(json).expect("Failed to load mappings");
    let converter = Converter::new();

    let mut total = 0;
    let mut parsed_ok = 0;
    let mut converted_ok = 0;
    let mut matched = 0;

    let mut classes = index.classes();
    classes.sort();

    for class_name in &classes {
        // 应用过滤
        if let Some(f) = filter {
            if !class_name.contains(f) {
                continue;
            }
        }

        total += 1;
        let expected = index.lookup(class_name).unwrap();

        // 1. 解析
        let parsed = match parse_class(class_name) {
            Ok(p) => p,
            Err(e) => {
                println!("--- {} ---", class_name);
                println!("  PARSE ERROR: {}", e);
                println!("  expected: {}", format_declarations(expected));
                println!();
                continue;
            }
        };
        parsed_ok += 1;

        // 2. 转换
        let rule = converter.convert(&parsed);

        // 3. 输出
        println!("--- {} ---", class_name);
        println!("  parsed:    plugin={:<16} value={:<20} neg={:<5} important={}",
            parsed.plugin,
            parsed.value.as_ref().map_or("-".to_string(), |v| format!("{:?}", v)),
            parsed.negative,
            parsed.important,
        );
        println!("  expected:  {}", format_declarations(expected));

        match &rule {
            Some(r) => {
                converted_ok += 1;
                let actual_str = format_declarations(&r.declarations);
                let expected_str = format_declarations(expected);
                let is_match = actual_str == expected_str;
                if is_match {
                    matched += 1;
                }
                println!("  actual:    {}", actual_str);
                println!("  status:    {}", if is_match { "MATCH" } else { "MISMATCH" });
            }
            None => {
                println!("  actual:    (converter returned None)");
                println!("  status:    NO_CONVERT");
            }
        }
        println!();
    }

    // 汇总
    println!("{}", "=".repeat(60));
    println!("Total: {}  Parsed: {}  Converted: {}  Matched: {}",
        total, parsed_ok, converted_ok, matched);
    println!("Parse rate:   {:.1}%", pct(parsed_ok, total));
    println!("Convert rate: {:.1}%", pct(converted_ok, total));
    println!("Match rate:   {:.1}%", pct(matched, total));
}

fn format_declarations(decls: &[headwind_core::Declaration]) -> String {
    decls
        .iter()
        .map(|d| format!("{}: {}", d.property, d.value))
        .collect::<Vec<_>>()
        .join("; ")
}

fn pct(n: usize, total: usize) -> f64 {
    if total == 0 { 0.0 } else { n as f64 / total as f64 * 100.0 }
}
