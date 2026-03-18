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
    build_source_config, default_source_residence, detect_path, import_into_local_source,
    persist_prepared_source, prepare_source, refresh_source, source_kind_residence,
    source_storage_path, source_storage_path_for_config, source_storage_path_in,
    source_storage_root, LocalImport, PreparedSource, RefreshSource,
};
pub use manifest::{MarketplaceManifest, PluginManifest};
pub use parsed::{ParsedSource, SourceKind};
pub use url::SourceUrl;
