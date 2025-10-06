# tui-searcher

A small, configurable TUI searcher (fzf-like).

Features
- Interactive TUI built on `ratatui`.
- Uses `frizbee` fuzzy matching for typo-tolerant search.
- Builder-style API to configure prompts, column headers and widths.

Quick example

```rust
use tui_searcher::Searcher;

let outcome = Searcher::new(data)
    .with_input_title("Search repo")
    .with_tag_headers(vec!["Tag", "Count", "Score"])
    .run()?;
```

Run the example

```bash
cargo run -p tui_searcher --example demo
```

Notes
- This crate is meant to be a reusable component. You can integrate it into your own CLIs and customize headers/column widths using the builder API.
- The underlying matching behavior is provided by `frizbee`.
