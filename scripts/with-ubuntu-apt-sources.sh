#!/usr/bin/env bash
# Scope browser dependency installation to the runner's signed Ubuntu sources.
# Other preinstalled repositories remain untouched, including their cached lists.
set -euo pipefail
[ "$#" -gt 0 ] || { echo "a browser installation command is required" >&2; exit 2; }
sources="${FORMAL_AI_UBUNTU_APT_SOURCES:-/etc/apt/sources.list.d/ubuntu.sources}"
real_sudo="${FORMAL_AI_UBUNTU_APT_SUDO:-/usr/bin/sudo}"
[[ "$sources" = /* ]] && [ -f "$sources" ] && [ ! -L "$sources" ] &&
  [ "$(wc -c < "$sources")" -le 65536 ] || {
  echo "Ubuntu sources must be a bounded regular non-symlink file" >&2; exit 2;
}
[[ "$real_sudo" = /* ]] && [ -f "$real_sudo" ] && [ -x "$real_sudo" ] || {
  echo "an absolute sudo executable is required" >&2; exit 2;
}
# Reuse the existing signed-source-preserving mirror preparation, once.
script_dir="$(cd -P -- "$(dirname -- "$0")" && pwd)"
"$real_sudo" bash "$script_dir/prepare-ubuntu-apt-mirror.sh"
work="$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-ubuntu-apt.XXXXXX")"
trap 'rm -rf -- "$work"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
case "$work" in *\"*|*\\*|*$'\n'*) echo "unsupported APT configuration path" >&2; exit 2 ;; esac
mkdir "$work/sources.list.d" "$work/bin"
cp -- "$sources" "$work/sources.list.d/ubuntu.sources"
: > "$work/empty.list"
cat > "$work/apt.conf" <<EOF
Dir::Etc::sourcelist "$work/empty.list";
Dir::Etc::sourceparts "$work/sources.list.d";
APT::Get::List-Cleanup "false";
EOF
# Playwright elevates just apt-get with sudo. Preserve this one scoped variable
# across that boundary; browser downloads still run as the original user.
# Do not use sudo -E, modify sudoers, or change system-wide APT configuration.
cat > "$work/bin/sudo" <<'EOF'
#!/usr/bin/env bash
exec "$FORMAL_AI_UBUNTU_APT_SUDO" --preserve-env=APT_CONFIG "$@"
EOF
chmod 700 "$work/bin/sudo"
export FORMAL_AI_UBUNTU_APT_SUDO="$real_sudo"
export APT_CONFIG="$work/apt.conf"
export PATH="$work/bin:$PATH"
echo "Installing browser dependencies with the unchanged signed Ubuntu sources only."
"$@"
