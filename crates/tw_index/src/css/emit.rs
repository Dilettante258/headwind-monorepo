use swc_css_ast::Stylesheet;
use swc_css_codegen::{
    writer::basic::{BasicCssWriter, BasicCssWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
};

/// 使用 swc_css_codegen 生成 CSS 字符串
pub fn emit_css(stylesheet: &Stylesheet) -> Result<String, std::fmt::Error> {
    let mut output = String::new();

    let writer_config = BasicCssWriterConfig {
        indent_type: swc_css_codegen::writer::basic::IndentType::Space,
        indent_width: 2,
        linefeed: swc_css_codegen::writer::basic::LineFeed::LF,
    };

    let mut wr = BasicCssWriter::new(&mut output, None, writer_config);
    let mut gen = CodeGenerator::new(&mut wr, CodegenConfig { minify: false });

    gen.emit(stylesheet)?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::ir::create_stylesheet;
    use headwind_core::Declaration;

    #[test]
    fn test_emit_single_rule() {
        let stylesheet = create_stylesheet(
            "test".to_string(),
            vec![Declaration::new("padding", "1rem")],
        );

        let css = emit_css(&stylesheet).unwrap();

        assert!(css.contains("padding"));
        assert!(css.contains("1rem"));
    }

    #[test]
    fn test_emit_multiple_declarations() {
        let stylesheet = create_stylesheet(
            "test".to_string(),
            vec![
                Declaration::new("padding", "1rem"),
                Declaration::new("margin", "0.5rem"),
            ],
        );

        let css = emit_css(&stylesheet).unwrap();

        assert!(css.contains("padding"));
        assert!(css.contains("margin"));
    }

    #[test]
    fn test_emit_stability() {
        let stylesheet = create_stylesheet(
            "test".to_string(),
            vec![
                Declaration::new("padding", "1rem"),
                Declaration::new("margin", "0.5rem"),
            ],
        );

        let css1 = emit_css(&stylesheet).unwrap();
        let css2 = emit_css(&stylesheet).unwrap();

        assert_eq!(css1, css2);
    }
}
