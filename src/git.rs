use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn find_non_submodule_root(start: &Path) -> Result<PathBuf> {
    let mut current = start
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", start.display()))?;
    loop {
        let git_path = current.join(".git");
        if git_path.is_dir() {
            return Ok(current);
        } else if git_path.is_file() {
            current = current.parent().map(Path::to_path_buf).ok_or_else(|| {
                anyhow::anyhow!("reached filesystem root without finding non-submodule repo")
            })?;
            continue;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                anyhow::bail!("{} is not inside a git repository", start.display())
            }
        }
    }
}

pub fn open_repository(start: Option<&Path>) -> Result<(gix::Repository, PathBuf)> {
    let start = start.unwrap_or_else(|| Path::new("."));
    let root = find_non_submodule_root(start)?;
    let repo = gix::open(&root)
        .with_context(|| format!("failed to open git repository at {}", root.display()))?;
    Ok((repo, root))
}

pub fn require_worktree<'repo>(repo: &'repo gix::Repository) -> Result<gix::Worktree<'repo>> {
    repo.worktree().ok_or_else(|| {
        anyhow::anyhow!("repository is bare; a worktree is required for this operation")
    })
}
