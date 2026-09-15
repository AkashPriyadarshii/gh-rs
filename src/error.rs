//! Unified error type. Every fallible path returns `Result<T, AppError>`.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not logged in — run `gh-rs auth login` first")]
    AuthMissing,

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("git subprocess failed: {0}")]
    GitSubprocess(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("credential store error: {0}")]
    Keyring(String),
}

impl From<octocrab::Error> for AppError {
    fn from(e: octocrab::Error) -> Self {
        AppError::GitHubApi(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::GitHubApi(e.to_string())
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        AppError::Keyring(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::GitSubprocess(e.to_string())
    }
}
