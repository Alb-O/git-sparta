use git_sparta::tui::{self, FileRow, SearchData, TagRow};

fn main() -> anyhow::Result<()> {
    let tags = vec![
        TagRow {
            name: "backend".into(),
            count: 7,
        },
        TagRow {
            name: "frontend".into(),
            count: 5,
        },
        TagRow {
            name: "integration".into(),
            count: 3,
        },
        TagRow {
            name: "mobile".into(),
            count: 4,
        },
        TagRow {
            name: "qa".into(),
            count: 2,
        },
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
    ];

    let data = SearchData {
        repo_display: "workspace-prototype".into(),
        user_filter: "workspace".into(),
        tags,
        files,
    };

    let outcome = tui::run(data)?;
    if outcome.accepted {
        println!("Workspace walkthrough accepted");
    } else {
        println!("Workspace walkthrough cancelled");
    }

    Ok(())
}
