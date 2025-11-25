use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use dunce::canonicalize;
use gix::attrs::StateRef;
use gix::bstr::ByteSlice;
use walkdir::WalkDir;

use crate::{git, picker};

pub fn run(tag: Option<&str>, auto_yes: bool, repo_dir: Option<&Path>, attribute: &str) -> Result<()> {
	let (repo, root) = git::open_repository(repo_dir)?;
	let worktree = git::require_worktree(&repo)?;

	// If no tag provided and not auto-yes, discover available tags and show picker
	// Track whether we selected the tag interactively to avoid showing a second picker
	let (selected_tag, tag_was_interactive) = match tag {
		Some(t) => (t.to_owned(), false),
		None => {
			if auto_yes {
				anyhow::bail!(
					"tag argument is required when using --yes; run without --yes to select interactively"
				);
			}
			(select_tag_interactively(&repo, &worktree, &root, attribute)?, true)
		}
	};

	let mut state = CollectState::new();
	collect_repository(&repo, &worktree, &selected_tag, "", &mut state, attribute)?;

	if state.matches.is_empty() {
		anyhow::bail!(
			"no matching attribute entries found for tag '{}' in {}",
			selected_tag,
			root.display()
		);
	}

	// Skip the preview picker if:
	// - auto_yes is set, OR
	// - the tag was already selected interactively (user already made their choice)
	if auto_yes || tag_was_interactive {
		for pattern in &state.patterns {
			println!("{}", pattern);
		}
		return Ok(());
	}

	// Show preview picker only when tag was provided via CLI (let user confirm/browse)
	let patterns: Vec<String> = state.patterns.into_iter().collect();
	let attributes = state
		.tag_counts
		.into_iter()
		.map(|(name, count)| picker::AttributeRow::new(name, count))
		.collect();
	let files = state
		.file_map
		.into_iter()
		.map(|(path, tags)| picker::FileRow::new(path, tags))
		.collect();

	let data = picker::SearchData::new()
		.with_context(root.display().to_string())
		.with_initial_query(&selected_tag)
		.with_attributes(attributes)
		.with_files(files);

	let outcome = picker::SearchUi::new(data)
		.with_ui_config(picker::UiConfig::tags_and_files())
		.run()?;

	if !outcome.accepted {
		anyhow::bail!("aborted by user");
	}

	for pattern in patterns {
		println!("{}", pattern);
	}

	Ok(())
}

/// Discover all available tags in the repository and show a picker for selection.
fn select_tag_interactively(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
	root: &Path,
	attribute: &str,
) -> Result<String> {
	let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
	discover_all_tags(repo, worktree, "", &mut tag_counts, attribute)?;

	if tag_counts.is_empty() {
		anyhow::bail!(
			"no '{}' attributes found in {}; ensure .gitattributes files define the '{}' attribute",
			attribute,
			root.display(),
			attribute
		);
	}

	let attributes: Vec<picker::AttributeRow> = tag_counts
		.into_iter()
		.map(|(name, count)| picker::AttributeRow::new(name, count))
		.collect();

	let data = picker::SearchData::new()
		.with_context(root.display().to_string())
		.with_attributes(attributes);

	let outcome = picker::SearchUi::new(data)
		.with_input_title("Select a project tag")
		.with_ui_config(picker::UiConfig::tags_and_files())
		.run()?;

	if !outcome.accepted {
		anyhow::bail!("aborted by user");
	}

	match outcome.selection {
		Some(picker::SearchSelection::Attribute(attr)) => Ok(attr.name),
		Some(picker::SearchSelection::File(_)) => {
			anyhow::bail!("unexpected file selection; please select a tag")
		}
		None => {
			// User typed a custom query without selecting an item - use the query as the tag
			if outcome.query.trim().is_empty() {
				anyhow::bail!("no tag selected");
			}
			Ok(outcome.query.trim().to_owned())
		}
	}
}

