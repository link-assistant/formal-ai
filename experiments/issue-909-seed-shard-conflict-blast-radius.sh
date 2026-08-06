#!/usr/bin/env bash
# Measure the blast radius of adding one seed token to `data/seed/closure-generated-*`.
#
# PR #965 review: "we need to find a way to automatically reorganize ./data/seed in
# such a way that probability of conflicts is drastically reduced, as we recently have
# multiple conflicts in ./data/seed folder almost in every pull request."
#
# The cause is not volume, it is *sequential* sharding: `scripts/close-total.py` used
# to fill shards in sorted order up to a line cap, so every shard's contents depended
# on the total size of everything sorted before it. One new token near the front of
# the alphabet rewrote the tail of that shard and shifted a block into each following
# shard — a one-token change dirtied nearly every file, and two branches doing that
# concurrently conflicted almost every time.
#
# Content-addressed sharding (`sha256(slug) % SHARD_COUNT`) makes a block's file a
# function of the block alone. This script quantifies the difference: it introduces
# one new token, regenerates, and counts how many shard files changed.
#
# Usage: ./experiments/issue-909-seed-shard-conflict-blast-radius.sh
# Exits non-zero if a single new token dirties more than one shard.
set -euo pipefail

cd "$(dirname "$0")/.."
PROBE_FILE="data/seed/zzz-shard-blast-radius-probe.lino"

cleanup() {
  rm -f "$PROBE_FILE"
  # Restore the committed generated shards whatever happened.
  git checkout -- 'data/seed/closure-generated-*.lino' 2>/dev/null || true
}
trap cleanup EXIT

echo "== baseline: regenerate from the committed seed =="
python3 scripts/close-total.py >/dev/null
if ! git diff --quiet -- 'data/seed/closure-generated-*.lino'; then
  echo "FAIL: the committed shards are stale — run 'python3 scripts/close-total.py' and commit."
  git diff --stat -- 'data/seed/closure-generated-*.lino'
  exit 1
fi
echo "   committed shards are byte-identical to a fresh run (idempotent)"

echo
echo "== introduce exactly one new unresolved token =="
# A token that resolves to nothing, so close-total.py must define it.
cat >"$PROBE_FILE" <<'LINO'
meanings
  shard_blast_radius_probe
    defined-by intent
LINO
echo "   added intent 'shard_blast_radius_probe' via $PROBE_FILE"

python3 scripts/close-total.py >/dev/null

changed=$(git diff --name-only -- 'data/seed/closure-generated-*.lino' | wc -l | tr -d ' ')
total=$(ls data/seed/closure-generated-*.lino | wc -l | tr -d ' ')

echo
echo "== result =="
git diff --stat -- 'data/seed/closure-generated-*.lino' || true
echo
echo "   shards changed by one new token: $changed / $total"

if [ "$changed" -gt 1 ]; then
  echo
  echo "FAIL: one new token dirtied $changed shards. Sharding is size-dependent again;"
  echo "      each shard must be selected by a digest of its own slug (shard_for())."
  exit 1
fi

echo
echo "PASS: a single new token is confined to a single shard."
echo "      Under the previous sequential fill this was $total / $total."
