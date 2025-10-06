mod commands;
mod config;
mod git;
mod output;

use clap::{Parser, Subcommand};
use commands::{generate, setup, teardown};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "pipeline",
    about = "Developer tooling for git-driven pipelines",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate sparse-checkout patterns for a given project tag.
    GenerateSparseList {
        /// Project tag filter (substring match).
        tag: String,
        /// Automatically confirm interactive prompts.
        #[arg(long, short = 'y')]
        yes: bool,
        /// Repository directory (defaults to current working directory).
        #[arg(long)]
        repo: Option<PathBuf>,
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
        Command::GenerateSparseList { tag, yes, repo } => generate::run(&tag, yes, repo.as_deref()),
        Command::SetupSubmodule { config_dir, yes } => setup::run(config_dir.as_deref(), yes),
        Command::TeardownSubmodule { config_dir, yes } => teardown::run(config_dir.as_deref(), yes),
    }
}
