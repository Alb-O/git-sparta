use git_sparta::tui::{self, FileRow, SearchData, TagRow};

fn main() -> anyhow::Result<()> {
    let tags = vec![
        TagRow {
            label: "app/core".into(),
            count: 12,
        },
        TagRow {
            label: "app/ui".into(),
            count: 9,
        },
        TagRow {
            label: "docs".into(),
            count: 4,
        },
        TagRow {
            label: "ops".into(),
            count: 6,
        },
        TagRow {
            label: "tooling".into(),
            count: 8,
        },
        TagRow {
            label: "infra".into(),
            count: 5,
        },
        TagRow {
            label: "ci".into(),
            count: 3,
        },
        TagRow {
            label: "tests".into(),
            count: 7,
        },
        TagRow {
            label: "examples".into(),
            count: 2,
        },
        TagRow {
            label: "legacy".into(),
            count: 1,
        },
        TagRow {
            label: "frontend".into(),
            count: 10,
        },
        TagRow {
            label: "backend".into(),
            count: 11,
        },
        TagRow {
            label: "api".into(),
            count: 6,
        },
        TagRow {
            label: "db".into(),
            count: 4,
        },
        TagRow {
            label: "scripts".into(),
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
        context_value: "demo".into(),
        primary_rows: tags,
        files,
    };

    let outcome = tui::run(data)?;
    if outcome.accepted {
        println!("Demo accepted – imagine emitting sparse patterns here");
    } else {
        println!("Demo aborted");
    }

    Ok(())
}
