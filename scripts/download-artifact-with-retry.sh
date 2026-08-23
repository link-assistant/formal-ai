#!/usr/bin/env bash
# Download a workflow artifact, retrying the transient storage failures that
# make a hosted runner look like our defect.
#
# Issue #1039: run 32555911181 reddened `main` with
#
#   ##[error]Unable to download artifact(s): Unable to download and extract
#   artifact: Artifact download failed after 5 retries.
#
# on `macOS Core Tests / Run macOS core slice 5/16`. Fifteen sibling slices
# downloaded the same artifact from the same run and passed. No test ran, no
# commit in the pipeline was implicated, and the blob URL in the log points at
# `productionresultssa8.blob.core.windows.net` -- GitHub's own storage backend,
# not this repository. The pipeline still reported `failure` and no release was
# published.
#
# `actions/download-artifact` does retry internally, and this is what its five
# attempts exhausting looks like: they are spent back-to-back against a backend
# that is having a bad minute, and then the step is over. A retry is only worth
# anything if it can outlast the outage it is retrying, which means pausing
# between attempts rather than hammering.
#
# So this wrapper adds a *second*, slower layer around the action's own: each
# attempt gets its own deadline, and the pauses between them are long enough
# that the next attempt meets a different minute. `apt-install-with-retry.sh` is
# the same shape one failure class over, and the same budget guard applies --
# a retry that cannot finish inside the enclosing step budget converts a
# transient failure into a *terminated* step, which is the failure this exists
# to prevent.
#
# The name is matched as a *prefix*, which also fixes a second failure. The
# macOS archive is uploaded as `macos-core-tests-<run_id>-<run_attempt>`, so on
# a partial rerun (`gh run rerun --failed`) the reran slices are on attempt 2
# while the archive job -- which succeeded and is not rerun -- left its artifact
# named `...-1`. An exact-name download then fails with "artifact not found" and
# a partial rerun of any macOS slice is impossible; the whole pipeline has to be
# rerun instead. Resolving the newest artifact whose name starts with
# `macos-core-tests-<run_id>` finds the archive whichever attempt uploaded it.
#
# Usage: download-artifact-with-retry.sh <artifact-name-prefix> <destination-directory>
#
# Environment:
#   GH_TOKEN / GITHUB_TOKEN            token for `gh` (required)
#   GITHUB_REPOSITORY                  owner/repo (required)
#   GITHUB_RUN_ID                      run holding the artifact (required)
#   FORMAL_AI_DOWNLOAD_ATTEMPTS        attempts before giving up (4)
#   FORMAL_AI_DOWNLOAD_ATTEMPT_SECONDS deadline for one download attempt (120)
#   FORMAL_AI_DOWNLOAD_LOOKUP_SECONDS  deadline for the name lookup (20); it is
#                                      one small API response, so it does not
#                                      need the transfer's budget
#   FORMAL_AI_DOWNLOAD_RETRY_DELAY_SECONDS  pause between attempts (15)
#   TEST_BUDGET_SECONDS                enclosing step budget, checked when set
#   FORMAL_AI_GH                       gh binary (gh); tests point it at a
#                                      stand-in
set -euo pipefail

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# `run-with-deadline.sh`, not GNU `timeout`: macOS ships no `timeout`, and this
# wrapper runs on the macOS slices (issue #1021).
deadline="$script_directory/run-with-deadline.sh"

artifact_prefix="${1:?artifact name prefix is required}"
destination="${2:?destination directory is required}"

attempts="${FORMAL_AI_DOWNLOAD_ATTEMPTS:-4}"
attempt_seconds="${FORMAL_AI_DOWNLOAD_ATTEMPT_SECONDS:-120}"
lookup_seconds="${FORMAL_AI_DOWNLOAD_LOOKUP_SECONDS:-20}"
retry_delay_seconds="${FORMAL_AI_DOWNLOAD_RETRY_DELAY_SECONDS:-15}"
budget_seconds="${TEST_BUDGET_SECONDS:-}"
gh_binary="${FORMAL_AI_GH:-gh}"

for name in FORMAL_AI_DOWNLOAD_ATTEMPTS:$attempts \
  FORMAL_AI_DOWNLOAD_ATTEMPT_SECONDS:$attempt_seconds \
  FORMAL_AI_DOWNLOAD_LOOKUP_SECONDS:$lookup_seconds \
  FORMAL_AI_DOWNLOAD_RETRY_DELAY_SECONDS:$retry_delay_seconds; do
  if ! [[ "${name#*:}" =~ ^[0-9]+$ ]]; then
    echo "${name%%:*} must be a non-negative integer, got ${name#*:}" >&2
    exit 2
  fi
