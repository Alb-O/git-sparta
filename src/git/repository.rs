//! Repository operations using gix.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Open a git repository at the given path (or discover from current directory).
pub fn open_repository(start: Option<&Path>) -> Result<(gix::Repository, PathBuf)> {
	let start = start.unwrap_or_else(|| Path::new("."));
	let repo = gix::ThreadSafeRepository::discover(start)
		.with_context(|| format!("failed to discover git repository at {}", start.display()))?
		.to_thread_local();

	let root = repo
		.workdir()
		.or_else(|| repo.git_dir().parent())
		.ok_or_else(|| anyhow::anyhow!("repository has no worktree or parent git directory"))?;

	let root = fs::canonicalize(root)
		.with_context(|| format!("failed to canonicalize {}", root.display()))?;

	Ok((repo, root))
}

/// Require that the repository has a worktree.
pub fn require_worktree(repo: &gix::Repository) -> Result<gix::Worktree<'_>> {
	repo.worktree().ok_or_else(|| {
		anyhow::anyhow!("repository is bare; a worktree is required for this operation")
	})
}

/// Initialize a bare git repository at the given path.
pub fn init_bare(path: &Path) -> Result<()> {
	super::git().args(["init", "--bare", "-q"]).arg(path).run()
}

/// Check if a path is a git repository.
pub fn is_repository(path: &Path) -> bool {
	path.join(".git").exists() || path.join("HEAD").exists()
}
