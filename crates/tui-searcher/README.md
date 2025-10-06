# tui-searcher

A small, configurable TUI searcher (fzf-like).

Features
- Interactive TUI built on `ratatui`.
- Uses `frizbee` fuzzy matching for typo-tolerant search.
- Builder-style API to configure prompts, column headers, widths and UI text.

Quick example

```rust
use tui_searcher::{ListRow, ModeTexts, SearchData, Searcher};

let data = SearchData {
    repo_display: "example/repo".into(),
    context_value: "frontend".into(),
    primary_rows: vec![ListRow {
        label: "ui".into(),
        count: 12,
    }],
    files: vec![],
};

let outcome = Searcher::new(data)
    .with_input_title("Search repo")
    .with_primary_headers(vec!["Label", "Count", "Score"])
    .with_primary_mode_texts(ModeTexts {
        title: "Label search".into(),
        hint: "Type to search labels".into(),
        table_title: "Matching labels".into(),
        count_label: "Labels".into(),
        detail_label: None,
    })
    .run()?;
```

Run the example

```bash
cargo run -p tui_searcher --example demo
```

Notes
- This crate is meant to be a reusable component. You can integrate it into your own CLIs and customize headers/column widths using the builder API.
- The builder also lets you override UI text (branding, hints, table titles) via helpers such as `with_primary_mode_texts` or by supplying a full `UiConfig`.
- The underlying matching behavior is provided by `frizbee`.
