#!/usr/bin/env bash
# Reproduce issue #1486's captured draft failure offline (no Docker, release or Rust build).
# 1. Download the draft session artifact of run 41191429608 and extract the replay:
#      gh run download 41191429608 --repo link-assistant/formal-ai --pattern 'hive-mind-draft-session-*' --dir /tmp/art-1486
#      node scripts/extract-draft-replay.mjs --session /tmp/art-1486/<artifact>/ --issue 1486 --run 41191429608 \
#        --out data/benchmarks/fixtures/draft-replays/issue-1486-read-policy.json
# 2. Run the replay against the captured source (fails with ReadPolicyRefused) and against HEAD (passes).
set -euo pipefail
repo="$(git rev-parse --show-toplevel)"
source_commit="${1:-edf219e604cb9c244abaaed3115ce2fc1277e427}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
git -C "$repo" archive "$source_commit" js data scripts experiments/js_dogfood package.json | tar -x -C "$work"
mkdir -p "$work/rust/tests/web" "$work/data/benchmarks/fixtures/draft-replays"
cp "$repo/scripts/lib/draft-replay.mjs" "$work/scripts/lib/"
cp "$repo/data/benchmarks/focused-test-map.json" "$work/data/benchmarks/"
cp "$repo"/data/benchmarks/fixtures/draft-replays/*.json "$work/data/benchmarks/fixtures/draft-replays/"
# Sources before #1560 answered a work item with a JSON object instead of a report record.
sed "s#^import { workItemRecordOf } from .*#const workItemRecordOf = plan => { try { return JSON.parse(plan.answer); } catch { return null; } };#" \
  "$repo/rust/tests/web/draft-request-replay.test.mjs" > "$work/rust/tests/web/draft-request-replay.test.mjs"
ln -s "$repo/node_modules" "$work/node_modules"
git -C "$work" init -q && git -C "$work" add -A
echo "== captured source $source_commit (expected: fails with ReadPolicyRefused)"
(cd "$work" && node --test --test-timeout=120000 rust/tests/web/draft-request-replay.test.mjs) && echo "UNEXPECTED PASS" || echo "failed as captured"
echo "== HEAD (expected: passes)"
(cd "$repo" && node --test --test-timeout=120000 rust/tests/web/draft-request-replay.test.mjs)
