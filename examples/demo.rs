use git_sparta::tui::{self, FileRow, SearchData, TagRow};

fn main() -> anyhow::Result<()> {
    let tags = vec![
        TagRow {
            name: "app/core".into(),
            count: 12,
        },
        TagRow {
            name: "app/ui".into(),
            count: 9,
        },
        TagRow {
            name: "docs".into(),
            count: 4,
        },
        TagRow {
            name: "ops".into(),
            count: 6,
        },
        TagRow {
            name: "tooling".into(),
            count: 8,
        },
        TagRow {
            name: "infra".into(),
            count: 5,
        },
        TagRow {
            name: "ci".into(),
            count: 3,
        },
        TagRow {
            name: "tests".into(),
            count: 7,
        },
        TagRow {
            name: "examples".into(),
            count: 2,
        },
        TagRow {
            name: "legacy".into(),
            count: 1,
        },
        TagRow {
            name: "frontend".into(),
            count: 10,
        },
        TagRow {
            name: "backend".into(),
            count: 11,
        },
        TagRow {
            name: "api".into(),
            count: 6,
        },
        TagRow {
            name: "db".into(),
            count: 4,
        },
        TagRow {
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
        tags,
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
