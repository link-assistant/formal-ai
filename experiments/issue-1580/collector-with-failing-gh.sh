#!/usr/bin/env bash
# Issue #1580: reproduce Job Headroom run 40307862657 -- one `gh api` call
# fails mid-sample -- against a fake `gh`, and print what the collector does.
#
#   experiments/issue-1580/collector-with-failing-gh.sh [MAX_UNREAD_PERCENT]
#
# Run 101 fails once and then answers; run 102 never answers. Before the fix
# the script exited 1 with no message at run 101; after it, run 101 is read on
# the retry and run 102 is skipped with a warning, unless the unread share is
# above MAX_UNREAD_PERCENT (default 50 here), in which case it fails with an
# `::error::`.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT
cat > "${work}/gh" <<'FAKE'
#!/usr/bin/env bash
# Prints already-filtered rows, as `gh api --jq` would.
for argument in "$@"; do
  case "${argument}" in
    *"actions/runs?"*"page=1") printf '101\tCI\n102\tCI\n103\tCI\n'; exit 0 ;;
    *"actions/runs?"*) exit 0 ;;
    *"runs/101/jobs"*)
      if [ ! -e "${FAKE_STATE}/101" ]; then
        touch "${FAKE_STATE}/101"
        echo "HTTP 502: Bad Gateway (https://api.github.com/...)" >&2
        exit 1
      fi
      printf 'build\tsuccess\t2026-01-01T00:00:00Z\t2026-01-01T00:01:00Z\n'; exit 0 ;;
    *"runs/102/jobs"*) echo "HTTP 403: API rate limit exceeded" >&2; exit 1 ;;
    *"runs/103/jobs"*)
      printf 'test\tsuccess\t2026-01-01T00:00:00Z\t2026-01-01T00:02:00Z\n'; exit 0 ;;
  esac
done
exit 0
FAKE
chmod +x "${work}/gh"
mkdir "${work}/state"
status=0
PATH="${work}:${PATH}" FAKE_STATE="${work}/state" GITHUB_REPOSITORY=o/r \
  COLLECT_RETRY_DELAY=0 COLLECT_MAX_UNREAD_PERCENT="${1:-50}" \
  bash "${root}/scripts/collect-job-durations.sh" main 3 \
  > "${work}/rows.tsv" 2> "${work}/log.txt" || status=$?
echo "exit status: ${status}"
echo "--- rows"; cat "${work}/rows.tsv"
echo "--- log"; cat "${work}/log.txt"
