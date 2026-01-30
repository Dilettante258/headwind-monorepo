pub mod emit;
pub mod ir;

// Re-export main functions
pub use emit::emit_css;
pub use ir::{create_qualified_rule, create_stylesheet, create_swc_declaration, merge_stylesheets};

// Re-export SWC CSS types
pub use swc_css_ast::Stylesheet;
