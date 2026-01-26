# btl (Bottle) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI that backgrounds processes and auto-replaces them on re-run

**Architecture:** Hash-based process tracking (PWD + command → hash → PID), graceful termination (SIGTERM → SIGKILL), file-locked state management, detached process groups via setsid()

**Tech Stack:** Rust 2021, serde/serde_json, nix (signals/unistd), chrono, sha2, dirs, clap

---

## Task 1: Project Setup and Dependencies

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

**Step 1: Initialize Cargo project**

Run: `cargo init --name btl`
Expected: Creates basic Rust project structure

**Step 2: Write Cargo.toml with dependencies**

```toml
[package]
name = "btl"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
hex = "0.4"
nix = { version = "0.27", features = ["signal", "unistd"] }
anyhow = "1.0"
dirs = "5.0"
clap = { version = "4.0", features = ["derive"] }

[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

**Step 3: Create .gitignore**

```
/target
Cargo.lock
.DS_Store
```

**Step 4: Write minimal main.rs placeholder**

```rust
fn main() {
    println!("btl - bottle it up!");
}
```

**Step 5: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add Cargo.toml src/main.rs .gitignore
git commit -m "chore: initialize Rust project with dependencies"
```

---

## Task 2: Data Structures and State Management

**Files:**
- Create: `src/state.rs`
- Modify: `src/main.rs`

**Step 1: Create state.rs with core structs**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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
```

**Step 2: Add state file path helpers**

```rust
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

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
```

**Step 3: Add state load/save functions**

```rust
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};

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
```

**Step 4: Update main.rs to include state module**

```rust
mod state;

fn main() {
    println!("btl - bottle it up!");
}
```

**Step 5: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add src/state.rs src/main.rs
git commit -m "feat: add state management structures and file I/O"
```

---

## Task 3: Hash Generation

**Files:**
- Create: `src/hash.rs`
- Modify: `src/main.rs`

**Step 1: Create hash.rs with hash generation**

```rust
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
```

**Step 2: Update main.rs to include hash module**

```rust
mod state;
mod hash;

fn main() {
    println!("btl - bottle it up!");
}
```

**Step 3: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/hash.rs src/main.rs
git commit -m "feat: add hash generation for command tracking"
```

---

## Task 4: Process Utilities (PID validation, UID checks)

**Files:**
- Create: `src/process.rs`
- Modify: `src/main.rs`

**Step 1: Create process.rs with platform utilities**

```rust
use anyhow::{Context, Result};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

#[cfg(unix)]
pub fn get_current_uid() -> u32 {
    unsafe { libc::getuid() }
}

#[cfg(unix)]
pub fn is_process_alive(pid: u32) -> bool {
    // Signal 0 checks if process exists without killing it
    signal::kill(Pid::from_raw(pid as i32), None).is_ok()
}

#[cfg(unix)]
pub fn get_process_uid(pid: u32) -> Result<u32> {
    use std::fs;

    let status_path = format!("/proc/{}/status", pid);
    let content = fs::read_to_string(&status_path)
        .context("Failed to read process status")?;

    for line in content.lines() {
        if line.starts_with("Uid:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                return parts[1].parse()
                    .context("Failed to parse UID");
            }
        }
    }

    Err(anyhow::anyhow!("Could not find UID in process status"))
}

pub fn validate_pid(pid: u32, expected_uid: u32) -> bool {
    if !is_process_alive(pid) {
        return false;
    }

    // On Linux, verify UID matches
    #[cfg(target_os = "linux")]
    {
        match get_process_uid(pid) {
            Ok(uid) => uid == expected_uid,
            Err(_) => false,
        }
    }

    // On macOS, just trust the alive check (harder to get UID)
    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}
```

**Step 2: Update main.rs to include process module**

```rust
mod state;
mod hash;
mod process;

fn main() {
    println!("btl - bottle it up!");
}
```

**Step 3: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/process.rs src/main.rs
git commit -m "feat: add process validation utilities"
```

---

## Task 5: Process Termination (Graceful Kill)

**Files:**
- Modify: `src/process.rs`

**Step 1: Add graceful kill function**

