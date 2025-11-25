//! Git submodule operations.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use dunce::canonicalize;
use walkdir::WalkDir;

use super::git;
use crate::output;

/// Convert a path to a Unix-style string (forward slashes).
pub fn path_to_unix_string(path: &Path) -> String {
	let mut result = String::new();
	for component in path.components() {
		if !result.is_empty() {
			result.push('/');
		}
		result.push_str(&component.as_os_str().to_string_lossy());
	}
	result
}

/// Discover submodules by scanning the `.git/modules` directory.
///
/// This is useful for finding submodules that may not be in the index yet,
/// or for discovering nested submodules.
pub fn discover_submodules(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
) -> Result<Vec<String>> {
	let modules_root = repo.git_dir().join("modules");
	if !modules_root.exists() {
		return Ok(Vec::new());
	}

	let mut submodules = Vec::new();
	for entry in WalkDir::new(&modules_root) {
		let entry = entry.with_context(|| {
			format!(
				"failed to traverse git modules at {}",
				modules_root.display()
			)
		})?;
		if !entry.file_type().is_file() || entry.file_name() != "config" {
			continue;
		}

		let Some(module_dir) = entry.path().parent() else {
			continue;
		};
		let rel_in_modules = match module_dir.strip_prefix(&modules_root) {
			Ok(rel) => rel,
			Err(_) => continue,
		};
		let module_dir_rel = path_to_unix_string(rel_in_modules);
		if module_dir_rel.is_empty() {
			continue;
		}

		let Some(worktree_rel) = module_worktree_relative(entry.path(), worktree.base())? else {
			continue;
		};

		if module_dir_rel != worktree_rel {
			let normalized = module_dir_rel.replace("/modules/", "/");
			if normalized == worktree_rel {
				continue;
			}
		}

		submodules.push(worktree_rel);
	}

	Ok(submodules)
}

/// Extract the worktree-relative path from a module's config file.
fn module_worktree_relative(config_path: &Path, repo_base: &Path) -> Result<Option<String>> {
	let Some(module_dir) = config_path.parent() else {
		return Ok(None);
	};
	let config = fs::read_to_string(config_path)
		.with_context(|| format!("failed to read {}", config_path.display()))?;
	let worktree_value = config.lines().find_map(|line| {
		let trimmed = line.trim();
		trimmed
			.strip_prefix("worktree")
			.and_then(|rest| {
				let rest = rest.trim_start();
				rest.strip_prefix('=').map(str::trim_start)
			})
			.map(str::trim)
			.filter(|value| !value.is_empty())
			.map(str::to_owned)
	});
	let Some(worktree_rel) = worktree_value else {
		return Ok(None);
	};
	let candidate = module_dir.join(&worktree_rel);
	let abs_path = match canonicalize(&candidate) {
		Ok(path) => path,
		Err(_) => return Ok(None),
	};
	let rel_path = match abs_path.strip_prefix(repo_base) {
		Ok(rel) => rel,
		Err(_) => return Ok(None),
	};
	let rel_str = path_to_unix_string(rel_path);
	if rel_str.is_empty() {
		return Ok(None);
	}
	Ok(Some(rel_str))
}

/// Initialize submodule metadata in the parent repository.
pub fn init(repo_path: &Path, submodule_path: &Path) -> Result<()> {
	git()
		.cwd(repo_path)
		.args(["submodule", "init", "--"])
		.arg(submodule_path)
		.run()
}

/// Add a gitlink entry to the index.
pub fn add_gitlink(repo_path: &Path, submodule_path: &Path, commit_sha: &str) -> Result<()> {
	git()
		.cwd(repo_path)
		.args(["update-index", "--add", "--cacheinfo", "160000", commit_sha])
		.arg(submodule_path)
		.run()
}

