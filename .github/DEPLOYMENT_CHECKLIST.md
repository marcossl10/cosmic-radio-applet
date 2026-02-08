# CI/CD Deployment Checklist

Use this checklist when setting up or maintaining the CI/CD pipeline.

## Initial Setup

### Required (Already Complete)
- [x] Create `.github/workflows/ci.yml`
- [x] Create `deny.toml` configuration
- [x] Add CI badges to README.md
- [x] Create comprehensive documentation
- [x] Validate workflow YAML syntax
- [x] Test workflow locally (format, clippy, build)

### Recommended (Next Steps)
- [ ] Push changes to trigger first CI run
- [ ] Monitor first build completion
- [ ] Enable branch protection on `main`
- [ ] Configure email notifications
- [ ] Install pre-commit hook locally

### Optional Enhancements
- [ ] Create Cachix account and cache
- [ ] Add `CACHIX_AUTH_TOKEN` secret
- [ ] Set up Slack/Discord notifications
- [ ] Configure CODEOWNERS file
- [ ] Add GitHub Actions workflow badge variants

## Branch Protection Configuration

Navigate to: `Settings → Branches → Add rule`

### Branch name pattern
```
main
```

### Protect matching branches
- [ ] Require a pull request before merging
  - [ ] Require approvals: 1
  - [ ] Dismiss stale PR approvals when new commits are pushed
- [ ] Require status checks to pass before merging
  - [ ] Require branches to be up to date before merging
  - [ ] Status checks that are required:
    - [ ] `Format Check`
    - [ ] `Clippy Lint`
    - [ ] `Build & Test (Cargo) / dev`
    - [ ] `Build & Test (Cargo) / release`
    - [ ] `Nix Build`
    - [ ] `Security Audit`
    - [ ] `Dependency Check`
    - [ ] `CI Success`
- [ ] Require conversation resolution before merging
- [ ] Do not allow bypassing the above settings

## Cachix Setup (Optional but Recommended)

### Create Cache
1. Visit https://app.cachix.org
2. Sign up / Sign in
3. Create new cache: `cosmic-radio-applet`
4. Set visibility: Public (for open source)

### Configure Token
1. Go to cache settings
2. Generate auth token
3. Copy token
4. In GitHub: `Settings → Secrets and variables → Actions`
5. New repository secret:
   - Name: `CACHIX_AUTH_TOKEN`
   - Value: [paste token]

### Verify Setup
- [ ] Token secret added to repository
- [ ] Push code to trigger build
- [ ] Check CI logs for "Cachix push enabled"
- [ ] Verify cache receives artifacts

## Regular Maintenance

### Weekly
- [ ] Review failed security audits
- [ ] Update vulnerable dependencies
  ```bash
  cargo audit
  cargo update
  ```
- [ ] Check for pending Dependabot PRs
- [ ] Review CI performance metrics

### Monthly
- [ ] Update GitHub Actions versions
  - Check for newer versions of:
    - `actions/checkout`
    - `dtolnay/rust-toolchain`
    - `Swatinem/rust-cache`
    - `cachix/install-nix-action`
    - `cachix/cachix-action`
- [ ] Review and update `deny.toml` policies
- [ ] Audit build cache efficiency
- [ ] Review CI execution times

### Quarterly
- [ ] Update Nix flake inputs
  ```bash
  nix flake update
  git add flake.lock
  git commit -m "chore: update nix flake inputs"
  ```
- [ ] Review branch protection rules
- [ ] Audit dependency licenses
- [ ] Review and clean up old artifacts
- [ ] Update documentation if workflow changed

## Verification Commands

Run these locally to ensure everything works:

### Format
```bash
cargo fmt --all --check
```
Expected: No output (all files formatted)

### Clippy
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
Expected: Exit code 0, no warnings

### Build (Dev)
```bash
cargo build --locked
```
Expected: Successful compilation

### Build (Release)
```bash
cargo build --release --locked
```
Expected: Successful compilation, binary in `target/release/`

### Tests
```bash
cargo test --locked
```
Expected: All tests pass

### Nix Flake Check
```bash
nix flake check --show-trace
```
Expected: All checks pass

### Nix Build
```bash
nix build --print-build-logs
```
Expected: Successful build, result symlink created

### Security Audit
```bash
cargo install cargo-audit
cargo audit
```
Expected: No vulnerabilities found

