# CI/CD Pipeline Documentation

## Overview

This project uses GitHub Actions for continuous integration and deployment. The pipeline ensures code quality, security, and reproducible builds across both traditional Cargo and Nix build systems.

## Workflow Jobs

### 1. Format Check (`format`)
**Purpose:** Fast-fail on code style violations
**Runs on:** Every push and PR
**Duration:** ~30 seconds

Validates that all Rust code follows the project's formatting standards using `rustfmt`.

```bash
# Run locally before committing
cargo fmt --all --check
```

### 2. Clippy Lint (`clippy`)
**Purpose:** Catch common mistakes and enforce best practices
**Runs on:** Every push and PR
**Duration:** ~2-3 minutes
**Cached:** Yes (cargo registry + build artifacts)

Runs Clippy with all warnings treated as errors (`-D warnings`).

```bash
# Run locally
cargo clippy --all-targets --all-features -- -D warnings
```

### 3. Build & Test (`build-and-test`)
**Purpose:** Validate compilation and test suite
**Runs on:** Every push and PR
**Strategy:** Matrix (dev + release profiles)
**Duration:** ~5-8 minutes per profile
**Cached:** Yes (separate caches per profile)

Builds the project and runs the test suite in both debug and release modes. Release builds are uploaded as artifacts.

```bash
# Run locally (dev)
cargo build --locked
cargo test --locked

# Run locally (release)
cargo build --release --locked
cargo test --release --locked
```

**Artifacts:**
- Binary: `cosmic-radio-applet-<commit-sha>`
- Retention: 7 days
- Access: Actions tab → specific workflow run

### 4. Nix Build (`nix-build`)
**Purpose:** Ensure reproducible builds via Nix flakes
**Runs on:** Every push and PR
**Duration:** ~10-15 minutes (first run), ~3-5 minutes (cached)
**Cached:** Yes (Nix store + Cachix)

Validates the Nix flake configuration and builds the package using pure Nix tooling.

```bash
# Run locally
nix flake check --show-trace
nix build --print-build-logs
nix develop --command cargo build
```

**Cachix Integration:**
- Cache name: `cosmic-radio-applet`
- Set `CACHIX_AUTH_TOKEN` secret to enable push
- Public cache available for faster builds

### 5. Security Audit (`security-audit`)
**Purpose:** Check for known vulnerabilities
**Runs on:** Every push and PR
**Duration:** ~1-2 minutes

Uses `cargo-audit` to check dependencies against the RustSec advisory database.

```bash
# Run locally
cargo install cargo-audit
cargo audit
```

### 6. Dependency Check (`dependency-check`)
**Purpose:** Validate licenses and detect duplicate dependencies
**Runs on:** Every push and PR
**Duration:** ~1-2 minutes

Uses `cargo-deny` to enforce dependency policies defined in `deny.toml`.

```bash
# Run locally
cargo install cargo-deny
cargo deny check
```

**Checks:**
- ✓ Security advisories
- ✓ License compliance (MIT, Apache-2.0, BSD allowed)
- ✓ Duplicate versions (warns)
- ✓ Banned crates
- ✓ Source validation (crates.io + allowed git repos)

## Performance Optimizations

### Cargo Caching
Uses `Swatinem/rust-cache@v2` with profile-specific keys:
- `cosmic-radio-applet-dev` - debug builds
- `cosmic-radio-applet-release` - release builds
- `cosmic-radio-applet-clippy` - clippy analysis

**Cache includes:**
- `~/.cargo/registry` - crate downloads
- `~/.cargo/git` - git dependencies
- `target/` - compiled artifacts

### Nix Caching
Two-layer caching strategy:
1. **Local GitHub Actions cache** - Nix store paths
2. **Cachix binary cache** - Shared across all users

**First run:** ~10-15 minutes (build everything)
**Cached run:** ~3-5 minutes (download from cache)
**No changes:** ~1-2 minutes (verify only)

### Parallel Execution
Jobs run in parallel when possible:
- Format, Clippy, Build, Nix Build run simultaneously
- Security and dependency checks run independently
- Only `ci-success` waits for all jobs

## Required System Dependencies

The CI installs these Ubuntu packages for Wayland/COSMIC development:

