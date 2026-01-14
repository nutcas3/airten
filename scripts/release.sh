#!/bin/bash
set -euo pipefail

VERSION=${1:-}

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

echo "=== Releasing AirTen v$VERSION ==="

# Verify we're on main branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo "Error: Must be on main branch to release"
    exit 1
fi

# Verify working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Working directory is not clean"
    exit 1
fi

# Run tests
echo "Running tests..."
cargo test --workspace --all-features

# Run clippy
echo "Running clippy..."
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Update version in all Cargo.toml files
echo "Updating version to $VERSION..."
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak

# Update CHANGELOG
echo "Please update CHANGELOG.md with release notes for v$VERSION"
echo "Press Enter to continue..."
read -r

# Commit version bump
git add -A
git commit -m "chore: bump version to $VERSION"

# Create tag
git tag -a "v$VERSION" -m "Release v$VERSION"

echo "=== Release Prepared ==="
echo "To publish:"
echo "  git push origin main"
echo "  git push origin v$VERSION"