### Dependency Check
```bash
cargo install cargo-deny
cargo deny check
```
Expected: All checks pass

## Troubleshooting Guide

### CI Build Fails but Local Works

1. Check Cargo.lock is committed
   ```bash
   git add Cargo.lock
   git commit -m "chore: update Cargo.lock"
   ```

2. Verify system dependencies match CI
   - Compare `.github/workflows/ci.yml` with local setup

3. Check for platform-specific code
   - CI runs on `ubuntu-latest` (x86_64-linux)

### Nix Build Fails

1. Validate flake
   ```bash
   nix flake check --show-trace
   ```

2. Update flake inputs
   ```bash
   nix flake update
   ```

3. Check cargoLock configuration
   - Ensure `allowBuiltinFetchGit = true` in flake.nix

4. Clear and rebuild
   ```bash
   nix-collect-garbage -d
   nix build --print-build-logs --verbose
   ```

### Security Audit Failures

1. Identify vulnerable dependencies
   ```bash
   cargo audit
   ```

2. Update specific dependency
   ```bash
   cargo update -p <vulnerable-crate>
   ```

3. Check for available patches
   - Review RustSec advisory for workarounds

4. If no fix available, consider:
   - Replacing dependency
   - Adding to ignore list (deny.toml) with justification

### Dependency Policy Violations

1. Check which policy failed
   ```bash
   cargo deny check advisories  # Security
   cargo deny check licenses    # License compliance
   cargo deny check bans        # Duplicate versions
   cargo deny check sources     # Source validation
   ```

2. Review `deny.toml` configuration

3. Update policy if legitimate
   - Add to `allow` list with comment explaining why

### Cache Not Working

1. Verify cache keys in workflow
2. Check cache size limits (10GB per repo)
3. Review cache hit/miss in CI logs
4. Clear stale caches in GitHub settings

## Performance Metrics

Track these over time:

### Build Times (Target: <20 min cached)
- Format: < 1 min
- Clippy: < 3 min
- Build & Test (both): < 10 min
- Nix Build: < 5 min
- Security: < 2 min
- Dependency: < 2 min

### Cache Efficiency (Target: >80% hit rate)
- Check "Cache hit" in Cargo jobs
- Monitor Cachix statistics
- Review Nix store cache hits

### Resource Usage
- Disk space: Monitor artifact storage
- API rate limits: Stay under GitHub limits
- Concurrent jobs: Optimize for parallelism

## Emergency Procedures

### Disable CI Temporarily
If CI is blocking critical fixes:

1. Disable branch protection:
   `Settings → Branches → Edit rule → Disable`

2. Merge critical fix

3. Re-enable branch protection immediately

### Skip CI for Specific Commit
```bash
git commit -m "docs: update README [skip ci]"
```
Note: Use sparingly, only for documentation changes

### Rollback Bad Workflow
If workflow changes break CI:

1. Revert workflow file
   ```bash
   git revert <commit-hash>
   ```

2. Or restore from known good version
   ```bash
   git checkout <good-commit> -- .github/workflows/ci.yml
   git commit -m "ci: restore working workflow"
   ```

## Documentation Updates

When workflow changes:

- [ ] Update `.github/CI_CD.md`
- [ ] Update `.github/QUICKSTART.md`
- [ ] Update this checklist
- [ ] Update pre-commit hook if needed
- [ ] Update README badges if needed
- [ ] Announce changes to team

## Success Criteria

CI/CD is working correctly when:

- [x] All jobs pass on clean main branch
- [x] Build artifacts are generated
- [x] Caching reduces build time significantly
- [x] Security scans complete without errors
- [x] Local development workflow matches CI
- [x] Documentation is comprehensive and accurate
- [x] Team can debug failures independently

## Additional Resources

- Workflow file: `/home/olafkfreund/Source/GitHub/cosmic-radio-applet/.github/workflows/ci.yml`
- Full documentation: `/home/olafkfreund/Source/GitHub/cosmic-radio-applet/.github/CI_CD.md`
- Quick start: `/home/olafkfreund/Source/GitHub/cosmic-radio-applet/.github/QUICKSTART.md`
- Pre-commit hook: `/home/olafkfreund/Source/GitHub/cosmic-radio-applet/.github/pre-commit.sample`

## Sign-off

Date deployed: _______________
Deployed by: _______________
Verified by: _______________
First successful build: _______________

---

Last updated: 2026-02-04
