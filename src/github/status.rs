//! GitHub work overview status query (`gh-rs status`).
//!
//! Aggregates assigned issues, open pull requests, and review requests across repositories.

use crate::error::AppError;
use crate::github::client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusItem {
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    pub assigned_issues: Vec<StatusItem>,
    pub my_prs: Vec<StatusItem>,
    pub review_requests: Vec<StatusItem>,
}

/// Parses items from a GitHub search API response.
pub fn parse_search_items(val: &Value, exclude: &[String]) -> Vec<StatusItem> {
    let mut items = Vec::new();
    let arr = match val.get("items").and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return items,
    };

    for item in arr {
        let number = item.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let html_url = item
            .get("html_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let updated_at = item
            .get("updated_at")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let repo = if let Some(repo_url) = item.get("repository_url").and_then(|v| v.as_str()) {
            repo_url
                .strip_prefix("https://api.github.com/repos/")
                .unwrap_or(repo_url)
                .to_string()
        } else {
            "unknown/repo".to_string()
        };

        if exclude.iter().any(|ex| ex.eq_ignore_ascii_case(&repo)) {
            continue;
        }

        items.push(StatusItem {
            repo,
            number,
            title,
            html_url,
            updated_at,
        });
    }

    items
}

/// Fetches the aggregated status report.
pub async fn fetch_status(org: Option<&str>, exclude: &[String]) -> Result<StatusReport, AppError> {
    let token = client::load_token()?;
    let http = client::http();

    let org_filter = org.map(|o| format!("+org:{o}")).unwrap_or_default();

    let q_issues = format!("is:issue+is:open+assignee:@me{org_filter}");
    let q_prs = format!("is:pr+is:open+author:@me{org_filter}");
    let q_reviews = format!("is:pr+is:open+review-requested:@me{org_filter}");

    let fetch_query = |q: String| {
        let http = http.clone();
        let token = token.clone();
        async move {
            let url = format!("https://api.github.com/search/issues?q={q}&per_page=10");
            let resp = http
                .get(&url)
                .header("Authorization", format!("Bearer {token}"))
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await?;
            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await?;
                return Err(AppError::GitHubApi(format!(
                    "search failed HTTP {status}: {text}"
                )));
            }
            let json: Value = resp.json().await?;
            Ok(json)
        }
    };

    let (issues_val, prs_val, reviews_val) = tokio::try_join!(
        fetch_query(q_issues),
        fetch_query(q_prs),
        fetch_query(q_reviews),
    )?;

    Ok(StatusReport {
        assigned_issues: parse_search_items(&issues_val, exclude),
        my_prs: parse_search_items(&prs_val, exclude),
        review_requests: parse_search_items(&reviews_val, exclude),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_items_and_exclude() {
        let sample = serde_json::json!({
            "items": [
                {
                    "number": 42,
                    "title": "Bug in parser",
                    "html_url": "https://github.com/org/repo-a/issues/42",
                    "repository_url": "https://api.github.com/repos/org/repo-a",
                    "updated_at": "2026-11-19T10:00:00Z"
                },
                {
                    "number": 99,
                    "title": "Excluded item",
                    "html_url": "https://github.com/org/excluded-repo/issues/99",
                    "repository_url": "https://api.github.com/repos/org/excluded-repo",
                    "updated_at": "2026-11-19T09:00:00Z"
                }
            ]
        });

        let exclude = vec!["org/excluded-repo".to_string()];
        let parsed = parse_search_items(&sample, &exclude);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].number, 42);
        assert_eq!(parsed[0].repo, "org/repo-a");
        assert_eq!(parsed[0].title, "Bug in parser");
    }
}
