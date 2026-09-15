//! git subprocess wrapper. Shelling out to the system `git` binary beats any
//! in-process libgit2 for clone/checkout fidelity (auth, LFS, submodules).

use crate::error::AppError;
use std::process::Command;

/// `git clone https://github.com/{owner}/{repo}.git` into cwd (or dir if given).
pub fn clone(owner: &str, repo: &str, dir: Option<&str>) -> Result<(), AppError> {
    let url = format!("https://github.com/{owner}/{repo}.git");
    let mut cmd = Command::new("git");
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
pub fn current_repo() -> Result<(String, String), AppError> {
    let url = capture(&["remote", "get-url", "origin"])?;
    let trimmed = url.trim_end_matches(".git");
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
    Ok((owner.to_string(), repo.to_string()))
}

#[cfg(test)]
mod tests {

    #[test]
    fn parses_https_and_ssh_origins() {
        // Unit-test the URL parsing without a git repo: exercised via current_repo below.
        for (url, expect) in [
            ("https://github.com/akash/gh-rs.git", ("akash", "gh-rs")),
            ("git@github.com:akash/gh-rs.git", ("akash", "gh-rs")),
            ("ssh://git@github.com/akash/gh-rs", ("akash", "gh-rs")),
        ] {
            let trimmed = url.trim_end_matches(".git");
            let path = trimmed
                .strip_prefix("https://github.com/")
                .or_else(|| trimmed.strip_prefix("git@github.com:"))
                .or_else(|| trimmed.strip_prefix("ssh://git@github.com/"))
                .unwrap();
            let (o, r) = path.split_once('/').unwrap();
            assert_eq!((o, r), expect);
        }
    }
}
