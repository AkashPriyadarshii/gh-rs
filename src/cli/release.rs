//! `gh-rs release` — Manage GitHub releases.

use crate::cli::repo::split_repo;
use crate::error::AppError;
use crate::git;
use crate::github::release as api;
use crate::render;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ReleaseArgs {
    /// List releases in a repository
    List {
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
        /// Max rows (default 30)
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
    /// View release details
    View {
        /// Release tag (default: latest release)
        tag: Option<String>,
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
    /// Create a new release
    Create {
        /// Git tag name
        tag: String,
        /// Title for the release
        #[arg(short = 't', long)]
        title: Option<String>,
        /// Release notes
        #[arg(short = 'n', long)]
        notes: Option<String>,
        /// Save as draft (unpublished)
        #[arg(short = 'd', long)]
        draft: bool,
        /// Identify as a pre-release
        #[arg(short = 'p', long)]
        prerelease: bool,
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
    /// Delete a release by tag
    Delete {
        /// Git tag name of the release to delete
        tag: String,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
    },
    /// Download release assets
    Download {
        /// Git tag name (default: latest)
        tag: Option<String>,
        /// Filename pattern filter
        #[arg(short = 'p', long)]
        pattern: Option<String>,
        /// Destination directory
        #[arg(short = 'D', long, default_value = ".")]
        dir: String,
        /// owner/repo (default: from git origin)
        #[arg(long)]
        repo: Option<String>,
    },
}

fn resolve_repo(explicit: Option<&str>) -> Result<(String, String), AppError> {
    match explicit {
        Some(r) => split_repo(r),
        None => git::current_repo(),
    }
}

pub async fn run(args: ReleaseArgs) -> Result<(), AppError> {
    match args {
        ReleaseArgs::List {
            repo,
            limit,
            json,
            jq,
            plain,
        } => {
            let (owner, name) = resolve_repo(repo.as_deref())?;
            let releases = api::list(&owner, &name, limit.unwrap_or(30)).await?;

            let out = render::format_output(
                &releases,
                json,
                jq.as_deref(),
                plain,
                |rels| {
                    if rels.is_empty() {
                        return "No releases found.".to_string();
                    }
                    let mut lines = Vec::new();
                    for r in rels {
                        let status = if r.draft {
                            "[Draft]"
                        } else if r.prerelease {
                            "[Pre-release]"
                        } else {
                            "[Latest]"
                        };
                        let name_disp = r.name.as_deref().unwrap_or("-");
                        lines.push(format!(
                            "{:<15} {:<15} {:<30} {}",
                            r.tag_name,
                            status,
                            name_disp,
                            r.published_at.as_deref().unwrap_or("-")
                        ));
                    }
                    lines.join("\n")
                },
                Some(|rels: &Vec<api::ReleaseSummary>| -> String {
                    rels.iter()
                        .map(|r| {
                            format!(
                                "{}\t{}\t{}\t{}",
                                r.tag_name,
                                if r.draft {
                                    "draft"
                                } else if r.prerelease {
                                    "prerelease"
                                } else {
                                    "latest"
                                },
                                r.name.as_deref().unwrap_or("-"),
                                r.published_at.as_deref().unwrap_or("-")
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }),
            )?;
            println!("{out}");
            Ok(())
        }
        ReleaseArgs::View {
            tag,
            repo,
            json,
            jq,
            plain,
        } => {
            let (owner, name) = resolve_repo(repo.as_deref())?;
            let rel = api::view(&owner, &name, tag.as_deref()).await?;

            let out = render::format_output(
                &rel,
                json,
                jq.as_deref(),
                plain,
                render::release,
                Some(|r: &api::ReleaseSummary| -> String {
                    format!(
                        "{}\t{}\t{}\t{}",
                        r.tag_name,
                        r.name.as_deref().unwrap_or("-"),
                        r.html_url,
                        r.published_at.as_deref().unwrap_or("-")
                    )
                }),
            )?;
            println!("{out}");
            Ok(())
        }
        ReleaseArgs::Create {
            tag,
            title,
            notes,
            draft,
            prerelease,
            repo,
            json,
            jq,
        } => {
            let (owner, name) = resolve_repo(repo.as_deref())?;
            let rel = api::create(
                &owner,
                &name,
                &tag,
                title.as_deref(),
                notes.as_deref(),
                draft,
                prerelease,
            )
            .await?;

            let out = render::format_output(
                &rel,
                json,
                jq.as_deref(),
                false,
                |r| {
                    format!(
                        "Created release {} ({}): {}",
                        r.tag_name,
                        r.html_url,
                        if r.draft { "draft" } else { "published" }
                    )
                },
                None::<fn(&api::ReleaseSummary) -> String>,
            )?;
            println!("{out}");
            Ok(())
        }
        ReleaseArgs::Delete { tag, repo } => {
            let (owner, name) = resolve_repo(repo.as_deref())?;
            api::delete(&owner, &name, &tag).await?;
            println!("Deleted release {tag} in {owner}/{name}.");
            Ok(())
        }
        ReleaseArgs::Download {
            tag,
            pattern,
            dir,
            repo,
        } => {
            let (owner, name) = resolve_repo(repo.as_deref())?;
            let files =
                api::download(&owner, &name, tag.as_deref(), &dir, pattern.as_deref()).await?;
            if files.is_empty() {
                println!("No matching assets found to download.");
            } else {
                for f in files {
                    println!("Downloaded: {f}");
                }
            }
            Ok(())
        }
    }
}
