//! Git command execution.
//!
//! This module provides a builder for executing git plumbing commands
//! that aren't available in gix or other Rust git libraries:
//! - update-index, read-tree, checkout-index (index manipulation)
//! - update-ref, symbolic-ref (ref manipulation)
//! - fetch, submodule (network operations)
//! - lfs commands (Git LFS extension)

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{Context, Result};

/// Builder for git commands with --git-dir and --work-tree support.
#[derive(Debug, Default)]
pub struct Git {
	git_dir: Option<String>,
	work_tree: Option<String>,
	cwd: Option<String>,
	args: Vec<String>,
}

impl Git {
	/// Set the git directory (--git-dir).
	pub fn git_dir(mut self, path: &Path) -> Self {
		self.git_dir = Some(path.to_string_lossy().into_owned());
		self
	}

	/// Set the work tree (--work-tree).
	pub fn work_tree(mut self, path: &Path) -> Self {
		self.work_tree = Some(path.to_string_lossy().into_owned());
		self
	}

	/// Set the current working directory for the command.
	pub fn cwd(mut self, path: &Path) -> Self {
		self.cwd = Some(path.to_string_lossy().into_owned());
		self
	}

	/// Add multiple arguments.
	pub fn args<I, S>(mut self, args: I) -> Self
	where
		I: IntoIterator<Item = S>,
		S: AsRef<OsStr>,
	{
		self.args.extend(
			args.into_iter()
				.map(|s| s.as_ref().to_string_lossy().into_owned()),
		);
		self
	}

	/// Add a single argument.
	pub fn arg<S: AsRef<OsStr>>(mut self, arg: S) -> Self {
		self.args.push(arg.as_ref().to_string_lossy().into_owned());
		self
	}

	/// Execute and return raw output.
	pub fn output(self) -> Result<Output> {
		let mut cmd = Command::new("git");

		if let Some(ref dir) = self.git_dir {
			cmd.arg("--git-dir").arg(dir);
		}
		if let Some(ref tree) = self.work_tree {
			cmd.arg("--work-tree").arg(tree);
		}
		if let Some(ref cwd) = self.cwd {
			cmd.current_dir(cwd);
		}

		cmd.args(&self.args);
		cmd.output()
			.with_context(|| format!("failed to execute: git {}", self.args.join(" ")))
	}

	/// Execute and require success.
	pub fn run(self) -> Result<()> {
		let desc = self.args.join(" ");
		let out = self.output()?;
		if !out.status.success() {
			let stderr = String::from_utf8_lossy(&out.stderr);
			anyhow::bail!("git {} failed: {}", desc, stderr.trim());
		}
		Ok(())
	}

	/// Execute and return stdout as trimmed string.
	pub fn stdout(self) -> Result<String> {
		let desc = self.args.join(" ");
		let out = self.output()?;
		if !out.status.success() {
			let stderr = String::from_utf8_lossy(&out.stderr);
			anyhow::bail!("git {} failed: {}", desc, stderr.trim());
		}
		Ok(String::from_utf8(out.stdout)?.trim().to_string())
	}

	/// Execute and return success status (for existence checks).
	pub fn ok(self) -> Result<bool> {
		Ok(self.output()?.status.success())
	}
}

/// Create a new git command builder.
pub fn git() -> Git {
	Git::default()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_git_version() {
		let version = git().args(["--version"]).stdout().unwrap();
		assert!(version.contains("git version"));
	}
}
