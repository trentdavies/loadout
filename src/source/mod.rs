pub mod detect;
pub mod discover;
pub mod fetch;
pub mod manifest;
pub mod normalize;
pub mod parsed;
pub mod url;

pub use detect::SourceStructure;
pub use discover::{DiscoveredPlugin, DiscoveredSkill};
pub use manifest::{MarketplaceManifest, PluginManifest};
pub use parsed::{ParsedSource, SourceKind};
pub use url::SourceUrl;
