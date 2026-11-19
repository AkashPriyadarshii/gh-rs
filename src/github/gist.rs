//! GitHub Gists operations (`gh-rs gist`).
//!
//! Handles creating, listing, viewing, and deleting gists.

use crate::error::AppError;
use crate::github::client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GistFile {
    pub filename: String,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    pub language: Option<String>,
    pub raw_url: Option<String>,
    pub size: Option<usize>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GistSummary {
    pub id: String,
    pub description: Option<String>,
    pub public: bool,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub files: BTreeMap<String, GistFile>,
}

#[derive(Debug, Serialize)]
struct GistFileInput<'a> {
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct CreateGistPayload<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    public: bool,
    files: BTreeMap<&'a str, GistFileInput<'a>>,
}

/// Extracts a clean Gist ID from a raw ID or URL.
pub fn extract_gist_id(input: &str) -> &str {
    let trimmed = input.trim().trim_end_matches('/');
    if let Some((_, id)) = trimmed.rsplit_once('/') {
        id
    } else {
        trimmed
    }
}

/// Lists gists owned by the authenticated user.
pub async fn list(
    limit: usize,
    public_only: bool,
    secret_only: bool,
) -> Result<Vec<GistSummary>, AppError> {
    let token = client::load_token()?;
    let http = client::http();
    let url = format!("https://api.github.com/gists?per_page={limit}");

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
            "gist list failed HTTP {status}: {text}"
        )));
    }

    let mut gists: Vec<GistSummary> = resp.json().await?;
    if public_only {
        gists.retain(|g| g.public);
    } else if secret_only {
        gists.retain(|g| !g.public);
    }

    Ok(gists)
}

/// Views a gist by ID or URL.
pub async fn view(id_or_url: &str) -> Result<GistSummary, AppError> {
    let id = extract_gist_id(id_or_url);
    let token = client::load_token()?;
    let http = client::http();
    let url = format!("https://api.github.com/gists/{id}");

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
            "gist view failed HTTP {status}: {text}"
        )));
    }

    let gist: GistSummary = resp.json().await?;
    Ok(gist)
}

/// Creates a new gist with multiple files.
pub async fn create(
    files: &[(&str, &str)],
    description: Option<&str>,
    public: bool,
) -> Result<GistSummary, AppError> {
    if files.is_empty() {
        return Err(AppError::InvalidInput(
            "at least one file is required to create a gist".into(),
        ));
    }

    let mut file_map = BTreeMap::new();
    for (filename, content) in files {
        if content.trim().is_empty() {
            return Err(AppError::InvalidInput(format!(
                "file {filename} cannot be empty (GitHub API rejects empty gist files)"
            )));
        }
        file_map.insert(*filename, GistFileInput { content });
    }

    let payload = CreateGistPayload {
        description,
        public,
        files: file_map,
    };

    let token = client::load_token()?;
    let http = client::http();
    let url = "https://api.github.com/gists";

    let resp = http
        .post(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&payload)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await?;
        return Err(AppError::GitHubApi(format!(
            "gist create failed HTTP {status}: {text}"
        )));
    }

    let gist: GistSummary = resp.json().await?;
    Ok(gist)
}

/// Deletes a gist by ID.
pub async fn delete(id_or_url: &str) -> Result<(), AppError> {
    let id = extract_gist_id(id_or_url);
    let token = client::load_token()?;
    let http = client::http();
    let url = format!("https://api.github.com/gists/{id}");

    let resp = http
        .delete(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await?;
        return Err(AppError::GitHubApi(format!(
            "gist delete failed HTTP {status}: {text}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_gist_id() {
        assert_eq!(extract_gist_id("abc123def456"), "abc123def456");
        assert_eq!(
            extract_gist_id("https://gist.github.com/octocat/abc123def456"),
            "abc123def456"
        );
        assert_eq!(
            extract_gist_id("https://gist.github.com/abc123def456/"),
            "abc123def456"
        );
    }

    #[test]
    fn test_deserialize_gist_summary() {
        let raw = serde_json::json!({
            "id": "testgist123",
            "description": "My test snippet",
            "public": true,
            "html_url": "https://gist.github.com/testgist123",
            "created_at": "2026-11-19T00:00:00Z",
            "updated_at": "2026-11-19T01:00:00Z",
            "files": {
                "main.rs": {
                    "filename": "main.rs",
                    "type": "text/plain",
                    "language": "Rust",
                    "raw_url": "https://gist.githubusercontent.com/.../raw/main.rs",
                    "size": 128,
                    "content": "fn main() { println!(\"hello\"); }"
                }
            }
        });

        let summary: GistSummary = serde_json::from_value(raw).unwrap();
        assert_eq!(summary.id, "testgist123");
        assert!(summary.public);
        assert_eq!(summary.files.len(), 1);
        assert!(summary.files.contains_key("main.rs"));
        assert_eq!(
            summary.files["main.rs"].content.as_deref(),
            Some("fn main() { println!(\"hello\"); }")
        );
    }
}
