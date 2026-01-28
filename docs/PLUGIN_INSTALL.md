# Installing the btl Plugin for Claude Code

The btl plugin provides intelligent assistance for using btl (Bottle) - the background process manager - directly within Claude Code.

## Prerequisites

First, install the btl CLI tool:

```bash
# Via cargo (recommended)
cargo install rust-btl

# Or from source
git clone https://github.com/lukeocodes/btl
cd btl
cargo install --path .
```

Verify installation:
```bash
btl --version
# Should show: btl 0.1.3 (or later)
```

## Installation Methods

### Install from Marketplace (Recommended)

1. **Add the marketplace:**
   ```bash
   /plugin marketplace add lukeocodes/claude-plugins
   ```

2. **Install the plugin:**
   ```bash
   /plugin install btl@lukeocodes-claude-plugins
   ```

3. **Verify installation:**
   ```bash
   /plugin list
   # Should show: ❯ btl@lukeocodes-claude-plugins
   #              Version: 0.1.3
   #              Status: ✔ enabled
   ```

### Install from Local Clone (Development)

For plugin development or testing local changes:

1. **Clone the repository:**
   ```bash
   git clone https://github.com/lukeocodes/btl
   cd btl
   ```

2. **Add as local marketplace:**
   ```bash
   # Use absolute path to the repository root
   /plugin marketplace add ~/path/to/btl
   ```

   This creates a local marketplace named `btl`.

3. **Install the plugin:**
   ```bash
   /plugin install btl@btl
   ```

4. **Verify installation:**
   ```bash
   /plugin list
   # Should show: ❯ btl@btl
   #              Version: 0.1.3
   #              Status: ✔ enabled
   ```

## What Gets Installed

The plugin provides:

### Skills
- **using-btl**: Comprehensive guide for using btl effectively
  - Auto-replace process patterns
  - Directory-aware process management
  - Status-based debugging workflow
  - Critical guidance on avoiding `btl logs` hangs

## Usage

Once installed, Claude will automatically use the btl skill when:
- You mention backgrounding processes
- You ask about long-running commands
- You need process management help
- You're debugging process issues

Example prompts that trigger the skill:
- "I need to background my dev server"
- "How do I manage long-running build processes?"
- "My btl process isn't working, can you debug it?"

## Updating

### From GitHub Marketplace
```bash
# Update the marketplace
/plugin marketplace update lukeocodes-claude-plugins

# Update the plugin
/plugin update btl@lukeocodes-claude-plugins
```

### From Local Development Marketplace
```bash
# Pull latest changes
cd ~/path/to/btl
git pull origin main

# Update the marketplace
/plugin marketplace update btl

# Update the plugin
/plugin update btl@btl
```

## Uninstalling

```bash
# Remove the plugin
/plugin uninstall btl@lukeocodes-claude-plugins

# Or if installed from local clone
/plugin uninstall btl@btl

# Remove the marketplace (optional)
/plugin marketplace remove lukeocodes-claude-plugins
```

## Troubleshooting

### Plugin Not Found
If plugin installation fails:

1. Verify marketplace is added:
   ```bash
   /plugin marketplace list
   ```

2. For local installations, check the repository has plugin files at root:
   ```bash
   # Should show plugin.json at repository root
   ls ~/path/to/btl/plugin.json
   ls ~/path/to/btl/skills/
   ```

### Skill Not Loading
If the skill doesn't appear in conversations:

1. Verify plugin is enabled:
   ```bash
   claude plugin list | grep btl
   # Should show: ✔ enabled
   ```

2. Check skill file exists:
   ```bash
   find ~/.claude/plugins -name "using-btl.md"
   ```

3. Restart Claude Code if needed

### Out of Date Skill
If the skill seems outdated after updating:

```bash
# Force marketplace update
claude plugin marketplace update btl

# Reinstall plugin
claude plugin uninstall btl@btl
claude plugin install btl@btl
```

## Support

- **Issues**: https://github.com/lukeocodes/btl/issues
- **Discussions**: https://github.com/lukeocodes/btl/discussions
- **Documentation**: https://github.com/lukeocodes/btl

## Contributing

Found a bug or have a suggestion for the plugin? Please open an issue or pull request!

## License

MIT License - see [LICENSE](../LICENSE) for details
