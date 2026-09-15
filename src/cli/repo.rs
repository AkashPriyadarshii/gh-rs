//! `gh-rs repo` — clone, create, list, view.

use crate::error::AppError;
use crate::git;
use crate::github::repo as api;
use crate::render;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum RepoArgs {
    /// Clone a repository with the system git
    Clone {
        /// owner/repo
        repo: String,
        /// target directory (default: repo name)
        dir: Option<String>,
    },
    /// Create a repository under your account (private unless --public)
    Create {
        name: String,
        #[arg(long, help = "Create as public (default: private)")]
        public: bool,
    },
    /// List your repositories (newest push first)
    List {
        /// Max rows (1-100, default 30)
        #[arg(long)]
        limit: Option<u32>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
    },
    /// Show repository details (default: current repo from git origin)
    View {
        /// owner/repo (default: from git origin)
        repo: Option<String>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
    },
}

/// Print value as pretty JSON on --json, else run the plain-text renderer.
fn emit_json<T: serde::Serialize>(value: &T, as_json: bool, text: impl FnOnce() -> String) {
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_default()
        );
    } else {
        print!("{}", text());
    }
}

pub async fn run(args: RepoArgs) -> Result<(), AppError> {
    match args {
        RepoArgs::Clone { repo, dir } => {
            let (owner, name) = split_repo(&repo)?;
            git::clone(&owner, &name, dir.as_deref())?;
            println!("Cloned {owner}/{name}.");
            Ok(())
        }
        RepoArgs::Create { name, public } => {
            if name.contains('/') {
                return Err(AppError::InvalidInput(
                    "repo create takes a bare name (not owner/name)".into(),
                ));
            }
            let r = api::create(&name, public).await?;
            let vis = if r.private { "private" } else { "public" };
            println!("Created {} ({vis}): {}", r.full_name, r.html_url);
            Ok(())
        }
        RepoArgs::List { limit, json } => {
            let repos = api::list(api::clamp_limit(limit)).await?;
            if repos.is_empty() {
                println!("No repositories found.");
                return Ok(());
            }
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&repos).unwrap_or_default()
                );
                return Ok(());
            }
            for r in &repos {
                let vis = if r.private { "private" } else { "public" };
                let lang = r.language.as_deref().unwrap_or("-");
                let pushed = r.pushed_at.as_deref().unwrap_or("-");
                println!("{:<40} {:<7} {:<14} {}", r.full_name, vis, lang, pushed);
            }
            Ok(())
        }
        RepoArgs::View { repo, json } => {
            let (owner, name) = match repo.as_deref() {
                Some(r) => split_repo(r)?,
                None => git::current_repo()?,
            };
            let r = api::view(&owner, &name).await?;
            emit_json(&r, json, || render::repository(&r));
            Ok(())
        }
    }
}

/// Shared owner/repo splitter. pr/issue `--repo` flags reuse this (async `resolve`
/// would need a tokio runtime in tests — keep it sync). Rejects empty segments
/// (`foo/`, `/bar`, `norepo`) so typos fail fast instead of 404ing on clean URLs.
pub fn split_repo(s: &str) -> Result<(String, String), AppError> {
    match s.split_once('/') {
        Some((o, n)) if !o.is_empty() && !n.is_empty() => Ok((o.to_string(), n.to_string())),
        _ => Err(AppError::InvalidInput(format!(
            "expected owner/repo, got '{s}'"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::split_repo;

    #[test]
    fn splits_owner_repo() {
        assert_eq!(
            split_repo("AkashPriyadarshii/gh-rs").unwrap(),
            ("AkashPriyadarshii".to_string(), "gh-rs".to_string())
        );
    }

    #[test]
    fn rejects_bare_and_empty() {
        assert!(split_repo("gh-rs").is_err());
        assert!(split_repo("/").is_err());
        assert!(split_repo("owner/").is_err());
    }
}
