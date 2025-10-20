use crate::{config::Config, git, output};
use anyhow::{Context, Result};
use gix::bstr::{BStr, BString, ByteSlice};
use gix::config::{File as GitConfigFile, Source};
use std::fs;
use std::path::Path;

pub fn run(config_dir: Option<&Path>, auto_yes: bool) -> Result<()> {
    let config_dir = config_dir.unwrap_or_else(|| Path::new("."));
    let config = Config::load(config_dir)?;

    output::divider();
    output::heading("Submodule teardown summary");
    output::label_value("Submodule", &config.submodule_name);
    output::label_value("Path", config.submodule_path.display());
    output::label_value("Project Tag", &config.project_tag);
    output::divider();

    if !output::confirm(
        &format!(
            "Remove submodule '{}' and clean metadata?",
            config.submodule_name
        ),
        false,
        auto_yes,
    )? {
        anyhow::bail!("aborted by user");
    }

    let (repo, _) = git::open_repository(Some(&config.work_repo))?;
    let git_dir = repo.git_dir().to_path_buf();

    let gitmodules_changed = remove_from_gitmodules(&config)?;
    let git_config_changed = remove_from_local_git_config(&git_dir, &config)?;

    if gitmodules_changed {
        output::success("Removed entry from .gitmodules");
    }
    if git_config_changed {
        output::success("Removed entry from local git config");
    }

    if config.submodule_path.exists() {
        fs::remove_dir_all(&config.submodule_path)
            .with_context(|| format!("failed to remove {}", config.submodule_path.display()))?;
        output::success(&format!(
            "Deleted working directory {}",
            config.submodule_path.display()
        ));
    }

    let modules_path = git_dir
        .join("modules")
        .join(&config.submodule_path_relative);
    if modules_path.exists() {
        fs::remove_dir_all(&modules_path)
            .with_context(|| format!("failed to remove {}", modules_path.display()))?;
        prune_empty_parents(modules_path.parent().unwrap_or(&modules_path), &git_dir)?;
        output::success("Removed modules repository");
    }

    output::success(&format!("Submodule '{}' removed", config.submodule_name));
    output::note("Review git status and stage removals as needed.");
    Ok(())
}

fn remove_from_gitmodules(config: &Config) -> Result<bool> {
    let path = config.work_repo.join(".gitmodules");
    if !path.exists() {
        return Ok(false);
    }
    let mut file = GitConfigFile::from_path_no_includes(path.clone(), Source::Local)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let subsection = BString::from(config.submodule_name.clone());
    let subsection_bytes: &[u8] = subsection.as_ref();
    let subsection_ref: &BStr = subsection_bytes.as_bstr();
    let removed = file.remove_section("submodule", Some(subsection_ref));

    if removed.is_some() {
        let mut buf = Vec::new();
        file.write_to(&mut buf)?;
        fs::write(&path, buf)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn remove_from_local_git_config(git_dir: &Path, config: &Config) -> Result<bool> {
    let path = git_dir.join("config");
    if !path.exists() {
        return Ok(false);
    }
    let mut file = GitConfigFile::from_path_no_includes(path.clone(), Source::Local)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let subsection = BString::from(config.submodule_name.clone());
    let subsection_bytes: &[u8] = subsection.as_ref();
    let subsection_ref: &BStr = subsection_bytes.as_bstr();
    let removed = file.remove_section("submodule", Some(subsection_ref));

    if removed.is_some() {
        let mut buf = Vec::new();
        file.write_to(&mut buf)?;
        fs::write(&path, buf)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn prune_empty_parents(start: &Path, git_dir: &Path) -> Result<()> {
    let mut current = start.to_path_buf();
    let modules_root = git_dir.join("modules");
    while current.starts_with(&modules_root) && current != modules_root {
        if fs::remove_dir(&current).is_ok() {
            current = current
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| modules_root.clone());
        } else {
            break;
        }
    }
    Ok(())
}
