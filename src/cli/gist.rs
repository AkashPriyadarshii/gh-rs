//! `gh-rs gist` — Manage GitHub Gists.

use crate::error::AppError;
use crate::github::gist as api;
use crate::render;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum GistArgs {
    /// List your gists
    List {
        /// Max rows (default 10)
        #[arg(long)]
        limit: Option<usize>,
        /// Show only public gists
        #[arg(long)]
        public: bool,
        /// Show only secret gists
        #[arg(long)]
        secret: bool,
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
    /// View a gist
    View {
        /// Gist ID or URL
        id: String,
        /// View a specific file from the gist
        #[arg(short = 'f', long)]
        filename: Option<String>,
        /// Print raw file contents only
        #[arg(short = 'r', long)]
        raw: bool,
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
    /// Create a new gist
    Create {
        /// Files to include in the gist (use "-" for stdin)
        files: Vec<String>,
        /// Description of the gist
        #[arg(short = 'd', long = "desc")]
        description: Option<String>,
        /// Make the gist publicly listed (default: secret)
        #[arg(short = 'p', long)]
        public: bool,
        /// Filename to use when reading from stdin
        #[arg(short = 'f', long)]
        filename: Option<String>,
        /// Output as JSON (agent-friendly)
        #[arg(long)]
        json: bool,
        /// Filter JSON output with embedded jaq/jq
        #[arg(long)]
        jq: Option<String>,
    },
    /// Delete a gist by ID
    Delete {
        /// Gist ID or URL
        id: String,
    },
}

pub async fn run(args: GistArgs) -> Result<(), AppError> {
    match args {
        GistArgs::List {
            limit,
            public,
            secret,
            json,
            jq,
            plain,
        } => {
            let gists = api::list(limit.unwrap_or(10), public, secret).await?;

            let out = render::format_output(
                &gists,
                json,
                jq.as_deref(),
                plain,
                |list| {
                    if list.is_empty() {
                        return "No gists found.".to_string();
                    }
                    let mut lines = Vec::new();
                    for g in list {
                        let vis = if g.public { "public" } else { "secret" };
                        let desc = g.description.as_deref().unwrap_or("-");
                        let file_count = format!("{} file(s)", g.files.len());
                        lines.push(format!(
                            "{:<34} {:<8} {:<10} {:<30} {}",
                            g.id, vis, file_count, desc, g.updated_at
                        ));
                    }
                    lines.join("\n")
                },
                Some(|list: &Vec<api::GistSummary>| -> String {
                    list.iter()
                        .map(|g| {
                            format!(
                                "{}\t{}\t{}\t{}\t{}",
                                g.id,
                                if g.public { "public" } else { "secret" },
                                g.files.len(),
                                g.description.as_deref().unwrap_or("-"),
                                g.html_url
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }),
            )?;
            println!("{out}");
            Ok(())
        }
        GistArgs::View {
            id,
            filename,
            raw,
            json,
            jq,
            plain,
        } => {
            let gist = api::view(&id).await?;

            if raw {
                if let Some(target_file) = filename {
                    if let Some(f) = gist.files.get(&target_file) {
                        if let Some(content) = &f.content {
                            print!("{content}");
                            return Ok(());
                        }
                    }
                    return Err(AppError::InvalidInput(format!(
                        "file {target_file} not found in gist {id}"
                    )));
                } else if let Some((_, f)) = gist.files.iter().next() {
                    if let Some(content) = &f.content {
                        print!("{content}");
                        return Ok(());
                    }
                }
            }

            let out = render::format_output(
                &gist,
                json,
                jq.as_deref(),
                plain,
                render::gist,
                Some(|g: &api::GistSummary| -> String {
                    format!(
                        "{}\t{}\t{}\t{}",
                        g.id,
                        if g.public { "public" } else { "secret" },
                        g.description.as_deref().unwrap_or("-"),
                        g.html_url
                    )
                }),
            )?;
            println!("{out}");
            Ok(())
        }
        GistArgs::Create {
            files,
            description,
            public,
            filename,
            json,
            jq,
        } => {
            if files.is_empty() {
                return Err(AppError::InvalidInput(
                    "at least one file path or '-' (stdin) is required".into(),
                ));
            }

            let mut loaded_files = Vec::new();
            let mut file_contents = Vec::new();

            for path in &files {
                if path == "-" {
                    use std::io::Read;
                    let mut s = String::new();
                    std::io::stdin().read_to_string(&mut s).map_err(|e| {
                        AppError::InvalidInput(format!("failed to read stdin: {e}"))
                    })?;
                    let fname = filename
                        .clone()
                        .unwrap_or_else(|| "gistfile1.txt".to_string());
                    file_contents.push((fname, s));
                } else {
                    let s = std::fs::read_to_string(path).map_err(|e| {
                        AppError::InvalidInput(format!("failed to read file {path}: {e}"))
                    })?;
                    let fname = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path)
                        .to_string();
                    file_contents.push((fname, s));
                }
            }

            for (name, content) in &file_contents {
                loaded_files.push((name.as_str(), content.as_str()));
            }

            let created = api::create(&loaded_files, description.as_deref(), public).await?;

            let out = render::format_output(
                &created,
                json,
                jq.as_deref(),
                false,
                |g| {
                    format!(
                        "Created {} gist {}: {}",
                        if g.public { "public" } else { "secret" },
                        g.id,
                        g.html_url
                    )
                },
                None::<fn(&api::GistSummary) -> String>,
            )?;
            println!("{out}");
            Ok(())
        }
        GistArgs::Delete { id } => {
            api::delete(&id).await?;
            println!("Deleted gist {id}.");
            Ok(())
        }
    }
}
