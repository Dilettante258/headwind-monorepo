pub mod parser;
pub mod types;

// Re-export main types
pub use parser::parse_class;
pub use types::{ArbitraryValue, Modifier, ParsedClass, ParsedValue};
