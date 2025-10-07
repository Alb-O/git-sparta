mod common;
use clap::Parser;
use common::{Opts, apply_theme};
use git_sparta::tui::{self, FacetRow, FileRow, SearchData};

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let facets = vec![
        FacetRow {
            name: "app/core".into(),
            count: 12,
        },
        FacetRow {
            name: "app/ui".into(),
            count: 9,
        },
        FacetRow {
            name: "docs".into(),
            count: 4,
        },
        FacetRow {
            name: "ops".into(),
            count: 6,
        },
        FacetRow {
            name: "tooling".into(),
            count: 8,
        },
        FacetRow {
            name: "infra".into(),
            count: 5,
        },
        FacetRow {
            name: "ci".into(),
            count: 3,
        },
        FacetRow {
            name: "tests".into(),
            count: 7,
        },
        FacetRow {
            name: "examples".into(),
            count: 2,
        },
        FacetRow {
            name: "legacy".into(),
            count: 1,
        },
        FacetRow {
            name: "frontend".into(),
            count: 10,
        },
        FacetRow {
            name: "backend".into(),
            count: 11,
        },
        FacetRow {
            name: "api".into(),
            count: 6,
        },
        FacetRow {
            name: "db".into(),
            count: 4,
        },
        FacetRow {
            name: "scripts".into(),
            count: 3,
        },
    ];

    let files = vec![
        FileRow::new(
            "src/main.rs".into(),
            vec!["app/core".into(), "app/ui".into()],
        ),
        FileRow::new(
            "src/ui/search.rs".into(),
            vec!["app/ui".into(), "tooling".into()],
        ),
        FileRow::new("docs/overview.md".into(), vec!["docs".into()]),
        FileRow::new(
            "ops/terraform/main.tf".into(),
            vec!["ops".into(), "tooling".into()],
        ),
        FileRow::new("tooling/dev-env.nix".into(), vec!["tooling".into()]),
        FileRow::new("infra/docker-compose.yml".into(), vec!["infra".into()]),
        FileRow::new("ci/build.yml".into(), vec!["ci".into()]),
        FileRow::new(
            "tests/test_main.rs".into(),
            vec!["tests".into(), "app/core".into()],
        ),
        FileRow::new(
            "examples/demo.rs".into(),
            vec!["examples".into(), "app/ui".into()],
        ),
        FileRow::new("legacy/old_code.rs".into(), vec!["legacy".into()]),
        FileRow::new(
            "frontend/app.jsx".into(),
            vec!["frontend".into(), "app/ui".into()],
        ),
        FileRow::new(
            "backend/service.rs".into(),
            vec!["backend".into(), "app/core".into()],
        ),
        FileRow::new("api/routes.rs".into(), vec!["api".into(), "backend".into()]),
        FileRow::new("db/schema.sql".into(), vec!["db".into()]),
        FileRow::new(
            "scripts/deploy.sh".into(),
            vec!["scripts".into(), "infra".into()],
        ),
        FileRow::new(
            "src/utils/helpers.rs".into(),
            vec!["app/core".into(), "tooling".into()],
        ),
        FileRow::new(
            "src/ui/components/button.rs".into(),
            vec!["app/ui".into(), "frontend".into()],
        ),
        FileRow::new(
            "ops/ansible/playbook.yml".into(),
            vec!["ops".into(), "infra".into()],
        ),
        FileRow::new(
            "tooling/lint.nix".into(),
            vec!["tooling".into(), "ci".into()],
        ),
        FileRow::new("docs/api.md".into(), vec!["docs".into(), "api".into()]),
    ];

    let data = SearchData {
        repo_display: "demo-repo".into(),
        user_filter: "demo".into(),
        facets,
        files,
    };

    let searcher = tui::Searcher::new(data).with_input_title("demo");
    let searcher = apply_theme(searcher, &opts);

    let outcome = searcher.run()?;
    if outcome.accepted {
        println!("Demo accepted – imagine emitting sparse patterns here");
    } else {
        println!("Demo aborted");
    }

    Ok(())
}
