#!/usr/bin/env sh
# formal-ai universal installer (issue #554).
#
# One script installs every formal-ai interface from the GitHub Releases the
# project already publishes:
#
#   desktop   the Electron desktop app (downloads the matching release asset)
#   vscode    the VS Code extension (downloads the .vsix, runs `code --install-extension`)
#   cli       the `formal-ai` command-line tool (via `cargo install formal-ai`)
#   telegram  the Telegram bot (alias for `cli`: the bot ships inside the CLI)
#   all       desktop + vscode + cli (best effort; skips what the host can't do)
#
# Usage (run directly):
#   ./scripts/install.sh [target]
#
# Usage (curl | bash — the only supported VS Code install method until the
# extension is on the Marketplace, issue #554 R3):
#   curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- vscode
#   wget -qO- https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- vscode
#
# Configuration (environment variables, so the curl|sh form needs no args):
#   FORMAL_AI_INSTALL_TARGET    desktop | vscode | cli | telegram | all (default: desktop)
#   FORMAL_AI_INSTALL_VERSION   pin a release tag, e.g. v0.215.0 (default: latest)
#   FORMAL_AI_INSTALL_DIR       where to place downloaded desktop assets
#                               (default: $HOME/Downloads, else the current dir)
#   FORMAL_AI_SKIP_VERIFY       set to 1 to skip the SHA-256 checksum check
#
# The script is wrapped in main() and only invoked on the final line so a
# truncated download (the classic curl|sh hazard) never executes a partial body.
set -eu

REPO="link-assistant/formal-ai"
API_LATEST="https://api.github.com/repos/${REPO}/releases/latest"
RELEASES_URL="https://github.com/${REPO}/releases"

# --- small helpers ---------------------------------------------------------

log() { printf '%s\n' "formal-ai: $*" >&2; }
err() { printf '%s\n' "formal-ai: error: $*" >&2; }
die() { err "$*"; exit 1; }

have() { command -v "$1" >/dev/null 2>&1; }

usage() {
  cat >&2 <<'EOF'
formal-ai universal installer

Usage: install.sh [desktop|vscode|cli|telegram|all]

Targets:
  desktop   Download the desktop app release asset for this OS/arch.
  vscode    Download the .vsix and install it with `code --install-extension`.
  cli       Install the `formal-ai` CLI with `cargo install formal-ai`.
  telegram  Install the CLI that powers the Telegram bot (alias for `cli`).
  all       Install everything this machine can support (best effort).

Environment:
  FORMAL_AI_INSTALL_TARGET    target when none is passed on the command line
  FORMAL_AI_INSTALL_VERSION   pin a release tag (default: latest)
  FORMAL_AI_INSTALL_DIR       directory for downloaded desktop assets
  FORMAL_AI_SKIP_VERIFY=1     skip the SHA-256 checksum verification
EOF
}

# Download <url> to <dest> using curl or wget, whichever exists.
download() {
  url="$1"
  dest="$2"
  if have curl; then
    curl -fsSL --proto '=https' "$url" -o "$dest"
  elif have wget; then
    wget -qO "$dest" "$url"
  else
    die "neither curl nor wget is available to download $url"
  fi
}

# Print <url> body to stdout.
fetch() {
  url="$1"
  if have curl; then
    curl -fsSL --proto '=https' "$url"
  elif have wget; then
    wget -qO- "$url"
  else
    die "neither curl nor wget is available to fetch $url"
  fi
}

# --- OS / arch detection ---------------------------------------------------

detect_os() {
  os="$(uname -s 2>/dev/null || echo unknown)"
  case "$os" in
    Darwin) echo macos ;;
    Linux) echo linux ;;
    MINGW* | MSYS* | CYGWIN* | Windows_NT) echo windows ;;
    *) echo unknown ;;
  esac
}

detect_arch() {
  arch="$(uname -m 2>/dev/null || echo unknown)"
  case "$arch" in
    x86_64 | amd64) echo x64 ;;
    arm64 | aarch64) echo arm64 ;;
    *) echo unknown ;;
  esac
}

# --- release resolution ----------------------------------------------------

