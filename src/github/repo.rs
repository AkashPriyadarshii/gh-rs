//! Repo operations: typed wrappers over `client.get`/`client.post` escape-hatch-free.
//! Uses a local `RepositorySummary` instead of octocrab's model — decoupled from
//! octocrab's Repository struct churn across versions.

use crate::error::AppError;
use crate::github::client::api_client;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug, Default)]
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

pub async fn list() -> Result<Vec<RepositorySummary>, AppError> {
    let client = api_client()?;
    // First page only (v0.1) — paginate when a `--page` flag is needed.
    let repos: Vec<RepositorySummary> = client.get("/user/repos", None::<&()>).await?;
    Ok(repos)
}

pub async fn view(owner: &str, repo: &str) -> Result<RepositorySummary, AppError> {
    let client = api_client()?;
    let summary: RepositorySummary = client
        .get(format!("/repos/{owner}/{repo}"), None::<&()>)
        .await?;
    Ok(summary)
}
