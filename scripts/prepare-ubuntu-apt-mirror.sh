#!/usr/bin/env bash
# Select the exact official Ubuntu HTTPS alternate without changing signed sources.
# Shared by bounded APT retries and bounded Playwright system-dependency setup.
set -euo pipefail
[ "$#" -le 1 ] || { echo "at most one mirror-list path is supported" >&2; exit 2; }
mirror_list="${1:-${FORMAL_AI_APT_MIRROR_LIST:-/etc/apt/apt-mirrors.txt}}"
[[ "$mirror_list" =~ ^/[A-Za-z0-9_./-]+/apt-mirrors\.txt$ ]] &&
  [ -f "$mirror_list" ] && [ ! -L "$mirror_list" ] && [ -O "$mirror_list" ] || {
  echo "official Ubuntu mirror list is absent or not a regular owned path" >&2; exit 2;
}
physical="$(cd -P -- "$(dirname -- "$mirror_list")" && pwd)/apt-mirrors.txt"
[ "$physical" = "$mirror_list" ] && [ "$(wc -c < "$mirror_list")" -le 65536 ] || {
  echo "official Ubuntu mirror list path or size is unsupported" >&2; exit 2;
}
[ "$(tail -c 1 "$mirror_list" | wc -l)" -eq 1 ] || {
  echo "official Ubuntu mirror list must end with a newline" >&2; exit 2;
}
original="$(mktemp "${mirror_list}.before.XXXXXX")"
staged=""
trap 'rm -f -- "$original" "${staged:-}"' EXIT
cp -p -- "$mirror_list" "$original"
staged="$(mktemp "${mirror_list}.next.XXXXXX")"
cp -p -- "$original" "$staged"
# Match one exact URI token, preserving priorities, comments and all
# other hosts. APT still verifies the unchanged signed catalog/hashes.
awk '$1 == "http://azure.archive.ubuntu.com/ubuntu" || $1 == "http://azure.archive.ubuntu.com/ubuntu/" {
  sub(/http:\/\/azure\.archive\.ubuntu\.com\/ubuntu/, "https://archive.ubuntu.com/ubuntu")
} { print }' "$original" > "$staged"
[ ! -L "$mirror_list" ] && cmp -s -- "$mirror_list" "$original" || {
  echo "official Ubuntu mirror list changed during preparation" >&2; exit 2;
}
if ! cmp -s -- "$original" "$staged"; then
  mv -- "$staged" "$mirror_list"
  echo "Using official Ubuntu archive HTTPS alternate."
fi
