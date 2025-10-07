use clap::{Parser, Subcommand};
use git_sparta::commands::{generate, setup, teardown};
use std::path::PathBuf;

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
    /// Optional theme for TUI components.
    #[arg(long, value_name = "THEME", value_parser = parse_theme)]
    theme: Option<String>,

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
    let theme = cli.theme.clone();
    match cli.command {
        Command::GenerateSparseList { tag, yes, repo } => {
            generate::run(&tag, yes, repo.as_deref(), theme)
        }
        Command::SetupSubmodule { config_dir, yes } => {
            setup::run(config_dir.as_deref(), yes, theme)
        }
        Command::TeardownSubmodule { config_dir, yes } => {
            teardown::run(config_dir.as_deref(), yes, theme)
        }
    }
}

// clap value parser that validates theme names against the single source of truth
// in the tui_searcher theme module. Returns the provided string on success or an
// error message listing valid choices.
fn parse_theme(s: &str) -> Result<String, String> {
    if git_sparta::tui::theme::by_name(s).is_some() {
        Ok(s.to_string())
    } else {
        let mut valid = String::new();
        for &n in git_sparta::tui::theme::NAMES {
            if !valid.is_empty() {
                valid.push_str(", ");
            }
            valid.push_str(n);
        }
        Err(format!("unknown theme '{}'; valid values: {}", s, valid))
    }
}
