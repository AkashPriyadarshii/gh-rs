//! gh-rs — GitHub CLI in Rust (v0.1).

mod cli;
mod error;
mod git;
mod github;
mod render;

use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = cli::Cli::parse();
    if let Err(e) = cli::run(args).await {
        eprintln!("gh-rs: {e}");
        std::process::exit(1);
    }
}
