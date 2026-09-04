#!/usr/bin/env bash
# Exercise the released-image -> candidate-image memory upgrade on one Docker
# named volume. CI supplies the candidate image built by the preceding step.

set -euo pipefail

PREVIOUS_IMAGE="${PREVIOUS_IMAGE:-ghcr.io/link-assistant/formal-ai:0.335.0}"
CANDIDATE_IMAGE="${CANDIDATE_IMAGE:-formal-ai:pr-check}"
MEMORY_PATH=/home/box/.formal-ai/memory.lino
BACKUP_PATH=/home/box/.formal-ai/rollback.lino
RECEIPT_PATH=/home/box/.formal-ai/upgrade-receipt.json
suffix="${GITHUB_RUN_ID:-local}-$$"
volume="formal-ai-memory-upgrade-$suffix"
server="formal-ai-memory-upgrade-server-$suffix"
workdir="$(mktemp -d)"

cleanup() {
  if docker inspect "$server" >/dev/null 2>&1; then
    docker inspect --format \
      'candidate server: status={{.State.Status}} exit={{.State.ExitCode}} oom={{.State.OOMKilled}} error={{.State.Error}}' \
      "$server" >&2 || true
    docker logs "$server" >&2 || true
  fi
  docker rm -f "$server" >/dev/null 2>&1 || true
  docker volume rm "$volume" >/dev/null 2>&1 || true
  rm -rf "$workdir"
}
trap cleanup EXIT

fail() {
  echo "memory upgrade E2E failed: $*" >&2
  exit 1
}

run_image() {
  local image="$1"
  shift
  docker run --rm --privileged -i \
    -e DIND_SKIP_DAEMON=1 \
    -v "$volume:/home/box/.formal-ai" \
    "$image" "$@"
}

container_sha() {
  run_image "$1" sha256sum "$2" \
    | sed -n 's/^\([0-9a-f]\{64\}\)  .*/\1/p' \
    | tail -n 1
}

assert_json_contract() {
  local file="$1"
  local contract="$2"
  node - "$file" "$contract" <<'NODE'
const fs = require('fs');
const [file, contract] = process.argv.slice(2);
const value = JSON.parse(fs.readFileSync(file, 'utf8'));
const valid = contract === 'status'
  ? value.detected_schema_version === 1
    && value.target_schema_version === 2
    && value.compatible === true
    && value.migration_required === true
    && value.migration_state === 'upgrade_required'
  : contract === 'receipt'
    ? value.changed === true
      && value.from_schema_version === 1
      && value.to_schema_version === 2
      && value.migration_id === 'demo_memory_v1_to_v2'
      && value.rollback_supported === true
      && value.event_count === 2
    : contract === 'health'
      && value.memory.schema_version === 2
      && value.memory.compatible === true
      && value.memory.migration_required === false
      && value.memory.migration_state === 'ready';
if (!valid) {
  console.error(`unexpected ${contract} JSON: ${JSON.stringify(value)}`);
  process.exit(1);
}
NODE
}

if ! docker image inspect "$PREVIOUS_IMAGE" >/dev/null 2>&1; then
  docker pull "$PREVIOUS_IMAGE"
fi
docker volume create "$volume" >/dev/null
# The dind image's entrypoint executes application commands as `box`. Seed the
# otherwise root-owned named volume with the same ownership before either
# released or candidate binary opens it.
docker run --rm \
  -v "$volume:/home/box/.formal-ai" \
  --entrypoint chown "$CANDIDATE_IMAGE" \
  -R box:box /home/box/.formal-ai

cat >"$workdir/released-memory.lino" <<'EOF'
demo_memory
  event "released-user"
    role "user"
    content "remember the amber migration canary"
  event "released-assistant"
    role "assistant"
    content "stored by the released container"
    evidence "release-session|migration-canary"
EOF

# The released binary itself persists the schema-1 events into the named
# volume. An extension field is then injected in the old container to model
# metadata from another producer which this old parser ignores and would have
# discarded on its next write.
run_image "$PREVIOUS_IMAGE" formal-ai memory import \
  --path - --into "$MEMORY_PATH" \
  <"$workdir/released-memory.lino"
run_image "$PREVIOUS_IMAGE" bash -c \
  "sed -i '/content \"remember the amber migration canary\"/a\    futureField \"preserve this extension\"' '$MEMORY_PATH' && sync"
run_image "$PREVIOUS_IMAGE" formal-ai memory show --path "$MEMORY_PATH" \
  | tee "$workdir/released-show.txt"
