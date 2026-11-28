The full audit entrypoint can report success when Git fails to enumerate the tracked lockfiles. This also occurs after Git emits a partial list, so a successful audit of that subset can conceal an incomplete inventory.

Affected source: [`scripts/audit-npm-locks.sh` lines 17–29](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/blob/c532c173801dfb12e681b0730c4abbf93aefd0ca/scripts/audit-npm-locks.sh#L17-L29), current `main` rechecked at `c532c173801dfb12e681b0730c4abbf93aefd0ca`. Full 53-line source Git blob: `e4c73c9df844a08e031b786cb8fdb891ce192789`; SHA-256: `40bdae551f88431166778fa4f041a437afcf1ff73bf5b82fb59d5fe586b2a7a2`.

Reproduce without installing dependencies or contacting an advisory registry:

```bash
source_script="$PWD/scripts/audit-npm-locks.sh"
scratch="$(mktemp -d)"
(
  cd "$scratch"
  bash "$source_script"
  printf 'audit exit: %s\n' "$?"
)
rm -rf -- "$scratch"
```

In an ordinary temporary directory outside a Git repository, Git prints `fatal: not a git repository`, but the script prints `::notice::No package-lock.json is tracked; nothing to audit.` and exits **0**. `--list` likewise succeeds. A controlled Git process that prints one existing lockfile and then exits 128 causes that single lock to be audited and reports `Audited 1 npm lockfile(s)` with exit 0, despite the failed inventory.

The failing producer runs inside `mapfile -t locks < <(git ls-files ...)`. The parent observes `mapfile` success, so `set -euo pipefail` does not propagate the producer's failure. A successful, empty inventory and a failed inventory become indistinguishable.

Suggested minimal repair, keeping the current pathspec, severity, full-audit loop and advisory failures:

```bash
tracked_locks="$(git ls-files -- ':(glob)**/package-lock.json')"
mapfile -t locks < <(printf '%s' "$tracked_locks")
```

The assignment must succeed before either `--list` or any audit runs. `printf '%s'` emits no record for a genuinely empty successful inventory. The disposable candidate propagates Git status 128 and performs no npm call for failed/partial inventories in either mode. Controls retain both tracked locks (including a path with spaces), exclude an untracked lock, retain `NPM_AUDIT_LEVEL=high`, reject an advisory verdict while auditing every selected lock, and accept a genuinely empty repository. Bash/npm child processes have five-second deadlines; npm is an offline stand-in, so no vulnerability or registry-state assertion is made.

Related work reviewed: closed issue #132 and merged PR #134 introduced npm coverage; merged PR #152 updated dependencies. The complete current 157-item issue/PR inventory and relevant conversation, inline-comment and review endpoints were checked before drafting. This report concerns propagation of the inventory failure, rather than adding npm coverage or changing dependency versions.

Found during https://github.com/link-assistant/formal-ai/pull/1581. Formal AI's analogous boundary has a local repair and regression; that does not repair this upstream helper. No upstream implementation change is claimed.
