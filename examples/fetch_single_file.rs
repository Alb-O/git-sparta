//! Example: Fetch a single file from a remote Git repository
//!
//! This demonstrates how to download a single file from a remote git repository
//! by resolving the appropriate raw file URL for different hosting providers.
//!
//! Supported providers:
//! - GitHub (github.com)
//! - GitLab (gitlab.com and self-hosted)
//! - Bitbucket (bitbucket.org)
//! - Codeberg (codeberg.org)
//! - sourcehut (git.sr.ht)
//!
//! Run with: cargo run --example fetch_single_file -- <repo_url> <file_path> [ref]
//!
//! Examples:
//!   cargo run --example fetch_single_file -- https://github.com/rust-lang/rust README.md
//!   cargo run --example fetch_single_file -- https://github.com/rust-lang/rust README.md main
//!   cargo run --example fetch_single_file -- git@github.com:rust-lang/rust.git Cargo.toml v1.75.0
//!   cargo run --example fetch_single_file -- https://gitlab.com/user/repo file.txt main

use std::borrow::Cow;

use anyhow::{Context, Result};

fn main() -> Result<()> {
	let args: Vec<String> = std::env::args().collect();

	if args.len() < 3 {
		eprintln!("Usage: {} <repo_url> <file_path> [ref]", args[0]);
		eprintln!();
		eprintln!("Arguments:");
		eprintln!("  repo_url   - Repository URL (HTTPS, SSH, or shorthand like 'owner/repo')");
		eprintln!("  file_path  - Path to file within the repository");
		eprintln!("  ref        - Branch, tag, or commit (default: HEAD or main)");
		eprintln!();
		eprintln!("Examples:");
		eprintln!("  {} https://github.com/rust-lang/rust README.md", args[0]);
		eprintln!(
			"  {} git@github.com:rust-lang/rust.git Cargo.toml main",
			args[0]
		);
		eprintln!("  {} rust-lang/rust README.md v1.75.0", args[0]);
		std::process::exit(1);
	}

	let repo_input = &args[1];
	let file_path = &args[2];
	let git_ref = args.get(3).map(|s| s.as_str());

	let repo_info = parse_repo_url(repo_input).context("failed to parse repository URL")?;

	println!("Repository: {}", repo_info);
	println!("File: {}", file_path);
	println!("Ref: {}", git_ref.unwrap_or("<default>"));

	let content = fetch_file(&repo_info, file_path, git_ref)?;

	println!("\n--- File Content ({} bytes) ---\n", content.len());

	// Print content (handle potential binary content)
	match std::str::from_utf8(&content) {
		Ok(text) => {
			// Limit output for large files
			if text.len() > 2000 {
				println!(
					"{}...\n\n[truncated, {} more bytes]",
					&text[..2000],
					text.len() - 2000
				);
			} else {
				println!("{}", text);
			}
		}
		Err(_) => {
			println!("[Binary content, {} bytes]", content.len());
		}
	}

	Ok(())
}

/// Information about a Git repository parsed from various URL formats
#[derive(Debug, Clone)]
struct RepoInfo {
	/// The hosting provider
	provider: Provider,
	/// Repository owner/namespace
	owner: String,
	/// Repository name
	repo: String,
	/// Full host (for self-hosted instances)
	host: String,
}

impl std::fmt::Display for RepoInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}/{}/{}", self.host, self.owner, self.repo)
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Provider {
	GitHub,
	GitLab,
	Bitbucket,
	Codeberg,
	Sourcehut,
	Unknown,
}

impl Provider {
	fn from_host(host: &str) -> Self {
		let host_lower = host.to_lowercase();
		if host_lower.contains("github.com") {
			Provider::GitHub
		} else if host_lower.contains("gitlab") {
			Provider::GitLab
		} else if host_lower.contains("bitbucket") {
			Provider::Bitbucket
		} else if host_lower.contains("codeberg.org") {
			Provider::Codeberg
		} else if host_lower.contains("sr.ht") || host_lower.contains("sourcehut") {
			Provider::Sourcehut
		} else {
			Provider::Unknown
		}
	}
}

