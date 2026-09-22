#!/usr/bin/env bash
# Bundle a relocatable CPython + instaloader into src-tauri/resources.
# Target triple: $1, $INSTALOADER_TARGET, or the host (uname).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/src-tauri/resources/instaloader-runtime"
PY_VERSION="3.12.14"
PY_TAG="20260901"

host_triple() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Darwin)
      case "$arch" in
        arm64|aarch64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *) echo "unsupported Darwin arch: $arch" >&2; return 1 ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-gnu" ;;
        aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
        *) echo "unsupported Linux arch: $arch" >&2; return 1 ;;
      esac
      ;;
    MINGW*|MSYS*|CYGWIN*)
      case "$arch" in
        x86_64|amd64|AMD64) echo "x86_64-pc-windows-msvc" ;;
        aarch64|arm64|ARM64) echo "aarch64-pc-windows-msvc" ;;
        *) echo "unsupported Windows arch: $arch" >&2; return 1 ;;
      esac
      ;;
    *)
      echo "unsupported OS: $os" >&2
      return 1
      ;;
  esac
}

find_python() {
  local dest="$1"
  if [[ -x "$dest/bin/python3" ]]; then
    echo "$dest/bin/python3"
  elif [[ -f "$dest/python.exe" ]]; then
    echo "$dest/python.exe"
  elif [[ -f "$dest/bin/python.exe" ]]; then
    echo "$dest/bin/python.exe"
  else
    return 1
  fi
}

write_launchers() {
  cat > "$DEST/instaloader" <<'EOF'
#!/bin/sh
ROOT="$(CDPATH= cd -- "$(dirname "$0")" && pwd)"
if [ -x "$ROOT/bin/python3" ]; then
  exec "$ROOT/bin/python3" -m instaloader "$@"
fi
if [ -f "$ROOT/python.exe" ]; then
  exec "$ROOT/python.exe" -m instaloader "$@"
fi
if [ -f "$ROOT/bin/python.exe" ]; then
  exec "$ROOT/bin/python.exe" -m instaloader "$@"
fi
echo "python not found in instaloader-runtime" >&2
exit 1
EOF
  chmod +x "$DEST/instaloader"

  cat > "$DEST/instaloader.cmd" <<'EOF'
@echo off
setlocal
set "ROOT=%~dp0"
if exist "%ROOT%python.exe" (
  "%ROOT%python.exe" -m instaloader %*
  exit /b %ERRORLEVEL%
)
if exist "%ROOT%bin\python.exe" (
  "%ROOT%bin\python.exe" -m instaloader %*
  exit /b %ERRORLEVEL%
)
echo python.exe not found in instaloader-runtime 1>&2
exit /b 1
EOF
}

TRIPLE="${1:-${INSTALOADER_TARGET:-}}"
if [[ -z "$TRIPLE" ]]; then
  TRIPLE="$(host_triple)"
fi

case "$TRIPLE" in
  aarch64-apple-darwin|x86_64-apple-darwin|x86_64-unknown-linux-gnu|aarch64-unknown-linux-gnu|x86_64-pc-windows-msvc|aarch64-pc-windows-msvc) ;;
  *)
    echo "unsupported target triple: $TRIPLE" >&2
    exit 1
    ;;
esac

if [[ -f "$DEST/.triple" ]] && [[ "$(tr -d '[:space:]' < "$DEST/.triple")" == "$TRIPLE" ]]; then
  if PYTHON="$(find_python "$DEST")" && "$PYTHON" -m instaloader --version >/dev/null 2>&1; then
    echo "bundled instaloader already present for $TRIPLE at $DEST"
    "$PYTHON" -m instaloader --version
    exit 0
  fi
fi

PY_URL="https://github.com/astral-sh/python-build-standalone/releases/download/${PY_TAG}/cpython-${PY_VERSION}+${PY_TAG}-${TRIPLE}-install_only_stripped.tar.gz"

echo "vendoring instaloader runtime for $TRIPLE into $DEST"
rm -rf "$DEST"
mkdir -p "$DEST"

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

curl -fsSL "$PY_URL" -o "$WORK/python.tgz"
tar -xzf "$WORK/python.tgz" -C "$WORK"
if [[ -d "$WORK/python" ]]; then
  # install_only archives unpack to a `python/` directory
  shopt -s dotglob
  mv "$WORK/python"/* "$DEST/"
  shopt -u dotglob
else
  echo "unexpected python archive layout" >&2
  ls -la "$WORK" >&2
  exit 1
fi

PYTHON="$(find_python "$DEST")"
"$PYTHON" -m pip install --upgrade pip
"$PYTHON" -m pip install --no-warn-script-location instaloader

write_launchers
printf '%s\n' "$TRIPLE" > "$DEST/.triple"
xattr -cr "$DEST" 2>/dev/null || true

echo -n "bundled "
HOST="$(host_triple || true)"
if [[ "$HOST" == "$TRIPLE" ]]; then
  "$PYTHON" -m instaloader --version
else
  echo "instaloader for $TRIPLE (not executed on host $HOST)"
fi