done
[ "$attempts" -ge 1 ] || {
  echo "FORMAL_AI_DOWNLOAD_ATTEMPTS must be at least 1" >&2
  exit 2
}

# The guard issue #1017 paid for, applied here: refuse to start rather than
# discover in CI that the retries cannot fit inside the budget above them.
#
# Both deadlined commands count -- resolving the name *and* downloading -- so
# an attempt that stalls in both spends the sum. Counting only the download
# would understate the worst case and let the job cap expire first, which
# reports as `cancelled` rather than `failure`: exactly what this prevents.
# The delays double, so their total is `delay * (2^(attempts-1) - 1)`, not
# `delay * (attempts - 1)`. Counting the flat sum would understate the worst
# case and let the job cap expire first -- reported as `cancelled`, which is
# the issue #1017 failure this guard exists to prevent.
backoff_total=0
backoff_step=$retry_delay_seconds
for ((backoff_index = 1; backoff_index < attempts; backoff_index++)); do
  backoff_total=$((backoff_total + backoff_step))
  backoff_step=$((backoff_step * 2))
done
worst_case=$((attempts * (attempt_seconds + lookup_seconds) + backoff_total))
if [ -n "$budget_seconds" ] && [ "$worst_case" -gt "$budget_seconds" ]; then
  echo "::error title=artifact download retry cannot finish inside its budget::\
${attempts} attempts of ${attempt_seconds}s plus ${retry_delay_seconds}s delays \
need ${worst_case}s, but the step budget is ${budget_seconds}s. Lower the \
attempts or the per-attempt deadline, or raise the budget with the job cap." >&2
  exit 2
fi

mkdir -p "$destination"

status=0
for ((attempt = 1; attempt <= attempts; attempt++)); do
  started=$SECONDS

  # A partial extraction from a failed attempt would make the next one look
  # like a success with a truncated tree, and the tree check downstream would
  # then blame this repository for a storage failure. Start each attempt clean.
  rm -rf "${destination:?}"/*

  # Resolve the prefix to a concrete name first: `gh run download --name` takes
  # an exact name, and the attempt suffix is not knowable from this job. The
  # newest match wins, so a genuine re-upload on a later attempt supersedes the
  # earlier archive rather than racing it.
  status=0
  artifact_name=$("$deadline" "$lookup_seconds" \
    "$gh_binary" api "repos/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}/artifacts" \
    --jq "[.artifacts[] | select(.name | startswith(\"${artifact_prefix}\"))] \
          | sort_by(.created_at) | last | .name") || status=$?

  if [ "$status" -eq 0 ] && [ -n "$artifact_name" ] && [ "$artifact_name" != "null" ]; then
    status=0
    "$deadline" "$attempt_seconds" \
      "$gh_binary" run download "${GITHUB_RUN_ID}" \
      --repo "${GITHUB_REPOSITORY}" \
      --name "$artifact_name" \
      --dir "$destination" || status=$?
  elif [ "$status" -eq 0 ]; then
    # The listing worked and matched nothing. That is not a storage stall, but
    # it can still be a race against an upload that has not registered yet, so
    # it is worth another attempt rather than an immediate failure.
    status=1
    artifact_name="$artifact_prefix*"
  fi
  elapsed=$((SECONDS - started))

  if [ "$status" -eq 0 ]; then
    printf 'artifact %s downloaded on attempt %s/%s after %ss.\n' \
      "$artifact_name" "$attempt" "$attempts" "$elapsed"
    exit 0
  fi

  echo "::warning title=artifact download failed attempt ${attempt}/${attempts}::\
Attempt ${attempt} exited ${status} after ${elapsed}s of its ${attempt_seconds}s \
deadline. Exit 124 is the deadline itself -- storage stalled, transient and \
upstream; anything else is the CLI's own status." >&2

  [ "$attempt" -lt "$attempts" ] || break

  # Back off further each time. A transfer that hit the deadline was not slow,
  # it was stuck: run 32601998... had seven slices finish in 20-77s while one
  # spent three full 165s attempts and never completed a byte. Retrying at a
  # fixed interval keeps meeting the same bad minute, so each wait doubles --
  # 15s, 30s, 60s -- to give the backend time to move the blob somewhere
  # healthy. The wrapper's budget guard already accounts for the total.
  sleep "$retry_delay_seconds"
  retry_delay_seconds=$((retry_delay_seconds * 2))
done

echo "::error title=artifact ${artifact_prefix} could not be downloaded::\
All ${attempts} attempts of ${attempt_seconds}s failed; the last exited \
${status}. This is no longer a transient stall -- check whether the archive job \
uploaded the artifact at all, and read the attempt warnings above." >&2
exit "$status"
