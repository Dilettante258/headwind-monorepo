pub mod bundle;
pub mod merge;
pub mod naming;
pub mod normalize;
pub mod types;

// Re-export commonly used types
pub use types::{
    BundleRequest, BundleResult, Declaration, Diagnostic, DiagnosticLevel, NamingMode,
};
