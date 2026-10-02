//! Pull-request operations via octocrab's typed pulls API.

use crate::error::AppError;
use crate::github::client::{api_client, http, load_token};
use octocrab::models::pulls::PullRequest;
use octocrab::params::pulls::MergeMethod;

/// Multi-page open-PR list. Walks pages up to `limit` (max 1000, default 30).
pub async fn list(owner: &str, repo: &str, limit: usize) -> Result<Vec<PullRequest>, AppError> {
    let client = api_client()?;
    let per_page = (limit.min(100)) as u8;
    let mut page = client
        .pulls(owner, repo)
        .list()
        .state(octocrab::params::State::Open)
        .per_page(per_page)
        .send()
        .await?;

    let mut prs = Vec::new();
    loop {
        let is_empty = page.items.is_empty();
        prs.extend(page.items);
        if prs.len() >= limit || is_empty {
            break;
        }
        page = match client.get_page::<PullRequest>(&page.next).await? {
            Some(next) => next,
            None => break,
        };
    }

    prs.truncate(limit);
    Ok(prs)
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
    let client = api_client()?;
    let handler = client.pulls(owner, repo);
    let mut builder = handler.create(title, head, base);
    if let Some(b) = body {
        builder = builder.body(b);
    }
    Ok(builder.send().await?)
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
