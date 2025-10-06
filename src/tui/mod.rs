pub mod app;
#[cfg(test)]
mod tests;
pub mod types;
pub mod utils;

pub use app::run;
pub use types::{FileRow, SearchData, SearchMode, TagRow};
