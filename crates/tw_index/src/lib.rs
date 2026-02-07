pub mod bundle;
pub mod bundler;
pub mod context;
pub mod converter;
pub mod css;
pub mod index;
pub mod loader;
pub mod merge;
pub mod naming;
pub mod normalize;
pub mod palette;
pub mod plugin_map;
pub mod shorthand;
pub mod theme_values;
pub mod value_map;
pub mod variant;

// Re-export main types
pub use bundle::TailwindIndexLookup;
pub use bundler::{Bundler, RuleGroup};
pub use context::ClassContext;
pub use converter::{Converter, CssRule};
pub use index::TailwindIndex;
pub use loader::{load_from_json, load_from_official_json};
pub use headwind_core::ColorMode;

// Implement TailwindIndexLookup for integration with bundle
use headwind_core::Declaration;

impl TailwindIndexLookup for TailwindIndex {
    fn lookup(&self, class: &str) -> Option<&[Declaration]> {
        self.lookup(class)
    }
}
