#!/usr/bin/env bash
# Write GitHub Release notes for a tag.
# Prefers the matching CHANGELOG.md / CHANGELOG.zh.md section; otherwise git log.
set -euo pipefail

TAG="${1:-${GITHUB_REF_NAME:-}}"
OUT="${2:-release-notes.md}"

if [[ -z "$TAG" ]]; then
  echo "usage: $0 <tag> [output-file]" >&2
  exit 1
fi

VERSION="${TAG#v}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

extract_changelog() {
  local file="$1"
  [[ -f "$file" ]] || return 1
  awk -v ver="$VERSION" '
    BEGIN { heading = "^##[ \t]+v?" ver "([ \t].*)?$" }
    $0 ~ heading { found = 1 }
    found && printed && /^##[ \t]+/ && $0 !~ heading { exit }
    found { print; printed = 1 }
  ' "$file"
}

NOTES=""
for file in CHANGELOG.md CHANGELOG.zh.md; do
  SECTION="$(extract_changelog "$file" || true)"
  if [[ -n "${SECTION// }" ]]; then
    NOTES="$SECTION"
    echo "using $file for $VERSION" >&2
    break
  fi
done

if [[ -z "$NOTES" ]]; then
  echo "generating notes from git history for $TAG" >&2
  PREV="$(git describe --tags --abbrev=0 "${TAG}^" 2>/dev/null || true)"
  NOTES="$(
    echo "## ${TAG}"
    echo
    if [[ -n "$PREV" ]]; then
      echo "Changes since ${PREV}:"
      echo
      git log --pretty=format:'- %s' "${PREV}..${TAG}"
      echo
    else
      git log --pretty=format:'- %s' "${TAG}"
      echo
    fi
  )"
fi

printf '%s\n' "$NOTES" > "$OUT"
echo "wrote $OUT"
