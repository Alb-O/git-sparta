//! Git attributes scanning and collection.
//!
//! This module provides utilities for scanning git repositories and collecting
//! attribute information, particularly for the "projects" attribute used by git-sparta.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use gix::attrs::StateRef;
use gix::bstr::ByteSlice;

use super::submodule::discover_submodules;
use crate::git;

/// Statistics about collected attributes/tags.
#[derive(Debug, Default)]
pub struct TagCounts(pub BTreeMap<String, usize>);

impl TagCounts {
	pub fn new() -> Self {
		Self::default()
	}

	/// Record a tag occurrence.
	pub fn record(&mut self, tag: &str) {
		*self.0.entry(tag.to_owned()).or_insert(0) += 1;
	}

	/// Check if any tags were found.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Consume and return the inner map.
	pub fn into_inner(self) -> BTreeMap<String, usize> {
		self.0
	}
}

/// State for collecting matching files and their tags.
#[derive(Debug, Default)]
pub struct CollectState {
	/// List of (pattern, tag) matches.
	pub matches: Vec<(String, String)>,
	/// Unique patterns that matched.
	pub patterns: BTreeSet<String>,
	/// Count of files per tag.
	pub tag_counts: BTreeMap<String, usize>,
	/// Map of pattern -> set of tags.
	pub file_map: BTreeMap<String, BTreeSet<String>>,
}

impl CollectState {
	pub fn new() -> Self {
		Self::default()
	}

	/// Record a match for the given pattern and token.
	pub fn record_match(&mut self, pattern: &str, token: &str, user_tag: &str) {
		if token == "global" || token.contains(user_tag) {
			let pattern_owned = pattern.to_owned();
			let token_owned = token.to_owned();
			self.matches
				.push((pattern_owned.clone(), token_owned.clone()));
			self.patterns.insert(pattern_owned.clone());
			*self.tag_counts.entry(token_owned.clone()).or_insert(0) += 1;
			self.file_map
				.entry(pattern_owned)
				.or_default()
				.insert(token_owned);
		}
	}
}

/// Discover all unique tags/attributes in a repository and its submodules.
///
/// This traverses the entire repository (and recursively into submodules)
/// to find all values of the specified attribute.
pub fn discover_all_tags(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
	attribute: &str,
) -> Result<TagCounts> {
	let mut tag_counts = TagCounts::new();
	discover_tags_recursive(repo, worktree, "", &mut tag_counts, attribute)?;
	Ok(tag_counts)
}

fn discover_tags_recursive(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
	prefix: &str,
	tag_counts: &mut TagCounts,
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

			discover_tags_recursive(
				&sub_repo,
				&sub_worktree,
				&next_prefix,
				tag_counts,
				attribute,
			)?;
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
					tag_counts.record("global");
				}
				StateRef::Value(value) => {
					let raw = value.as_bstr().to_str_lossy();
					for token in raw
						.split(',')
						.map(|token| token.trim())
						.filter(|s| !s.is_empty())
					{
						tag_counts.record(token);
					}
				}
			}
		}
		outcome.reset();
	}

	// Also check submodules discovered from .git/modules
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

		discover_tags_recursive(
			&sub_repo,
			&sub_worktree,
			&next_prefix,
			tag_counts,
			attribute,
		)?;
		processed_submodules.insert(submodule_path);
	}

	Ok(())
}

/// Collect files matching a specific tag from a repository and its submodules.
pub fn collect_matching_files(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
	tag: &str,
	attribute: &str,
) -> Result<CollectState> {
	let mut state = CollectState::new();
	collect_files_recursive(repo, worktree, tag, "", &mut state, attribute)?;
	Ok(state)
}

fn collect_files_recursive<'repo>(
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

			collect_files_recursive(
				&sub_repo,
				&sub_worktree,
				tag,
				&next_prefix,
				state,
				attribute,
			)?;
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
					state.record_match(&pattern, "global", tag);
				}
				StateRef::Value(value) => {
					let raw = value.as_bstr().to_str_lossy();
					for token in raw
						.split(',')
						.map(|token| token.trim())
						.filter(|s| !s.is_empty())
					{
						state.record_match(&pattern, token, tag);
					}
				}
			}
		}
		outcome.reset();
	}

	// Also check submodules discovered from .git/modules
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

		collect_files_recursive(
			&sub_repo,
			&sub_worktree,
			tag,
			&next_prefix,
			state,
			attribute,
		)?;
		processed_submodules.insert(submodule_path);
	}

	Ok(())
}

/// Scan a repository for patterns matching a tag (used by setup command).
///
/// This is a simplified version that only collects matching file patterns,
/// used when generating sparse checkout patterns.
pub fn collect_sparse_patterns(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
	tag: &str,
	attribute: &str,
) -> Result<BTreeSet<String>> {
	let mut patterns = BTreeSet::new();
	collect_patterns_recursive(repo, worktree, tag, attribute, &mut patterns)?;
	Ok(patterns)
}

fn collect_patterns_recursive(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
	tag: &str,
	attribute: &str,
	patterns: &mut BTreeSet<String>,
) -> Result<()> {
	let mut attr_stack = worktree
		.attributes(None)
		.context("failed to load git attribute stack")?;
	let mut outcome = attr_stack.selected_attribute_matches([attribute]);

	let index = repo.open_index().context("failed to load git index")?;

	for entry in index.entries() {
		let path = entry.path(&index);
		let platform = attr_stack
			.at_entry(path, Some(entry.mode))
			.with_context(|| format!("failed to evaluate attributes for {}", path))?;

		if platform.matching_attributes(&mut outcome)
			&& let Some(state) = outcome.iter_selected().next().map(|m| m.assignment.state)
		{
			match state {
				StateRef::Set => {
					// Global tag
					patterns.insert(path.to_str_lossy().into_owned());
				}
				StateRef::Value(value) => {
					let raw = value.as_bstr().to_str_lossy();
					for token in raw.split(',').map(|s| s.trim()) {
						if token == "global" || token.contains(tag) {
							patterns.insert(path.to_str_lossy().into_owned());
							break;
						}
					}
				}
				_ => {}
			}
		}
		outcome.reset();
	}

	Ok(())
}
