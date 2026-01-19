# Release Process

This document describes how to create a new release for the NQRust Identity installer.

## Binary & Package Names

> Note: the shipped binary/apt package name is `nqrust-identity`.

## Release Artifacts

Each release should include the following artifacts:

- `nqrust-identity_amd64.deb` — Debian package produced by `cargo deb` (stable alias for latest amd64).
- `SHA256SUMS` — Checksum file for verification.

## Creating a Release

### Automated (Recommended)

The project uses GitHub Actions for automated releases:

1. **Update version** in `Cargo.toml`:
   ```toml
   [package]
   version = "0.2.0"  # Increment version
   ```

2. **Commit and push**:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.2.0"
   git push origin main
   ```

3. **Create and push tag**:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **Wait for GitHub Actions** to complete (~2-5 minutes)
   - Workflow builds the `.deb` package
   - Generates `SHA256SUMS`
   - Creates GitHub Release automatically
   - Uploads artifacts

5. **Verify release** at:
   ```
   https://github.com/NexusQuantum/installer-NQRust-Identity/releases
   ```

### Manual Release

If you need to create a release manually:

1. **Build the .deb package**:
   ```bash
   cargo install cargo-deb
   cargo deb
   ```

2. **Generate checksums**:
   ```bash
   cd target/debian
   sha256sum *.deb > SHA256SUMS
   ```

3. **Create GitHub Release**:
   - Go to: https://github.com/NexusQuantum/installer-NQRust-Identity/releases/new
   - Tag: `v0.2.0`
   - Title: `Release v0.2.0`
   - Upload:
     - `nqrust-identity_0.2.0_amd64.deb`
     - `SHA256SUMS`

4. **Publish release**

## Testing the Release

After creating a release, test the one-liner installer:

```bash
curl -fsSL https://raw.githubusercontent.com/NexusQuantum/installer-NQRust-Identity/main/scripts/install/install.sh | bash
```

Verify:
- ✅ Downloads correct `.deb` package
- ✅ Verifies checksum
- ✅ Installs successfully
- ✅ Binary `nqrust-identity` is available in PATH
- ✅ Installer runs correctly

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.2.0): New features, backwards compatible
- **PATCH** (0.1.1): Bug fixes, backwards compatible

## Changelog

Update `CHANGELOG.md` with release notes before tagging.