# Echo the resolved release JSON. Honors FORMAL_AI_INSTALL_VERSION.
release_json() {
  if [ -n "${FORMAL_AI_INSTALL_VERSION:-}" ]; then
    fetch "https://api.github.com/repos/${REPO}/releases/tags/${FORMAL_AI_INSTALL_VERSION}"
  else
    fetch "$API_LATEST"
  fi
}

# Extract the semver (x.y.z) from a release JSON blob read on stdin.
release_version() {
  sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -n1 \
    | sed -n 's/^[A-Za-z-]*v\{0,1\}\([0-9][0-9.]*\([-+][0-9A-Za-z.-]*\)\{0,1\}\).*/\1/p'
}

# Find a browser_download_url for an asset whose name matches the grep regex,
# reading the release JSON from stdin.
asset_url_matching() {
  pattern="$1"
  tr ',' '\n' \
    | grep -o '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*"' \
    | sed -n 's/.*"\(https[^"]*\)"/\1/p' \
    | grep -E "$pattern" \
    | head -n1
}

# --- checksum verification -------------------------------------------------

# Verify <file> against SHA256SUMS.txt (resolved from <release-json-file>).
# No-op when FORMAL_AI_SKIP_VERIFY=1 or the tools/asset are unavailable.
verify_checksum() {
  file="$1"
  json_file="$2"
  [ "${FORMAL_AI_SKIP_VERIFY:-0}" = "1" ] && { log "skipping checksum verification"; return 0; }

  sums_url="$(asset_url_matching 'SHA256SUMS\.txt$' < "$json_file" || true)"
  [ -n "$sums_url" ] || { log "no SHA256SUMS.txt in release; skipping verification"; return 0; }

  hasher=""
  if have sha256sum; then hasher="sha256sum";
  elif have shasum; then hasher="shasum -a 256";
  else log "no sha256 tool found; skipping verification"; return 0; fi

  sums="$(fetch "$sums_url" || true)"
  [ -n "$sums" ] || { log "could not download SHA256SUMS.txt; skipping verification"; return 0; }

  base="$(basename "$file")"
  expected="$(printf '%s\n' "$sums" | sed -n "s/^\([a-fA-F0-9]\{64\}\)[[:space:]]*[*]\{0,1\}${base}\$/\1/p" | head -n1)"
  [ -n "$expected" ] || { log "no checksum line for $base; skipping verification"; return 0; }

  actual="$(eval "$hasher \"$file\"" | awk '{print $1}')"
  if [ "$actual" = "$expected" ]; then
    log "checksum OK for $base"
  else
    die "checksum MISMATCH for $base (expected $expected, got $actual)"
  fi
}

# --- destination directory -------------------------------------------------

resolve_install_dir() {
  if [ -n "${FORMAL_AI_INSTALL_DIR:-}" ]; then
    echo "$FORMAL_AI_INSTALL_DIR"
  elif [ -d "${HOME:-}/Downloads" ]; then
    echo "${HOME}/Downloads"
  else
    echo "."
  fi
}

# --- targets ---------------------------------------------------------------

install_desktop() {
  json_file="$1"
  os="$(detect_os)"
  arch="$(detect_arch)"
  [ "$os" = "unknown" ] && die "could not detect a supported OS for the desktop app"
  [ "$arch" = "unknown" ] && die "could not detect a supported CPU architecture"

  case "$os" in
    macos) pattern="formal-ai-desktop-macos-${arch}-[0-9].*\\.dmg$" ;;
    windows) pattern="formal-ai-desktop-windows-installer-${arch}-[0-9].*\\.exe$" ;;
    linux) pattern="formal-ai-desktop-linux-${arch}-[0-9].*\\.AppImage$" ;;
  esac

  url="$(asset_url_matching "$pattern" < "$json_file" || true)"
  [ -n "$url" ] || die "no desktop asset matching $pattern in the release. See $RELEASES_URL/latest"

  dir="$(resolve_install_dir)"
  mkdir -p "$dir"
  name="$(basename "$url")"
  dest="${dir}/${name}"
  log "downloading $name -> $dir"
  download "$url" "$dest"
  verify_checksum "$dest" "$json_file"

  case "$os" in
    linux)
      chmod +x "$dest" 2>/dev/null || true
      log "desktop AppImage saved to $dest"
      log "run it with: \"$dest\""
      ;;
    macos)
      log "desktop disk image saved to $dest"
      log "open it, drag 'formal-ai Desktop' to /Applications, then see the macOS"
      log "Gatekeeper notes at https://link-assistant.github.io/formal-ai/download/"
      ;;
    windows)
      log "desktop installer saved to $dest"
      log "run the installer to complete setup."
      ;;
  esac
}

