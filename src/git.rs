//! git subprocess wrapper. Shelling out to the system `git` binary beats any
//! in-process libgit2 for clone/checkout fidelity (auth, LFS, submodules).

use crate::error::AppError;
use std::process::Command;

/// `git clone https://github.com/{owner}/{repo}.git` into cwd (or dir if given).
pub fn clone(owner: &str, repo: &str, dir: Option<&str>) -> Result<(), AppError> {
    let url = format!("https://github.com/{owner}/{repo}.git");
    let mut cmd = Command::new("git");
    // Bearer header via -c: private repos clone with zero git-credential setup.
    // ponytail: token visible in local `ps` argv during clone; switch to
    // credential-helper plumbing when v0.2 needs paranoid multi-user hosts.
    if let Ok(token) = crate::github::client::load_token() {
        cmd.args([
            "-c",
            &format!("http.https://github.com/.extraHeader=Authorization: Bearer {token}"),
        ]);
    }
    cmd.args(["clone", &url]);
    if let Some(d) = dir {
        cmd.arg(d);
    }
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::GitSubprocess(format!(
            "git clone failed for {owner}/{repo}"
        )))
    }
}

/// Capture-mode runner for read-only git queries.
fn capture(args: &[&str]) -> Result<String, AppError> {
    let out = Command::new("git").args(args).output()?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(AppError::GitSubprocess(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ))
    }
}

/// Current branch name (empty if detached).
pub fn current_branch() -> Result<String, AppError> {
    capture(&["branch", "--show-current"])
}

/// Resolve owner/repo from the `origin` remote URL. Understands https and ssh forms.
/// Strip a remote URL down to the owner/repo path. Shared by current_repo + tests.
fn parse_origin(url: &str) -> Result<(String, String), AppError> {
    // Trailing slash first, then .git — `<repo>.git/` would otherwise survive both.
    let trimmed = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let path = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .or_else(|| trimmed.strip_prefix("git@github.com:"))
        .or_else(|| trimmed.strip_prefix("ssh://git@github.com/"))
        .ok_or_else(|| {
            AppError::GitSubprocess(format!("origin is not a github.com remote: {url}"))
        })?;
    let (owner, repo) = path.split_once('/').ok_or_else(|| {
        AppError::InvalidInput(format!("could not parse owner/repo from origin '{url}'"))
    })?;
    if owner.is_empty() || repo.is_empty() {
        return Err(AppError::InvalidInput(format!(
            "could not parse owner/repo from origin '{url}'"
        )));
    }
    Ok((owner.to_string(), repo.to_string()))
}

/// Resolve owner/repo from the `origin` remote URL. Understands https and ssh forms.
pub fn current_repo() -> Result<(String, String), AppError> {
    let url = capture(&["remote", "get-url", "origin"])?;
    parse_origin(&url)
}

#[cfg(test)]
mod tests {
    use super::parse_origin;

    #[test]
    fn parses_https_and_ssh_origins() {
        for (url, expect) in [
            ("https://github.com/akash/gh-rs.git", ("akash", "gh-rs")),
            ("https://github.com/akash/gh-rs.git/", ("akash", "gh-rs")),
            ("https://github.com/akash/gh-rs/", ("akash", "gh-rs")),
            ("git@github.com:akash/gh-rs.git", ("akash", "gh-rs")),
            ("ssh://git@github.com/akash/gh-rs", ("akash", "gh-rs")),
        ] {
            let (o, r) = parse_origin(url).unwrap();
            assert_eq!((o.as_str(), r.as_str()), expect);
        }
    }
}
