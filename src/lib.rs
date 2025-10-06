pub mod commands;
pub mod config;
pub mod git;
pub mod output;
// Re-export the `tui_searcher` crate under the old `tui` name so existing
// callers (internal and external) keep working.
pub use tui_searcher as tui;