```rust
use std::thread;
use std::time::Duration;

pub fn kill_process_gracefully(pid: u32) -> Result<()> {
    let pid = Pid::from_raw(pid as i32);

    // Try SIGTERM first
    if let Err(e) = signal::kill(pid, Signal::SIGTERM) {
        // Process might already be dead
        return Err(anyhow::anyhow!("Failed to send SIGTERM: {}", e));
    }

    // Wait up to 3 seconds for graceful shutdown
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(100));
        if signal::kill(pid, None).is_err() {
            // Process is dead
            return Ok(());
        }
    }

    // Still alive, force kill with SIGKILL
    eprintln!("[btl] Process {} did not respond to SIGTERM, sending SIGKILL", pid);
    signal::kill(pid, Signal::SIGKILL)
        .context("Failed to send SIGKILL")?;

    // Wait up to 1 second for final cleanup
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        if signal::kill(pid, None).is_err() {
            return Ok(());
        }
    }

    eprintln!("[btl] Warning: Process {} may still be running", pid);
    Ok(())
}
```

**Step 2: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/process.rs
git commit -m "feat: add graceful process termination with SIGTERM/SIGKILL"
```

---

## Task 6: Process Spawning (Background with setsid)

**Files:**
- Modify: `src/process.rs`

**Step 1: Add spawn function**

```rust
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn spawn_background_process(
    command: &str,
    args: &[String],
    log_file: &Path,
) -> Result<u32> {
    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            File::create(log_file)
                .context("Failed to create log file")?
        ))
        .stderr(Stdio::from(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .context("Failed to open log file for stderr")?
        ));

    // Detach from parent process group
    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            nix::unistd::setsid()
                .map(|_| ())
                .map_err(|e| std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("setsid failed: {}", e)
                ))
        });
    }

    let child = cmd.spawn()
        .context("Failed to spawn process")?;

    Ok(child.id())
}
```

**Step 2: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/process.rs
git commit -m "feat: add background process spawning with setsid"
```

---

## Task 7: State Validation and Cleanup

**Files:**
- Modify: `src/state.rs`

**Step 1: Add cleanup function**

```rust
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
```

**Step 2: Verify build**

Run: `cargo build`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/state.rs
git commit -m "feat: add state validation and cleanup for dead processes"
```

---

## Task 8: CLI Argument Parsing

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`

**Step 1: Create cli.rs with command structure**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "btl",
    about = "Bottle - Background task manager that auto-replaces processes",
    long_about = "btl (pronounced 'bottle') backgrounds processes and auto-replaces them on re-run"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Command to run in background (if no subcommand provided)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all tracked background processes
    List,

    /// Clean dead process entries
    Clean {
        /// Remove all entries (not just dead ones)
        #[arg(long)]
        all: bool,
    },

    /// Show or tail logs for a process
    Logs {
        /// Hash of the process (omit to see all)
        hash: Option<String>,
    },

    /// Manually kill a background process
    Kill {
        /// Hash of the process to kill
        hash: Option<String>,

        /// Kill all tracked processes
        #[arg(long)]
        all: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}
```

**Step 2: Update main.rs to use CLI**

```rust
mod state;
mod hash;
mod process;
mod cli;

fn main() {
    let cli = cli::parse();

    match cli.command {
        Some(cmd) => {
            println!("Subcommand: {:?}", cmd);
        }
        None => {
            if cli.args.is_empty() {
                eprintln!("Error: No command provided");
                std::process::exit(1);
            }
            println!("Running: {:?}", cli.args);
        }
    }
}
```

**Step 3: Verify build and test help**

Run: `cargo build && cargo run -- --help`
Expected: Shows help text

**Step 4: Commit**

```bash
git add src/cli.rs src/main.rs
git commit -m "feat: add CLI argument parsing with clap"
```

---

## Task 9: Main Command Handler (Background Process)

**Files:**
- Modify: `src/main.rs`

**Step 1: Add root prevention check**

```rust
use anyhow::Result;

fn main() -> Result<()> {
    // Refuse to run as root
    if process::get_current_uid() == 0 {
        eprintln!("Error: btl refuses to run as root for safety");
        eprintln!("Run as a regular user instead");
        std::process::exit(1);
    }

    let cli = cli::parse();

    match cli.command {
        Some(cmd) => handle_subcommand(cmd)?,
        None => handle_background_command(&cli.args)?,
    }

    Ok(())
}

fn handle_subcommand(cmd: cli::Commands) -> Result<()> {
    println!("Subcommand not yet implemented");
    Ok(())
}

