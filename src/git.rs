use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

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

pub fn require_worktree<'repo>(repo: &'repo gix::Repository) -> Result<gix::Worktree<'repo>> {
	repo.worktree().ok_or_else(|| {
		anyhow::anyhow!("repository is bare; a worktree is required for this operation")
	})
}
