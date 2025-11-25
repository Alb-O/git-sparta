use std::path::PathBuf;

use clap::{Parser, Subcommand};
use git_sparta::commands::{generate, setup, teardown};

#[derive(Parser, Debug)]
#[command(
	name = "git-sparta",
	about = "Git Sparse Tagging - Manage sparse submodule checkouts based on git attributes",
	long_about = "git-sparta enables efficient sparse checkouts of git submodules by using \
                  git attributes to tag files with project identifiers. Only files tagged \
                  with matching tags are checked out, reducing disk usage and clone times.",
	version
)]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
	/// Generate sparse-checkout patterns for a given project tag.
	///
	/// When run without a tag, an interactive picker displays all available
	/// tags/attributes found in the repository for you to choose from.
	GenerateSparseList {
		/// Project tag filter (substring match). If omitted, shows an interactive picker.
		tag: Option<String>,
		/// Automatically confirm interactive prompts.
		#[arg(long, short = 'y')]
		yes: bool,
		/// Repository directory (defaults to current working directory).
		#[arg(long)]
		repo: Option<PathBuf>,
		/// Git attribute name to search for tags.
		#[arg(long, short = 'a', default_value = "projects")]
		attribute: String,
	},
	/// Configure a sparse submodule clone according to JSON metadata.
	SetupSubmodule {
		/// Directory that contains the JSON configuration and .gitmodules file (defaults to current dir).
		#[arg(long)]
		config_dir: Option<PathBuf>,
		/// Automatically confirm interactive prompts.
		#[arg(long, short = 'y')]
		yes: bool,
	},
	/// Remove a previously configured sparse submodule clone.
	TeardownSubmodule {
		/// Directory that contains the JSON configuration and .gitmodules file (defaults to current dir).
		#[arg(long)]
		config_dir: Option<PathBuf>,
		/// Automatically confirm interactive prompts.
		#[arg(long, short = 'y')]
		yes: bool,
	},
}

fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();
	match cli.command {
		Command::GenerateSparseList {
			tag,
			yes,
			repo,
			attribute,
		} => generate::run(tag.as_deref(), yes, repo.as_deref(), &attribute),
		Command::SetupSubmodule { config_dir, yes } => setup::run(config_dir.as_deref(), yes),
		Command::TeardownSubmodule { config_dir, yes } => teardown::run(config_dir.as_deref(), yes),
	}
}
