pub mod detect;
pub mod fetch;
pub mod manifest;
pub mod normalize;
pub mod url;

pub use detect::SourceStructure;
pub use manifest::{SourceManifest, PluginManifest};
pub use url::SourceUrl;
