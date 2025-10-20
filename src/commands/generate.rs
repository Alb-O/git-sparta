use crate::{git, picker};
use anyhow::{Context, Result};
use dunce::canonicalize;
use gix::attrs::StateRef;
use gix::bstr::ByteSlice;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn run(tag: &str, auto_yes: bool, repo_dir: Option<&Path>) -> Result<()> {
    let (repo, root) = git::open_repository(repo_dir)?;
    let worktree = git::require_worktree(&repo)?;

    let mut matches: Vec<(String, String)> = Vec::new();
    let mut unique_patterns: BTreeSet<String> = BTreeSet::new();
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut file_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    collect_repository(
        &repo,
        &worktree,
        tag,
        "",
        &mut matches,
        &mut unique_patterns,
        &mut tag_counts,
        &mut file_map,
    )?;

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
    let attributes = tag_counts
        .into_iter()
        .map(|(name, count)| picker::AttributeRow::new(name, count))
        .collect();
    let files = file_map
        .into_iter()
        .map(|(path, tags)| picker::FileRow::new(path, tags.into_iter()))
        .collect();

    let data = picker::SearchData::new()
        .with_context(root.display().to_string())
        .with_initial_query(tag)
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

        let module_dir = match entry.path().parent() {
            Some(dir) => dir,
            None => continue,
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
    let module_dir = match config_path.parent() {
        Some(dir) => dir,
        None => return Ok(None),
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

fn record_match(
    matches: &mut Vec<(String, String)>,
    patterns: &mut BTreeSet<String>,
    pattern: &str,
    token: &str,
    user_tag: &str,
    tag_counts: &mut BTreeMap<String, usize>,
    file_map: &mut BTreeMap<String, BTreeSet<String>>,
) {
    if token == "global" || token.contains(user_tag) {
        let pattern_owned = pattern.to_owned();
        let token_owned = token.to_owned();
        matches.push((pattern_owned.clone(), token_owned.clone()));
        patterns.insert(pattern_owned.clone());
        *tag_counts.entry(token_owned.clone()).or_insert(0) += 1;
        file_map
            .entry(pattern_owned)
            .or_default()
            .insert(token_owned);
    }
}

fn collect_repository<'repo>(
    repo: &'repo gix::Repository,
    worktree: &gix::Worktree<'repo>,
    tag: &str,
    prefix: &str,
    matches: &mut Vec<(String, String)>,
    patterns: &mut BTreeSet<String>,
    tag_counts: &mut BTreeMap<String, usize>,
    file_map: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<()> {
    let base_display = worktree.base().display().to_string();
    let mut attr_stack = worktree
        .attributes(None)
        .with_context(|| format!("failed to load git attribute stack for {}", base_display))?;
    let mut outcome = attr_stack.selected_attribute_matches(["projects"]);

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

            collect_repository(
                &sub_repo,
                &sub_worktree,
                tag,
                &next_prefix,
                matches,
                patterns,
                tag_counts,
                file_map,
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
            && let Some(state) = outcome.iter_selected().next().map(|m| m.assignment.state)
        {
            match state {
                StateRef::Unspecified | StateRef::Unset => {}
                StateRef::Set => {
                    record_match(
                        matches, patterns, &pattern, "global", tag, tag_counts, file_map,
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
                            matches, patterns, &pattern, token, tag, tag_counts, file_map,
                        );
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

        collect_repository(
            &sub_repo,
            &sub_worktree,
            tag,
            &next_prefix,
            matches,
            patterns,
            tag_counts,
            file_map,
        )?;
        processed_submodules.insert(submodule_path);
    }

    Ok(())
}