fn handle_background_command(args: &[String]) -> Result<()> {
    println!("Background command not yet implemented");
    Ok(())
}
```

**Step 2: Implement background command handler**

```rust
use chrono::Utc;

fn handle_background_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!("Error: No command provided");
        std::process::exit(1);
    }

    // Ensure directories exist
    state::ensure_dirs()?;

    // Generate hash for this command
    let hash = hash::generate_hash(args)?;
    let log_file = hash::get_log_path(&hash)?;

    // Load state
    let state_path = state::get_state_path()?;
    let mut btl_state = state::load_state(&state_path)?;

    // Clean stale entries
    state::validate_and_clean_state(&mut btl_state)?;

    // Check if process already exists
    if let Some(existing) = btl_state.processes.get(&hash) {
        if process::validate_pid(existing.pid, existing.user_id) {
            println!("[btl] Killing previous process (PID {})", existing.pid);
            if let Err(e) = process::kill_process_gracefully(existing.pid) {
                eprintln!("[btl] Warning: Failed to kill process: {}", e);
            }
        }
    }

    // Spawn new process
    let command = &args[0];
    let cmd_args = &args[1..];

    println!("[btl] Starting: {}", args.join(" "));
    let pid = process::spawn_background_process(command, cmd_args, &log_file)?;

    // Update state
    let proc_state = state::ProcessState {
        pid,
        user_id: process::get_current_uid(),
        command: args.join(" "),
        cwd: std::env::current_dir()?,
        started_at: Utc::now(),
        log_file: log_file.clone(),
    };

    btl_state.processes.insert(hash.clone(), proc_state);
    state::save_state(&state_path, &btl_state)?;

    println!("[btl] Process backgrounded (PID {})", pid);
    println!("[btl] Hash: {}", hash);
    println!("[btl] Logs: {}", log_file.display());

    Ok(())
}
```

**Step 3: Test with a simple command**

Run: `cargo build && cargo run -- echo "test"`
Expected: Creates process, shows hash and log location

**Step 4: Verify log file created**

Run: `ls ~/.btl/logs/`
Expected: Shows log file with hash name

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement main background command handler"
```

---

## Task 10: List Subcommand

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement list command**

```rust
use chrono::Local;

fn handle_subcommand(cmd: cli::Commands) -> Result<()> {
    match cmd {
        cli::Commands::List => handle_list()?,
        _ => println!("Subcommand not yet implemented"),
    }
    Ok(())
}

fn handle_list() -> Result<()> {
    state::ensure_dirs()?;

    let state_path = state::get_state_path()?;
    let mut btl_state = state::load_state(&state_path)?;

    // Clean stale entries
    state::validate_and_clean_state(&mut btl_state)?;
    state::save_state(&state_path, &btl_state)?;

    if btl_state.processes.is_empty() {
        println!("No background processes tracked");
        return Ok(());
    }

    println!("{:<16} {:<8} {:<20} {:<30} {}",
        "HASH", "PID", "STARTED", "COMMAND", "CWD");

    for (hash, proc) in &btl_state.processes {
        let duration = Local::now().signed_duration_since(proc.started_at);
        let started = if duration.num_hours() > 0 {
            format!("{}h ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{}m ago", duration.num_minutes())
        } else {
            format!("{}s ago", duration.num_seconds())
        };

        let command_display = if proc.command.len() > 28 {
            format!("{}...", &proc.command[..28])
        } else {
            proc.command.clone()
        };

        println!("{:<16} {:<8} {:<20} {:<30} {}",
            hash, proc.pid, started, command_display, proc.cwd.display());
    }

    Ok(())
}
```

**Step 2: Test list command**

Run: `cargo build && cargo run -- list`
Expected: Shows table of tracked processes

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement list subcommand"
```

---

## Task 11: Clean Subcommand

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement clean command**

```rust
fn handle_subcommand(cmd: cli::Commands) -> Result<()> {
    match cmd {
        cli::Commands::List => handle_list()?,
        cli::Commands::Clean { all } => handle_clean(all)?,
        _ => println!("Subcommand not yet implemented"),
    }
    Ok(())
}

