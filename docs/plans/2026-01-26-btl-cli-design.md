# btl (Bottle) - Background Task Manager Design

**Date:** 2026-01-26
**Status:** Approved for Implementation

## Overview

**btl** (pronounced "bottle", as in "bottle it up") is a Rust CLI tool that manages background processes with automatic replacement. When you run a command with btl, it backgrounds the process and remembers it. Running the same command again kills the previous instance and starts fresh.

**Example usage:**
```bash
btl pnpm run dev
btl corepack pnpm dev
btl node server.js
```

## Architecture

### Core Components

1. **State Manager** - Handles reading/writing `~/.btl/state.json` storing hash-to-PID mappings
2. **Hash Generator** - Creates consistent hashes from PWD + command string
3. **Process Manager** - Spawns, backgrounds, and terminates processes
4. **Log Manager** - Redirects process output to `~/.btl/logs/<hash>.log`
5. **Cleanup Service** - Validates PIDs on startup and provides manual cleanup

### Data Flow

```
btl [command] → hash(PWD + command) → check state.json →
  if PID exists → validate + kill (SIGTERM → SIGKILL) →
  spawn new process → update state.json → background & log
```

### Key Decisions

- Single global state file (`~/.btl/state.json`) for simplicity
- Graceful-then-forceful termination (SIGTERM with 3s timeout → SIGKILL)
- Automatic log file per command for debugging
- PID validation before killing to prevent accidents
- User ID validation for defense-in-depth security
- Refuse to run as root for safety

## Data Structures and Storage

### State File Format

**Location:** `~/.btl/state.json`

```json
{
  "processes": {
    "a3f5b2c1d4e6f7a8": {
      "pid": 12345,
      "user_id": 501,
      "command": "pnpm run dev",
      "cwd": "/Users/luke/project",
      "started_at": "2026-01-26T19:04:32Z",
      "log_file": "/Users/luke/.btl/logs/a3f5b2c1d4e6f7a8.log"
    }
  }
}
```

### Hash Generation

- **Input:** `${PWD}::${command_args.join(" ")}`
- **Algorithm:** SHA256, truncated to first 16 hex chars
- **Example:** `/Users/luke/project::pnpm run dev` → `a3f5b2c1d4e6f7a8`

### Directory Structure

```
~/.btl/
├── state.json       # Process mappings
└── logs/
    ├── a3f5b2c1.log # Per-process logs
    └── b4e7f3d2.log
```

### Rust Structs

```rust
struct ProcessState {
    pid: u32,
    user_id: u32,  // UID on Unix systems
    command: String,
    cwd: PathBuf,
    started_at: DateTime<Utc>,
    log_file: PathBuf,
}

struct BtlState {
    processes: HashMap<String, ProcessState>,
}
```

## Security and Safety

### Root Prevention

```rust
fn main() -> Result<()> {
    let current_uid = get_current_uid();
    if current_uid == 0 {
        eprintln!("Error: btl refuses to run as root for safety");
        eprintln!("Run as a regular user instead");
        std::process::exit(1);
    }
    // ... rest of main
}
```

### User ID Validation

Before killing any process:
1. Verify stored `user_id` matches current user's UID
2. Verify process exists
3. Optionally verify running process UID matches expected

This provides defense-in-depth even though OS permissions and per-user home directories already provide isolation.

## Process Management

### Spawning and Backgrounding

```rust
fn spawn_background_process(command: &str, args: &[String], log_file: &Path) -> Result<u32> {
    let mut cmd = Command::new(command);
    cmd.args(args)
       .stdin(Stdio::null())
       .stdout(Stdio::from(File::create(log_file)?))
       .stderr(Stdio::from(OpenOptions::new().append(true).open(log_file)?));

    // Platform-specific backgrounding
    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| {
            // Create new process group so btl exit doesn't kill child
            nix::unistd::setsid().map(|_| ())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
    }

    let child = cmd.spawn()?;
    Ok(child.id())
}
```

**Key insight:** Use `setsid()` to detach spawned process from btl's process group, preventing terminal closure from cascade-killing background processes.

### Graceful Termination Sequence

1. Send SIGTERM to process
2. Poll every 100ms for up to 3 seconds
3. If still alive after timeout, send SIGKILL
4. Wait up to 1 second for final cleanup
5. If still won't die, log warning and continue anyway

### PID Validation

```rust
fn is_process_alive(pid: u32) -> bool {
    // On Unix: send signal 0 (doesn't kill, just checks)
    nix::sys::signal::kill(Pid::from_raw(pid as i32), None).is_ok()
}

fn validate_pid(pid: u32, expected_uid: u32) -> bool {
    if !is_process_alive(pid) { return false; }

    // Verify UID of running process matches
    get_process_uid(pid).map(|uid| uid == expected_uid).unwrap_or(false)
}
```

## Error Handling and Cleanup

### Error Types

