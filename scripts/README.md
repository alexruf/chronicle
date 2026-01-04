# Scripts

Development and release scripts for Chronicle.

## Version Management

### bump-version.sh

Updates the version in `Cargo.toml`.

**Usage:**
```bash
# Interactive mode (prompts for version type)
./scripts/bump-version.sh

# Direct version specification
./scripts/bump-version.sh 0.2.0
```

**What it does:**
- Reads current version from Cargo.toml
- Calculates suggested next versions (major/minor/patch)
- Prompts for confirmation
- Updates Cargo.toml with new version
- Does NOT create commits or tags

**Next steps after running:**
1. Review: `git diff Cargo.toml`
2. Update lockfile: `cargo build`
3. Commit: `git add Cargo.toml Cargo.lock && git commit -m 'chore: bump version to X.Y.Z'`
4. Release: `./scripts/release.sh X.Y.Z`

### release.sh

Creates a git tag and pushes it to trigger the GitHub Actions release workflow.

**Usage:**
```bash
./scripts/release.sh 0.2.0
```

**What it does:**
1. Validates version format
2. Checks Cargo.toml version matches the specified version
3. Ensures working directory is clean
4. Creates annotated git tag (vX.Y.Z)
5. Pushes tag to origin
6. Triggers GitHub Actions release workflow

**GitHub Actions will automatically:**
- Build binaries for macOS (Apple Silicon), Linux (x86_64), Windows (x86_64)
- Generate changelog using git-cliff
- Create GitHub release with artifacts and checksums
- Update CHANGELOG.md in the repository
- Update Homebrew formula in homebrew-tap

**Prerequisites:**
- Clean working directory (no uncommitted changes)
- Cargo.toml version must match the release version
- Typically run from main branch

## Complete Release Workflow

```bash
# 1. Bump version
./scripts/bump-version.sh 0.2.0

# 2. Build to update Cargo.lock
cargo build

# 3. Run tests
cargo test
cargo clippy
cargo fmt

# 4. Commit version bump
git add Cargo.toml Cargo.lock
git commit -m 'chore: bump version to 0.2.0'
git push

# 5. Create and push release tag
./scripts/release.sh 0.2.0

# 6. Monitor at:
#    - https://github.com/alexruf/chronicle/actions
#    - https://github.com/alexruf/chronicle/releases
```

## Development Notes

- Scripts use bash for portability (works on macOS and Linux)
- Version format must be semantic versioning (x.y.z)
- Scripts include color-coded output for readability
- All destructive operations require confirmation
- Scripts validate inputs before making changes