```bash
libxkbcommon-dev      # Keyboard handling
libwayland-dev        # Wayland protocol
libssl-dev            # TLS/crypto
libasound2-dev        # Audio (ALSA)
pkg-config            # Build tool
libfontconfig1-dev    # Font configuration
libfreetype6-dev      # Font rendering
libexpat1-dev         # XML parsing
libvulkan-dev         # GPU rendering
libinput-dev          # Input devices
libgbm-dev            # Graphics buffer manager
libudev-dev           # Device management
libdbus-1-dev         # IPC
```

## Triggering Workflows

### Automatic Triggers
- **Push to `main`** - Full CI pipeline
- **Pull Request to `main`** - Full CI pipeline

### Manual Triggers
```bash
# Via GitHub UI: Actions tab → CI workflow → Run workflow

# Via GitHub CLI
gh workflow run ci.yml
```

## Branch Protection Rules (Recommended)

Configure these in repository settings:

```yaml
Branch: main
Require status checks:
  ✓ Format Check
  ✓ Clippy Lint
  ✓ Build & Test (Cargo) / dev
  ✓ Build & Test (Cargo) / release
  ✓ Nix Build
  ✓ Security Audit
  ✓ Dependency Check
  ✓ CI Success
```

## Secrets Configuration

### Required Secrets
None - basic pipeline works without secrets

### Optional Secrets
- `CACHIX_AUTH_TOKEN` - Enable Cachix binary cache push
  ```bash
  # Generate at https://app.cachix.org
  # Settings → Repository secrets → New secret
  ```

## Debugging Failed Builds

### Format Failures
```bash
# Fix automatically
cargo fmt --all
git add -u
git commit -m "chore: format code"
```

### Clippy Failures
```bash
# See specific warnings
cargo clippy --all-targets --all-features

# Fix with auto-fix (when available)
cargo clippy --fix --all-targets --all-features
```

### Test Failures
```bash
# Run specific test
cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=full cargo test
```

### Nix Build Failures
```bash
# Check flake validity
nix flake check --show-trace

# Build with verbose output
nix build --print-build-logs --verbose

# Enter dev shell for debugging
nix develop
cargo build
```

### Dependency Issues
```bash
# Check which dependencies are problematic
cargo deny check advisories
cargo deny check licenses
cargo deny check bans

# Update to fix vulnerabilities
cargo update
```

## Local Pre-commit Checks

Run all checks locally before pushing:

```bash
# Quick check (format + clippy)
cargo fmt --all --check && \
cargo clippy --all-targets --all-features -- -D warnings

# Full check (matching CI)
cargo fmt --all --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --locked && \
cargo build --release --locked && \
nix flake check && \
nix build
```

Consider using a Git pre-commit hook:
```bash
#!/bin/bash
# .git/hooks/pre-commit
cargo fmt --all --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --locked
```

## Deployment Strategy

### Current State
The pipeline validates code but does not deploy. All jobs are CI (continuous integration).

### Future CD (Continuous Deployment)
When ready to add deployment:

1. **GitHub Releases** - Attach binaries to tags
   ```yaml
   on:
     push:
       tags: [ 'v*.*.*' ]
   ```

2. **Nix Flake Registry** - Publish to FlakeHub
3. **Container Images** - Build OCI containers
4. **Package Repositories** - Publish to AUR, nixpkgs

## Monitoring and Alerts

- **Status Badge:** Add to README.md
  ```markdown
  ![CI](https://github.com/USERNAME/cosmic-radio-applet/workflows/CI/badge.svg)
  ```

- **Notifications:** Configure in GitHub settings
  - Email on failure
  - Slack/Discord webhook integration

## Performance Metrics

Typical execution times (cached):
- Format: 30s
- Clippy: 2-3 min
- Build & Test (both): 6-10 min
- Nix Build: 3-5 min
- Security + Dependency: 2-3 min

**Total:** ~15-20 minutes for full pipeline (parallel execution)

## Maintenance

### Weekly
- Review security audit failures
- Update dependencies with vulnerabilities

### Monthly
- Review and update allowed licenses in `deny.toml`
- Check for outdated GitHub Actions versions
- Update Rust toolchain in `rust-toolchain.toml` (if added)

### Quarterly
- Review and update Nix flake inputs
- Audit cache performance and cleanup
- Review branch protection rules

## Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-deny Documentation](https://embarkstudios.github.io/cargo-deny/)
- [cargo-audit Documentation](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [Cachix Documentation](https://docs.cachix.org/)
- [Nix Flakes Book](https://nixos.wiki/wiki/Flakes)
