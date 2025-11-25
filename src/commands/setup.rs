use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use gix::bstr::ByteSlice;

use crate::config::Config;
use crate::git::{self, attributes, config as git_config, git, lfs, sparse, submodule};
use crate::output;

pub fn run(config_dir: Option<&Path>, auto_yes: bool) -> Result<()> {
	let config_dir = config_dir.unwrap_or_else(|| Path::new("."));
	let config = Config::load(config_dir)?;

	// Generate sparse patterns first
	let sparse_patterns = generate_sparse_patterns(&config)?;

	output::divider();
	output::heading("Submodule setup summary");
	output::label_value("Configuration", config.config_file.display());
	output::label_value("Submodule", &config.submodule_name);
	output::label_value("Path", config.submodule_path.display());
	output::label_value("URL", &config.submodule_url);
	output::label_value("Branch", &config.submodule_branch);
	output::label_value("Project Tag", &config.project_tag);
	output::label_value("Sparse Patterns", sparse_patterns.len());
	if let Some(mirror) = &config.shared_mirror_path {
		output::label_value("Mirror", mirror.display());
	} else {
		output::note("Mirror: <none>");
	}
	output::divider();

	if !output::confirm("Proceed with submodule setup?", true, auto_yes)? {
		anyhow::bail!("aborted by user");
	}

	// Open the current repository (which might be a submodule itself)
	let (repo, repo_root) = git::open_repository(Some(&config.work_repo))?;
	let git_dir = repo.git_dir().to_path_buf();

	output::note(&format!("Working in repository: {}", repo_root.display()));
	output::note(&format!("Git directory: {}", git_dir.display()));

	// Update .gitmodules and local git config using shared config module
	let submodule_cfg = git_config::SubmoduleConfig::new(&config.submodule_name);

	let gitmodules_changed = submodule_cfg.ensure_gitmodules(
		&config.work_repo.join(".gitmodules"),
		&config.submodule_path_relative.to_string_lossy(),
		&config.submodule_url,
		&config.submodule_branch,
	)?;

	let git_config_changed = submodule_cfg.ensure_local_config(
		&git_dir.join("config"),
		&config.submodule_url,
		&config.submodule_branch,
	)?;

	if gitmodules_changed {
		output::success("✓ Updated .gitmodules");
	}
	if git_config_changed {
		output::success("✓ Updated local git configuration");
	}

	// Calculate the modules path
	let modules_path = git_dir
		.join("modules")
		.join(&config.submodule_path_relative);

	// Check if gitlink already exists in index
	let gitlink_exists = check_gitlink_exists(&repo, &config.submodule_path_relative)?;

	if !gitlink_exists {
		output::note("Creating gitlink in index...");
		let commit_sha = fetch_commit_sha(&config)?;
		add_gitlink(&repo, &config.submodule_path_relative, &commit_sha)?;
		output::success("✓ Added gitlink to index");
	} else {
		output::note("Gitlink already exists in index");
	}

	// Initialize the submodule metadata
	output::note("Initializing submodule metadata...");
	git_submodule_init(&config.work_repo, &config.submodule_path_relative)?;
	output::success("✓ Submodule initialized");

	// Create the working tree directory
	fs::create_dir_all(&config.submodule_path)
		.with_context(|| format!("failed to create {}", config.submodule_path.display()))?;

	// Set up the modules directory (the actual .git directory for the submodule)
	setup_modules_directory(&modules_path, &config)?;
	output::success(&format!(
		"✓ Set up modules directory: {}",
		modules_path.display()
	));

	// Create the .git file in the submodule working tree
	let relative_modules = pathdiff::diff_paths(&modules_path, &config.submodule_path)
		.context("failed to compute relative path to modules directory")?;
	let gitfile_content = format!("gitdir: {}\n", relative_modules.display());
	fs::write(config.submodule_path.join(".git"), gitfile_content)?;
	output::success("✓ Created .git file in submodule working tree");

	// Configure core.bare and core.worktree
	configure_modules_repo(&modules_path, &config.submodule_path)?;
	output::success("✓ Configured modules repository");

	// Add remote if it doesn't exist
	add_remote_if_missing(&modules_path, &config.submodule_url)?;

	// Fetch the commit
	fetch_to_modules(&modules_path, &config, gitlink_exists)?;
	output::success("✓ Fetched remote content");

	// Set up sparse checkout
	setup_sparse_checkout(&modules_path, &sparse_patterns)?;
	output::success(&format!(
		"✓ Configured sparse checkout ({} patterns)",
		sparse_patterns.len()
	));

	// Materialize the sparse files
	materialize_sparse_files(&modules_path, &config.submodule_path)?;
	output::success("✓ Materialized sparse files");

	// Handle LFS if the repository uses it
	if repo_uses_lfs(&config.submodule_path) {
		fetch_lfs_objects(&modules_path, &config.submodule_path)?;
		output::success("✓ LFS objects fetched and checked out");
	}

	output::divider();
	output::success(&format!(
		"✓ Submodule '{}' successfully set up with sparse checkout!",
		config.submodule_name
	));
	output::note(&format!(
		"Working tree: {}",
		config.submodule_path.display()
	));

	Ok(())
}

