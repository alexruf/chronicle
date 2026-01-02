//! Terminal display module
//!
//! Handles rich terminal output with automatic TTY detection.

mod formatter;
mod terminal;

pub use formatter::print_markdown;
