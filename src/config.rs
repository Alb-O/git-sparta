use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Config {
	pub submodule_name: String,
	pub submodule_path: PathBuf,
	pub submodule_path_relative: PathBuf,
	pub submodule_url: String,
	pub submodule_branch: String,
	pub project_tag: String,
	pub shared_mirror_path: Option<PathBuf>,
	pub config_file: PathBuf,
	pub work_repo: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Overrides {
	pub submodule_url: Option<String>,
	pub shared_mirror_path: Option<PathBuf>,
}

impl Config {
	pub fn load(config_dir: &Path) -> Result<Self> {
		let config_dir = config_dir
			.canonicalize()
			.with_context(|| format!("Failed to canonicalize {}", config_dir.display()))?;
		let (mut base, config_file) = find_base_config(&config_dir)?;
		base.config_file = config_file;
		base.work_repo = config_dir.clone();

		// Apply local overrides first, then env overrides.
		let overrides = load_local_overrides(&config_dir)?;
		apply_overrides(&mut base, &overrides);
		let env_overrides = load_env_overrides();
		apply_overrides(&mut base, &env_overrides);

		// Ensure absolute paths and derive relative location inside work repo.
		if base.submodule_path.is_relative() {
			base.submodule_path = config_dir.join(&base.submodule_path);
		}
		base.submodule_path = normalize(&base.submodule_path);
		let relative =
			pathdiff::diff_paths(&base.submodule_path, &config_dir).ok_or_else(|| {
				anyhow::anyhow!(
					"unable to express submodule path {} relative to {}",
					base.submodule_path.display(),
					config_dir.display()
				)
			})?;
		base.submodule_path_relative = relative;

		if let Some(path) = base.shared_mirror_path.as_mut() {
			if path.is_relative() {
				*path = normalize(&config_dir.join(&path));
			} else {
				*path = normalize(path);
			}
		}

		Ok(base)
	}
}

fn find_base_config(config_dir: &Path) -> Result<(Config, PathBuf)> {
	let mut entries: Vec<_> = fs::read_dir(config_dir)?
		.filter_map(|entry| entry.ok())
		.map(|entry| entry.path())
		.filter(|path| {
			path.extension()
				.and_then(|ext| ext.to_str())
				.map(|ext| ext.eq_ignore_ascii_case("json"))
				.unwrap_or(false)
		})
		.collect();
	entries.sort();

	let required_keys = [
		"SUBMODULE_NAME",
		"SUBMODULE_PATH",
		"SUBMODULE_URL",
		"SUBMODULE_BRANCH",
		"PROJECT_TAG",
	];

	for candidate in entries {
		let contents = fs::read_to_string(&candidate)
			.with_context(|| format!("failed to read {}", candidate.display()))?;
		let json: Value = serde_json::from_str(&contents)
			.with_context(|| format!("failed to parse {} as JSON", candidate.display()))?;
		if let Some(object) = first_object_with_keys(&json, &required_keys) {
			let config = Config {
				submodule_name: get_string(object, "SUBMODULE_NAME")?,
				submodule_path: PathBuf::from(get_string(object, "SUBMODULE_PATH")?),
				submodule_path_relative: PathBuf::new(),
				submodule_url: get_string(object, "SUBMODULE_URL")?,
				submodule_branch: get_string(object, "SUBMODULE_BRANCH")?,
				project_tag: get_string(object, "PROJECT_TAG")?,
				shared_mirror_path: object
					.get("SHARED_MIRROR_PATH")
					.and_then(|v| v.as_str())
					.map(PathBuf::from),
				config_file: candidate.clone(),
				work_repo: config_dir.to_path_buf(),
			};
			return Ok((config, candidate));
		}
	}

	anyhow::bail!(
		"no JSON file in {} contained all required submodule keys",
		config_dir.display()
	);
}

fn load_local_overrides(config_dir: &Path) -> Result<Overrides> {
	let mut overrides = Overrides {
		submodule_url: None,
		shared_mirror_path: None,
	};

	let mut candidates: Vec<PathBuf> = Vec::new();
	for entry in (fs::read_dir(config_dir)?).flatten() {
		let path = entry.path();
		if path
			.extension()
			.and_then(|ext| ext.to_str())
			.map(|ext| ext.eq_ignore_ascii_case("json"))
			.unwrap_or(false)
			&& path
				.file_name()
				.and_then(|name| name.to_str())
				.map(|name| name.ends_with(".local.json"))
				.unwrap_or(false)
		{
			candidates.push(path);
		}
	}

	let dot_project = config_dir.join(".project_local.json");
	if dot_project.exists() {
		candidates.push(dot_project);
	}

	candidates.sort();
	for candidate in candidates {
		apply_single_override(candidate, &mut overrides)?;
	}

	Ok(overrides)
}

fn apply_single_override(path: PathBuf, overrides: &mut Overrides) -> Result<()> {
	if !path.exists() {
		return Ok(());
	}
	let contents = fs::read_to_string(&path)?;
	let json: Value = serde_json::from_str(&contents)?;

	if overrides.submodule_url.is_none()
		&& let Some(value) = first_value_for_key(&json, "SUBMODULE_URL")
	{
		overrides.submodule_url = Some(value);
	}
	if overrides.shared_mirror_path.is_none()
		&& let Some(value) = first_value_for_key(&json, "SHARED_MIRROR_PATH")
	{
		overrides.shared_mirror_path = Some(PathBuf::from(value));
	}
	Ok(())
}

fn load_env_overrides() -> Overrides {
	Overrides {
		submodule_url: std::env::var("SUBMODULE_URL")
			.ok()
			.filter(|s| !s.is_empty()),
		shared_mirror_path: std::env::var("SHARED_MIRROR_PATH")
			.ok()
			.filter(|s| !s.is_empty())
			.map(PathBuf::from),
	}
}

fn apply_overrides(config: &mut Config, overrides: &Overrides) {
	if let Some(url) = &overrides.submodule_url {
		config.submodule_url = url.clone();
	}
	if let Some(path) = &overrides.shared_mirror_path {
		config.shared_mirror_path = Some(path.clone());
	}
}

fn first_object_with_keys<'a>(
	value: &'a Value,
	keys: &[&str],
) -> Option<&'a serde_json::Map<String, Value>> {
	let mut queue = VecDeque::from([value]);
	while let Some(current) = queue.pop_front() {
		match current {
			Value::Object(map) => {
				if keys.iter().all(|key| map.contains_key(*key)) {
					return Some(map);
				}
				queue.extend(map.values());
			}
			Value::Array(items) => queue.extend(items.iter()),
			_ => {}
		}
	}
	None
}

fn first_value_for_key(value: &Value, key: &str) -> Option<String> {
	let mut queue = VecDeque::from([value]);
	while let Some(current) = queue.pop_front() {
		match current {
			Value::Object(map) => {
				if let Some(found) = map.get(key)
					&& let Some(s) = found.as_str()
				{
					return Some(s.to_owned());
				}
				queue.extend(map.values());
			}
			Value::Array(items) => queue.extend(items.iter()),
			_ => {}
		}
	}
	None
}

fn get_string(map: &serde_json::Map<String, Value>, key: &str) -> Result<String> {
	map.get(key)
		.and_then(|v| v.as_str())
		.map(|s| s.to_owned())
		.ok_or_else(|| anyhow::anyhow!("missing required key {key}"))
}

fn normalize(path: &Path) -> PathBuf {
	dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
