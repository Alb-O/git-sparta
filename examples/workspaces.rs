mod common;
use clap::Parser;
use common::{Opts, apply_theme};
use riz::{self, FacetRow, FileRow, SearchData, SearchSelection};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let facets: Vec<FacetRow> = [
        ("backend", 7),
        ("frontend", 5),
        ("integration", 3),
        ("mobile", 4),
        ("qa", 2),
        ("devops", 6),
        ("docs", 3),
        ("security", 2),
        ("infra", 4),
        ("legacy", 1),
        ("api", 5),
        ("db", 3),
        ("tests", 4),
        ("scripts", 2),
    ]
    .into_iter()
    .map(|(name, count)| FacetRow::new(name, count))
    .collect();

    let files: Vec<FileRow> = vec![
        FileRow::new("services/catalog/lib.rs", ["backend", "integration"]),
        FileRow::new("services/payments/api.rs", ["backend"]),
        FileRow::new("clients/web/src/app.tsx", ["frontend"]),
        FileRow::new(
            "clients/mobile/app/lib/main.dart",
            ["mobile", "integration"],
        ),
        FileRow::new("qa/scenarios/payment.feature", ["qa", "backend"]),
        FileRow::new("devops/ci.yml", ["devops"]),
        FileRow::new("docs/architecture.md", ["docs"]),
        FileRow::new("security/audit.log", ["security", "infra"]),
        FileRow::new("infra/terraform/main.tf", ["infra", "devops"]),
        FileRow::new("legacy/old_service.rs", ["legacy"]),
        FileRow::new("api/routes.rs", ["api", "backend"]),
        FileRow::new("db/schema.sql", ["db"]),
        FileRow::new("tests/test_api.rs", ["tests", "api"]),
        FileRow::new("scripts/deploy.sh", ["scripts", "devops"]),
        FileRow::new("clients/web/src/components/button.tsx", ["frontend"]),
        FileRow::new("clients/mobile/app/lib/utils.dart", ["mobile"]),
        FileRow::new("qa/scenarios/login.feature", ["qa", "frontend"]),
        FileRow::new("infra/ansible/playbook.yml", ["infra", "devops"]),
        FileRow::new("docs/api.md", ["docs", "api"]),
        FileRow::new("db/migrations/001_init.sql", ["db", "backend"]),
    ];

    let data = SearchData::new()
        .with_context("workspace-prototype")
        .with_initial_query("workspace")
        .with_facets(facets)
        .with_files(files);

    let searcher = riz::Searcher::new(data)
        .with_ui_config(riz::UiConfig::tags_and_files())
        .with_input_title("workspace");
    let searcher = apply_theme(searcher, &opts);

    let outcome = searcher.run()?;
    if !outcome.accepted {
        println!("Workspace walkthrough cancelled (query: {})", outcome.query);
        return Ok(());
    }

    match outcome.selection {
        Some(SearchSelection::Facet(facet)) => {
            println!("Selected workspace facet: {}", facet.name)
        }
        Some(SearchSelection::File(file)) => println!("Selected workspace file: {}", file.path),
        None => println!("Workspace walkthrough accepted"),
    }

    Ok(())
}