install_vscode() {
  json_file="$1"
  version="$(release_version < "$json_file" || true)"
  url="$(asset_url_matching "formal-ai-vscode-.*\\.vsix$" < "$json_file" || true)"
  [ -n "$url" ] || die "no .vsix in the release yet. Build one with 'npm run vscode:package' or see $RELEASES_URL/latest"

  tmp="$(mktemp -d 2>/dev/null || echo "${TMPDIR:-/tmp}/formal-ai-vsix.$$")"
  mkdir -p "$tmp"
  name="$(basename "$url")"
  dest="${tmp}/${name}"
  log "downloading $name"
  download "$url" "$dest"
  verify_checksum "$dest" "$json_file"

  code_cli=""
  if have code; then code_cli="code";
  elif have code-insiders; then code_cli="code-insiders";
  elif have codium; then code_cli="codium";
  fi

  if [ -n "$code_cli" ]; then
    log "installing the extension with '$code_cli --install-extension'"
    "$code_cli" --install-extension "$dest"
    log "VS Code extension installed${version:+ (v$version)}. Reload VS Code to activate it."
  else
    log "the 'code' CLI was not found on PATH."
    log "the .vsix is saved at: $dest"
    log "install it from VS Code: Extensions view -> ... menu -> 'Install from VSIX...'"
    log "or enable the CLI: VS Code Command Palette -> 'Shell Command: Install code command in PATH'."
  fi
}

install_cli() {
  if have cargo; then
    log "installing the formal-ai CLI with 'cargo install formal-ai'"
    if [ -n "${FORMAL_AI_INSTALL_VERSION:-}" ]; then
      ver="$(printf '%s' "$FORMAL_AI_INSTALL_VERSION" | sed 's/^v//')"
      cargo install formal-ai --version "$ver" || die "cargo install failed"
    else
      cargo install formal-ai || die "cargo install failed"
    fi
    log "CLI installed. Try: formal-ai --help"
  else
    die "cargo is required to install the CLI. Install Rust from https://rustup.rs then re-run."
  fi
}

# The Telegram bot ships inside the CLI, so installing it is the `cli` step plus
# a bot-specific next-step hint. Kept as its own target so users following the
# Telegram landing page can run `... | sh -s -- telegram` without an error.
install_telegram() {
  install_cli "$@"
  log "Telegram bot ready. Create a token with @BotFather, then run:"
  log "  TELEGRAM_BOT_TOKEN=<token> formal-ai telegram"
}

# --- main ------------------------------------------------------------------

main() {
  target="${1:-${FORMAL_AI_INSTALL_TARGET:-desktop}}"
  case "$target" in
    -h | --help | help) usage; exit 0 ;;
  esac

  case "$target" in
    desktop | vscode | cli | telegram | all) : ;;
    *) usage; die "unknown target: $target" ;;
  esac

  log "resolving ${FORMAL_AI_INSTALL_VERSION:-latest} release of $REPO"
  json_file="$(mktemp 2>/dev/null || echo "${TMPDIR:-/tmp}/formal-ai-release.$$")"
  release_json > "$json_file" || die "could not query the GitHub Releases API"
  [ -s "$json_file" ] || die "empty release response from the GitHub Releases API"

  case "$target" in
    desktop) install_desktop "$json_file" ;;
    vscode) install_vscode "$json_file" ;;
    cli) install_cli "$json_file" ;;
    telegram) install_telegram "$json_file" ;;
    all)
      # Best effort: install what this host supports, never abort the whole run
      # because one optional interface is missing its toolchain.
      install_desktop "$json_file" || err "desktop step did not complete"
      install_vscode "$json_file" || err "vscode step did not complete"
      install_cli "$json_file" || err "cli step did not complete"
      ;;
  esac

  rm -f "$json_file" 2>/dev/null || true
  log "done."
}

main "$@"
