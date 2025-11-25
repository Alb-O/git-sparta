//! Generate sparse-checkout patterns for a project tag.

use std::path::Path;

use anyhow::Result;

use crate::git::{self, attributes};
use crate::picker;

pub fn run(
	tag: Option<&str>,
	auto_yes: bool,
	repo_dir: Option<&Path>,
	attribute: &str,
) -> Result<()> {
	let (repo, root) = git::open_repository(repo_dir)?;
	let worktree = git::require_worktree(&repo)?;

	// If no tag provided and not auto-yes, discover available tags and show picker
	// Track whether we selected the tag interactively to avoid showing a second picker
	#[allow(non_snake_case)]
	let (selected_tag, tag_was_interactive) = match tag {
		Some(t) => (t.to_owned(), false),
		None => {
			if auto_yes {
				anyhow::bail!(
					"tag argument is required when using --yes; run without --yes to select interactively"
				);
			}
			(
				select_tag_interactively(&repo, &worktree, &root, attribute)?,
				true,
			)
		}
	};

	let state = attributes::collect_matching_files(&repo, &worktree, &selected_tag, attribute)?;

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
	let picker_attributes = state
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
		.with_attributes(picker_attributes)
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
#[allow(non_snake_case)]
fn select_tag_interactively(
	repo: &gix::Repository,
	worktree: &gix::Worktree<'_>,
	root: &Path,
	attribute: &str,
) -> Result<String> {
	let tag_counts = attributes::discover_all_tags(repo, worktree, attribute)?;

	if tag_counts.is_empty() {
		anyhow::bail!(
			"no '{}' attributes found in {}; ensure .gitattributes files define the '{}' attribute",
			attribute,
			root.display(),
			attribute
		);
	}

	let picker_attributes: Vec<picker::AttributeRow> = tag_counts
		.into_inner()
		.into_iter()
		.map(|(name, count)| picker::AttributeRow::new(name, count))
		.collect();

	let data = picker::SearchData::new()
		.with_context(root.display().to_string())
		.with_attributes(picker_attributes);

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
