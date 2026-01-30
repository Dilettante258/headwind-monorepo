pub mod index;
pub mod loader;

// Re-export main types
pub use index::TailwindIndex;
pub use loader::load_from_json;

// Implement TailwindIndexLookup for integration with bundle
use headwind_core::{bundle::TailwindIndexLookup, Declaration};

impl TailwindIndexLookup for TailwindIndex {
    fn lookup(&self, class: &str) -> Option<&[Declaration]> {
        self.lookup(class)
    }
}
