mod common;
use clap::Parser;
use common::{Opts, apply_theme};
use git_sparta::picker::{AttributeRow, SearchData, SearchSelection, SearchUi, UiConfig};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let attributes: Vec<AttributeRow> = [
        ("app/core", 12),
        ("app/ui", 9),
        ("docs", 4),
        ("ops", 6),
        ("tooling", 8),
        ("infra", 5),
        ("ci", 3),
        ("tests", 7),
        ("examples", 2),
        ("legacy", 1),
        ("frontend", 10),
        ("backend", 11),
        ("api", 6),
        ("db", 4),
        ("scripts", 3),
    ]
    .into_iter()
    .map(|(name, count)| AttributeRow::new(name, count))
    .collect();

    let data = SearchData::new()
        .with_context("demo-repo")
        .with_attributes(attributes);

    let searcher = SearchUi::new(data)
        .with_ui_config(UiConfig::tags_and_files())
        .with_input_title("demo");
    let searcher = apply_theme(searcher, &opts);

    let outcome = searcher.run()?;
    if !outcome.accepted {
        println!("Demo aborted (query: {})", outcome.query);
        return Ok(());
    }

    match outcome.selection {
        Some(SearchSelection::Attribute(attribute)) => {
            println!("Selected attribute: {}", attribute.name)
        }
        Some(SearchSelection::File(_)) => unreachable!("demo only returns attributes"),
        _ => println!("Demo accepted – imagine emitting sparse patterns here"),
    }

    Ok(())
}
