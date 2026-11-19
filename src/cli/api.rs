//! `gh-rs api` — Make an authenticated GitHub API request.

use crate::error::AppError;
use crate::github::api as github_api;
use crate::render;
use clap::Args;

#[derive(Args, Debug)]
pub struct ApiArgs {
    /// Endpoint path (e.g. user, repos/{owner}/{repo}, or graphql)
    pub endpoint: String,

    /// HTTP method (default: GET, or POST if fields are provided)
    #[arg(short = 'X', long)]
    pub method: Option<String>,

    /// Add a string parameter in key=value format
    #[arg(short = 'f', long = "raw-field")]
    pub raw_fields: Vec<String>,

    /// Add a typed parameter in key=value format
    #[arg(short = 'F', long = "field")]
    pub fields: Vec<String>,

    /// Add a custom HTTP request header in key:value format
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// Automatically fetch all pages of results
    #[arg(long)]
    pub paginate: bool,

    /// Combine all page results into an array
    #[arg(long)]
    pub slurp: bool,

    /// File to use as request body ("-" for stdin)
    #[arg(long)]
    pub input: Option<String>,

    /// Filter JSON output with embedded jaq/jq
    #[arg(long)]
    pub jq: Option<String>,

    /// Output pretty JSON (default for api)
    #[arg(long)]
    pub json: bool,

    /// Output plain tab-separated values without quotes
    #[arg(long)]
    pub plain: bool,
}

pub async fn run(args: ApiArgs) -> Result<(), AppError> {
    let result = github_api::execute(github_api::ApiRequest {
        endpoint: &args.endpoint,
        method: args.method.as_deref(),
        raw_fields: &args.raw_fields,
        fields: &args.fields,
        headers: &args.headers,
        input_file: args.input.as_deref(),
        paginate: args.paginate,
        slurp: args.slurp,
    })
    .await?;

    let out = render::format_output(
        &result,
        true, // default to JSON formatting for api command
        args.jq.as_deref(),
        args.plain,
        |val| serde_json::to_string_pretty(val).unwrap_or_else(|_| "{}".into()),
        Some(|val: &serde_json::Value| -> String {
            match val {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Array(arr) => arr
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join("\t"),
                other => other.to_string(),
            }
        }),
    )?;

    println!("{out}");
    Ok(())
}