/// Recursively discover all unique tags/attributes in the repository and submodules.
fn discover_all_tags(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
	prefix: &str,
	tag_counts: &mut BTreeMap<String, usize>,
	attribute: &str,
) -> Result<()> {
	let base_display = worktree.base().display().to_string();
	let mut attr_stack = worktree
		.attributes(None)
		.with_context(|| format!("failed to load git attribute stack for {}", base_display))?;
	let mut outcome = attr_stack.selected_attribute_matches([attribute]);

	let index = repo.open_index().with_context(|| {
		format!(
			"failed to load git index for repository at {}",
			base_display
		)
	})?;

	let mut processed_submodules: BTreeSet<String> = BTreeSet::new();

	for entry in index.entries() {
		let path = entry.path(&index);
		let path_display = path.to_str_lossy();
		let local_path = path_display.as_ref();

		if entry.mode == gix::index::entry::Mode::COMMIT {
			processed_submodules.insert(local_path.to_owned());
			let submodule_worktree_path = worktree.base().join(local_path);
			if !submodule_worktree_path.exists() {
				continue;
			}

			let (sub_repo, _) =
				git::open_repository(Some(&submodule_worktree_path)).with_context(|| {
					format!(
						"failed to open submodule at {}",
						submodule_worktree_path.display()
					)
				})?;
			let sub_worktree = git::require_worktree(&sub_repo).with_context(|| {
				format!(
					"submodule at {} is bare; a worktree is required for this operation",
					submodule_worktree_path.display()
				)
			})?;

			let next_prefix = if prefix.is_empty() {
				local_path.to_owned()
			} else {
				format!("{}/{}", prefix, local_path)
			};

			discover_all_tags(&sub_repo, &sub_worktree, &next_prefix, tag_counts, attribute)?;
			continue;
		}

		let platform = attr_stack
			.at_entry(path, Some(entry.mode))
			.with_context(|| {
				format!(
					"failed to evaluate attributes for {}",
					if prefix.is_empty() {
						local_path.to_owned()
					} else {
						format!("{}/{}", prefix, local_path)
					}
				)
			})?;
		if platform.matching_attributes(&mut outcome)
			&& let Some(attr_state) = outcome.iter_selected().next().map(|m| m.assignment.state)
		{
			match attr_state {
				StateRef::Unspecified | StateRef::Unset => {}
				StateRef::Set => {
					*tag_counts.entry("global".to_owned()).or_insert(0) += 1;
				}
				StateRef::Value(value) => {
					let raw = value.as_bstr().to_str_lossy();
					for token in raw
						.split(',')
						.map(|token| token.trim())
						.filter(|s| !s.is_empty())
					{
						*tag_counts.entry(token.to_owned()).or_insert(0) += 1;
					}
				}
			}
		}
		outcome.reset();
	}

	for submodule_path in discover_submodules(repo, worktree)? {
		if processed_submodules.contains(&submodule_path) {
			continue;
		}

		let submodule_worktree_path = worktree.base().join(&submodule_path);
		if !submodule_worktree_path.exists() {
			continue;
		}

		let (sub_repo, _) =
			git::open_repository(Some(&submodule_worktree_path)).with_context(|| {
				format!(
					"failed to open submodule at {}",
					submodule_worktree_path.display()
				)
			})?;
		let sub_worktree = git::require_worktree(&sub_repo).with_context(|| {
			format!(
				"submodule at {} is bare; a worktree is required for this operation",
				submodule_worktree_path.display()
			)
		})?;

		let next_prefix = if prefix.is_empty() {
			submodule_path.clone()
		} else {
			format!("{}/{}", prefix, submodule_path)
		};

		discover_all_tags(&sub_repo, &sub_worktree, &next_prefix, tag_counts, attribute)?;
		processed_submodules.insert(submodule_path);
	}

	Ok(())
}

