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

### Install from Local Clone

1. **Clone the repository:**
   ```bash
   git clone https://github.com/lukeocodes/btl
   cd btl
   ```

2. **Add the marketplace:**
   ```bash
   # Use absolute path to the claude-plugin directory
   claude plugin marketplace add ~/path/to/btl/claude-plugin
   ```

   This will create a marketplace named `btl`.

3. **Install the plugin:**
   ```bash
   claude plugin install btl@btl
   ```

4. **Verify installation:**
   ```bash
   claude plugin list | grep btl
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
claude plugin marketplace update btl

# Update the plugin
claude plugin update btl@btl
```

### From Local Development Marketplace
```bash
# Pull latest changes
cd ~/path/to/btl
git pull origin main

# Update the marketplace
claude plugin marketplace update btl

# Update the plugin
claude plugin update btl@btl
```

## Uninstalling

```bash
# Remove the plugin
claude plugin uninstall btl@btl

# Remove the marketplace (optional)
claude plugin marketplace remove btl
```

## Troubleshooting

### Plugin Not Found
If `claude plugin install btl@btl` fails:

1. Verify marketplace is added:
   ```bash
   claude plugin marketplace list
   ```

2. Check marketplace path is correct:
   ```bash
   # Should point to the claude-plugin directory
   ls ~/path/to/btl/claude-plugin/.claude-plugin/marketplace.json
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
