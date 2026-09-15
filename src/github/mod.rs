//! GitHub API commands: client.rs (credentials + octocrab), auth.rs (device flow).
//! Repo operations (gh-rs repo *) are thin wrappers over octocrab's typed models.

pub mod auth;
pub mod client;

pub mod pr;
pub mod repo;
