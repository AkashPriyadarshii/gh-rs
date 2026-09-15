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
