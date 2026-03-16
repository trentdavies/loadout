pub mod kit;
pub mod cli;
pub mod completions;
pub mod config;
pub mod marketplace;
pub mod output;
pub mod prompt;
pub mod registry;
pub mod source;
pub mod agent;

// Re-export for convenience
pub use cli::Cli;
