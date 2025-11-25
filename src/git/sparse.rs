//! Sparse checkout operations.

use std::fs;
use std::path::Path;

use anyhow::Result;

use super::git;

/// Configure sparse checkout for a repository.
pub fn configure(git_dir: &Path, patterns: &[String]) -> Result<()> {
	// Enable sparse checkout
	git()
		.git_dir(git_dir)
		.args(["config", "core.sparseCheckout", "true"])
		.run()?;

	// Write sparse-checkout file
	let sparse_file = git_dir.join("info/sparse-checkout");
	fs::create_dir_all(git_dir.join("info"))?;
	fs::write(&sparse_file, patterns.join("\n") + "\n")?;

	Ok(())
}

/// Materialize sparse checkout files into the worktree.
pub fn checkout(git_dir: &Path, worktree: &Path) -> Result<()> {
	// Run read-tree to update the index with sparse patterns
	git()
		.git_dir(git_dir)
		.work_tree(worktree)
		.args(["read-tree", "-mu", "HEAD"])
		.run()?;

	// Checkout the files
	git()
		.git_dir(git_dir)
		.work_tree(worktree)
		.args(["checkout-index", "--all", "--force"])
		.run()
}
