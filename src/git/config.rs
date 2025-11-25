//! Git configuration file manipulation.
//!
//! This module provides utilities for reading and writing git configuration files
//! (`.gitmodules`, local config, etc.) using the gix library.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use gix::bstr::{BStr, BString, ByteSlice};
use gix::config::{File as GitConfigFile, Source};
use gix::sec::Trust;

/// A wrapper around a git configuration file for easier manipulation.
pub struct ConfigFile {
	file: GitConfigFile<'static>,
	path: std::path::PathBuf,
	dirty: bool,
}

impl ConfigFile {
	/// Open an existing config file, or create a new one if it doesn't exist.
	pub fn open_or_create(path: &Path) -> Result<Self> {
		let file = if path.exists() {
			GitConfigFile::from_path_no_includes(path.to_path_buf(), Source::Local)
				.with_context(|| format!("failed to load {}", path.display()))?
		} else {
			let metadata = gix::config::file::Metadata::from(Source::Local)
				.at(path)
				.with(Trust::Full);
			GitConfigFile::new(metadata)
		};

		Ok(Self {
			file,
			path: path.to_path_buf(),
			dirty: false,
		})
	}

	/// Open an existing config file, returning an error if it doesn't exist.
	pub fn open(path: &Path) -> Result<Self> {
		let file = GitConfigFile::from_path_no_includes(path.to_path_buf(), Source::Local)
			.with_context(|| format!("failed to load {}", path.display()))?;

		Ok(Self {
			file,
			path: path.to_path_buf(),
			dirty: false,
		})
	}

	/// Set a value in the configuration file.
	///
	/// Returns `true` if the value was changed (or newly set).
	pub fn set_value(
		&mut self,
		section: &str,
		subsection: Option<&str>,
		key: &str,
		value: &str,
	) -> Result<bool> {
		let subsection_bstring = subsection.map(BString::from);
		let subsection_ref: Option<&BStr> = subsection_bstring.as_ref().map(|s| s.as_bstr());

		let value_bstring = BString::from(value);
		let value_ref: &BStr = value_bstring.as_bstr();

		let previous =
			self.file
				.set_raw_value_by(section, subsection_ref, key.to_owned(), value_ref)?;

		let changed = previous
			.map(|prev| prev.as_ref() != value_ref)
			.unwrap_or(true);

		if changed {
			self.dirty = true;
		}

		Ok(changed)
	}

	/// Remove a section from the configuration file.
	///
	/// Returns `true` if the section was removed.
	pub fn remove_section(&mut self, section: &str, subsection: Option<&str>) -> bool {
		let subsection_bstring = subsection.map(BString::from);
		let subsection_ref: Option<&BStr> = subsection_bstring.as_ref().map(|s| s.as_bstr());

		let removed = self.file.remove_section(section, subsection_ref);
		if removed.is_some() {
			self.dirty = true;
			true
		} else {
			false
		}
	}

	/// Write changes to disk if the file has been modified.
	///
	/// Returns `true` if the file was written.
	pub fn save(&self) -> Result<bool> {
		if !self.dirty {
			return Ok(false);
		}

		let mut buf = Vec::new();
		self.file.write_to(&mut buf)?;
		fs::write(&self.path, buf)?;
		Ok(true)
	}

	/// Check if the file has unsaved changes.
	pub fn is_dirty(&self) -> bool {
		self.dirty
	}
}

/// Helper for managing submodule configuration in `.gitmodules` and local config.
pub struct SubmoduleConfig<'a> {
	name: &'a str,
}

impl<'a> SubmoduleConfig<'a> {
	pub fn new(name: &'a str) -> Self {
		Self { name }
	}

	/// Ensure submodule entry exists in `.gitmodules` with the given values.
	///
	/// Returns `true` if any changes were made.
	pub fn ensure_gitmodules(
		&self,
		gitmodules_path: &Path,
		path: &str,
		url: &str,
		branch: &str,
	) -> Result<bool> {
		let mut config = ConfigFile::open_or_create(gitmodules_path)?;

		let mut changed = false;
		changed |= config.set_value("submodule", Some(self.name), "path", path)?;
		changed |= config.set_value("submodule", Some(self.name), "url", url)?;
		changed |= config.set_value("submodule", Some(self.name), "branch", branch)?;

		config.save()?;
		Ok(changed)
	}

	/// Ensure submodule entry exists in local git config with the given values.
	///
	/// Returns `true` if any changes were made.
	pub fn ensure_local_config(
		&self,
		git_config_path: &Path,
		url: &str,
		branch: &str,
	) -> Result<bool> {
		let mut config = ConfigFile::open(git_config_path)?;

		let mut changed = false;
		changed |= config.set_value("submodule", Some(self.name), "url", url)?;
		changed |= config.set_value("submodule", Some(self.name), "branch", branch)?;

		config.save()?;
		Ok(changed)
	}

	/// Remove submodule entry from `.gitmodules`.
	///
	/// Returns `true` if the entry was removed.
	pub fn remove_from_gitmodules(&self, gitmodules_path: &Path) -> Result<bool> {
		if !gitmodules_path.exists() {
			return Ok(false);
		}

		let mut config = ConfigFile::open(gitmodules_path)?;
		let removed = config.remove_section("submodule", Some(self.name));
		config.save()?;
		Ok(removed)
	}

	/// Remove submodule entry from local git config.
	///
	/// Returns `true` if the entry was removed.
	pub fn remove_from_local_config(&self, git_config_path: &Path) -> Result<bool> {
		if !git_config_path.exists() {
			return Ok(false);
		}

		let mut config = ConfigFile::open(git_config_path)?;
		let removed = config.remove_section("submodule", Some(self.name));
		config.save()?;
		Ok(removed)
	}
}