fn discover_submodules(
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

fn path_to_unix_string(path: &Path) -> String {
	let mut result = String::new();
	for component in path.components() {
		if !result.is_empty() {
			result.push('/');
		}
		result.push_str(&component.as_os_str().to_string_lossy());
	}
	result
}

fn record_match(state: &mut CollectState, pattern: &str, token: &str, user_tag: &str) {
	if token == "global" || token.contains(user_tag) {
		let pattern_owned = pattern.to_owned();
		let token_owned = token.to_owned();
		state
			.matches
			.push((pattern_owned.clone(), token_owned.clone()));
		state.patterns.insert(pattern_owned.clone());
		*state.tag_counts.entry(token_owned.clone()).or_insert(0) += 1;
		state
			.file_map
			.entry(pattern_owned)
			.or_default()
			.insert(token_owned);
	}
}

struct CollectState {
	matches: Vec<(String, String)>,
	patterns: BTreeSet<String>,
	tag_counts: BTreeMap<String, usize>,
	file_map: BTreeMap<String, BTreeSet<String>>,
}

impl CollectState {
	fn new() -> Self {
		Self {
			matches: Vec::new(),
			patterns: BTreeSet::new(),
			tag_counts: BTreeMap::new(),
			file_map: BTreeMap::new(),
		}
	}
}

fn collect_repository<'repo>(
	repo: &'repo gix::Repository,
	worktree: &gix::Worktree<'repo>,
	tag: &str,
	prefix: &str,
	state: &mut CollectState,
	attribute: &str,
) -> Result<()> {
	let base_display = worktree.base().display().to_string();
	let mut attr_stack = worktree
		.attributes(None)
		.with_context(|| format!("failed to load git attribute stack for {}", base_display))?;
	let mut outcome = attr_stack.selected_attribute_matches([attribute]);

	let index = repo.open_index().with_context(|| {
		format!(
			"failed to load git index for repository at {}",
			base_display
		)
	})?;

	let mut processed_submodules: BTreeSet<String> = BTreeSet::new();

	for entry in index.entries() {
		let path = entry.path(&index);
		let path_display = path.to_str_lossy();
		let local_path = path_display.as_ref();

		if entry.mode == gix::index::entry::Mode::COMMIT {
			processed_submodules.insert(local_path.to_owned());
			let submodule_worktree_path = worktree.base().join(local_path);
			if !submodule_worktree_path.exists() {
				continue;
			}

			let (sub_repo, _) =
				git::open_repository(Some(&submodule_worktree_path)).with_context(|| {
					format!(
						"failed to open submodule at {}",
						submodule_worktree_path.display()
					)
				})?;
			let sub_worktree = git::require_worktree(&sub_repo).with_context(|| {
				format!(
					"submodule at {} is bare; a worktree is required for this operation",
					submodule_worktree_path.display()
				)
			})?;

			let next_prefix = if prefix.is_empty() {
				local_path.to_owned()
			} else {
				format!("{}/{}", prefix, local_path)
			};

			collect_repository(&sub_repo, &sub_worktree, tag, &next_prefix, state, attribute)?;
			continue;
		}

		let pattern = if prefix.is_empty() {
			local_path.to_owned()
		} else {
			format!("{}/{}", prefix, local_path)
		};

		let platform = attr_stack
			.at_entry(path, Some(entry.mode))
			.with_context(|| format!("failed to evaluate attributes for {}", pattern))?;
		if platform.matching_attributes(&mut outcome)
			&& let Some(attr_state) = outcome.iter_selected().next().map(|m| m.assignment.state)
		{
			match attr_state {
				StateRef::Unspecified | StateRef::Unset => {}
				StateRef::Set => {
					record_match(state, &pattern, "global", tag);
				}
				StateRef::Value(value) => {
					let raw = value.as_bstr().to_str_lossy();
					for token in raw
						.split(',')
						.map(|token| token.trim())
						.filter(|s| !s.is_empty())
					{
						record_match(state, &pattern, token, tag);
					}
				}
			}
		}
		outcome.reset();
	}

	for submodule_path in discover_submodules(repo, worktree)? {
		if processed_submodules.contains(&submodule_path) {
			continue;
		}

		let submodule_worktree_path = worktree.base().join(&submodule_path);
		if !submodule_worktree_path.exists() {
			continue;
		}

		let (sub_repo, _) =
			git::open_repository(Some(&submodule_worktree_path)).with_context(|| {
				format!(
					"failed to open submodule at {}",
					submodule_worktree_path.display()
				)
			})?;
		let sub_worktree = git::require_worktree(&sub_repo).with_context(|| {
			format!(
				"submodule at {} is bare; a worktree is required for this operation",
				submodule_worktree_path.display()
			)
		})?;

		let next_prefix = if prefix.is_empty() {
			submodule_path.clone()
		} else {
			format!("{}/{}", prefix, submodule_path)
		};

		collect_repository(&sub_repo, &sub_worktree, tag, &next_prefix, state, attribute)?;
		processed_submodules.insert(submodule_path);
	}

	Ok(())
}

