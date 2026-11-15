#!/usr/bin/env bash
# Issue #1486: show that a delta image archive (the candidate minus its pinned
# base layers) loads to the exact producer image ID when the base is present,
# and fails closed when it is not.
#
#   bash experiments/issue-1486/delta-image-probe.sh [BASE_REF]
#
# Uses a small public base so the probe stays cheap; the Hive delivery uses the
# same code path with konard/hive-mind-dind pinned by RepoDigest.
set -euo pipefail
repo="$(cd "$(dirname "$0")/../.." && pwd)"
base_ref="${1:-alpine:3.20}"
work="$(mktemp -d)"
trap 'rm -rf -- "$work"; docker image rm -f delta-probe:candidate >/dev/null 2>&1 || true' EXIT

docker pull --quiet "$base_ref" >/dev/null
base_id="$(docker image inspect "$base_ref" --format '{{.Id}}')"
base_pinned="$(docker image inspect "$base_ref" --format '{{index .RepoDigests 0}}')"
head -c 200000 /dev/urandom > "$work/payload"
printf 'FROM %s\nRUN echo one > /one\nCOPY payload /payload\n' "$base_pinned" > "$work/Dockerfile"
docker build --quiet --tag delta-probe:candidate "$work" >/dev/null
candidate_id="$(docker image inspect delta-probe:candidate --format '{{.Id}}')"

node --input-type=module -e "
  import { deltaPlan } from '$repo/scripts/lib/oci-delta-archive.mjs';
  import { execFileSync } from 'node:child_process';
  import { writeFileSync } from 'node:fs';
  const layers = ref => JSON.parse(execFileSync('docker', ['image', 'inspect', ref, '--format', '{{json .RootFS.Layers}}'], { encoding: 'utf8' }));
  writeFileSync('$work/plan.json', JSON.stringify(deltaPlan({ baseImageId: '$base_id', baseRef: '$base_pinned', baseDiffIds: layers('$base_ref'), candidateDiffIds: layers('delta-probe:candidate') })));
"
docker save delta-probe:candidate | gzip -1 > "$work/full.tar.gz"
docker save delta-probe:candidate | node "$repo/scripts/lib/oci-delta-archive.mjs" filter --plan "$work/plan.json" --report "$work/report.json" | gzip -1 > "$work/delta.tar.gz"
printf 'full archive:  %s bytes\ndelta archive: %s bytes\n' "$(stat -c %s "$work/full.tar.gz")" "$(stat -c %s "$work/delta.tar.gz")"

docker image rm -f delta-probe:candidate >/dev/null
docker load --quiet --input "$work/delta.tar.gz" >/dev/null
loaded="$(docker image inspect delta-probe:candidate --format '{{.Id}}')"
[[ "$loaded" == "$candidate_id" ]] || { echo "FAIL: loaded $loaded, produced $candidate_id"; exit 1; }
echo "base present: loaded the exact candidate $loaded"

docker image rm -f delta-probe:candidate >/dev/null
docker image rm -f "$base_ref" "$base_pinned" >/dev/null 2>&1 || true
# BuildKit's cache keeps its own reference to the base layers it built from;
# a fresh runner has none, so drop it to model the consumer (opt out with
# DELTA_PROBE_KEEP_BUILD_CACHE=1).
[[ "${DELTA_PROBE_KEEP_BUILD_CACHE:-0}" == 1 ]] || docker builder prune --all --force >/dev/null
if docker load --quiet --input "$work/delta.tar.gz" >/dev/null 2>"$work/load.err"; then
  echo "FAIL: the delta loaded without its base"; exit 1
fi
echo "base absent: load failed closed ($(tr '\n' ' ' < "$work/load.err" | cut -c1-160))"
docker pull --quiet "$base_ref" >/dev/null