/// Parse various repository URL formats into structured info
fn parse_repo_url(input: &str) -> Result<RepoInfo> {
	let input = input.trim();

	// Handle shorthand: "owner/repo" -> assumes GitHub
	if !input.contains(':') && !input.contains('/') == false && !input.starts_with("http") {
		if let Some((owner, repo)) = input.split_once('/') {
			if !owner.is_empty() && !repo.is_empty() && !repo.contains('/') {
				return Ok(RepoInfo {
					provider: Provider::GitHub,
					owner: owner.to_string(),
					repo: repo.trim_end_matches(".git").to_string(),
					host: "github.com".to_string(),
				});
			}
		}
	}

	// Handle SSH URLs: git@host:owner/repo.git
	if input.starts_with("git@")
		|| input.contains('@') && input.contains(':') && !input.contains("://")
	{
		return parse_ssh_url(input);
	}

	// Handle HTTPS/HTTP URLs
	if input.starts_with("http://") || input.starts_with("https://") {
		return parse_https_url(input);
	}

	// Handle git:// protocol
	if input.starts_with("git://") {
		return parse_git_protocol_url(input);
	}

	anyhow::bail!(
		"Unrecognized URL format: '{}'\n\
         Expected formats:\n\
         - HTTPS: https://github.com/owner/repo\n\
         - SSH: git@github.com:owner/repo.git\n\
         - Shorthand: owner/repo (assumes GitHub)",
		input
	)
}

fn parse_ssh_url(input: &str) -> Result<RepoInfo> {
	// Format: git@host:owner/repo.git or user@host:owner/repo.git
	let without_user = input.split('@').nth(1).context("invalid SSH URL format")?;

	let (host, path) = without_user
		.split_once(':')
		.context("invalid SSH URL format: missing ':'")?;

	let path = path.trim_start_matches('/');
	parse_path_components(host, path)
}

fn parse_https_url(input: &str) -> Result<RepoInfo> {
	let url = gix::url::parse(input.into()).context("failed to parse URL")?;

	let host = url.host().context("URL has no host")?.to_string();

	let path = url.path.to_string();
	let path = path.trim_start_matches('/');

	parse_path_components(&host, path)
}

fn parse_git_protocol_url(input: &str) -> Result<RepoInfo> {
	// Format: git://host/owner/repo.git
	let without_protocol = input.strip_prefix("git://").unwrap();
	let (host, path) = without_protocol
		.split_once('/')
		.context("invalid git:// URL format")?;

	parse_path_components(host, path)
}

fn parse_path_components(host: &str, path: &str) -> Result<RepoInfo> {
	let path = path.trim_end_matches(".git");
	let path = path.trim_matches('/');

	// Split path into components
	let parts: Vec<&str> = path.split('/').collect();

	if parts.len() < 2 {
		anyhow::bail!("URL path must contain at least owner/repo, got: '{}'", path);
	}

	// For GitLab, handle nested namespaces (owner can be "group/subgroup")
	let provider = Provider::from_host(host);
	let (owner, repo) = if provider == Provider::GitLab && parts.len() > 2 {
		// GitLab can have nested groups: gitlab.com/group/subgroup/repo
		let repo = parts.last().unwrap();
		let owner = parts[..parts.len() - 1].join("/");
		(owner, (*repo).to_string())
	} else {
		(parts[0].to_string(), parts[1].to_string())
	};

	Ok(RepoInfo {
		provider,
		owner,
		repo,
		host: host.to_string(),
	})
}

