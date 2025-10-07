use clap::Parser;

/// Common CLI options used by example binaries.
#[derive(Parser, Debug)]
pub struct Opts {
    /// Optional theme name (eg. "light"). Valid names are defined by the crate theme list.
    #[arg(long, value_name = "THEME")]
    pub theme: Option<String>,
}

/// Apply the chosen theme (if any) to the provided Searcher and return it.
use riz::Searcher; // Add this at the top if the type is at the crate root, or adjust the path as needed

pub fn apply_theme(mut searcher: Searcher, opts: &Opts) -> Searcher {
    if let Some(name) = opts.theme.as_deref() {
        searcher = searcher.with_theme_name(name);
    }
    searcher
}
