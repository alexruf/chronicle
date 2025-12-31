//! Data models module
//!
//! Defines domain models for Git, TODO, Notes, and Chronicle.
//! Includes ChangeKind, Commit, Branch, Repository, Todo, Note, Chronicle.

pub mod chronicle;
pub mod source;

pub use chronicle::Chronicle;
pub use source::{Branch, ChangeKind, Commit, Note, Repository, Todo, TodoStatus};
