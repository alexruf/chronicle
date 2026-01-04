#!/usr/bin/env bash
#
# bump-version.sh - Update Chronicle version in Cargo.toml
#
# Usage:
#   ./scripts/bump-version.sh [VERSION]
#
# If VERSION is not provided, prompts interactively for:
#   - major (0.x.y -> 1.0.0)
#   - minor (x.Y.z -> x.(Y+1).0)
#   - patch (x.y.Z -> x.y.(Z+1))
#   - custom (x.y.z)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current directory (should be project root when run as ./scripts/bump-version.sh)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

# Check if Cargo.toml exists
if [[ ! -f "$CARGO_TOML" ]]; then
    echo -e "${RED}Error: Cargo.toml not found at $CARGO_TOML${NC}"
    exit 1
fi

# Extract current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/')

if [[ -z "$CURRENT_VERSION" ]]; then
    echo -e "${RED}Error: Could not extract current version from Cargo.toml${NC}"
    exit 1
fi

echo -e "${BLUE}Current version: ${YELLOW}$CURRENT_VERSION${NC}"
echo

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Calculate suggested versions
NEXT_MAJOR="$((MAJOR + 1)).0.0"
NEXT_MINOR="$MAJOR.$((MINOR + 1)).0"
NEXT_PATCH="$MAJOR.$MINOR.$((PATCH + 1))"

# If version provided as argument, use it
if [[ $# -eq 1 ]]; then
    NEW_VERSION="$1"
else
    # Interactive prompt
    echo "Select version bump type:"
    echo "  1) major: $NEXT_MAJOR (breaking changes)"
    echo "  2) minor: $NEXT_MINOR (new features, backward compatible)"
    echo "  3) patch: $NEXT_PATCH (bug fixes)"
    echo "  4) custom (enter manually)"
    echo
    read -rp "Choice [1-4]: " choice

    case $choice in
        1)
            NEW_VERSION="$NEXT_MAJOR"
            ;;
        2)
            NEW_VERSION="$NEXT_MINOR"
            ;;
        3)
            NEW_VERSION="$NEXT_PATCH"
            ;;
        4)
            read -rp "Enter new version (e.g., 1.2.3): " NEW_VERSION
            ;;
        *)
            echo -e "${RED}Invalid choice${NC}"
            exit 1
            ;;
    esac
fi

# Validate version format (simple regex for x.y.z)
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Invalid version format '$NEW_VERSION'. Expected format: x.y.z${NC}"
    exit 1
fi

# Check if version is actually newer
if [[ "$NEW_VERSION" == "$CURRENT_VERSION" ]]; then
    echo -e "${YELLOW}Warning: New version is same as current version${NC}"
    read -rp "Continue anyway? [y/N]: " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo "Aborted"
        exit 0
    fi
fi

echo
echo -e "${BLUE}Updating version: ${YELLOW}$CURRENT_VERSION${NC} → ${GREEN}$NEW_VERSION${NC}"
echo

# Confirm before making changes
read -rp "Proceed with version update? [y/N]: " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "Aborted"
    exit 0
fi

# Update Cargo.toml
# Using sed with -i flag (note: macOS and Linux differ slightly)
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
else
    # Linux
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
fi

# Verify the change
NEW_VERSION_CHECK=$(grep '^version = ' "$CARGO_TOML" | head -n1 | sed 's/version = "\(.*\)"/\1/')
if [[ "$NEW_VERSION_CHECK" == "$NEW_VERSION" ]]; then
    echo -e "${GREEN}✓ Version updated successfully in Cargo.toml${NC}"
    echo
    echo "Next steps:"
    echo "  1. Review the change: git diff Cargo.toml"
    echo "  2. Update Cargo.lock: cargo build"
    echo "  3. Commit the change: git add Cargo.toml Cargo.lock && git commit -m 'chore: bump version to $NEW_VERSION'"
    echo "  4. Create release: ./scripts/release.sh $NEW_VERSION"
else
    echo -e "${RED}✗ Version update failed${NC}"
    exit 1
fi
