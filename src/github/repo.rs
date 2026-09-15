//! Repo operations: typed wrappers over `client.get`/`client.post` escape-hatch-free.
//! Uses a local `RepositorySummary` instead of octocrab's model — decoupled from
//! octocrab's Repository struct churn across versions.

use crate::error::AppError;
use crate::github::client::api_client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct RepositorySummary {
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub html_url: String,
    pub clone_url: String,
    pub default_branch: String,
    pub language: Option<String>,
    pub stargazers_count: u64,
    pub pushed_at: Option<String>,
}

pub async fn create(name: &str, public: bool) -> Result<RepositorySummary, AppError> {
    let client = api_client()?;
    let repo: RepositorySummary = client
        .post::<_, RepositorySummary>(
            "/user/repos",
            Some(&json!({ "name": name, "private": !public })),
        )
        .await?;
    Ok(repo)
}

/// Single-page list, sorted pushed_at DESC. `per_page` = clamped --limit (max 100).
/// ponytail: no Link-header walker — add cursor pagination when repos exceed 100.
pub async fn list(per_page: u8) -> Result<Vec<RepositorySummary>, AppError> {
    let client = api_client()?;
    let mut repos: Vec<RepositorySummary> = client
        .get(
            format!("/user/repos?per_page={per_page}&sort=pushed&direction=desc"),
            None::<&()>,
        )
        .await?;
    // API honors sort=pushed, but belt-and-braces local sort keeps output stable
    // across API quirks (pushed_at None sorts last).
    repos.sort_by(|a, b| b.pushed_at.cmp(&a.pushed_at));
    Ok(repos)
}

pub async fn view(owner: &str, repo: &str) -> Result<RepositorySummary, AppError> {
    let client = api_client()?;
    let summary: RepositorySummary = client
        .get(format!("/repos/{owner}/{repo}"), None::<&()>)
        .await?;
    Ok(summary)
}

/// Clamp user --limit to GitHub's per_page ceiling (1..=100, default 30).
pub fn clamp_limit(limit: Option<u32>) -> u8 {
    match limit {
        Some(n) if n >= 1 => n.min(100) as u8,
        _ => 30,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_limit() {
        assert_eq!(clamp_limit(None), 30);
        assert_eq!(clamp_limit(Some(0)), 30);
        assert_eq!(clamp_limit(Some(5)), 5);
        assert_eq!(clamp_limit(Some(100)), 100);
        assert_eq!(clamp_limit(Some(500)), 100);
    }

    #[test]
    fn serializes_summary() {
        let r = RepositorySummary {
            full_name: "a/b".into(),
            ..Default::default()
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"full_name\":\"a/b\""));
    }
}
