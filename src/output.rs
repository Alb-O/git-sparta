use std::io::{self, Write};

use anyhow::Result;
use owo_colors::OwoColorize;

pub fn divider() {
	eprintln!("{}", "─".repeat(56).blue());
}

pub fn heading(text: &str) {
	eprintln!("{}", text.bold().cyan());
}

pub fn note(text: &str) {
	eprintln!("{}", text.dimmed());
}

pub fn label_value(label: &str, value: impl std::fmt::Display) {
	eprintln!("{} {}", format!("{}:", label).bold(), value);
}

pub fn bullet_list(lines: impl IntoIterator<Item = String>) {
	for line in lines.into_iter().filter(|line| !line.is_empty()) {
		eprintln!("  {} {}", "•".green(), line);
	}
}

pub fn confirm(prompt: &str, default_yes: bool, auto_yes: bool) -> Result<bool> {
	if auto_yes {
		return Ok(true);
	}

	let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
	eprint!("{} {} ", prompt.bold(), hint.dimmed());
	io::stderr().flush()?;

	let mut line = String::new();
	io::stdin().read_line(&mut line)?;
	let reply = line.trim();
	if reply.is_empty() {
		return Ok(default_yes);
	}
	match reply.to_ascii_lowercase().as_str() {
		"y" | "yes" => Ok(true),
		"n" | "no" => Ok(false),
		_ => Ok(default_yes),
	}
}

pub fn success(message: &str) {
	eprintln!("{}", message.green().bold());
}

pub fn warn(message: &str) {
	eprintln!("{}", message.yellow().bold());
}
