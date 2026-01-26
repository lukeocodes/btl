# Publishing btl - Complete Guide

This guide covers publishing btl to both crates.io and the Claude Code plugin marketplace.

## Part 1: Publish to crates.io

### Step 1: Create crates.io Account

1. Visit https://crates.io/
2. Click "Log in with GitHub"
3. Authorize the application
4. Your account is created!

### Step 2: Get API Token

1. Go to https://crates.io/settings/tokens
2. Click "New Token"
3. Give it a name (e.g., "btl-publishing")
4. Select scopes: `publish-new` and `publish-update`
5. Click "Generate"
6. **Copy the token immediately** (shown only once!)

### Step 3: Configure Cargo Credentials

```bash
# Login with your token
cargo login <your-token-here>

# This saves to ~/.cargo/credentials.toml
```

### Step 4: Pre-Publication Checklist

Verify everything is ready:

```bash
# 1. Build and test
cargo build --release
cargo test
./test_btl.sh

# 2. Check package contents
cargo package --list

# 3. Verify metadata in Cargo.toml
# - name, version, authors
# - description, license, repository
# - keywords, categories, readme

# 4. Dry run (catches issues without publishing)
cargo publish --dry-run
```

### Step 5: Publish to crates.io

```bash
# Publish for real!
cargo publish

# Wait for crates.io to process (usually < 1 minute)
# Then verify at: https://crates.io/crates/btl
```

### Step 6: Verify Installation

```bash
# In a fresh directory, try installing
cargo install rust-btl

# Test it works
btl --version
btl help
```

### Common Issues

**Issue**: "crate already exists"
**Solution**: Bump version in Cargo.toml, commit, publish again

**Issue**: "failed to verify package tarball"
**Solution**: Check that all files in Cargo.toml are present and committed

**Issue**: "missing documentation"
**Solution**: Add `#![warn(missing_docs)]` and document public APIs

## Part 2: Publish to Claude Code Marketplace

### Step 1: Prepare Plugin Package

The plugin is already created in `claude-plugin/`:
```
claude-plugin/
├── plugin.json          # Plugin metadata
├── README.md           # Marketplace description
└── skills/
    └── using-btl.md    # Skill for Claude Code
```

### Step 2: Create GitHub Repository (Plugin)

You have two options:

**Option A: Separate Plugin Repo (Recommended)**
```bash
cd claude-plugin
git init
git add .
git commit -m "Initial btl plugin for Claude Code"

# Create repo on GitHub: claude-plugin
git remote add origin https://github.com/lukeocodes/claude-plugin.git
git push -u origin main

# Tag the release
git tag v0.1.0
git push origin v0.1.0
```

**Option B: Subdirectory in btl Repo**
```bash
# In btl repo root
git add claude-plugin
git commit -m "Add Claude Code plugin"
git push

# Tag specifically for plugin
git tag plugin-v0.1.0
git push origin plugin-v0.1.0
```

### Step 3: Submit to Claude Code Marketplace

1. Visit the Claude Code marketplace submission page
2. Fill in the submission form:
   - **Plugin Name**: btl
   - **Repository URL**: https://github.com/lukeocodes/claude-plugin (or btl repo URL)
   - **Version**: 0.1.0
   - **Description**: Copy from plugin.json
   - **Installation Requirements**: Specify btl binary needed

3. Submit for review

### Step 4: Wait for Approval

The Claude Code team will:
- Review your plugin structure
- Test the skill
- Verify metadata
- Approve or request changes

Typical review time: 1-3 business days

### Step 5: Plugin Goes Live

Once approved:
- Plugin appears in Claude Code marketplace
- Users can install with: `claude plugins install btl`
- Skill automatically available in their sessions

## Part 3: Post-Publication Maintenance

### Updating crates.io

```bash
# 1. Make changes, test
# 2. Bump version in Cargo.toml
# 3. Commit and tag
git add Cargo.toml
git commit -m "Bump version to 0.1.1"
git tag v0.1.1
git push origin main --tags

# 4. Publish update
cargo publish
```

### Updating Claude Code Plugin

```bash
# 1. Update plugin.json version
# 2. Update skills or metadata
# 3. Commit and tag
git add .
git commit -m "Update plugin to 0.1.1"
git tag plugin-v0.1.1
git push origin main --tags

# 4. Submit update to marketplace
# (Users will be notified of update)
```

### Version Strategy

Use semantic versioning (semver):
- **0.1.0** → **0.1.1**: Bug fixes
- **0.1.1** → **0.2.0**: New features (backwards compatible)
- **0.2.0** → **1.0.0**: First stable release
- **1.0.0** → **2.0.0**: Breaking changes

Keep plugin version in sync with crate version for simplicity.

## Part 4: Marketing & Community

### Announce Your Plugin

1. **Twitter/X**:
   ```
   🎉 Just published btl to crates.io!

   Background processes that auto-replace on restart.
   Perfect for dev servers, watchers, builds.

   cargo install rust-btl

   Also available as @ClaudeCode plugin!

   #rustlang #devtools
   ```

2. **Reddit**:
   - r/rust: "Show HN: btl - Background task manager for dev workflows"
   - r/commandline: Focus on CLI productivity

3. **Hacker News**:
   - Title: "btl – Background task manager that auto-replaces processes"
   - Add "Show HN:" prefix if it's your first submission

4. **Dev.to**:
   - Write tutorial: "Managing Dev Servers with btl"
   - Include Claude Code integration tips

### Documentation

Create additional docs:
- **examples/**: Real-world usage patterns
- **CHANGELOG.md**: Version history
- **CONTRIBUTING.md**: Contribution guidelines

### Gather Feedback

- Monitor GitHub issues
- Respond to crates.io comments
- Join Claude Code community discussions
- Iterate based on user needs

## Checklist Summary

### Before Publishing:
- [ ] All tests pass (15/15 integration tests)
- [ ] README.md is comprehensive
- [ ] LICENSE file present (MIT)
- [ ] Cargo.toml metadata complete
- [ ] Binary builds on target platforms
- [ ] Documentation is clear

### Crates.io:
- [ ] Create account and get API token
- [ ] Run `cargo publish --dry-run`
- [ ] Publish with `cargo publish`
- [ ] Verify installation works
- [ ] Add crate badge to README

### Claude Code Plugin:
- [ ] Plugin structure complete
- [ ] Skill tested with Claude
- [ ] Create/update GitHub repo
- [ ] Tag release (v0.1.0)
- [ ] Submit to marketplace
- [ ] Wait for approval

### Post-Launch:
- [ ] Announce on social media
- [ ] Write blog post/tutorial
- [ ] Monitor for issues
- [ ] Plan next version

## Need Help?

- **Crates.io Issues**: https://users.rust-lang.org/
- **Claude Code Plugin**: Check Claude Code documentation
- **btl Specific**: Open issue at https://github.com/lukeocodes/btl/issues

Good luck with your launch! 🚀
