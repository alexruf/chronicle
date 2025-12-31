//! Data collectors module
//!
//! Implements collectors for different data sources:
//! - GitCollector: Extract commits and branches from Git repositories
//! - TodoCollector: Parse TODO/Inbox markdown files
//! - NotesCollector: Scan note directories for modified files

pub mod git;
pub mod notes;
pub mod todo;

pub use git::GitCollector;
pub use notes::NotesCollector;
pub use todo::TodoCollector;
