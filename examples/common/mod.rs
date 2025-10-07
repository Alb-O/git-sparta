use clap::Parser;

/// Common CLI options used by example binaries.
#[derive(Parser, Debug)]
pub struct Opts {
    /// Optional theme name (eg. "light"). Valid names are defined by the crate theme list.
    #[arg(long, value_name = "THEME")]
    pub theme: Option<String>,
}

/// Apply the chosen theme (if any) to the provided Searcher and return it.
pub fn apply_theme(
    mut searcher: git_sparta::tui::Searcher,
    opts: &Opts,
) -> git_sparta::tui::Searcher {
    if let Some(name) = opts.theme.as_deref() {
        searcher = searcher.with_theme_name(name);
    }
    searcher
}
