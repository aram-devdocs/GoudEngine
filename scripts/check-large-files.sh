#!/usr/bin/env bash
# Fail if any tracked (or about-to-be-tracked) non-image file exceeds the size
# budget. Large binary blobs bloat clones and should use Git LFS or be ignored.
#
# Mirrors the check formerly inlined in the pre-push hook so it can run from the
# canonical verify pipeline and CI alike.
set -uo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$REPO_ROOT" || exit 1

MAX_BYTES=$((1024 * 1024)) # 1 MiB

# Candidate files over budget, excluding build output and images (images are
# expected to be large and are handled separately by the LFS policy).
#
# 3D model assets (glb/gltf/fbx) are also excluded pending the Git LFS migration
# tracked as a follow-up issue — the existing example assets predate this check
# and moving them rewrites history. Remove these exclusions once LFS is in place.
LARGE_FILES="$(find . -type f -size +1M \
  ! -path './.git/*' ! -path './target/*' ! -path '*/bin/*' ! -path '*/obj/*' \
  ! -path '*/node_modules/*' \
  ! -name '*.png' ! -name '*.jpg' ! -name '*.jpeg' ! -name '*.gif' \
  ! -name '*.glb' ! -name '*.gltf' ! -name '*.fbx' ! -name '*.bin' 2>/dev/null)"

[ -z "$LARGE_FILES" ] && exit 0

# Keep only files that git actually tracks (not gitignored).
TRACKED_LARGE="$(echo "$LARGE_FILES" | while IFS= read -r f; do
  [ -z "$f" ] && continue
  git check-ignore -q "$f" 2>/dev/null || echo "$f"
done)"

if [ -n "$TRACKED_LARGE" ]; then
  echo "Error: large tracked files detected (> $((MAX_BYTES / 1024)) KiB):"
  echo "$TRACKED_LARGE" | sed 's/^/  /'
  echo "Use Git LFS for binary assets or add the path to .gitignore."
  exit 1
fi

exit 0
