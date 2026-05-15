#!/usr/bin/env bash
# Synchronise the canonical Links Notation seed (`data/seed/*.lino`) into the
# static web demo (`src/web/seed/`). The web copy is a deploy artefact: the
# canonical source of truth is `data/seed/`, shared by the Rust solver, CLI,
# Telegram bot, HTTP server, and the browser worker.
#
# Run this script before serving the demo locally and as part of CI before the
# GitHub Pages artefact is uploaded.
#
# Usage:
#   scripts/sync-seed.sh         # copy data/seed/*.lino → src/web/seed/
#   scripts/sync-seed.sh --check # exit 1 if the two trees diverge

set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
SRC_DIR="$ROOT_DIR/data/seed"
DEST_DIR="$ROOT_DIR/src/web/seed"

mode="copy"
if [[ "${1-}" == "--check" ]]; then
  mode="check"
fi

if [[ ! -d "$SRC_DIR" ]]; then
  echo "sync-seed: source directory not found: $SRC_DIR" >&2
  exit 1
fi

mkdir -p "$DEST_DIR"

shopt -s nullglob
sources=("$SRC_DIR"/*.lino)
shopt -u nullglob

if [[ ${#sources[@]} -eq 0 ]]; then
  echo "sync-seed: no .lino files found in $SRC_DIR" >&2
  exit 1
fi

status=0

for src in "${sources[@]}"; do
  name=$(basename "$src")
  dest="$DEST_DIR/$name"
  if [[ "$mode" == "check" ]]; then
    if [[ ! -f "$dest" ]] || ! cmp -s "$src" "$dest"; then
      echo "sync-seed: out of sync — $name" >&2
      status=1
    fi
  else
    cp "$src" "$dest"
    echo "sync-seed: $name"
  fi
done

if [[ "$mode" == "check" ]]; then
  # Detect orphan files in the destination that no longer exist in source.
  shopt -s nullglob
  dests=("$DEST_DIR"/*.lino)
  shopt -u nullglob
  for dst in "${dests[@]}"; do
    name=$(basename "$dst")
    if [[ ! -f "$SRC_DIR/$name" ]]; then
      echo "sync-seed: orphan in destination — $name" >&2
      status=1
    fi
  done
fi

exit "$status"
