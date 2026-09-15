//! Pull-request operations via octocrab's typed pulls API.

use crate::error::AppError;
use crate::github::client::{api_client, http, load_token};
use octocrab::models::pulls::PullRequest;
use octocrab::params::pulls::MergeMethod;
use serde_json::json;

pub async fn list(owner: &str, repo: &str) -> Result<Vec<PullRequest>, AppError> {
    let client = api_client()?;
    // First page of open PRs (v0.1) — paginate when a flag is needed.
    let page = client
        .pulls(owner, repo)
        .list()
        .state(octocrab::params::State::Open)
        .per_page(50)
        .send()
        .await?;
    Ok(page.items)
}

pub async fn view(owner: &str, repo: &str, number: u64) -> Result<PullRequest, AppError> {
    let client = api_client()?;
    Ok(client.pulls(owner, repo).get(number).await?)
}

pub async fn create(
    owner: &str,
    repo: &str,
    title: &str,
    head: &str,
    base: &str,
    body: Option<&str>,
) -> Result<PullRequest, AppError> {
    let client = http();
    let token = load_token()?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}/pulls");
    let body = json!({
        "title": title,
        "head": head,
        "base": base,
        "body": body,
    });
    let resp: PullRequest = client
        .post(url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(resp)
}

pub async fn merge(
    owner: &str,
    repo: &str,
    number: u64,
    method: MergeMethod,
) -> Result<octocrab::models::pulls::Merge, AppError> {
    let client = api_client()?;
    Ok(client
        .pulls(owner, repo)
        .merge(number)
        .method(method)
        .send()
        .await?)
}

/// Raw unified diff text. Not in octocrab's typed API — the diff is only served
/// with a custom Accept header, so this uses the reqwest escape hatch.
pub async fn diff(owner: &str, repo: &str, number: u64) -> Result<String, AppError> {
    let client = http();
    let token = load_token()?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}/pulls/{number}");
    let text = client
        .get(url)
        .bearer_auth(token)
        .header("Accept", "application/vnd.github.diff")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(text)
}
