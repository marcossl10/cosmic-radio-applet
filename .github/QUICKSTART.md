# CI/CD Quick Start Guide

## Getting Started

The CI/CD pipeline is configured and ready to use. Here's what you need to know.

## Automatic Checks

Every push to `main` and every pull request automatically runs:

1. Code formatting check
2. Clippy linting
3. Build (debug + release)
4. Test suite
5. Nix flake build
6. Security vulnerability scan
7. Dependency policy check

**Total time:** ~15-20 minutes (with cache)

## Local Development

Before pushing code, run these checks locally to catch issues early:

### Quick Check (Recommended)
```bash
cargo fmt --all --check && \
cargo clippy --all-targets --all-features -- -D warnings
```

### Full Check (Matches CI)
```bash
cargo fmt --all --check && \
cargo clippy --all-targets --all-features -- -D warnings && \
cargo test --locked && \
cargo build --release --locked && \
nix flake check && \
nix build
```

### Install Pre-commit Hook (Optional)
```bash
cp .github/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

This will run format, clippy, and tests automatically before each commit.

## Viewing Build Status

### Repository Badge
The README shows the current CI status:
- Green: All checks passing
- Red: Build failed
- Yellow: Build in progress

### GitHub Actions Tab
View detailed build logs at:
`https://github.com/marcossl10/cosmic-radio-applet/actions`

## Common Scenarios

### Fix Formatting Issues
```bash
cargo fmt --all
git add -u
git commit --amend --no-edit
```

### Fix Clippy Warnings
```bash
# See all warnings
cargo clippy --all-targets --all-features

# Auto-fix when possible
cargo clippy --fix --all-targets --all-features
```

### Update Dependencies
```bash
# Update to latest compatible versions
cargo update

# Update Nix flake inputs
nix flake update

# Commit lock files
git add Cargo.lock flake.lock
git commit -m "chore: update dependencies"
```

### Security Vulnerabilities
If security audit fails:

```bash
# Check which dependencies are vulnerable
cargo audit

# Update specific dependency
cargo update -p <vulnerable-crate>

# Or update all dependencies
cargo update
```

## Understanding Build Failures

### Format Check Failed
**Fix:** Run `cargo fmt --all`

### Clippy Failed
**Fix:** Run `cargo clippy --all-targets --all-features` and fix warnings

### Tests Failed
**Fix:** Run `cargo test` locally and fix failing tests

### Nix Build Failed
**Fix:** Run `nix flake check --show-trace` for detailed error

### Security Audit Failed
**Fix:** Run `cargo audit` and update vulnerable dependencies

### Dependency Check Failed
**Fix:** Run `cargo deny check` to see which policy was violated

## Build Artifacts

Release builds are uploaded as artifacts:
1. Go to Actions tab
2. Click on the workflow run
3. Download artifacts at the bottom

Artifacts are kept for 7 days.

## Branch Protection

Consider enabling branch protection for `main`:
1. Settings → Branches → Add rule
2. Pattern: `main`
3. Enable "Require status checks to pass before merging"
4. Select all CI jobs

This prevents broken code from being merged.

## Caching

The pipeline uses aggressive caching for speed:
- **Cargo:** Registry, git dependencies, build artifacts
- **Nix:** Store paths, build outputs
- **Cachix:** Binary cache (optional)

First build: ~30-40 minutes
Subsequent builds: ~15-20 minutes

## Optional: Cachix Setup

For faster Nix builds, set up Cachix:

1. Create account at https://app.cachix.org
2. Create cache named `cosmic-radio-applet`
3. Generate auth token
4. Add as repository secret: `CACHIX_AUTH_TOKEN`

Benefits:
- Faster CI builds
- Shared cache for all developers
- Public cache for users

## Troubleshooting

### "Permission denied" on pre-commit hook
```bash
chmod +x .git/hooks/pre-commit
```

### Cargo cache not working
```bash
# Clear local cache
cargo clean
rm -rf ~/.cargo/registry/cache
```

### Nix build fails locally but CI works
```bash
# Ensure flake inputs are up-to-date
nix flake update

# Clear Nix cache
nix-collect-garbage -d
```

## Getting Help

- View workflow file: `.github/workflows/ci.yml`
- Read full docs: `.github/CI_CD.md`
- Check deny config: `deny.toml`
- GitHub Actions docs: https://docs.github.com/en/actions

## Performance Tips

1. Run `cargo check` instead of `cargo build` for faster iteration
2. Use `cargo clippy --fix` to auto-fix warnings
3. Install pre-commit hook to catch issues early
4. Keep dependencies updated to reduce security scan time
5. Use `nix develop` for consistent development environment

## Next Steps

1. Push changes to trigger first CI run
2. Watch the Actions tab for build progress
3. Fix any failures that appear
4. Enable branch protection
5. Share the repository badge URL

## Questions?

Check the comprehensive documentation in `.github/CI_CD.md` or open an issue.
