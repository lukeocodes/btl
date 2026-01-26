use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessState {
    pub pid: u32,
    pub user_id: u32,
    pub command: String,
    pub cwd: PathBuf,
    pub started_at: DateTime<Utc>,
    pub log_file: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BtlState {
    pub processes: HashMap<String, ProcessState>,
}

impl BtlState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn get_btl_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".btl"))
}

pub fn get_state_path() -> Result<PathBuf> {
    Ok(get_btl_dir()?.join("state.json"))
}

pub fn get_logs_dir() -> Result<PathBuf> {
    Ok(get_btl_dir()?.join("logs"))
}

pub fn ensure_dirs() -> Result<()> {
    let btl_dir = get_btl_dir()?;
    let logs_dir = get_logs_dir()?;

    fs::create_dir_all(&btl_dir)
        .context("Failed to create .btl directory")?;
    fs::create_dir_all(&logs_dir)
        .context("Failed to create logs directory")?;

    Ok(())
}

pub fn load_state(state_path: &Path) -> Result<BtlState> {
    if !state_path.exists() {
        return Ok(BtlState::new());
    }

    let file = File::open(state_path)
        .context("Failed to open state file")?;
    let reader = BufReader::new(file);

    serde_json::from_reader(reader)
        .context("Failed to parse state file")
        .or_else(|_| {
            // If corrupted, backup and start fresh
            let backup_path = state_path.with_extension("json.backup");
            fs::copy(state_path, &backup_path)
                .context("Failed to backup corrupted state")?;
            eprintln!("[btl] Warning: Corrupted state file backed up to {:?}", backup_path);
            Ok(BtlState::new())
        })
}

pub fn save_state(state_path: &Path, state: &BtlState) -> Result<()> {
    let json = serde_json::to_string_pretty(state)
        .context("Failed to serialize state")?;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(state_path)
        .context("Failed to open state file for writing")?;

    file.write_all(json.as_bytes())
        .context("Failed to write state file")?;

    Ok(())
}

pub fn validate_and_clean_state(state: &mut BtlState) -> Result<()> {
    let current_uid = crate::process::get_current_uid();
    let mut to_remove = Vec::new();

    for (hash, proc_state) in &state.processes {
        // Remove if process is dead
        if !crate::process::is_process_alive(proc_state.pid) {
            to_remove.push(hash.clone());
            continue;
        }

        // Remove if UID doesn't match (defensive)
        if proc_state.user_id != current_uid {
            eprintln!(
                "[btl] Removing process {} - belongs to different user",
                hash
            );
            to_remove.push(hash.clone());
        }
    }

    for hash in to_remove {
        state.processes.remove(&hash);
    }

    Ok(())
}
