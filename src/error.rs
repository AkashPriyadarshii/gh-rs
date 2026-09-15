//! Unified error type. Every fallible path returns `Result<T, AppError>`.
//! Conversion impls (`From`) are added per-module in Sprints 1-3, only where used.

use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)] // wired into cli dispatch from Sprint 1
pub enum AppError {
    #[error("not logged in — run `gh-rs auth login` first")]
    AuthMissing,

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("git subprocess failed: {0}")]
    GitSubprocess(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}
