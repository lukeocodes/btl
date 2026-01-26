# btl Plugin for Claude Code

**Bottle up your processes** - Background and auto-replace long-running commands with intelligent process management.

## What is btl?

btl (pronounced "bottle") is a CLI tool that backgrounds processes and automatically replaces them when you run the same command again. Perfect for dev servers, watchers, and other long-running commands that need easy restart management.

## Features

- 🔄 **Auto-Replace**: Running the same command kills the previous instance
- 📍 **Directory-Aware**: Same command in different directories runs separately
- 📋 **Persistent Logs**: All output saved to `~/.btl/logs/`
- 🔒 **Safe**: User isolation, graceful shutdown, root prevention
- 💾 **State Tracking**: Survives terminal closure, tracks PIDs
- 🧹 **Auto-Cleanup**: Dead processes removed automatically

## Installation

### Via Cargo (Rust)
```bash
cargo install btl
```

### From Source
```bash
git clone https://github.com/lukeocodes/btl
cd btl
cargo install --path .
```

## Quick Start

```bash
# Background a dev server (kills previous instance if running)
btl pnpm run dev

# List all tracked processes
btl list

# View logs
btl logs <hash>

# Kill specific or all processes
btl kill <hash>
btl kill --all
```

## Usage with Claude Code

This plugin provides a skill that helps Claude Code suggest btl for process management:

**Example conversations:**
- "I need to background my dev server"
- "How do I manage long-running build processes?"
- "Start my API server in the background"

Claude will recognize these patterns and suggest using btl with the appropriate commands.

## Common Patterns

### Development Workflow
```bash
# Start your dev environment
btl pnpm dev

# Work on code, make changes...

# Restart automatically (no need to find and kill PID)
btl pnpm dev
```

### Multiple Services
```bash
# In api/ directory
btl pnpm run api

# In frontend/ directory
btl pnpm run frontend

# In worker/ directory
btl node worker.js

# All tracked separately by directory + command
btl list
```

## How It Works

btl creates a unique hash from:
- Current working directory (PWD)
- Full command string

This hash identifies processes, so:
- Same command in same directory = auto-replace ✓
- Same command in different directories = run separately ✓
- Different commands in same directory = run separately ✓

## Safety Features

- **Root Prevention**: Refuses to run as root
- **User Isolation**: Only manages processes owned by current user
- **Graceful Shutdown**: SIGTERM (3s) → SIGKILL fallback
- **PID Validation**: Verifies process ownership before killing
- **Auto-Cleanup**: Dead processes removed from tracking

## State & Logs

**Process State**: `~/.btl/state.json`
- Tracks all background processes
- PIDs, commands, start times, log locations

**Log Files**: `~/.btl/logs/<hash>.log`
- All stdout/stderr captured
- Persistent across restarts
- Easy to tail: `btl logs <hash>`

## Contributing

Contributions welcome!

- **Repository**: https://github.com/lukeocodes/btl
- **Issues**: https://github.com/lukeocodes/btl/issues
- **Crate**: https://crates.io/crates/btl

## License

MIT License - see [LICENSE](../LICENSE) for details

## Author

Luke Oliff <luke@lukeoliff.com>

## Links

- [GitHub Repository](https://github.com/lukeocodes/btl)
- [Crates.io](https://crates.io/crates/btl)
- [Documentation](https://github.com/lukeocodes/btl#readme)
