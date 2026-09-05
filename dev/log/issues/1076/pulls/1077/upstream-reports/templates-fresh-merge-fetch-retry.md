# templates: `simulate-fresh-merge.sh` fails the job when a single `git fetch` hits a network blip

Filed against the rust, js and python
`link-foundation/*-ai-driven-development-pipeline-template` repositories.

## What happens

`scripts/simulate-fresh-merge.sh` fetches the base branch once, unretried:

```sh
git fetch origin "$BASE_REF"       # js, python
git fetch --no-tags origin "${BASE_REF}"   # rust
```

Under `set -euo pipefail` any transient failure of that one call ends the job,
and because the script runs as an early step, every later step is skipped.

We hit it in link-assistant/formal-ai run 33973154494, job 101325331000
(`macOS Core Tests / Build macOS test archive`), 30 seconds in:

```text
=== Synchronizing PR with latest main ===
Fetching latest main...
fatal: unable to access 'https://github.com/<owner>/<repo>/': Could not resolve host: github.com
##[error]Process completed with exit code 128
```

Name resolution failed on the runner. Nothing about the commit under test
caused it, and the pull request went red -- a false positive in a required
check.

## Reproduction

Put a `git` earlier on `PATH` that fails its first `fetch` the way the runner
did, and delegate everything else:

```sh
mkdir -p /tmp/fakebin && cat > /tmp/fakebin/git <<'SH'
#!/usr/bin/env bash
if [ "$1" = fetch ]; then
  n=$(( $(cat /tmp/fetch-attempts 2>/dev/null || echo 0) + 1 ))
  echo "$n" > /tmp/fetch-attempts
  if [ "$n" -eq 1 ]; then
    echo "fatal: unable to access 'https://github.com/o/r/': Could not resolve host: github.com" >&2
    exit 128
  fi
  exit 0
fi
case "$1" in
  config) exit 0 ;;
  rev-parse) echo 0000000000000000000000000000000000000000 ;;
  rev-list) echo 0 ;;
  *) exit 0 ;;
esac
SH
chmod +x /tmp/fakebin/git
rm -f /tmp/fetch-attempts
PATH=/tmp/fakebin:$PATH BASE_REF=main bash scripts/simulate-fresh-merge.sh; echo "exit=$?"
```

Today: `exit=128` after one attempt. With the fix below: `exit=0` after two.

## Workaround

Re-run the job. That is exactly the cost the fix removes.

## Suggested fix

Bound the retry rather than removing the failure -- a base branch that truly
cannot be fetched must still fail, because silently skipping the merge
simulation is the false negative the check exists to prevent:

```sh
fetch_with_retry() {
  local attempt=1
  local max_attempts=5
  local delay="${FRESH_MERGE_RETRY_DELAY_SECONDS:-5}"

  while :; do
    if git fetch origin "$@"; then
      return 0
    fi
    if [ "$attempt" -ge "$max_attempts" ]; then
      echo "::error::git fetch origin $* failed $max_attempts times; the base branch could not be read"
      return 1
    fi
    echo "git fetch origin $* failed (attempt $attempt/$max_attempts); retrying in $((delay * attempt))s"
    sleep "$((delay * attempt))"
    attempt=$((attempt + 1))
  done
}

fetch_with_retry "$BASE_REF"
```

The `FRESH_MERGE_RETRY_DELAY_SECONDS` override keeps the behaviour testable
without a 30-second test.

Our implementation, with two tests that drive it through a stand-in `git`
(transient failure recovers, persistent failure still fails), is in
link-assistant/formal-ai: `scripts/simulate-fresh-merge.sh` and
`tests/unit/ci-cd/fresh_merge_fetch_retry.rs`.