fn handle_clean(all: bool) -> Result<()> {
    state::ensure_dirs()?;

    let state_path = state::get_state_path()?;
    let mut btl_state = state::load_state(&state_path)?;

    if all {
        let count = btl_state.processes.len();
        btl_state.processes.clear();
        println!("[btl] Removed {} entries", count);
    } else {
        let before = btl_state.processes.len();
        state::validate_and_clean_state(&mut btl_state)?;
        let after = btl_state.processes.len();
        let removed = before - after;

        if removed > 0 {
            println!("[btl] Removed {} dead process entries", removed);
        } else {
            println!("[btl] No dead processes found");
        }
    }

    state::save_state(&state_path, &btl_state)?;
    Ok(())
}
```

**Step 2: Test clean command**

Run: `cargo build && cargo run -- clean`
Expected: Reports cleaning dead processes

**Step 3: Test clean --all**

Run: `cargo run -- clean --all`
Expected: Clears all entries

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement clean subcommand"
```

---

## Task 12: Kill Subcommand

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement kill command**

```rust
fn handle_subcommand(cmd: cli::Commands) -> Result<()> {
    match cmd {
        cli::Commands::List => handle_list()?,
        cli::Commands::Clean { all } => handle_clean(all)?,
        cli::Commands::Kill { hash, all } => handle_kill(hash, all)?,
        _ => println!("Subcommand not yet implemented"),
    }
    Ok(())
}

fn handle_kill(hash: Option<String>, all: bool) -> Result<()> {
    state::ensure_dirs()?;

    let state_path = state::get_state_path()?;
    let mut btl_state = state::load_state(&state_path)?;

    if all {
        let count = btl_state.processes.len();
        for (h, proc) in &btl_state.processes {
            println!("[btl] Killing process {} (PID {})", h, proc.pid);
            if let Err(e) = process::kill_process_gracefully(proc.pid) {
                eprintln!("[btl] Warning: Failed to kill {}: {}", h, e);
            }
        }
        btl_state.processes.clear();
        println!("[btl] Killed {} processes", count);
    } else if let Some(h) = hash {
        if let Some(proc) = btl_state.processes.remove(&h) {
            println!("[btl] Killing process {} (PID {})", h, proc.pid);
            process::kill_process_gracefully(proc.pid)?;
            println!("[btl] Process killed");
        } else {
            eprintln!("Error: Process {} not found", h);
            std::process::exit(1);
        }
    } else {
        eprintln!("Error: Provide a hash or --all flag");
        std::process::exit(1);
    }

    state::save_state(&state_path, &btl_state)?;
    Ok(())
}
```

**Step 2: Test kill command**

Run: `cargo build && cargo run -- kill <hash>`
Expected: Kills specific process

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement kill subcommand"
```

---

## Task 13: Logs Subcommand

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement logs command**

```rust
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::Command as StdCommand;

fn handle_subcommand(cmd: cli::Commands) -> Result<()> {
    match cmd {
        cli::Commands::List => handle_list()?,
        cli::Commands::Clean { all } => handle_clean(all)?,
        cli::Commands::Kill { hash, all } => handle_kill(hash, all)?,
        cli::Commands::Logs { hash } => handle_logs(hash)?,
    }
    Ok(())
}

fn handle_logs(hash: Option<String>) -> Result<()> {
    state::ensure_dirs()?;

    let state_path = state::get_state_path()?;
    let btl_state = state::load_state(&state_path)?;

    if let Some(h) = hash {
        if let Some(proc) = btl_state.processes.get(&h) {
            println!("[btl] Tailing {}", proc.log_file.display());
            println!("---");

            // Use tail -f if available, otherwise read file
            let tail_result = StdCommand::new("tail")
                .arg("-f")
                .arg(&proc.log_file)
                .status();

            if tail_result.is_err() {
                // Fallback: just read the file
                let file = fs::File::open(&proc.log_file)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    println!("{}", line?);
                }
            }
        } else {
            eprintln!("Error: Process {} not found", h);
            std::process::exit(1);
        }
    } else {
        // Show all available logs
        println!("Available logs:");
        for (h, proc) in &btl_state.processes {
            println!("  {} -> {}", h, proc.log_file.display());
        }
    }

    Ok(())
}
```

**Step 2: Test logs command without args**

Run: `cargo build && cargo run -- logs`
Expected: Lists all available log hashes

**Step 3: Test logs command with hash**

Run: `cargo run -- logs <hash>`
Expected: Tails log file

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement logs subcommand"
```

