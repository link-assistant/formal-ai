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

command_name="$(matrix_client_field "$CLIENT" command)" \
  || matrix_fail "$CLIENT has no command in the seed registry"

matrix_note "installing $CLIENT via $installer ($spec), command '$command_name'"

case "$installer" in
  npm)
    if [ "${MATRIX_ISOLATED_NPM:-0}" = 1 ]; then
      # One prefix per client. In CI each leg is its own runner and installs one
      # CLI, so a shared global tree is fine; on a single machine running every
      # leg it is not. Installing all of them into one bun global tree hoists a
      # single copy of each transitive dependency, and the grok leg died on
      # `TypeError: ansiStyles.color.ansi is not a function` — grok's `slice-ansi`
      # resolving to another client's `ansi-styles` major. That is a packaging
      # accident of the host, not a defect in the client or in our server, and a
      # leg must not report it as either. `run_matrix.sh` puts these prefixes on
      # PATH ahead of the shared one.
      prefix="$(matrix_client_prefix "$CLIENT")"
      mkdir -p "$prefix"
      npm install --silent --prefix "$prefix" "$spec" \
        || matrix_fail "npm install --prefix $prefix $spec failed"
      installed="$prefix/node_modules/.bin/$command_name"
    else
      # bun installs onto $HOME/.bun/bin, which the workflow adds to PATH.
      bun add -g "$spec" || matrix_fail "bun add -g $spec failed"
      installed="${BUN_INSTALL:-$HOME/.bun}/bin/$command_name"
    fi
    ;;
  npm-native)
    # A package with a native addon it builds at install time. `bun add -g`
    # cannot install these here for two independent reasons: it skips lifecycle
    # scripts unless the dependency is trusted, and even with `--trust` it
    # builds under bun's own Node 20, where node-gyp 13 dies with
    # `webidl.util.markAsUncloneable is not a function`. Left half-installed,
    # t3 starts, migrates its database and *then* exits 1 with
    # `NodePtyModuleLoadError` — a leg failure that looks like our server.
    #
    # t3 also declares `engines.node ^22.16 || ^23.11 || >=24.10`, so the
    # version check below is the client's own requirement, not our preference.
    major="$(node -p 'process.versions.node.split(".")[0]' 2> /dev/null || echo 0)"
    [ "${major:-0}" -ge 22 ] \
      || matrix_fail "$CLIENT needs Node >= 22 (its own engines field); this host has $(node -v 2> /dev/null || echo 'no node')"
    npm install -g "$spec" || matrix_fail "npm install -g $spec failed"
    # Pin the interpreter into the entry point, not merely into the install.
    # The global bin is a `#!/usr/bin/env node` script, so a leg that starts it
    # with a different Node first on PATH loads a native addon built for another
    # ABI — a leftover Node 20 is exactly how t3 came to exit before listening,
    # after the install itself had succeeded under Node 22.
    shim="${MATRIX_SHIM_DIR:-$HOME/.local/bin}"
    mkdir -p "$shim"
    printf '#!/usr/bin/env bash\nexec "%s" "%s" "$@"\n' \
      "$(command -v node)" "$(npm prefix -g)/bin/$command_name" > "$shim/$command_name"
    chmod +x "$shim/$command_name"
    installed="$shim/$command_name"
    ;;
  pipx)
    python3 -m pip install --user pipx > /dev/null 2>&1 || true
    # aider-chat declares `Requires-Python >=3.10,<3.13`, so on a host whose
    # default python is newer, pip resolves *backwards* to aider 0.16.0 (2024)
    # instead of failing — a leg that installs a two-year-old CLI under the name
    # of a pinned modern one is worse than no leg. Pin the interpreter too.
    interpreter=""
    for candidate in python3.12 python3.11 python3.10; do
      command -v "$candidate" > /dev/null 2>&1 && {
        interpreter="$candidate"
        break
      }
    done
    [ -n "$interpreter" ] \
      || matrix_fail "$CLIENT needs python 3.10-3.12; none found (see aider's Requires-Python)"
    pipx install --python "$interpreter" "$spec" \
      || matrix_fail "pipx install --python $interpreter $spec failed"
    installed="${PIPX_BIN_DIR:-$HOME/.local/bin}/$command_name"
    ;;
  tarball)
    # The VS Code extension leg needs a real editor binary to host it, and the
    # `code` package lives in Microsoft's apt repository, which a stock runner
    # does not have — `apt-get install code` there fails with "no installation
    # candidate" and takes root to even try. The vendor tarball is pinned by
    # version, needs no root, and is the same bytes on every host.
    dest="${MATRIX_VSCODE_DIR:-$HOME/.formal-ai-matrix/vscode}"
    if [ ! -x "$dest/bin/code" ]; then
      mkdir -p "$dest"
      curl -fsSL "https://update.code.visualstudio.com/$spec/linux-x64/stable" \
        -o "$dest/code.tar.gz" \
        || matrix_fail "downloading VS Code $spec failed"
      tar -xzf "$dest/code.tar.gz" -C "$dest" --strip-components=1 \
        || matrix_fail "unpacking VS Code $spec failed"
    fi
    shim="${MATRIX_SHIM_DIR:-$HOME/.local/bin}"
    mkdir -p "$shim"
    printf '#!/usr/bin/env bash\nexec "%s" "$@"\n' "$dest/bin/code" > "$shim/code"
    chmod +x "$shim/code"
    installed="$shim/code"
    # An editor with no OpenCode extension is not the client this row names: the
    # leg would launch a bare editor and prove nothing about our server. The
    # extension is what talks to the base URL the wrapper configures.
    "$dest/bin/code" --install-extension sst-dev.opencode --force \
      --user-data-dir "$dest/user-data" --extensions-dir "$dest/extensions" \
      || matrix_fail "installing the sst-dev.opencode VS Code extension failed"
    ;;
  appimage)
    # OpenCode Desktop ships as an AppImage (see docs/case-studies/issue-762).
    # It is extracted rather than mounted because GitHub runners have no FUSE,
    # and the extracted `AppRun` is the real Electron binary the leg drives.
    dest="${MATRIX_APPIMAGE_DIR:-$HOME/.formal-ai-matrix/opencode-desktop}"
    mkdir -p "$dest"
    # The desktop AppImage rides the ordinary `v<version>` release tag — there is
    # no separate `desktop-v…` tag, and asking for one 404s.
    url="https://github.com/sst/opencode/releases/download/v$spec/opencode-desktop-linux-x86_64.AppImage"
    curl -fsSL "$url" -o "$dest/app.AppImage" \
      || matrix_fail "downloading $url failed"
    chmod +x "$dest/app.AppImage"
    ( cd "$dest" && ./app.AppImage --appimage-extract > /dev/null ) \
      || matrix_fail "extracting the OpenCode Desktop AppImage failed"
    # The seed registry launches this client by the name `opencode-desktop`, so
    # the extracted `AppRun` needs that name on PATH — without the shim the leg
    # fails as "command not found", which reads like a broken wrapper rather
    # than a missing install.
    shim="${MATRIX_SHIM_DIR:-$HOME/.local/bin}"
    mkdir -p "$shim"
    printf '#!/usr/bin/env bash\nexec "%s" "$@"\n' "$dest/squashfs-root/AppRun" \
      > "$shim/opencode-desktop"
    chmod +x "$shim/opencode-desktop"
    installed="$shim/opencode-desktop"
    echo "FORMAL_AI_OPENCODE_DESKTOP_BIN=$dest/squashfs-root/AppRun" >> "${GITHUB_ENV:-/dev/null}"
    ;;
  script)
    curl -fsSL "$spec" | bash || matrix_fail "vendor install script $spec failed"
    # A vendor script puts the binary where it likes, so this is the one kind
    # whose path has to be discovered. It is discovered *after* the install, and
    # under the client's own command name — cursor's script also drops an
    # `agent` alias next to `cursor-agent`, and picking that one up would have
    # recorded another client's command as this one's binary.
    installed="$(command -v "$command_name" 2> /dev/null)"
    ;;
  *)
    matrix_fail "unknown installer '$installer' for $CLIENT"
    ;;
esac

# Record what was installed so every leg drives *this* binary regardless of what
# else on the host answers to the same name — see `matrix_record_binary`.
matrix_record_binary "$CLIENT" "$installed"

matrix_pass "$CLIENT installed ($spec)"
