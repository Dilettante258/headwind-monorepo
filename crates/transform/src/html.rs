use crate::collector::ClassCollector;

/// HTML 转换器 —— 扫描 HTML 源码中的 class="..." 属性，
/// 将 Tailwind 类替换为生成的类名。
///
/// 使用简单的状态机解析，避免引入正则依赖。
/// 支持双引号和单引号。
pub fn transform_html_source(source: &str, collector: &mut ClassCollector) -> String {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // 查找 "class" 关键字
        if i + 5 < len && matches_class_attr(bytes, i) {
            // 跳过 "class"
            let attr_start = i;
            i += 5;

            // 跳过可选空白
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            // 期望 '='
            if i < len && bytes[i] == b'=' {
                i += 1;

                // 跳过可选空白
                while i < len && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }

                // 期望引号
                if i < len && (bytes[i] == b'"' || bytes[i] == b'\'') {
                    let quote = bytes[i];
                    i += 1;
                    let value_start = i;

                    // 查找匹配的闭合引号
                    while i < len && bytes[i] != quote {
                        i += 1;
                    }

                    if i < len {
                        let class_value = &source[value_start..i];
                        i += 1; // 跳过闭合引号

                        // 处理类值
                        let new_class = collector.process_classes(class_value);
                        if !new_class.is_empty() {
                            result.push_str("class=");
                            result.push(quote as char);
                            result.push_str(&new_class);
                            result.push(quote as char);
                        } else {
                            // 空类值，保留原样
                            result.push_str(&source[attr_start..i]);
                        }
                        continue;
                    }
                }
            }

            // 未匹配完整的 class="..." 模式，回退
            result.push_str(&source[attr_start..i]);
            continue;
        }

        result.push(source[i..].chars().next().unwrap());
        i += source[i..].chars().next().unwrap().len_utf8();
    }

    result
}

/// 检查位置 i 是否为 class 属性开头
/// 匹配 "class" 后面跟空白或 '='（区别于 className 等）
fn matches_class_attr(bytes: &[u8], i: usize) -> bool {
    let len = bytes.len();

    // 检查前面的字符确保是属性开始位置（空白或 <）
    if i > 0 && !bytes[i - 1].is_ascii_whitespace() && bytes[i - 1] != b'<' {
        return false;
    }

    // 匹配 "class"
    if i + 5 > len {
        return false;
    }
    if &bytes[i..i + 5] != b"class" {
        return false;
    }

    // class 后面必须是空白或 '='（排除 className 等）
    if i + 5 < len {
        let next = bytes[i + 5];
        return next == b'=' || next.is_ascii_whitespace();
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use headwind_core::NamingMode;

    #[test]
    fn test_html_basic() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let html = r#"<div class="p-4 m-2">Hello</div>"#;
        let result = transform_html_source(html, &mut collector);

        assert!(!result.contains("p-4 m-2"));
        assert!(result.contains("class=\"c_"));
        assert!(result.contains(">Hello</div>"));
    }

    #[test]
    fn test_html_single_quotes() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let html = "<div class='p-4 m-2'>Hello</div>";
        let result = transform_html_source(html, &mut collector);

        assert!(!result.contains("p-4 m-2"));
        assert!(result.contains("class='c_"));
    }

    #[test]
    fn test_html_multiple_elements() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let html = r#"<div class="p-4"><span class="text-center m-2">text</span></div>"#;
        let result = transform_html_source(html, &mut collector);

        assert!(!result.contains("p-4"));
        assert!(!result.contains("text-center m-2"));
        assert_eq!(collector.class_map().len(), 2);
    }

    #[test]
    fn test_html_preserves_non_class_attrs() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let html = r#"<div id="main" class="p-4" data-value="test">content</div>"#;
        let result = transform_html_source(html, &mut collector);

        assert!(result.contains("id=\"main\""));
        assert!(result.contains("data-value=\"test\""));
        assert!(!result.contains("\"p-4\""));
    }

    #[test]
    fn test_html_does_not_match_classname() {
        let mut collector = ClassCollector::new(NamingMode::Hash);
        let html = r#"<div className="p-4">content</div>"#;
        let result = transform_html_source(html, &mut collector);

        // className 不应被匹配（HTML 只处理 class）
        assert!(result.contains("className=\"p-4\""));
        assert!(collector.class_map().is_empty());
    }
}
