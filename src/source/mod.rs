pub mod detect;
pub mod discover;
pub mod fetch;
pub mod manage;
pub mod manifest;
pub mod normalize;
pub mod parsed;
pub mod url;

pub use detect::SourceStructure;
pub use discover::{DiscoveredPlugin, DiscoveredSkill};
pub use manage::{
    build_source_config, detect_path, persist_prepared_source, prepare_source, refresh_source,
    PreparedSource, RefreshSource,
};
pub use manifest::{MarketplaceManifest, PluginManifest};
pub use parsed::{ParsedSource, SourceKind};
pub use url::SourceUrl;
