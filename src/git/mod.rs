//! Git operations for git-sparta.
//!
//! This module provides abstractions for git operations, using `gix` where possible
//! and falling back to shell commands for operations not yet supported by `gix`.

pub mod attributes;
pub mod cmd;
pub mod config;
pub mod lfs;
pub mod repository;
pub mod sparse;
pub mod submodule;

// Re-export commonly used items
pub use cmd::git;
pub use repository::{open_repository, require_worktree};
