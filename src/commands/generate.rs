use crate::{git, tui};
use anyhow::{Context, Result};
use gix::attrs::StateRef;
use gix::bstr::ByteSlice;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn run(tag: &str, auto_yes: bool, repo_dir: Option<&Path>) -> Result<()> {
    let (repo, root) = git::open_repository(repo_dir)?;
    let worktree = git::require_worktree(&repo)?;
    let mut attr_stack = worktree
        .attributes(None)
        .context("failed to load git attribute stack")?;
    let mut outcome = attr_stack.selected_attribute_matches(["projects"]);

    let index = repo
        .open_index()
        .context("failed to load git index for repository")?;

    let mut matches: Vec<(String, String)> = Vec::new();
    let mut unique_patterns: BTreeSet<String> = BTreeSet::new();
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut file_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for entry in index.entries() {
        let path = entry.path(&index);
        let platform = attr_stack
            .at_entry(path, Some(entry.mode))
            .with_context(|| format!("failed to evaluate attributes for {}", path))?;
        if platform.matching_attributes(&mut outcome)
            && let Some(state) = outcome.iter_selected().next().map(|m| m.assignment.state)
        {
            match state {
                StateRef::Unspecified | StateRef::Unset => {}
                StateRef::Set => {
                    let token = "global".to_string();
                    record_match(
                        &mut matches,
                        &mut unique_patterns,
                        path,
                        &token,
                        tag,
                        &mut tag_counts,
                        &mut file_map,
                    );
                }
                StateRef::Value(value) => {
                    let raw = value.as_bstr().to_str_lossy();
                    for token in raw
                        .split(',')
                        .map(|token| token.trim())
                        .filter(|s| !s.is_empty())
                    {
                        record_match(
                            &mut matches,
                            &mut unique_patterns,
                            path,
                            token,
                            tag,
                            &mut tag_counts,
                            &mut file_map,
                        );
                    }
                }
            }
        }
        outcome.reset();
    }

    if matches.is_empty() {
        anyhow::bail!(
            "no matching attribute entries found for tag '{}' in {}",
            tag,
            root.display()
        );
    }

    if auto_yes {
        for pattern in &unique_patterns {
            println!("{}", pattern);
        }
        return Ok(());
    }

    let patterns: Vec<String> = unique_patterns.into_iter().collect();
    let tags = tag_counts
        .into_iter()
        .map(|(name, count)| tui::TagRow { name, count })
        .collect();
    let files = file_map
        .into_iter()
        .map(|(path, tags)| tui::FileRow::new(path, tags.into_iter().collect()))
        .collect();

    let outcome = tui::run(tui::SearchData {
        repo_display: root.display().to_string(),
        user_filter: tag.to_string(),
        tags,
        files,
    })?;

    if !outcome.accepted {
        anyhow::bail!("aborted by user");
    }

    for pattern in patterns {
        println!("{}", pattern);
    }

    Ok(())
}

fn record_match(
    matches: &mut Vec<(String, String)>,
    patterns: &mut BTreeSet<String>,
    path: &gix::bstr::BStr,
    token: &str,
    user_tag: &str,
    tag_counts: &mut BTreeMap<String, usize>,
    file_map: &mut BTreeMap<String, BTreeSet<String>>,
) {
    if token == "global" || token.contains(user_tag) {
        let pattern = path.to_str_lossy().into_owned();
        let token_owned = token.to_owned();
        matches.push((pattern.clone(), token_owned.clone()));
        patterns.insert(pattern.clone());
        *tag_counts.entry(token_owned.clone()).or_insert(0) += 1;
        file_map.entry(pattern).or_default().insert(token_owned);
    }
}