/// Get the commit SHA for a gitlink in the index.
pub fn get_gitlink_sha(repo_path: &Path, submodule_path: &Path) -> Result<String> {
	let output = git()
		.cwd(repo_path)
		.args(["ls-files", "--stage", "--"])
		.arg(submodule_path)
		.stdout()?;

	output
		.lines()
		.next()
		.and_then(|line| line.split_whitespace().nth(1))
		.map(|s| s.to_string())
		.context("no gitlink found in index")
}

/// Set up the modules directory (bare repository) for a submodule.
pub fn setup_modules_directory(modules_path: &Path, mirror_path: Option<&Path>) -> Result<()> {
	if !modules_path.exists() {
		output::note(&format!(
			"Initializing bare repository at {}",
			modules_path.display()
		));
		super::repository::init_bare(modules_path)?;
	}

	// Configure alternates if using mirror
	if let Some(mirror) = mirror_path {
		configure_alternates(modules_path, mirror)?;
	}

	Ok(())
}

/// Configure git alternates to share objects with a mirror.
pub fn configure_alternates(git_dir: &Path, mirror_path: &Path) -> Result<()> {
	let mirror_objects = mirror_path.join(".git/objects");
	if !mirror_objects.exists() {
		return Ok(());
	}

	let alternates_dir = git_dir.join("objects/info");
	fs::create_dir_all(&alternates_dir)?;
	let alternates_file = alternates_dir.join("alternates");

	let current = if alternates_file.exists() {
		fs::read_to_string(&alternates_file)?
	} else {
		String::new()
	};

	let mirror_path_str = mirror_objects.display().to_string();
	if !current.lines().any(|line| line == mirror_path_str) {
		let new_content = if current.is_empty() {
			format!("{}\n", mirror_path_str)
		} else {
			format!("{}{}\n", current, mirror_path_str)
		};
		fs::write(&alternates_file, new_content)?;
		output::note("Configured git alternates from mirror");
	}

	Ok(())
}

/// Configure the modules repository with worktree settings.
pub fn configure_modules_repo(modules_path: &Path, worktree_path: &Path) -> Result<()> {
	git()
		.git_dir(modules_path)
		.args(["config", "core.bare", "false"])
		.run()?;

	git()
		.git_dir(modules_path)
		.args(["config", "core.worktree"])
		.arg(worktree_path)
		.run()
}

/// Add a remote to a repository if it doesn't exist.
pub fn add_remote_if_missing(git_dir: &Path, name: &str, url: &str) -> Result<bool> {
	let exists = git()
		.git_dir(git_dir)
		.args(["remote", "get-url", name])
		.ok()?;

	if !exists {
		git()
			.git_dir(git_dir)
			.args(["remote", "add", name, url])
			.run()?;
		output::note(&format!("Added remote '{}'", name));
		Ok(true)
	} else {
		Ok(false)
	}
}

/// Fetch from a remote with optional shallow clone.
pub fn fetch(git_dir: &Path, remote: &str, refspec: &str, depth: Option<u32>) -> Result<()> {
	let mut args = vec!["fetch".to_string()];
	if let Some(d) = depth {
		args.push(format!("--depth={}", d));
	}
	args.push(remote.to_string());
	args.push(refspec.to_string());

	git().git_dir(git_dir).args(args).run()
}

/// Update HEAD and branch refs to point to a commit.
pub fn update_refs(git_dir: &Path, commit_sha: &str, branch: &str) -> Result<()> {
	// Update HEAD to point to the commit
	git()
		.git_dir(git_dir)
		.args(["update-ref", "HEAD", commit_sha])
		.run()?;

	// Update branch tracking
	git()
		.git_dir(git_dir)
		.args(["update-ref", &format!("refs/heads/{}", branch), commit_sha])
		.run()?;

	// Set symbolic ref
	git()
		.git_dir(git_dir)
		.args(["symbolic-ref", "HEAD", &format!("refs/heads/{}", branch)])
		.run()
}

/// Check if a commit exists in the object database.
pub fn has_commit(git_dir: &Path, commit_sha: &str) -> Result<bool> {
	git()
		.git_dir(git_dir)
		.args(["cat-file", "-e", commit_sha])
		.ok()
}
