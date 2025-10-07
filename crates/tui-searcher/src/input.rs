//! Single-line input widget using tui-textarea
//!
//! This module provides a wrapper around `TextArea` configured for single-line input,
//! similar to `<input>` in HTML.

use crate::theme::Theme;
use ratatui::{Frame, layout::Rect};
use tui_textarea::{Input, Key, TextArea};

/// A single-line text input widget
pub struct SearchInput<'a> {
    textarea: TextArea<'a>,
}

impl<'a> SearchInput<'a> {
    /// Create a new search input with optional initial text
    pub fn new(initial_text: impl Into<String>) -> Self {
        let text = initial_text.into().replace(['\n', '\r'], " ");
        let mut textarea = TextArea::new(vec![text]);
        textarea.remove_line_number();
        Self { textarea }
    }

    /// Handle input events, ignoring Enter and Ctrl+M to keep it single-line
    pub fn input(&mut self, input: impl Into<Input>) -> bool {
        let input = input.into();

        // Ignore keys that would insert newlines
        match input {
            Input {
                key: Key::Char('m'),
                ctrl: true,
                alt: false,
                ..
            }
            | Input {
                key: Key::Enter, ..
            } => false,
            _ => {
                self.textarea.input(input);
                true
            }
        }
    }

    /// Get the current input text
    pub fn text(&self) -> &str {
        self.textarea.lines()[0].as_str()
    }

    /// Set the input text
    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into().replace(['\n', '\r'], " ");
        self.textarea = TextArea::new(vec![text]);
        self.textarea.remove_line_number();
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.set_text("");
    }

    /// Render the textarea widget directly (shows cursor and proper text editing)
    pub fn render_textarea(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(&self.textarea, area);
    }

    /// Render with a prompt prefix (for display purposes)
    pub fn render_with_prompt(&self, frame: &mut Frame, area: Rect, prompt: &str) {
        use ratatui::widgets::{Paragraph, Widget};
        let display = if prompt.is_empty() {
            self.text().to_string()
        } else {
            format!("{} > {}", prompt, self.text())
        };
        let para = Paragraph::new(display).style(Theme::default().prompt_style());
        para.render(area, frame.buffer_mut());
    }

    /// Get a reference to the underlying TextArea for advanced usage
    pub fn textarea(&self) -> &TextArea<'a> {
        &self.textarea
    }

    /// Get a mutable reference to the underlying TextArea for advanced usage
    pub fn textarea_mut(&mut self) -> &mut TextArea<'a> {
        &mut self.textarea
    }
}

impl<'a> Default for SearchInput<'a> {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_input() {
        let input = SearchInput::new("test");
        assert_eq!(input.text(), "test");
    }

    #[test]
    fn test_newlines_replaced() {
        let input = SearchInput::new("test\nwith\rnewlines");
        assert_eq!(input.text(), "test with newlines");
    }

    #[test]
    fn test_clear() {
        let mut input = SearchInput::new("test");
        input.clear();
        assert_eq!(input.text(), "");
    }

    #[test]
    fn test_set_text() {
        let mut input = SearchInput::new("initial");
        input.set_text("updated");
        assert_eq!(input.text(), "updated");
    }
}
