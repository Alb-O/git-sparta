pub mod app;
#[cfg(test)]
mod tests;
pub mod types;

pub use app::run;
pub use types::{FileRow, SearchData, SearchMode, TagRow};
