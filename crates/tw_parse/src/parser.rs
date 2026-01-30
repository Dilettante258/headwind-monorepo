use crate::types::{ArbitraryValue, Modifier, ParsedClass, ParsedValue};

/// 解析 Tailwind class 字符串
///
/// 支持的格式：
/// - 简单类：`p-4`, `m-2`, `bg-red-500`
/// - 修饰符：`hover:bg-blue-500`, `md:p-4`, `dark:text-white`
/// - 多修饰符：`md:hover:bg-blue-500`
/// - 负值：`-m-4`, `md:-top-1`
/// - 任意值：`w-[13px]`, `bg-[#ff0000]`
/// - 透明度：`bg-blue-500/50`, `text-black/75`
/// - 重要性：`p-4!`, `md:bg-red-500!`
///
/// # 示例
///
/// ```
/// use headwind_tw_parse::parse_class;
///
/// let parsed = parse_class("md:hover:bg-blue-500/50!").unwrap();
/// assert_eq!(parsed.modifiers.len(), 2);
/// assert_eq!(parsed.plugin, "bg");
/// assert_eq!(parsed.alpha, Some("50".to_string()));
/// assert_eq!(parsed.important, true);
/// ```
pub fn parse_class(input: &str) -> Result<ParsedClass, ParseError> {
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    let mut parser = Parser::new(input);
    parser.parse()
}

/// 解析错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    EmptyInput,
    InvalidFormat(String),
    UnmatchedBracket,
    MissingPlugin,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Empty input"),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::UnmatchedBracket => write!(f, "Unmatched bracket in arbitrary value"),
            ParseError::MissingPlugin => write!(f, "Missing plugin/command"),
        }
    }
}

impl std::error::Error for ParseError {}

