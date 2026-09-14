#!/usr/bin/env bash
set -euo pipefail

root="${1:-tests/fixtures/coding-discovery/python-docs}"
mkdir -p "$root"
base="https://docs.python.org/3.12/library"
user_agent="formal-ai/${FORMAL_AI_VERSION:-coding-discovery} (https://github.com/link-assistant/formal-ai; fixture capture)"
pages=(functions stdtypes math itertools heapq collections)

for page in "${pages[@]}"; do
  curl --fail --silent --show-error --location --compressed --max-time 30 \
    --user-agent "$user_agent" "$base/$page.html" --output "$root/$page.html"
done

captured_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
manifest="$root/capture-manifest.lino"
{
  echo "capture_manifest"
  for page in "${pages[@]}"; do
    file="$page.html"
    sha256="$(shasum -a 256 "$root/$file" | awk '{print $1}')"
    bytes="$(wc -c <"$root/$file" | tr -d ' ')"
    echo "  capture \"$file\""
    echo "    url \"$base/$file\""
    echo "    fetched_at \"$captured_at\""
    echo "    sha256 \"$sha256\""
    echo "    bytes \"$bytes\""
    echo "    license \"PSF-2.0\""
  done
} >"$manifest"
