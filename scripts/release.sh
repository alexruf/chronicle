#!/usr/bin/env bash
#
# release.sh - Create and push a release tag
#
# Usage:
#   ./scripts/release.sh [VERSION]
#
# This script:
#   1. Verifies Cargo.toml version matches the tag
#   2. Checks working directory is clean
#   3. Creates a git tag (v{VERSION})
#   4. Pushes the tag to origin
#   5. Triggers GitHub Actions release workflow

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

# Check if version provided
if [[ $# -ne 1 ]]; then
    echo -e "${RED}Error: Version argument required${NC}"
    echo "Usage: $0 VERSION"
    echo "Example: $0 0.2.0"
    exit 1
fi

VERSION="$1"

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Invalid version format '$VERSION'. Expected format: x.y.z${NC}"
    exit 1
fi

# Extract version from Cargo.toml
CARGO_VERSION=$(grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/')

if [[ -z "$CARGO_VERSION" ]]; then
    echo -e "${RED}Error: Could not extract version from Cargo.toml${NC}"
    exit 1
fi

# Verify version matches Cargo.toml
if [[ "$VERSION" != "$CARGO_VERSION" ]]; then
    echo -e "${RED}Error: Version mismatch${NC}"
    echo -e "  Provided:    ${YELLOW}$VERSION${NC}"
    echo -e "  Cargo.toml:  ${YELLOW}$CARGO_VERSION${NC}"
    echo
    echo "Run ./scripts/bump-version.sh first to update Cargo.toml"
    exit 1
fi

# Check if working directory is clean
if [[ -n $(git status --porcelain) ]]; then
    echo -e "${RED}Error: Working directory is not clean${NC}"
    echo "Commit or stash your changes before creating a release"
    echo
    git status --short
    exit 1
fi

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [[ "$CURRENT_BRANCH" != "main" ]]; then
    echo -e "${YELLOW}Warning: Not on main branch (currently on: $CURRENT_BRANCH)${NC}"
    read -rp "Continue anyway? [y/N]: " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo "Aborted"
        exit 0
    fi
fi

TAG="v$VERSION"

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${RED}Error: Tag $TAG already exists${NC}"
    exit 1
fi

echo -e "${BLUE}Creating release for version ${GREEN}$VERSION${NC}"
echo
echo "This will:"
echo "  1. Create git tag: $TAG"
echo "  2. Push tag to origin"
echo "  3. Trigger GitHub Actions release workflow"
echo
echo "The automated workflow will:"
echo "  - Build binaries for macOS, Linux, Windows"
echo "  - Generate changelog with git-cliff"
echo "  - Create GitHub release with artifacts"
echo "  - Update Homebrew formula"
echo

read -rp "Proceed with release? [y/N]: " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "Aborted"
    exit 0
fi

# Create annotated tag
echo
echo -e "${BLUE}Creating tag $TAG...${NC}"
git tag -a "$TAG" -m "Release $VERSION"

# Push tag to origin
echo -e "${BLUE}Pushing tag to origin...${NC}"
git push origin "$TAG"

echo
echo -e "${GREEN}✓ Release tag created and pushed successfully${NC}"
echo
echo "GitHub Actions workflow started:"
echo "  https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
echo
echo "Monitor release progress at:"
echo "  https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/releases"
