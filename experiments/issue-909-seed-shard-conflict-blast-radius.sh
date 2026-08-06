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

total=$(ls data/seed/closure-generated-*.lino | wc -l | tr -d ' ')

echo
echo "== introduce one new unresolved token, at four points in the sort order =="
echo "   (sequential fill cascades forward from the insertion point, so its blast"
echo "    radius depends on where the token sorts; a digest's must not)"
echo
status=0

for slug in aaa_shard_probe mmm_shard_probe sss_shard_probe zzz_shard_probe; do
  # The probe must be a dangling *value* token, not a definition: close-total.py
  # defines the unresolved values it finds, so a token that defines itself under
  # `meanings` already resolves and would produce no work at all.
  cat >"$PROBE_FILE" <<LINO
meanings
  shard_blast_radius_holder
    defined-by intent
    intent $slug
LINO
  python3 scripts/close-total.py >/dev/null
  changed=$(git diff --name-only -- 'data/seed/closure-generated-*.lino' | wc -l | tr -d ' ')
  printf '   %-18s -> %2d / %s shards dirtied\n' "$slug" "$changed" "$total"

  if [ "$changed" -eq 0 ]; then
    echo "   FAIL: the probe changed nothing, so this run proved nothing. The probe"
    echo "         must reach close-total.py as an unresolved value (base_tokens())."
    status=1
  elif [ "$changed" -gt 1 ]; then
    echo "   FAIL: one new token dirtied $changed shards. Sharding is size-dependent"
    echo "         again; each block must be placed by a digest of its own slug."
    status=1
  fi
  git checkout -- 'data/seed/closure-generated-*.lino'
done

rm -f "$PROBE_FILE"

echo
if [ "$status" -ne 0 ]; then
  echo "FAIL: sharding is not content-addressed."
  exit 1
fi

echo "PASS: every new token is confined to exactly one shard, wherever it sorts."
echo
echo "   For comparison, the same four probes against the previous sequential fill"
echo "   (measured on commit 198b11cd, 11 shards) dirtied 11, 9, 2 and 1 shards"
echo "   respectively — a mid-alphabet token rewrote most of data/seed, which is"
echo "   why nearly every pull request conflicted there."