```rust
enum BtlError {
    StateFileLocked,
    ProcessNotFound(u32),
    PermissionDenied(String),
    InvalidCommand,
    HashCollision(String),
    IoError(std::io::Error),
}
```

### Startup Validation

Runs on every btl invocation to clean stale state:

```rust
fn validate_and_clean_state(state: &mut BtlState) -> Result<()> {
    let current_uid = get_current_uid();
    let mut to_remove = Vec::new();

    for (hash, proc_state) in &state.processes {
        // Remove if process is dead
        if !is_process_alive(proc_state.pid) {
            to_remove.push(hash.clone());
            continue;
        }

        // Remove if UID doesn't match (defensive)
        if proc_state.user_id != current_uid {
            to_remove.push(hash.clone());
        }
    }

    for hash in to_remove {
        state.processes.remove(&hash);
    }

    Ok(())
}
```

### Failure Scenarios

- **State file corrupted** → Backup old file, create fresh state
- **Can't kill process** → Log warning, update state anyway (PID might be hung/zombie)
- **Log directory full** → Warn but continue (backgrounding still works)
- **Hash collision** (extremely rare) → Append counter suffix: `a3f5b2c1-2`

The tool prioritizes reliability: if something goes wrong during kill/cleanup, it still spawns the new process rather than failing the entire operation.

## CLI Interface

### Primary Usage

```bash
btl <command> [args...]    # Run command in background, killing previous instance
```

### Additional Commands

```bash
btl list                   # Show all tracked background processes
btl clean                  # Remove dead process entries from state
btl clean --all            # Remove all entries (cleanup state file)
btl logs <hash>            # Tail the log file for a specific process
btl logs                   # Show available log hashes
btl kill <hash>            # Manually kill a specific background process
btl kill --all             # Kill all tracked processes
```

### Output Examples

```bash
$ btl pnpm run dev
[btl] Killing previous process (PID 12345)
[btl] Starting: pnpm run dev
[btl] Process backgrounded (PID 67890)
[btl] Hash: a3f5b2c1
[btl] Logs: ~/.btl/logs/a3f5b2c1.log

$ btl list
HASH      PID    STARTED          COMMAND           CWD
a3f5b2c1  67890  2m ago          pnpm run dev      ~/project
b4e7f3d2  45678  1h ago          node server.js    ~/other

$ btl logs a3f5b2c1
[Tailing ~/.btl/logs/a3f5b2c1.log]
[... log output ...]
```

### Argument Parsing

- Everything after `btl` (except recognized subcommands) is treated as the command to run
- No special escaping needed: `btl pnpm run dev` just works
- Use `--` if command starts with a flag: `btl -- -some-weird-command args`

## Implementation Details

### Dependencies

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
dirs = "5.0"  # Cross-platform home directory
clap = { version = "4.0", features = ["derive"] }  # Better arg parsing

[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

### Platform Support

- **Primary:** Linux, macOS (Unix-like systems with setsid support)
- **Windows:** Not initially supported (different backgrounding model, no signals)
- **Future:** Windows support would need `CREATE_NEW_PROCESS_GROUP` and different termination logic

### File Locking

```rust
fn with_state_lock<F, R>(f: F) -> Result<R>
where F: FnOnce(&mut BtlState) -> Result<R>
{
    let state_path = get_state_path()?;
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(state_path)?;

    // Advisory lock (flock on Unix)
    file.try_lock_exclusive()?;

    let mut state = load_state(&file)?;
    let result = f(&mut state)?;

    save_state(&file, &state)?;
    // Lock released on drop

    Ok(result)
}
```

Prevents race conditions if multiple btl instances run simultaneously.

## Distribution and Installation

### Initial Release: Cargo

```bash
cargo install btl
```

Published to crates.io for easy installation.

### Future Package Managers

Once tool is stable and has users, expand to:

**Homebrew (macOS/Linux):**
- Create `homebrew-btl` tap for immediate availability
- Submit to homebrew-core for wider distribution

**Other Package Managers:**
- **AUR (Arch Linux):** Community-maintained PKGBUILD
- **Nix/NixOS:** Derivation in nixpkgs
- **Scoop (Windows):** If Windows support added

**GitHub Releases:**
- Pre-built binaries for macOS (Intel/ARM), Linux (x86_64/ARM)
- Automated via GitHub Actions on version tags
- Enables `cargo-binstall` support

## Success Criteria

1. Single command to background and auto-replace processes
2. Processes survive terminal closure
3. No orphaned processes from stale PIDs
4. Safe multi-user operation (won't kill other users' processes)
5. Logs preserved for debugging
6. Simple, predictable CLI interface
7. Zero configuration required

## Future Enhancements (Post-MVP)

- Process monitoring/health checks
- Notification on process crashes
- Environment variable preservation
- Process groups (kill multiple related processes)
- Web UI for process management
- Windows support
- Configuration file for default behaviors
