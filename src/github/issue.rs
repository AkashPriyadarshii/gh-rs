//! Issue operations via octocrab's typed issues API.

use crate::error::AppError;
use crate::github::client::api_client;
use octocrab::models::issues::Issue;

pub async fn list(owner: &str, repo: &str) -> Result<Vec<Issue>, AppError> {
    let client = api_client()?;
    let page = client
        .issues(owner, repo)
        .list()
        .state(octocrab::params::State::Open)
        .per_page(50)
        .send()
        .await?;
    Ok(page.items)
}

pub async fn view(owner: &str, repo: &str, number: u64) -> Result<Issue, AppError> {
    let client = api_client()?;
    // Get Issue
    let issue = client.issues(owner, repo).get(number).await?;
    Ok(issue)
}

pub async fn create(
    owner: &str,
    repo: &str,
    title: &str,
    body: Option<&str>,
) -> Result<Issue, AppError> {
    let client = api_client()?;
    // Builder lifetimes in octocrab 0.54: chain in one expression or None breaks.
    let issue = match body {
        Some(b) => {
            client
                .issues(owner, repo)
                .create(title)
                .body(b)
                .send()
                .await?
        }
        None => client.issues(owner, repo).create(title).send().await?,
    };
    Ok(issue)
}

pub async fn close(owner: &str, repo: &str, number: u64) -> Result<Issue, AppError> {
    let client = api_client()?;
    let issue = client
        .issues(owner, repo)
        .update(number)
        .state(octocrab::models::IssueState::Closed)
        .send()
        .await?;
    Ok(issue)
}
