//! `gh-rs repo` — clone, create, list, view.

use crate::error::AppError;
use crate::git;
use crate::github::repo as api;
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
    /// List your repositories (first page only)
    List,
    /// Show repository details
    View {
        /// owner/repo
        repo: String,
    },
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
        RepoArgs::List => {
            let repos = api::list().await?;
            if repos.is_empty() {
                println!("No repositories found.");
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
        RepoArgs::View { repo } => {
            let (owner, name) = split_repo(&repo)?;
            let r = api::view(&owner, &name).await?;
            let vis = if r.private { "private" } else { "public" };
            println!("{} ({vis})", r.full_name);
            if let Some(d) = &r.description {
                println!();
                println!("  {d}");
            }
            println!();
            println!("  language:       {}", r.language.as_deref().unwrap_or("-"));
            println!("  stars:          {}", r.stargazers_count);
            println!("  default branch: {}", r.default_branch);
            println!(
                "  pushed:         {}",
                r.pushed_at.as_deref().unwrap_or("-")
            );
            println!("  clone:          {}", r.clone_url);
            println!("  web:            {}", r.html_url);
            Ok(())
        }
    }
}

fn split_repo(s: &str) -> Result<(String, String), AppError> {
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
