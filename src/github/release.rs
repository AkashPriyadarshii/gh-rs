//! GitHub releases operations (`gh-rs release`).
//!
//! Handles release listing, viewing, creation, deletion, and asset downloading.

use crate::error::AppError;
use crate::github::client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub id: u64,
    pub name: String,
    pub size: u64,
    pub content_type: String,
    pub download_count: u64,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseSummary {
    pub id: u64,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub html_url: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Serialize)]
struct CreateReleasePayload<'a> {
    tag_name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<&'a str>,
    draft: bool,
    prerelease: bool,
}

/// Lists releases for a repository.
pub async fn list(owner: &str, repo: &str, limit: usize) -> Result<Vec<ReleaseSummary>, AppError> {
    let token = client::load_token()?;
    let http = client::http();
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page={limit}");

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
            "releases list failed HTTP {status}: {text}"
        )));
    }

    let releases: Vec<ReleaseSummary> = resp.json().await?;
    Ok(releases)
}

/// Views a specific release by tag, or the latest release.
pub async fn view(owner: &str, repo: &str, tag: Option<&str>) -> Result<ReleaseSummary, AppError> {
    let token = client::load_token()?;
    let http = client::http();
    let url = match tag {
        Some(t) => format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{t}"),
        None => format!("https://api.github.com/repos/{owner}/{repo}/releases/latest"),
    };

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
            "releases view failed HTTP {status}: {text}"
        )));
    }

    let release: ReleaseSummary = resp.json().await?;
    Ok(release)
}

/// Creates a new release in a repository.
pub async fn create(
    owner: &str,
    repo: &str,
    tag: &str,
    title: Option<&str>,
    notes: Option<&str>,
    draft: bool,
    prerelease: bool,
) -> Result<ReleaseSummary, AppError> {
    let token = client::load_token()?;
    let http = client::http();
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases");

    let payload = CreateReleasePayload {
        tag_name: tag,
        name: title,
        body: notes,
        draft,
        prerelease,
    };

    let resp = http
        .post(&url)
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
            "release create failed HTTP {status}: {text}"
        )));
    }

    let release: ReleaseSummary = resp.json().await?;
    Ok(release)
}

/// Deletes a release by tag.
pub async fn delete(owner: &str, repo: &str, tag: &str) -> Result<(), AppError> {
    let release = view(owner, repo, Some(tag)).await?;
    let token = client::load_token()?;
    let http = client::http();
    let url = format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/{}",
        release.id
    );

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
            "release delete failed HTTP {status}: {text}"
        )));
    }

    Ok(())
}

/// Downloads assets from a release matching a pattern.
pub async fn download(
    owner: &str,
    repo: &str,
    tag: Option<&str>,
    dir: &str,
    pattern: Option<&str>,
) -> Result<Vec<String>, AppError> {
    let release = view(owner, repo, tag).await?;
    let token = client::load_token()?;
    let http = client::http();
    let out_dir = std::path::Path::new(dir);
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir).map_err(|e| {
            AppError::InvalidInput(format!("failed to create output dir {dir}: {e}"))
        })?;
    }

    let mut downloaded = Vec::new();

    for asset in &release.assets {
        if let Some(pat) = pattern {
            if !asset.name.contains(pat) {
                continue;
            }
        }

        let resp = http
            .get(&asset.browser_download_url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            return Err(AppError::GitHubApi(format!(
                "failed to download asset {} HTTP {status}",
                asset.name
            )));
        }

        let bytes = resp.bytes().await?;
        let target_path = out_dir.join(&asset.name);
        std::fs::write(&target_path, bytes).map_err(|e| {
            AppError::GitSubprocess(format!(
                "failed to write asset file {}: {e}",
                target_path.display()
            ))
        })?;

        downloaded.push(asset.name.clone());
    }

    Ok(downloaded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_release_summary() {
        let raw = serde_json::json!({
            "id": 12345,
            "tag_name": "v1.0.0",
            "name": "First Release",
            "body": "Initial launch changelog",
            "draft": false,
            "prerelease": false,
            "created_at": "2026-11-01T00:00:00Z",
            "published_at": "2026-11-01T01:00:00Z",
            "html_url": "https://github.com/octocat/hello-world/releases/tag/v1.0.0",
            "assets": [
                {
                    "id": 999,
                    "name": "binary-x86_64.tar.gz",
                    "size": 1048576,
                    "content_type": "application/gzip",
                    "download_count": 42,
                    "browser_download_url": "https://github.com/octocat/hello-world/releases/download/v1.0.0/binary-x86_64.tar.gz"
                }
            ]
        });

        let summary: ReleaseSummary = serde_json::from_value(raw).unwrap();
        assert_eq!(summary.id, 12345);
        assert_eq!(summary.tag_name, "v1.0.0");
        assert_eq!(summary.assets.len(), 1);
        assert_eq!(summary.assets[0].name, "binary-x86_64.tar.gz");
        assert_eq!(summary.assets[0].download_count, 42);
    }
}
