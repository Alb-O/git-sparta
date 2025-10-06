use git_sparta::tui::{self, FileRow, SearchData, TagRow};

fn main() -> anyhow::Result<()> {
    let tags = vec![
        TagRow {
            label: "backend".into(),
            count: 7,
        },
        TagRow {
            label: "frontend".into(),
            count: 5,
        },
        TagRow {
            label: "integration".into(),
            count: 3,
        },
        TagRow {
            label: "mobile".into(),
            count: 4,
        },
        TagRow {
            label: "qa".into(),
            count: 2,
        },
        TagRow {
            label: "devops".into(),
            count: 6,
        },
        TagRow {
            label: "docs".into(),
            count: 3,
        },
        TagRow {
            label: "security".into(),
            count: 2,
        },
        TagRow {
            label: "infra".into(),
            count: 4,
        },
        TagRow {
            label: "legacy".into(),
            count: 1,
        },
        TagRow {
            label: "api".into(),
            count: 5,
        },
        TagRow {
            label: "db".into(),
            count: 3,
        },
        TagRow {
            label: "tests".into(),
            count: 4,
        },
        TagRow {
            label: "scripts".into(),
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

    let data = SearchData {
        repo_display: "workspace-prototype".into(),
        context_value: "workspace".into(),
        primary_rows: tags,
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