fn generate_sparse_patterns(config: &Config) -> Result<Vec<String>> {
	output::note("Generating sparse patterns...");

	// Use the mirror if available, otherwise use the local submodule path
	let repo_path = if let Some(mirror) = &config.shared_mirror_path {
		mirror.clone()
	} else {
		config.submodule_path.clone()
	};

	// Check if the path is a git repository (either .git directory or .git file for worktrees)
	let git_path = repo_path.join(".git");
	if !git_path.exists() {
		if config.shared_mirror_path.is_some() {
			anyhow::bail!(
				"No git repository found at mirror path: {}\n\
				 Ensure SHARED_MIRROR_PATH points to a valid git repository.",
				repo_path.display()
			);
		} else {
			anyhow::bail!(
				"No git repository found at: {}\n\n\
				 To generate sparse patterns, git-sparta needs access to the repository's \
				 .gitattributes files. You can either:\n\
				 1. Set SHARED_MIRROR_PATH in your config to point to a local clone/mirror\n\
				 2. Set the SHARED_MIRROR_PATH environment variable\n\
				 3. Clone the repository first and run setup again\n\n\
				 Example config:\n\
				 {{\n\
				   \"SHARED_MIRROR_PATH\": \"/path/to/local/mirror\"\n\
				 }}",
				repo_path.display()
			);
		}
	}

	let (repo, _) = git::open_repository(Some(&repo_path))?;
	let worktree = git::require_worktree(&repo)?;

	// Use the shared attributes module to collect sparse patterns
	let patterns =
		attributes::collect_sparse_patterns(&repo, &worktree, &config.project_tag, "projects")?;

	if patterns.is_empty() {
		anyhow::bail!("No patterns found for tag '{}'", config.project_tag);
	}

	Ok(patterns.into_iter().collect())
}

fn check_gitlink_exists(repo: &gix::Repository, submodule_path: &Path) -> Result<bool> {
	let index = match repo.open_index() {
		Ok(index) => index,
		Err(_) => {
			// No index means no gitlink exists
			return Ok(false);
		}
	};
	let path_str = submodule_path.to_string_lossy();

	for entry in index.entries() {
		let entry_path = entry.path(&index).to_str_lossy();
		if entry_path == path_str.as_ref() && entry.mode == gix::index::entry::Mode::COMMIT {
			return Ok(true);
		}
	}

	Ok(false)
}

fn fetch_commit_sha(config: &Config) -> Result<String> {
	output::note("Fetching commit SHA from remote...");

	// Use a temporary directory for the fetch
	let temp_dir = tempfile::tempdir()?;
	let temp_path = temp_dir.path();

	// Initialize a bare repository
	git::repository::init_bare(temp_path)?;

	// Add remote
	submodule::add_remote_if_missing(temp_path, "origin", &config.submodule_url)?;

	// Configure alternates if using mirror
	if let Some(mirror) = &config.shared_mirror_path {
		submodule::configure_alternates(temp_path, mirror)?;
	}

	// Fetch
	submodule::fetch(temp_path, "origin", &config.submodule_branch, Some(1))?;

	// Get the SHA
	let sha = git()
		.git_dir(temp_path)
		.args(["rev-parse", "FETCH_HEAD"])
		.stdout()?;

	Ok(sha)
}

fn add_gitlink(repo: &gix::Repository, submodule_path: &Path, commit_sha: &str) -> Result<()> {
	let repo_path = repo
		.workdir()
		.context("repository has no working directory")?;
	submodule::add_gitlink(repo_path, submodule_path, commit_sha)
}

fn git_submodule_init(work_repo: &Path, submodule_path: &Path) -> Result<()> {
	submodule::init(work_repo, submodule_path)
}

fn setup_modules_directory(modules_path: &Path, config: &Config) -> Result<()> {
	submodule::setup_modules_directory(modules_path, config.shared_mirror_path.as_deref())
}

fn configure_modules_repo(modules_path: &Path, worktree_path: &Path) -> Result<()> {
	submodule::configure_modules_repo(modules_path, worktree_path)
}

fn add_remote_if_missing(modules_path: &Path, remote_url: &str) -> Result<()> {
	submodule::add_remote_if_missing(modules_path, "origin", remote_url)?;
	Ok(())
}

fn fetch_to_modules(modules_path: &Path, config: &Config, _gitlink_exists: bool) -> Result<()> {
	// Get the commit SHA from the gitlink
	let commit_sha =
		submodule::get_gitlink_sha(&config.work_repo, &config.submodule_path_relative)?;

	// Check if we already have the commit
	if !submodule::has_commit(modules_path, &commit_sha)? {
		output::note(&format!("Fetching commit {}...", commit_sha));
		submodule::fetch(modules_path, "origin", &config.submodule_branch, Some(1))?;
	}

	// Update refs
	submodule::update_refs(modules_path, &commit_sha, &config.submodule_branch)?;

	Ok(())
}

fn setup_sparse_checkout(modules_path: &Path, patterns: &[String]) -> Result<()> {
	sparse::configure(modules_path, patterns)
}

fn materialize_sparse_files(modules_path: &Path, worktree_path: &Path) -> Result<()> {
	sparse::checkout(modules_path, worktree_path)
}

/// Check if the repository uses Git LFS by looking for filter=lfs in .gitattributes
fn repo_uses_lfs(worktree_path: &Path) -> bool {
	lfs::is_enabled(worktree_path)
}

/// Fetch and checkout LFS objects for the sparse checkout
fn fetch_lfs_objects(modules_path: &Path, worktree_path: &Path) -> Result<()> {
	lfs::fetch_and_checkout(modules_path, worktree_path)
}
