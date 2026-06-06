#!/usr/bin/env bash
# Launch CollectionLoom from a portable kit folder.
set -euo pipefail
cd "$(dirname "$0")"

if [[ "$OSTYPE" == darwin* ]]; then
  if [[ -d "CollectionLoom.app" ]]; then
    exec open -a "$(pwd)/CollectionLoom.app"
  fi
  echo "CollectionLoom.app not found in $(pwd)" >&2
  exit 1
fi

# Linux: prefer AppImage beside the kit root
shopt -s nullglob
appimages=( *.AppImage )
if (( ${#appimages[@]} > 0 )); then
  chmod +x "${appimages[0]}"
  exec "${appimages[0]}" "$@"
fi

if [[ -x "./collectionloom" ]]; then
  exec ./collectionloom "$@"
fi

echo "No CollectionLoom binary found in $(pwd)" >&2
exit 1
