# Release Process

This document describes how to create a Caret release.

## Prerequisites

- Release manager privileges
- Clean working directory
- All tests passing
- Updated documentation

## Release Checklist

### Pre-Release

- [ ] All tests passing: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Documentation builds: `cargo doc --workspace`
- [ ] CHANGELOG.md updated
- [ ] Version numbers updated
- [ ] STATE.md updated

## Version Bumping

### Version Scheme

Caret follows semantic versioning:
- **MAJOR**: Breaking changes
- **MINOR**: New features (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)

### Update Versions

1. **Root `Cargo.toml`**:

```toml
[workspace.package]
version = "0.2.0"  # Update this
```

2. **Individual crates** inherit from workspace:

```toml
[package]
version.workspace = true
```

3. **Binary crates** may need explicit updates.

### Git Tags

```bash
# Tag the release
git tag -a v0.2.0 -m "Release v0.2.0"

# Push tag
git push origin v0.2.0
```

## Building Release Artifacts

### Build for All Platforms

```bash
# Linux x86_64
cross build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu

# macOS x86_64
cargo build --release --target x86_64-apple-darwin

# macOS ARM64
cargo build --release --target aarch64-apple-darwin

# Windows x86_64
cargo build --release --target x86_64-pc-windows-msvc
```

### Package Artifacts

```bash
# Create release directory
mkdir release
cd release

# Copy binaries
cp ../target/x86_64-unknown-linux-gnu/release/caret caret-linux-x86_64
cp ../target/aarch64-unknown-linux-gnu/release/caret caret-linux-aarch64
cp ../target/x86_64-apple-darwin/release/caret caret-darwin-x86_64
cp ../target/aarch64-apple-darwin/release/caret caret-darwin-aarch64
cp ../target/x86_64-pc-windows-msvc/release/caret caret-windows-x86_64.exe

# Create archives
tar czf caret-linux-x86_64.tar.gz caret-linux-x86_64
tar czf caret-linux-aarch64.tar.gz caret-linux-aarch64
tar czf caret-darwin-x86_64.tar.gz caret-darwin-x86_64
tar czf caret-darwin-aarch64.tar.gz caret-darwin-aarch64
zip caret-windows-x86_64.zip caret-windows-x86_64.exe

# Generate checksums
shasum -a 256 * > SHA256SUMS
```

## Publishing

### GitHub Release

1. Go to [Releases](https://github.com/staticpayload/caret-eternity/releases)
2. Click "Draft a new release"
3. Tag: `v0.2.0`
4. Title: `Caret v0.2.0`
5. Description: Use CHANGELOG content
6. Upload binaries and SHA256SUMS
7. Publish release

### crates.io (When Published)

```bash
# Publish workspace crates in order
cargo publish -p caret_core
cargo publish -p caret_graph
# ... etc

# Finally binary
cargo publish -p caret
```

## Announcements

### Template

```
# Caret v0.2.0 Released

Caret v0.2.0 has been released!

## Highlights

- Feature 1
- Feature 2
- Bug fix 1

## Downloads

- Linux: [link]
- macOS: [link]
- Windows: [link]

## Documentation

- [User Guide](link)
- [Migration Guide](link)

## Upgrading

See [CHANGELOG](link) for upgrade notes.

## Thanks

Thanks to all contributors!
```

### Channels

- GitHub Release
- Blog (if available)
- Twitter/Mastodon (if available)
- Discord/Slack (if available)

## Post-Release

### Update Branches

```bash
# Merge main into develop
git checkout develop
git merge main
git push origin develop

# Start next development version
# Update version to 0.3.0-dev or 0.2.1-dev
```

### Update Documentation

1. Update docs with new features
2. Update examples
3. Add migration guide if breaking changes

## Hotfix Releases

For urgent fixes:

```bash
# Create hotfix branch from release tag
git checkout -b hotfix/v0.2.1 v0.2.0

# Make fix and test
# ...

# Tag and release
git tag -a v0.2.1 -m "Hotfix v0.2.1"
git push origin v0.2.1
```

## Release Notes Template

```markdown
# Release v0.2.0

## Added
- New feature 1
- New feature 2

## Changed
- Modified behavior 1
- Modified behavior 2

## Deprecated
- Feature to be removed in next release

## Removed
- Feature removed in this release

## Fixed
- Bug fix 1
- Bug fix 2

## Security
- Security fix 1

## Upgrade Notes
### Breaking Changes
- Describe breaking change and migration path

### Deprecated Features
- List deprecated features
```

## Version History

| Version | Date | Notes |
|---------|------|-------|
| 0.2.0 | TBD | Distributed execution |
| 0.1.0 | TBD | Initial release |

## Resources

- [Semantic Versioning](https://semver.org/)
- [How to Publish a Crate](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
