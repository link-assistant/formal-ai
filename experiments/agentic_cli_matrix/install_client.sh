#!/usr/bin/env bash
# Install one pinned client from `clients.lock` for the issue-#671 matrix.
#
#   experiments/agentic_cli_matrix/install_client.sh codex
#
# The installer kind and the exact version both come from the lockfile, never
# from this script, so bumping a client is a one-line lockfile commit and a CI
# leg installs exactly what a local reproduction installs.

source "$(cd "$(dirname "$0")" && pwd)/lib.sh"

CLIENT="${1:?usage: install_client.sh <client-id>}"
installer="$(matrix_lock_installer "$CLIENT")"
spec="$(matrix_lock_spec "$CLIENT")"
[ -n "$installer" ] || matrix_fail "$CLIENT is not pinned in $LOCKFILE"

matrix_note "installing $CLIENT via $installer ($spec)"

case "$installer" in
  npm)
    # bun installs onto $HOME/.bun/bin, which the workflow adds to PATH.
    bun add -g "$spec" || matrix_fail "bun add -g $spec failed"
    ;;
  pipx)
    python3 -m pip install --user pipx > /dev/null 2>&1 || true
    pipx install "$spec" || matrix_fail "pipx install $spec failed"
    ;;
  apt)
    # The VS Code extension leg needs a real editor binary to host it.
    if ! command -v "$spec" > /dev/null 2>&1; then
      sudo apt-get update -qq \
        && sudo apt-get install -y -qq "$spec" \
        || matrix_fail "apt-get install $spec failed"
    fi
    ;;
  appimage)
    # OpenCode Desktop ships as an AppImage (see docs/case-studies/issue-762).
    # It is extracted rather than mounted because GitHub runners have no FUSE,
    # and the extracted `AppRun` is the real Electron binary the leg drives.
    dest="${MATRIX_APPIMAGE_DIR:-$HOME/.formal-ai-matrix/opencode-desktop}"
    mkdir -p "$dest"
    url="https://github.com/sst/opencode/releases/download/desktop-v$spec/opencode-desktop-linux-x86_64.AppImage"
    curl -fsSL "$url" -o "$dest/app.AppImage" \
      || matrix_fail "downloading $url failed"
    chmod +x "$dest/app.AppImage"
    ( cd "$dest" && ./app.AppImage --appimage-extract > /dev/null ) \
      || matrix_fail "extracting the OpenCode Desktop AppImage failed"
    echo "FORMAL_AI_OPENCODE_DESKTOP_BIN=$dest/squashfs-root/AppRun" >> "${GITHUB_ENV:-/dev/null}"
    ;;
  script)
    curl -fsSL "$spec" | bash || matrix_fail "vendor install script $spec failed"
    ;;
  *)
    matrix_fail "unknown installer '$installer' for $CLIENT"
    ;;
esac

matrix_pass "$CLIENT installed ($spec)"
