mod common;
use clap::Parser;
use common::{apply_theme, Opts};
use git_sparta::picker::{AttributeRow, FileRow, SearchData, SearchSelection, SearchUi, UiConfig};

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

    let files: Vec<FileRow> = vec![
        FileRow::new("src/main.rs", ["app/core", "app/ui"]),
        FileRow::new("src/ui/search.rs", ["app/ui", "tooling"]),
        FileRow::new("docs/overview.md", ["docs"]),
        FileRow::new("ops/terraform/main.tf", ["ops", "tooling"]),
        FileRow::new("tooling/dev-env.nix", ["tooling"]),
        FileRow::new("infra/docker-compose.yml", ["infra"]),
        FileRow::new("ci/build.yml", ["ci"]),
        FileRow::new("tests/test_main.rs", ["tests", "app/core"]),
        FileRow::new("examples/demo.rs", ["examples", "app/ui"]),
        FileRow::new("legacy/old_code.rs", ["legacy"]),
        FileRow::new("frontend/app.jsx", ["frontend", "app/ui"]),
        FileRow::new("backend/service.rs", ["backend", "app/core"]),
        FileRow::new("api/routes.rs", ["api", "backend"]),
        FileRow::new("db/schema.sql", ["db"]),
        FileRow::new("scripts/deploy.sh", ["scripts", "infra"]),
        FileRow::new("src/utils/helpers.rs", ["app/core", "tooling"]),
        FileRow::new("src/ui/components/button.rs", ["app/ui", "frontend"]),
        FileRow::new("ops/ansible/playbook.yml", ["ops", "infra"]),
        FileRow::new("tooling/lint.nix", ["tooling", "ci"]),
        FileRow::new("docs/api.md", ["docs", "api"]),
    ];

    let data = SearchData::new()
        .with_context("demo-repo")
        .with_initial_query("demo")
        .with_attributes(attributes)
        .with_files(files);

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
        Some(SearchSelection::File(file)) => println!("Selected file: {}", file.path),
        None => println!("Demo accepted – imagine emitting sparse patterns here"),
    }

    Ok(())
}
