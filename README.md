# btl (Bottle)

**btl** (pronounced "bottle") is a lightweight background task manager that automatically replaces processes when you re-run the same command. Think of it as a smart, automatic process manager that keeps only the latest instance of each command running.

## Why btl?

- **Auto-replace**: Running the same command again automatically kills the old process
- **Zero configuration**: No config files, no setup, just works
- **Safe by default**: Refuses to run as root, validates PIDs, isolates users
- **Simple interface**: Intuitive commands that feel natural
- **Process isolation**: Each user's processes are tracked separately
- **Persistent state**: Survives terminal sessions and restarts

## Installation

### From Source

```bash
git clone https://github.com/lukeocodes/btl.git
cd btl
cargo build --release
sudo cp target/release/btl /usr/local/bin/
```

### Using Cargo

```bash
cargo install btl
```

## Usage

### Background a Process

```bash
btl your-command arg1 arg2
```

Example:
```bash
btl npm run dev
# Or
btl python -m http.server 8080
# Or
btl cargo watch -x run
```

**Auto-replace**: If you run the same command again, btl automatically kills the old process and starts a new one. This is perfect for development workflows where you frequently restart servers.

### List Running Processes

```bash
btl list
```

Shows all tracked background processes with their hash, PID, start time, command, and working directory.

Example output:
```
HASH             PID      STARTED              COMMAND                        CWD
ba2f49187372    45123    2m ago              npm run dev                    /Users/you/project
13fae122192f    45201    30s ago             python -m http.server          /Users/you/server
```

### View Logs

```bash
btl logs           # List all available logs
btl logs <hash>    # Tail logs for a specific process
```

Logs are stored in `~/.local/share/btl/logs/` and can be accessed even after the process terminates.

### Kill a Process

```bash
btl kill <hash>    # Kill specific process
btl kill --all     # Kill all tracked processes
```

The kill command attempts a graceful shutdown (SIGTERM) first, with a 2-second timeout before forcing (SIGKILL).

### Clean Up

```bash
btl clean          # Remove dead process entries
btl clean --all    # Remove all entries (running or not)
```

btl automatically cleans up stale entries, but you can manually clean if needed.

## How It Works

### Hash-Based Process Identification

btl generates a deterministic hash from your command and current working directory:

```
hash = SHA256(command + cwd)[:16]
```

This means:
- Same command in the same directory = same hash = auto-replace
- Same command in different directory = different hash = both run
- Different commands = different hashes = both run

### Background Process Management

When you run `btl command args`:

1. **Hash Generation**: Computes hash from command + working directory
2. **State Check**: Loads state from `~/.local/share/btl/state.json`
3. **Auto-Replace**: If hash exists and process is alive, kills it gracefully
4. **Process Spawn**: Forks and creates new session with `setsid()`
5. **State Update**: Records PID, UID, command, cwd, timestamp, log path
6. **Log Redirection**: stdout/stderr redirected to `~/.local/share/btl/logs/<hash>.log`

### State Management

State is stored in `~/.local/share/btl/state.json`:

```json
{
  "processes": {
    "ba2f49187372": {
      "pid": 45123,
      "user_id": 501,
      "command": "npm run dev",
      "cwd": "/Users/you/project",
      "started_at": "2026-01-26T20:30:00Z",
      "log_file": "/Users/you/.local/share/btl/logs/ba2f49187372.log"
    }
  }
}
```

### Process Validation

btl validates processes by:
- Checking if PID exists via `kill(pid, 0)`
- Verifying the process belongs to the current user (UID match)
- Auto-cleaning stale entries on any state read

## Safety Features

### Root Protection

btl refuses to run as root to prevent accidental system-wide changes:

```bash
$ sudo btl command
Error: btl refuses to run as root for safety
Run as a regular user instead
```

### User Isolation

Each user's processes are tracked independently:
- State files use user-specific paths
- PID validation checks UID ownership
- Users cannot see or affect other users' processes

### Graceful Shutdown

The kill operation tries to be kind:
1. Send SIGTERM (graceful shutdown signal)
2. Wait up to 2 seconds for process to exit
3. If still running, send SIGKILL (force kill)

### PID Reuse Protection

btl validates that a PID still belongs to the expected process:
- Checks process existence
- Verifies UID matches
- Removes stale entries automatically

## Platform Support

### Supported Platforms

- **Linux**: Full support (tested on Ubuntu, Debian, Arch)
- **macOS**: Full support (tested on macOS 11+)
- **Unix-like**: Should work on any POSIX-compliant system

### Requirements

- Rust 1.70+ (for building from source)
- POSIX-compliant system with:
  - `setsid()` for session management
  - `fork()` for process spawning
  - `/proc` or similar for PID validation

### Not Supported

- **Windows**: Native Windows is not supported due to Unix-specific process management APIs

## Files and Directories

- **State**: `~/.local/share/btl/state.json` - Process tracking state
- **Logs**: `~/.local/share/btl/logs/<hash>.log` - Per-process log files

These directories are created automatically on first run.

## Common Use Cases

### Development Servers

```bash
# Start dev server
btl npm run dev

# Make changes, restart server
btl npm run dev  # Automatically kills old server
```

### Long-Running Jobs

```bash
# Start a long build
btl cargo build --release

# Check if still running
btl list

# View progress
btl logs <hash>
```

### Multiple Environments

```bash
# Terminal 1 - Frontend
cd ~/project/frontend
btl npm start

# Terminal 2 - Backend
cd ~/project/backend
btl npm start

# Both run simultaneously (different working directories)
```

## Troubleshooting

### Process Not Showing in List

If a process isn't listed, it may have:
- Exited immediately (check logs with `btl logs`)
- Failed to start (check command syntax)
- Been cleaned up (run `btl clean` to verify)

### Can't Kill Process

If `btl kill <hash>` doesn't work:
- Process may have already exited
- Run `btl clean` to remove stale entries
- Check logs for error messages

### State Corruption

If state seems corrupted:
```bash
btl clean --all  # Remove all entries
rm ~/.local/share/btl/state.json  # Nuclear option
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Author

Luke Oliff (luke@lukeoliff.com)

## Links

- GitHub: https://github.com/lukeocodes/btl
- Issues: https://github.com/lukeocodes/btl/issues