/// Construct the raw file URL for the given provider
fn build_raw_url(repo: &RepoInfo, file_path: &str, git_ref: Option<&str>) -> String {
	let ref_part = git_ref.unwrap_or("HEAD");
	let file_path = file_path.trim_start_matches('/');

	match repo.provider {
		Provider::GitHub => {
			// https://raw.githubusercontent.com/owner/repo/ref/path
			format!(
				"https://raw.githubusercontent.com/{}/{}/{}/{}",
				repo.owner, repo.repo, ref_part, file_path
			)
		}
		Provider::GitLab => {
			// https://gitlab.com/owner/repo/-/raw/ref/path
			format!(
				"https://{}/{}/-/raw/{}/{}",
				repo.host,
				format!("{}/{}", repo.owner, repo.repo),
				ref_part,
				file_path
			)
		}
		Provider::Bitbucket => {
			// https://bitbucket.org/owner/repo/raw/ref/path
			format!(
				"https://{}/{}/{}/raw/{}/{}",
				repo.host, repo.owner, repo.repo, ref_part, file_path
			)
		}
		Provider::Codeberg => {
			// https://codeberg.org/owner/repo/raw/branch/ref/path
			format!(
				"https://{}/{}/{}/raw/branch/{}/{}",
				repo.host, repo.owner, repo.repo, ref_part, file_path
			)
		}
		Provider::Sourcehut => {
			// https://git.sr.ht/~owner/repo/blob/ref/path
			let owner = if repo.owner.starts_with('~') {
				Cow::Borrowed(&repo.owner)
			} else {
				Cow::Owned(format!("~{}", repo.owner))
			};
			format!(
				"https://{}/{}/{}/blob/{}/{}",
				repo.host, owner, repo.repo, ref_part, file_path
			)
		}
		Provider::Unknown => {
			// Try GitLab-style URL as fallback (common for self-hosted)
			format!(
				"https://{}/{}/{}/-/raw/{}/{}",
				repo.host, repo.owner, repo.repo, ref_part, file_path
			)
		}
	}
}

/// Fetch a file from a remote Git repository
fn fetch_file(repo: &RepoInfo, file_path: &str, git_ref: Option<&str>) -> Result<Vec<u8>> {
	let url = build_raw_url(repo, file_path, git_ref);
	println!("Fetching: {}", url);

	// Use a simple blocking HTTP client
	let mut response = ureq::get(&url)
		.header("User-Agent", "git-sparta/0.1")
		.call()
		.map_err(|e| {
			// Check for HTTP status errors
			if let ureq::Error::StatusCode(code) = &e {
				if *code == 404 {
					return anyhow::anyhow!(
						"File not found: '{}' at ref '{}'\n\
                         URL: {}",
						file_path,
						git_ref.unwrap_or("HEAD"),
						url
					);
				} else {
					return anyhow::anyhow!("HTTP error {}\nURL: {}", code, url);
				}
			}
			anyhow::anyhow!("Request failed: {}\nURL: {}", e, url)
		})?;

	// Read body with increased limit for larger files
	let bytes = response
		.body_mut()
		.with_config()
		.limit(50 * 1024 * 1024) // 50MB limit
		.read_to_vec()
		.context("failed to read response body")?;

	Ok(bytes)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_github_https() {
		let info = parse_repo_url("https://github.com/rust-lang/rust").unwrap();
		assert_eq!(info.provider, Provider::GitHub);
		assert_eq!(info.owner, "rust-lang");
		assert_eq!(info.repo, "rust");
	}

	#[test]
	fn test_parse_github_ssh() {
		let info = parse_repo_url("git@github.com:rust-lang/rust.git").unwrap();
		assert_eq!(info.provider, Provider::GitHub);
		assert_eq!(info.owner, "rust-lang");
		assert_eq!(info.repo, "rust");
	}

	#[test]
	fn test_parse_shorthand() {
		let info = parse_repo_url("rust-lang/rust").unwrap();
		assert_eq!(info.provider, Provider::GitHub);
		assert_eq!(info.owner, "rust-lang");
		assert_eq!(info.repo, "rust");
	}

	#[test]
	fn test_parse_gitlab_nested() {
		let info = parse_repo_url("https://gitlab.com/group/subgroup/repo").unwrap();
		assert_eq!(info.provider, Provider::GitLab);
		assert_eq!(info.owner, "group/subgroup");
		assert_eq!(info.repo, "repo");
	}

	#[test]
	fn test_build_github_raw_url() {
		let repo = RepoInfo {
			provider: Provider::GitHub,
			owner: "rust-lang".to_string(),
			repo: "rust".to_string(),
			host: "github.com".to_string(),
		};
		let url = build_raw_url(&repo, "README.md", Some("main"));
		assert_eq!(
			url,
			"https://raw.githubusercontent.com/rust-lang/rust/main/README.md"
		);
	}
}
