#!/usr/bin/env bash
set -euo pipefail

# Release script for headwind-vscode
# Usage: ./tools/scripts/release-vscode.sh <version>
# Example: ./tools/scripts/release-vscode.sh 0.3.0

VERSION="${1:?Usage: release-vscode.sh <version>}"

# Strip leading 'v' if provided
VERSION="${VERSION#v}"

cd "$(git rev-parse --show-toplevel)"

CHANGELOG="apps/vscode-extension/CHANGELOG.md"
PKG="apps/vscode-extension/package.json"

# Extract release notes for this version from CHANGELOG.md
# Matches everything between "## [VERSION]" and the next "## [" heading
NOTES=$(sed -n "/^## \[$VERSION\]/,/^## \[/{/^## \[$VERSION\]/d;/^## \[/d;p;}" "$CHANGELOG")

if [ -z "$NOTES" ]; then
  echo "Warning: No changelog entry found for version $VERSION in $CHANGELOG"
  echo "The tag will be created without release notes."
  NOTES="Release v$VERSION"
fi

echo "--- Release notes for v$VERSION ---"
echo "$NOTES"
echo "-----------------------------------"

# Update version in package.json
npm pkg set version="$VERSION" --prefix apps/vscode-extension
echo "Updated $PKG to version $VERSION"

# Stage, commit, tag with changelog notes, push
git add "$PKG"
git commit -m "chore(vscode): release v$VERSION"
git tag -a "v$VERSION" -m "$(printf 'v%s\n\n%s' "$VERSION" "$NOTES")"
git push && git push origin "v$VERSION"

echo ""
echo "Done! Tag v$VERSION pushed — CI will publish automatically."
