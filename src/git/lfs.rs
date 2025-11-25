//! Git LFS operations.

use std::fs;
use std::path::Path;

use anyhow::Result;

use super::git;
use crate::output;

/// Check if a repository uses Git LFS by looking for filter=lfs in .gitattributes.
pub fn is_enabled(worktree_path: &Path) -> bool {
	let gitattributes = worktree_path.join(".gitattributes");
	if let Ok(content) = fs::read_to_string(&gitattributes) {
		return content.contains("filter=lfs");
	}
	false
}

/// Install LFS hooks in a repository.
pub fn install(git_dir: &Path, worktree: &Path) -> Result<bool> {
	let output = git()
		.git_dir(git_dir)
		.work_tree(worktree)
		.args(["lfs", "install", "--local"])
		.output()?;

	if !output.status.success() {
		output::warn(&format!(
			"git lfs install failed (LFS may not be installed): {}",
			String::from_utf8_lossy(&output.stderr)
		));
		return Ok(false);
	}

	Ok(true)
}

/// Fetch LFS objects for the current checkout.
pub fn fetch(git_dir: &Path, worktree: &Path) -> Result<()> {
	let output = git()
		.git_dir(git_dir)
		.work_tree(worktree)
		.args(["lfs", "fetch"])
		.output()?;

	if !output.status.success() {
		output::warn(&format!(
			"git lfs fetch warning: {}",
			String::from_utf8_lossy(&output.stderr)
		));
		// Don't fail - alternates may already provide the objects
	}

	Ok(())
}

/// Checkout (smudge) LFS files in the worktree.
pub fn checkout(git_dir: &Path, worktree: &Path) -> Result<()> {
	git()
		.git_dir(git_dir)
		.work_tree(worktree)
		.args(["lfs", "checkout"])
		.run()
}

/// Fetch and checkout LFS objects for a sparse checkout.
pub fn fetch_and_checkout(git_dir: &Path, worktree: &Path) -> Result<()> {
	output::note("Fetching LFS objects...");

	// Install LFS hooks
	if !install(git_dir, worktree)? {
		return Ok(());
	}

	// Fetch objects
	fetch(git_dir, worktree)?;

	// Checkout files
	checkout(git_dir, worktree)
}
