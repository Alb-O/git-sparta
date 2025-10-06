use crate::{git, output};
use anyhow::{Context, Result};
use gix::attrs::StateRef;
use gix::bstr::ByteSlice;
use std::collections::BTreeSet;
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
    let mut unique_tags: BTreeSet<String> = BTreeSet::new();

    for entry in index.entries() {
        let path = entry.path(&index);
        let platform = attr_stack
            .at_entry(path, Some(entry.mode))
            .with_context(|| format!("failed to evaluate attributes for {}", path))?;
        if platform.matching_attributes(&mut outcome)
            && let Some(state) = outcome.iter_selected().next().map(|m| m.assignment.state) {
                match state {
                    StateRef::Unspecified | StateRef::Unset => {}
                    StateRef::Set => {
                        let token = "global".to_string();
                        record_match(
                            &mut matches,
                            &mut unique_patterns,
                            &mut unique_tags,
                            path,
                            &token,
                            tag,
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
                                &mut unique_tags,
                                path,
                                token,
                                tag,
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

    output::divider();
    output::heading(&format!("Matched tags for input tag: {tag}"));
    output::note("(including \"global\" when present)");
    output::label_value("Tags", unique_tags.len());
    if unique_tags.is_empty() {
        output::note("  <none>");
    } else {
        output::bullet_list(unique_tags.iter().cloned());
    }
    output::label_value("Patterns", unique_patterns.len());
    output::divider();

    if !output::confirm("Proceed?", false, auto_yes)? {
        anyhow::bail!("aborted by user");
    }

    for pattern in unique_patterns {
        println!("{}", pattern);
    }

    Ok(())
}

fn record_match(
    matches: &mut Vec<(String, String)>,
    patterns: &mut BTreeSet<String>,
    tags: &mut BTreeSet<String>,
    path: &gix::bstr::BStr,
    token: &str,
    user_tag: &str,
) {
    if token == "global" || token.contains(user_tag) {
        let pattern = path.to_str_lossy().into_owned();
        matches.push((pattern.clone(), token.to_owned()));
        patterns.insert(pattern);
        tags.insert(token.to_owned());
    }
}
