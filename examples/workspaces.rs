mod common;
use clap::Parser;
use common::{Opts, apply_theme};
use git_sparta::tui::{self, FacetRow, FileRow, SearchData};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let facets = vec![
        FacetRow::new("backend", 7),
        FacetRow::new("frontend", 5),
        FacetRow::new("integration", 3),
        FacetRow::new("mobile", 4),
        FacetRow::new("qa", 2),
        FacetRow::new("devops", 6),
        FacetRow::new("docs", 3),
        FacetRow::new("security", 2),
        FacetRow::new("infra", 4),
        FacetRow::new("legacy", 1),
        FacetRow::new("api", 5),
        FacetRow::new("db", 3),
        FacetRow::new("tests", 4),
        FacetRow::new("scripts", 2),
    ];

    let files = vec![
        FileRow::new(
            "services/catalog/lib.rs".into(),
            vec!["backend".into(), "integration".into()],
        ),
        FileRow::new("services/payments/api.rs".into(), vec!["backend".into()]),
        FileRow::new("clients/web/src/app.tsx".into(), vec!["frontend".into()]),
        FileRow::new(
            "clients/mobile/app/lib/main.dart".into(),
            vec!["mobile".into(), "integration".into()],
        ),
        FileRow::new(
            "qa/scenarios/payment.feature".into(),
            vec!["qa".into(), "backend".into()],
        ),
        FileRow::new("devops/ci.yml".into(), vec!["devops".into()]),
        FileRow::new("docs/architecture.md".into(), vec!["docs".into()]),
        FileRow::new(
            "security/audit.log".into(),
            vec!["security".into(), "infra".into()],
        ),
        FileRow::new(
            "infra/terraform/main.tf".into(),
            vec!["infra".into(), "devops".into()],
        ),
        FileRow::new("legacy/old_service.rs".into(), vec!["legacy".into()]),
        FileRow::new("api/routes.rs".into(), vec!["api".into(), "backend".into()]),
        FileRow::new("db/schema.sql".into(), vec!["db".into()]),
        FileRow::new(
            "tests/test_api.rs".into(),
            vec!["tests".into(), "api".into()],
        ),
        FileRow::new(
            "scripts/deploy.sh".into(),
            vec!["scripts".into(), "devops".into()],
        ),
        FileRow::new(
            "clients/web/src/components/button.tsx".into(),
            vec!["frontend".into()],
        ),
        FileRow::new(
            "clients/mobile/app/lib/utils.dart".into(),
            vec!["mobile".into()],
        ),
        FileRow::new(
            "qa/scenarios/login.feature".into(),
            vec!["qa".into(), "frontend".into()],
        ),
        FileRow::new(
            "infra/ansible/playbook.yml".into(),
            vec!["infra".into(), "devops".into()],
        ),
        FileRow::new("docs/api.md".into(), vec!["docs".into(), "api".into()]),
        FileRow::new(
            "db/migrations/001_init.sql".into(),
            vec!["db".into(), "backend".into()],
        ),
    ];

    let data = SearchData::new().with_facets(facets).with_files(files);

    let searcher = tui::Searcher::new(data).with_input_title("workspace");
    let searcher = apply_theme(searcher, &opts);

    let outcome = searcher.run()?;
    if outcome.is_accepted() {
        println!("Workspace walkthrough accepted");
    } else {
        println!("Workspace walkthrough cancelled");
    }

    Ok(())
}
