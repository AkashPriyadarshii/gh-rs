//! Issue operations via raw REST (reqwest escape hatch).
//!
//! octocrab 0.54's `models::issues::Issue` stack-overflows on deserialize in this
//! toolchain (nested Author/Milestone models). We use a local slim `IssueSummary`
//! like S1's `RepositorySummary` — decoupled and immune to upstream model churn.
//! Note: the slim struct deserializer expects `user.login` flattened as
//! `user_login` via the github API — instead we read `user: {login}` manually.

use crate::error::AppError;
use crate::github::client::{api_client, http, load_token};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct IssueSummary {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub comments: u64,
    #[serde(rename = "user")]
    pub user: IssueUser,
    /// Present on PRs (the /issues endpoint returns both). Filtered in list().
    pub pull_request: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct IssueUser {
    pub login: String,
}

pub async fn list(owner: &str, repo: &str) -> Result<Vec<IssueSummary>, AppError> {
    let client = api_client()?;
    let issues: Vec<IssueSummary> = client
        .get(
            format!("/repos/{owner}/{repo}/issues?state=open&per_page=50"),
            None::<&()>,
        )
        .await?;
    // /issues mixes PRs into the response — drop anything carrying pull_request.
    Ok(issues
        .into_iter()
        .filter(|i| i.pull_request.is_none())
        .collect())
}

pub async fn view(owner: &str, repo: &str, number: u64) -> Result<IssueSummary, AppError> {
    let client = api_client()?;
    let issue: IssueSummary = client
        .get(
            format!("/repos/{owner}/{repo}/issues/{number}"),
            None::<&()>,
        )
        .await?;
    Ok(issue)
}

pub async fn create(
    owner: &str,
    repo: &str,
    title: &str,
    body: Option<&str>,
) -> Result<IssueSummary, AppError> {
    let client = http();
    let token = load_token()?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}/issues");
    let resp = client
        .post(url)
        .bearer_auth(token)
        .json(&json!({ "title": title, "body": body }))
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let detail = resp.text().await.unwrap_or_default();
        return Err(AppError::GitHubApi(format!("{status}: {detail}")));
    }
    Ok(resp.json().await?)
}

pub async fn close(owner: &str, repo: &str, number: u64) -> Result<IssueSummary, AppError> {
    let client = http();
    let token = load_token()?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}/issues/{number}");
    let resp = client
        .patch(url)
        .bearer_auth(token)
        .json(&json!({ "state": "closed" }))
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let detail = resp.text().await.unwrap_or_default();
        return Err(AppError::GitHubApi(format!("{status}: {detail}")));
    }
    Ok(resp.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_issue_summary() {
        let json = r#"{"number":3,"title":"Bug","body":"fix me","state":"open","comments":2,"user":{"login":"akash"}}"#;
        let parsed: IssueSummary = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.number, 3);
        assert_eq!(parsed.user.login, "akash");
        assert_eq!(parsed.state, "open");
        assert!(parsed.pull_request.is_none());
    }

    #[test]
    fn pr_payload_detected() {
        let json = r#"{"number":4,"title":"PR","state":"open","comments":0,"user":{"login":"akash"},"pull_request":{"url":"https://api.github.com/x"}}"#;
        let parsed: IssueSummary = serde_json::from_str(json).unwrap();
        assert!(parsed.pull_request.is_some());
    }
}
