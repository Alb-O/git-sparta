use clap::Parser;

/// Common CLI options used by example binaries.
#[derive(Parser, Debug)]
pub struct Opts {
    /// Optional theme name (eg. "light"). Valid names are defined by the crate theme list.
    #[arg(long, value_name = "THEME")]
    pub theme: Option<String>,
}

/// Apply the chosen theme (if any) to the provided Searcher and return it.
///
/// Theme selection is currently a no-op because the `nucleo-picker` integration
/// does not expose theme configuration.
use git_sparta::picker::SearchUi;

pub fn apply_theme(mut searcher: SearchUi, opts: &Opts) -> SearchUi {
    if let Some(_name) = opts.theme.as_deref() {
        // Theme selection is not currently supported by the nucleo picker integration.
    }
    searcher
}
