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

/// Multi-page list, sorted pushed_at DESC. Walks pages up to `limit` (max 1000, default 30).
pub async fn list(limit: usize) -> Result<Vec<RepositorySummary>, AppError> {
    let client = api_client()?;
    let mut repos = Vec::new();
    let mut page: u32 = 1;

    while repos.len() < limit {
        let fetch_size = (limit - repos.len()).min(100);
        let batch: Vec<RepositorySummary> = client
            .get(
                format!("/user/repos?per_page={fetch_size}&page={page}&sort=pushed&direction=desc"),
                None::<&()>,
            )
            .await?;

        if batch.is_empty() {
            break;
        }

        let batch_len = batch.len();
        repos.extend(batch);

        if batch_len < fetch_size {
            break;
        }
        page += 1;
    }

    repos.truncate(limit);
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

/// Clamp user --limit (1..=1000, default 30).
pub fn clamp_limit(limit: Option<usize>) -> usize {
    match limit {
        Some(n) if n >= 1 => n.min(1000),
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
        assert_eq!(clamp_limit(Some(500)), 500);
        assert_eq!(clamp_limit(Some(2000)), 1000);
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