grep -Fq 'remember the amber migration canary' "$workdir/released-show.txt" \
  || fail "released container did not persist the fixture"
original_sha="$(container_sha "$CANDIDATE_IMAGE" "$MEMORY_PATH")"

# Preflight must be machine-readable and byte-side-effect-free.
run_image "$CANDIDATE_IMAGE" formal-ai memory upgrade-status \
  --path "$MEMORY_PATH" --format json >"$workdir/status.json"
assert_json_contract "$workdir/status.json" status
after_preflight_sha="$(container_sha "$CANDIDATE_IMAGE" "$MEMORY_PATH")"
[[ "$after_preflight_sha" == "$original_sha" ]] \
  || fail "preflight modified released memory"

run_image "$CANDIDATE_IMAGE" formal-ai memory migrate \
  --path "$MEMORY_PATH" --backup "$BACKUP_PATH" \
  --receipt "$RECEIPT_PATH" --format json >"$workdir/receipt.json"
assert_json_contract "$workdir/receipt.json" receipt
backup_sha="$(container_sha "$CANDIDATE_IMAGE" "$BACKUP_PATH")"
[[ "$backup_sha" == "$original_sha" ]] || fail "rollback backup is not byte-exact"

# Candidate CLI paths must load, query, and export the upgraded volume without
# dropping released identifiers, ordering, or unknown event metadata.
run_image "$CANDIDATE_IMAGE" formal-ai memory query \
  --path "$MEMORY_PATH" --prompt 'recall amber migration canary' \
  | tee "$workdir/query.txt"
grep -Fq 'amber migration canary' "$workdir/query.txt" \
  || fail "candidate query did not recall released data"
run_image "$CANDIDATE_IMAGE" formal-ai memory export \
  --from "$MEMORY_PATH" --path - --events-only >"$workdir/exported.lino"
grep -Fq 'schema_version "2"' "$workdir/exported.lino" \
  || fail "candidate export omitted the target schema"
grep -Fq 'futureField "preserve this extension"' "$workdir/exported.lino" \
  || fail "candidate export dropped unknown metadata"
grep -Fq 'evidence "release-session|migration-canary"' "$workdir/exported.lino" \
  || fail "candidate export dropped evidence"
first_line="$(grep -n 'event "released-user"' "$workdir/exported.lino" | cut -d: -f1)"
second_line="$(grep -n 'event "released-assistant"' "$workdir/exported.lino" | cut -d: -f1)"
[[ "$first_line" -lt "$second_line" ]] || fail "candidate export changed event order"

# The real candidate server opens the same upgraded named volume and reports
# the compatibility contract from /health.
docker run --rm -d --privileged --name "$server" \
  -p 127.0.0.1::8080 \
  -e DIND_SKIP_DAEMON=1 \
  -e FORMAL_AI_MEMORY_PATH="$MEMORY_PATH" \
  -e FORMAL_AI_DREAMING=0 \
  -v "$volume:/home/box/.formal-ai" \
  "$CANDIDATE_IMAGE" formal-ai serve --host 0.0.0.0 --port 8080 >/dev/null
host_port="$(docker port "$server" 8080/tcp | awk -F: 'NR == 1 {print $NF}')"
curl -fsS --retry 30 --retry-delay 1 --retry-all-errors --max-time 40 \
  "http://127.0.0.1:$host_port/health" >"$workdir/health.json"
assert_json_contract "$workdir/health.json" health
docker rm -f "$server" >/dev/null

# Schema-1 readers deliberately ignore the additive root marker. Then restore
# the verified backup and prove the old image can reopen the byte-exact state.
run_image "$PREVIOUS_IMAGE" formal-ai memory show --path "$MEMORY_PATH" \
  | grep -Fq 'amber migration canary'
run_image "$CANDIDATE_IMAGE" bash -c \
  "cp '$BACKUP_PATH' '$MEMORY_PATH' && sync"
rolled_back_sha="$(container_sha "$CANDIDATE_IMAGE" "$MEMORY_PATH")"
[[ "$rolled_back_sha" == "$original_sha" ]] || fail "rollback was not byte-exact"
run_image "$PREVIOUS_IMAGE" formal-ai memory show --path "$MEMORY_PATH" \
  | tee "$workdir/rollback-show.txt"
grep -Fq 'amber migration canary' "$workdir/rollback-show.txt" \
  || fail "released container could not reopen rolled-back memory"

echo "memory upgrade E2E passed: released write, preflight, migration, candidate load/query/export, rollback, released reopen"
