//! `gh-rs pr` — create, list, view, diff, merge.
//! Target repo: explicit `--repo owner/repo` wins, else derived from git origin.

use crate::cli::repo::split_repo;
use crate::error::AppError;
use crate::git;
use crate::github::pr as api;
use crate::render;
use clap::Subcommand;
use octocrab::models::pulls::PullRequest;
use octocrab::params::pulls::MergeMethod;

#[derive(Subcommand)]
pub enum PrArgs {
    /// Create a pull request (head = current branch or --head, base = --base or repo default)
    Create {
        /// Title
        #[arg(long)]
        title: String,
        /// Body
        #[arg(long)]
        body: Option<String>,
        /// Head branch (default: current git branch)
        #[arg(long)]
        head: Option<String>,
        /// Base branch (default: repo default branch)
        #[arg(long)]
        base: Option<String>,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
    },
    /// List open pull requests
    List {
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Max rows (1-1000, default 30)
        #[arg(long)]
        limit: Option<usize>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
        /// Output plain tab-separated values without headers
        #[arg(long)]
        plain: bool,
    },
    /// Show pull request details
    View {
        /// PR number
        number: u64,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
        /// Output plain tab-separated values
        #[arg(long)]
        plain: bool,
    },
    /// Show the unified diff
    Diff {
        /// PR number
        number: u64,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
    },
    /// Merge a pull request
    Merge {
        /// PR number
        number: u64,
        /// Merge strategy: merge (default), squash, rebase
        #[arg(long, default_value = "merge")]
        method: String,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
    },
}

pub async fn run(args: PrArgs) -> Result<(), AppError> {
    match args {
        PrArgs::Create {
            title,
            body,
            head,
            base,
            repo,
            json,
            jq,
        } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let head = match head {
                Some(h) => h,
                None => {
                    let b = git::current_branch()?;
                    if b.is_empty() {
                        return Err(AppError::InvalidInput(
                            "detached HEAD — pass --head explicitly".into(),
                        ));
                    }
                    b
                }
            };
            let base = match base {
                Some(b) => b,
                None => {
                    crate::github::repo::view(&owner, &name)
                        .await?
                        .default_branch
                }
            };
            let pr = api::create(&owner, &name, &title, &head, &base, body.as_deref()).await?;
            let out = render::format_output(
                &pr,
                json,
                jq.as_deref(),
                false,
                |p| {
                    format!(
                        "Created #{} {} → {}/{}",
                        p.number,
                        p.title.as_deref().unwrap_or(""),
                        owner,
                        name,
                    )
                },
                None::<fn(&PullRequest) -> String>,
            )?;
            println!("{out}");
            Ok(())
        }
        PrArgs::List {
            repo,
            limit,
            json,
            jq,
            plain,
        } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let prs = api::list(&owner, &name, crate::github::repo::clamp_limit(limit)).await?;
            if prs.is_empty() && !json && jq.is_none() {
                println!("No open pull requests in {owner}/{name}.");
                return Ok(());
            }
            let out = render::format_output(
                &prs,
                json,
                jq.as_deref(),
                plain,
                |list| {
                    let mut lines = Vec::new();
                    for pr in list {
                        let login = pr.user.as_ref().map(|u| u.login.as_str()).unwrap_or("-");
                        let head = &pr.head.ref_field;
                        lines.push(format!(
                            "#{:<5} {:<60} {head:<24} {}",
                            pr.number,
                            pr.title.as_deref().unwrap_or("-"),
                            login,
                        ));
                    }
                    lines.join("\n")
                },
                Some(|list: &Vec<PullRequest>| {
                    let mut lines = Vec::new();
                    for pr in list {
                        let login = pr.user.as_ref().map(|u| u.login.as_str()).unwrap_or("-");
                        let head = &pr.head.ref_field;
                        lines.push(format!(
                            "{}\t{}\t{}\t{}",
                            pr.number,
                            pr.title.as_deref().unwrap_or("-"),
                            head,
                            login
                        ));
                    }
                    lines.join("\n")
                }),
            )?;
            if !out.is_empty() {
                println!("{out}");
            }
            Ok(())
        }
        PrArgs::View {
            number,
            repo,
            json,
            jq,
            plain,
        } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let pr = api::view(&owner, &name, number).await?;
            let out = render::format_output(
                &pr,
                json,
                jq.as_deref(),
                plain,
                render::pull_request,
                Some(|p: &PullRequest| {
                    let login = p.user.as_ref().map(|u| u.login.as_str()).unwrap_or("-");
                    format!(
                        "{}\t{}\t{}\t{}",
                        p.number,
                        p.title.as_deref().unwrap_or("-"),
                        p.head.ref_field,
                        login
                    )
                }),
            )?;
            println!("{out}");
            Ok(())
        }
        PrArgs::Diff { number, repo } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let diff = api::diff(&owner, &name, number).await?;
            print!("{diff}");
            Ok(())
        }
        PrArgs::Merge {
            number,
            method,
            repo,
            json,
            jq,
        } => {
            let (owner, name) = resolve(repo.as_deref()).await?;
            let method = match method.as_str() {
                "merge" => MergeMethod::Merge,
                "squash" => MergeMethod::Squash,
                "rebase" => MergeMethod::Rebase,
                other => {
                    return Err(AppError::InvalidInput(format!(
                        "unknown merge method '{other}' (expected merge|squash|rebase)"
                    )))
                }
            };
            let merged = api::merge(&owner, &name, number, method).await?;
            let out = render::format_output(
                &merged,
                json,
                jq.as_deref(),
                false,
                |m| {
                    format!(
                        "Merged #{} ({}).",
                        number,
                        if m.merged { "merged" } else { "not merged" }
                    )
                },
                None::<fn(&octocrab::models::pulls::Merge) -> String>,
            )?;
            println!("{out}");
            Ok(())
        }
    }
}

async fn resolve(explicit: Option<&str>) -> Result<(String, String), AppError> {
    match explicit {
        Some(repo) => split_repo(repo),
        None => git::current_repo(),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;

    #[tokio::test]
    async fn explicit_repo_wins() {
        let (o, r) = resolve(Some("AkashPriyadarshii/kharcha")).await.unwrap();
        assert_eq!(o, "AkashPriyadarshii");
        assert_eq!(r, "kharcha");
    }

    #[tokio::test]
    async fn rejects_malformed_repo() {
        assert!(resolve(Some("norepo")).await.is_err());
    }
}
