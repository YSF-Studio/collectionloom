#!/bin/bash
# Push collectionloom repo with repo-specific README swap
# Usage: ./scripts/push-all.sh "commit message"

set -e

MSG="${1:-update}"
BASE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$BASE"

# Save the real root README.md
cp README.md README-monorepo.md

echo "=== Pushing to origin (collectionloom) ==="
cp README-collectionloom.md README.md
git add README.md
git commit --amend --no-edit --allow-empty 2>/dev/null || true
git push origin main

# Restore root README
cp README-monorepo.md README.md
git add README.md
git commit --amend --no-edit --allow-empty 2>/dev/null || true
rm README-monorepo.md

echo "=== Done! ==="
