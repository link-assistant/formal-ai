#!/usr/bin/env sh
# formal-ai universal installer (issue #554).
#
# One script installs every formal-ai interface from the GitHub Releases the
# project already publishes:
#
#   desktop   the Electron desktop app (downloads the matching release asset)
#   vscode    the VS Code extension (downloads the .vsix, runs `code --install-extension`)
#   cli       the `formal-ai` command-line tool (prebuilt release archive,
#             falls back to `cargo install formal-ai`; issue #1181)
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
#   FORMAL_AI_INSTALL_BIN       where to place the prebuilt CLI binary
#                               (default: $HOME/.cargo/bin when it exists, else $HOME/.local/bin)
#   FORMAL_AI_SKIP_VERIFY       set to 1 to skip the SHA-256 checksum check
#
# The script is wrapped in main() and only invoked on the final line so a
# truncated download (the classic curl|sh hazard) never executes a partial body.
#
# Issue #812 asked why this is the only script in the repository without
# `pipefail`: because it is the only one that runs under `#!/usr/bin/env sh`.
# `pipefail` is a bash/ksh extension, not POSIX, and `set -o pipefail` aborts
# dash -- the `/bin/sh` on Debian and Ubuntu -- which is exactly the shell a
# `curl … | sh` install lands in. Portability wins here; every pipeline below
# is written so its exit status comes from the rightmost command deliberately.
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
  cli       Install the `formal-ai` CLI from the prebuilt release archive
            (falls back to `cargo install formal-ai`).
  telegram  Install the CLI that powers the Telegram bot (alias for `cli`).
  all       Install everything this machine can support (best effort).

Environment:
  FORMAL_AI_INSTALL_TARGET    target when none is passed on the command line
  FORMAL_AI_INSTALL_VERSION   pin a release tag (default: latest)
  FORMAL_AI_INSTALL_DIR       directory for downloaded desktop assets
  FORMAL_AI_INSTALL_BIN       directory for the prebuilt CLI binary
  FORMAL_AI_SKIP_VERIFY=1     skip the SHA-256 checksum verification
EOF
}

# Download <url> to <dest> using curl or wget, whichever exists.
#
# Both retry: a release asset that stops arriving mid-transfer is a dropped
# connection, not a missing release, and curl calls that fatal unless told
# otherwise. `--retry` on its own does not cover the truncation (curl exit 18);
# `--retry-all-errors` does. wget needs no flag -- it retries twenty times by
# default.
download() {
  url="$1"
  dest="$2"
  if have curl; then
    curl -fsSL --proto '=https' --retry 3 --retry-delay 2 --retry-all-errors "$url" -o "$dest"
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
    curl -fsSL --proto '=https' --retry 3 --retry-delay 2 --retry-all-errors "$url"
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

# The host triple the `cli` job of the Desktop Release workflow publishes a
# prebuilt archive for (issue #1181). Linux uses the static musl builds, so
# one archive runs on any distro regardless of its glibc. Empty when this
# OS/arch has no prebuilt archive (the caller falls back to cargo install).
cli_target_triple() {
  os="$(detect_os)"
  arch="$(detect_arch)"
  case "$os:$arch" in
    macos:x64) echo "x86_64-apple-darwin" ;;
    macos:arm64) echo "aarch64-apple-darwin" ;;
    linux:x64) echo "x86_64-unknown-linux-musl" ;;
    linux:arm64) echo "aarch64-unknown-linux-musl" ;;
    windows:x64) echo "x86_64-pc-windows-msvc" ;;
    *) echo "" ;;
  esac
}

# Where the prebuilt binary goes. ~/.cargo/bin wins when it exists because it
# is already on PATH for every rustup user; otherwise the XDG default.
resolve_cli_bin_dir() {
  if [ -n "${FORMAL_AI_INSTALL_BIN:-}" ]; then
    echo "$FORMAL_AI_INSTALL_BIN"
  elif [ -d "${HOME:-}/.cargo/bin" ]; then
    echo "${HOME}/.cargo/bin"
  else
    echo "${HOME:-.}/.local/bin"
  fi
}

# Download and install the prebuilt CLI archive for this host. Returns
# non-zero -- without dying -- whenever the prebuilt path is unavailable or
# unusable, so install_cli can fall back to `cargo install`.
install_cli_prebuilt() {
  json_file="$1"
  triple="$(cli_target_triple)"
  [ -n "$triple" ] || { log "no prebuilt CLI archive for this OS/arch"; return 1; }

  exe=""
  case "$triple" in
    *-windows-*) ext="zip"; pattern="formal-ai-cli-${triple}\\.zip$"; exe=".exe" ;;
    *) ext="tar.gz"; pattern="formal-ai-cli-${triple}\\.tar\\.gz$" ;;
  esac

  url="$(asset_url_matching "$pattern" < "$json_file" || true)"
  [ -n "$url" ] || {
    log "no prebuilt CLI archive (formal-ai-cli-${triple}.${ext}) in this release"
    return 1
  }

  tmp="$(mktemp -d 2>/dev/null || echo "${TMPDIR:-/tmp}/formal-ai-cli.$$")"
  mkdir -p "$tmp"
  name="$(basename "$url")"
  archive="${tmp}/${name}"
  log "downloading $name"
  download "$url" "$archive" || { log "download failed"; return 1; }
  verify_checksum "$archive" "$json_file"

  if [ "$ext" = "tar.gz" ]; then
    tar -xzf "$archive" -C "$tmp" || { log "archive extraction failed"; return 1; }
  elif have unzip; then
    unzip -o "$archive" -d "$tmp" || { log "archive extraction failed"; return 1; }
  else
    # Git Bash ships no unzip; its bsdtar (Windows System32 tar) reads zip,
    # and if the available tar is GNU tar it fails cleanly here.
    tar -xf "$archive" -C "$tmp" || { log "archive extraction failed (no unzip)"; return 1; }
  fi

  src="${tmp}/formal-ai-cli-${triple}/formal-ai${exe}"
  [ -f "$src" ] || { log "the archive did not contain the expected binary"; return 1; }

  bin_dir="$(resolve_cli_bin_dir)"
  mkdir -p "$bin_dir" || { log "could not create $bin_dir"; return 1; }
  dest="${bin_dir}/formal-ai${exe}"
  cp "$src" "$dest" && chmod +x "$dest" 2>/dev/null || {
    log "could not install the binary into $bin_dir"
    return 1
  }
  rm -rf "$tmp" 2>/dev/null || true

  if ! "$dest" --version >/dev/null 2>&1; then
    log "the downloaded binary did not run on this machine"
    rm -f "$dest"
    return 1
  fi
  log "CLI installed at $dest"
  case ":${PATH}:" in
    *":${bin_dir}:"*) : ;;
    *) log "note: $bin_dir is not on your PATH; add it with:"
       log "  export PATH=\"$bin_dir:\$PATH\"" ;;
  esac
  log "Try: formal-ai --help"
}

install_cli() {
  # Issue #1181: prefer the prebuilt release archive -- no Rust toolchain, no
  # compile -- and only fall back to `cargo install` when there is no archive
  # for this host or it cannot be used.
  if install_cli_prebuilt "$1"; then
    return 0
  fi
  log "falling back to 'cargo install formal-ai'"
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
    die "no prebuilt CLI archive for this machine and cargo is unavailable. Install Rust from https://rustup.rs then re-run."
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