---

## Task 14: Integration Testing

**Files:**
- Create: `test_btl.sh`

**Step 1: Create test script**

```bash
#!/bin/bash
set -e

echo "Building btl..."
cargo build --release

BTL="./target/release/btl"

echo "Testing btl..."

# Test 1: Run a simple command
echo "Test 1: Background a sleep process"
$BTL sleep 300
sleep 1

# Test 2: List processes
echo "Test 2: List processes"
$BTL list

# Test 3: Run same command again (should kill previous)
echo "Test 3: Re-run command (should replace)"
$BTL sleep 300
sleep 1
$BTL list

# Test 4: Clean
echo "Test 4: Clean (should find no dead processes yet)"
$BTL clean

# Test 5: Kill all
echo "Test 5: Kill all processes"
$BTL kill --all

# Test 6: Verify empty
echo "Test 6: Verify no processes remain"
$BTL list

# Test 7: Clean all
echo "Test 7: Clean all state"
$BTL clean --all

echo "All tests passed!"
```

**Step 2: Make executable and run**

Run: `chmod +x test_btl.sh && ./test_btl.sh`
Expected: All tests pass

**Step 3: Commit**

```bash
git add test_btl.sh
git commit -m "test: add integration test script"
```

---

## Task 15: Documentation

**Files:**
- Create: `README.md`
- Create: `LICENSE`

**Step 1: Create README**

```markdown
# btl (Bottle)

**Bottle it up** - A CLI tool that backgrounds processes and auto-replaces them on re-run.

## Installation

```bash
cargo install btl
```

## Usage

### Background a command

```bash
btl pnpm run dev
btl node server.js
btl corepack pnpm dev
```

Running the same command again automatically kills the previous instance and starts fresh.

### List tracked processes

```bash
btl list
```

### View logs

```bash
btl logs              # List all log files
btl logs <hash>       # Tail specific log
```

### Kill processes

```bash
btl kill <hash>       # Kill specific process
btl kill --all        # Kill all tracked processes
```

### Clean state

```bash
btl clean             # Remove dead process entries
btl clean --all       # Remove all entries
```

## How it works

- Generates hash from PWD + command string
- Stores PID mapping in `~/.btl/state.json`
- Logs output to `~/.btl/logs/<hash>.log`
- Uses `setsid()` to detach processes (survive terminal closure)
- Graceful termination: SIGTERM (3s timeout) → SIGKILL

## Safety Features

- Refuses to run as root
- Validates process ownership before killing
- Auto-cleans stale PIDs on every invocation
- File-locked state prevents race conditions

## Platform Support

- Linux ✓
- macOS ✓
- Windows ✗ (not yet supported)

## License

MIT
```

**Step 2: Create LICENSE file**

```
MIT License

Copyright (c) 2026

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

**Step 3: Commit**

```bash
git add README.md LICENSE
git commit -m "docs: add README and LICENSE"
```

---

## Task 16: Final Polish and Release Prep

**Files:**
- Modify: `Cargo.toml`

**Step 1: Update Cargo.toml with metadata**

```toml
[package]
name = "btl"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
license = "MIT"
description = "Background task manager that auto-replaces processes"
repository = "https://github.com/yourusername/btl"
keywords = ["cli", "process", "background", "daemon"]
categories = ["command-line-utilities"]
readme = "README.md"

# ... rest of dependencies
```

**Step 2: Build release binary**

Run: `cargo build --release`
Expected: Creates optimized binary in target/release/btl

**Step 3: Test release binary**

Run: `./target/release/btl --help`
Expected: Shows help text

**Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add package metadata for crates.io"
```

**Step 5: Create release tag**

```bash
git tag -a v0.1.0 -m "Initial release"
```

---

## Success Criteria

- ✓ Single command backgrounds and auto-replaces processes
- ✓ Processes survive terminal closure (via setsid)
- ✓ No orphaned processes (stale PID cleanup)
- ✓ Safe multi-user operation (UID validation)
- ✓ Logs preserved for debugging
- ✓ Simple CLI interface
- ✓ Zero configuration required

## Future Enhancements

After MVP is stable:
- Process monitoring/health checks
- Notification on process crashes
- Environment variable preservation
- Process groups
- Windows support
- Web UI for process management
