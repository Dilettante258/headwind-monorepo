pub mod bundler;
pub mod context;
pub mod converter;
pub mod index;
pub mod loader;
pub mod plugin_map;
pub mod theme_values;
pub mod value_map;

// Re-export main types
pub use bundler::{Bundler, RuleGroup};
pub use context::ClassContext;
pub use converter::{Converter, CssRule};
pub use index::TailwindIndex;
pub use loader::{load_from_json, load_from_official_json};

// Implement TailwindIndexLookup for integration with bundle
use headwind_core::{bundle::TailwindIndexLookup, Declaration};

impl TailwindIndexLookup for TailwindIndex {
    fn lookup(&self, class: &str) -> Option<&[Declaration]> {
        self.lookup(class)
    }
}
