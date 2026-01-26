use anyhow::Result;
use sha2::{Digest, Sha256};
use std::env;
use std::path::PathBuf;

pub fn generate_hash(command_args: &[String]) -> Result<String> {
    let cwd = env::current_dir()?;
    let command_str = command_args.join(" ");
    let input = format!("{}::{}", cwd.display(), command_str);

    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();

    // Take first 16 hex chars for readability
    Ok(hex::encode(&result[..8]))
}

pub fn get_log_path(hash: &str) -> Result<PathBuf> {
    let logs_dir = crate::state::get_logs_dir()?;
    Ok(logs_dir.join(format!("{}.log", hash)))
}
