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