/// 内部解析器
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse(&mut self) -> Result<ParsedClass, ParseError> {
        // 1. 解析修饰符（modifier:modifier:...）
        let modifiers = self.parse_modifiers();

        // 2. 解析负值标记
        let negative = self.consume_if('-');

        // 3. 解析插件和值
        let (plugin, value) = self.parse_plugin_and_value()?;

        // 4. 解析透明度（/50）
        let alpha = self.parse_alpha();

        // 5. 解析重要性（!）
        let important = self.consume_if('!');

        // 确保已解析完整个字符串
        if self.pos < self.input.len() {
            return Err(ParseError::InvalidFormat(format!(
                "Unexpected characters at position {}: '{}'",
                self.pos,
                &self.input[self.pos..]
            )));
        }

        Ok(ParsedClass {
            modifiers,
            negative,
            plugin,
            value,
            alpha,
            important,
        })
    }

    /// 解析修饰符列表
    fn parse_modifiers(&mut self) -> Vec<Modifier> {
        let mut modifiers = Vec::new();

        loop {
            // 尝试找到下一个冒号
            let start = self.pos;
            while self.pos < self.input.len() && self.current_char() != ':' {
                self.pos += 1;
            }

            // 如果没有找到冒号，说明修饰符结束
            if self.pos >= self.input.len() || self.current_char() != ':' {
                self.pos = start; // 回退
                break;
            }

            // 提取修饰符
            let modifier_str = &self.input[start..self.pos];

            // 跳过冒号
            self.pos += 1;

            // 检查这是否真的是修饰符（后面还有内容）
            if self.pos >= self.input.len() {
                self.pos = start; // 回退，这不是修饰符
                break;
            }

            // 检查是否为有效的修饰符（不包含特殊字符）
            if modifier_str.is_empty()
                || modifier_str.contains('[')
                || modifier_str.contains('/')
                || modifier_str.contains('!')
            {
                self.pos = start; // 回退
                break;
            }

            modifiers.push(Modifier::from_str(modifier_str));
        }

        modifiers
    }

    /// 解析插件和值
    ///
    /// 策略：扫描整个字符串，找到所有 `-[` 模式的位置
    /// - 如果存在 `-[`，则将其之前的部分作为 plugin
    /// - 否则，在第一个 `-` 处分割
    fn parse_plugin_and_value(&mut self) -> Result<(String, Option<ParsedValue>), ParseError> {
        let start = self.pos;

        // 查找 `-[` 模式的位置
        let mut dash_bracket_pos = None;
        let mut temp_pos = self.pos;

        while temp_pos + 1 < self.input.len() {
            if self.input[temp_pos..].starts_with("-[") {
                dash_bracket_pos = Some(temp_pos);
                break;
            }
            temp_pos += 1;
        }

        // 确定 plugin 的结束位置
        if let Some(db_pos) = dash_bracket_pos {
            // 找到了 `-[`，plugin 到这里结束
            self.pos = db_pos;
        } else {
            // 没有 `-[`，在第一个 `-`、`[`、`/`、`!` 处分割
            while self.pos < self.input.len() {
                let ch = self.current_char();
                if ch == '-' || ch == '[' || ch == '/' || ch == '!' {
                    break;
                }
                self.pos += 1;
            }
        }

        let plugin = self.input[start..self.pos].to_string();

        if plugin.is_empty() {
            return Err(ParseError::MissingPlugin);
        }

        // 解析值（如果存在）
        let value = if self.pos < self.input.len() {
            let ch = self.current_char();

            if ch == '-' {
                // 跳过 '-'
                self.pos += 1;

                // 检查是否为任意值
                if self.pos < self.input.len() && self.current_char() == '[' {
                    Some(ParsedValue::Arbitrary(self.parse_arbitrary_value()?))
                } else {
                    // 标准值
                    let val = self.parse_standard_value();
                    if !val.is_empty() {
                        Some(ParsedValue::Standard(val))
                    } else {
                        None
                    }
                }
            } else if ch == '[' {
                // 直接的任意值
                Some(ParsedValue::Arbitrary(self.parse_arbitrary_value()?))
            } else {
                None
            }
        } else {
            None
        };

        Ok((plugin, value))
    }

    /// 解析标准值
    fn parse_standard_value(&mut self) -> String {
        let start = self.pos;

        // 读取直到遇到 /、! 或字符串结尾
        while self.pos < self.input.len() {
            let ch = self.current_char();
            if ch == '/' || ch == '!' {
                break;
            }
            self.pos += 1;
        }

        self.input[start..self.pos].to_string()
    }

    /// 解析任意值（方括号内容）
    fn parse_arbitrary_value(&mut self) -> Result<ArbitraryValue, ParseError> {
        if self.current_char() != '[' {
            return Err(ParseError::InvalidFormat(
                "Arbitrary value must start with '['".to_string(),
            ));
        }

        let start = self.pos;

        // 跳过 '['
        self.pos += 1;

        // 找到匹配的 ']'
        let mut depth = 1;
        while self.pos < self.input.len() && depth > 0 {
            match self.current_char() {
                '[' => depth += 1,
                ']' => depth -= 1,
                _ => {}
            }
            self.pos += 1;
        }

        if depth != 0 {
            return Err(ParseError::UnmatchedBracket);
        }

        let raw = self.input[start..self.pos].to_string();
        Ok(ArbitraryValue::new(raw))
    }

    /// 解析透明度修饰符
    fn parse_alpha(&mut self) -> Option<String> {
        if self.pos < self.input.len() && self.current_char() == '/' {
            self.pos += 1; // 跳过 '/'

            let start = self.pos;

            // 读取数字或百分比
            while self.pos < self.input.len() {
                let ch = self.current_char();
                if ch == '!' || !ch.is_ascii_alphanumeric() {
                    break;
                }
                self.pos += 1;
            }

            let alpha = self.input[start..self.pos].to_string();
            if !alpha.is_empty() {
                return Some(alpha);
            }
        }

        None
    }

    /// 消费指定字符（如果存在）
    fn consume_if(&mut self, expected: char) -> bool {
        if self.pos < self.input.len() && self.current_char() == expected {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// 获取当前字符
    fn current_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_class() {
        let parsed = parse_class("p-4").unwrap();
        assert_eq!(parsed.plugin, "p");
        assert_eq!(
            parsed.value,
            Some(ParsedValue::Standard("4".to_string()))
        );
        assert!(!parsed.negative);
        assert!(!parsed.important);
    }

    #[test]
    fn test_class_without_value() {
        let parsed = parse_class("flex").unwrap();
        assert_eq!(parsed.plugin, "flex");
        assert_eq!(parsed.value, None);
    }

    #[test]
    fn test_single_modifier() {
        let parsed = parse_class("hover:bg-blue-500").unwrap();
        assert_eq!(parsed.modifiers.len(), 1);
        assert!(parsed.modifiers[0].is_pseudo_class());
        assert_eq!(parsed.plugin, "bg");
        assert_eq!(
            parsed.value,
            Some(ParsedValue::Standard("blue-500".to_string()))
        );
    }

    #[test]
    fn test_multiple_modifiers() {
        let parsed = parse_class("md:hover:bg-blue-500").unwrap();
        assert_eq!(parsed.modifiers.len(), 2);
        assert!(parsed.modifiers[0].is_responsive());
        assert!(parsed.modifiers[1].is_pseudo_class());
    }

    #[test]
    fn test_negative_value() {
        let parsed = parse_class("-m-4").unwrap();
        assert!(parsed.negative);
        assert_eq!(parsed.plugin, "m");
        assert_eq!(
            parsed.value,
            Some(ParsedValue::Standard("4".to_string()))
        );
    }

    #[test]
    fn test_arbitrary_value() {
        let parsed = parse_class("w-[13px]").unwrap();
        assert_eq!(parsed.plugin, "w");
        assert!(parsed.value.as_ref().unwrap().is_arbitrary());

        if let Some(ParsedValue::Arbitrary(arb)) = parsed.value {
            assert_eq!(arb.content, "13px");
        } else {
            panic!("Expected arbitrary value");
        }
    }

    #[test]
    fn test_alpha_modifier() {
        let parsed = parse_class("bg-blue-500/50").unwrap();
        assert_eq!(parsed.plugin, "bg");
        assert_eq!(parsed.alpha, Some("50".to_string()));
    }

    #[test]
    fn test_important() {
        let parsed = parse_class("p-4!").unwrap();
        assert_eq!(parsed.plugin, "p");
        assert!(parsed.important);
    }

    #[test]
    fn test_complex_class() {
        let parsed = parse_class("md:hover:bg-blue-500/50!").unwrap();
        assert_eq!(parsed.modifiers.len(), 2);
        assert_eq!(parsed.plugin, "bg");
        assert_eq!(
            parsed.value,
            Some(ParsedValue::Standard("blue-500".to_string()))
        );
        assert_eq!(parsed.alpha, Some("50".to_string()));
        assert!(parsed.important);
    }

    #[test]
    fn test_negative_with_modifier() {
        let parsed = parse_class("md:-top-1").unwrap();
        assert_eq!(parsed.modifiers.len(), 1);
        assert!(parsed.negative);
        assert_eq!(parsed.plugin, "top");
    }

    #[test]
    fn test_arbitrary_color() {
        let parsed = parse_class("bg-[#ff0000]").unwrap();
        assert_eq!(parsed.plugin, "bg");

        if let Some(ParsedValue::Arbitrary(arb)) = parsed.value {
            assert_eq!(arb.content, "#ff0000");
        } else {
            panic!("Expected arbitrary value");
        }
    }

    #[test]
    fn test_nested_brackets() {
        let parsed = parse_class("grid-cols-[repeat(3,minmax(0,1fr))]").unwrap();
        // grid-cols 是一个整体，-[ 表示后面是任意值
        assert_eq!(parsed.plugin, "grid-cols");

        if let Some(ParsedValue::Arbitrary(arb)) = parsed.value {
            assert_eq!(arb.content, "repeat(3,minmax(0,1fr))");
        } else {
            panic!("Expected arbitrary value");
        }
    }

    #[test]
    fn test_empty_input() {
        let result = parse_class("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParseError::EmptyInput);
    }
}
