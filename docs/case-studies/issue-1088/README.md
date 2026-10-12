# Issue #1088 (E110, #1085 D7) — evidence moves out of the source repository

## The measurement (2026-09-30, branch qa-reasoning-coding-bulk-fixes)

| Slice | Files | Size |
|---|---|---|
| `dev/log/issues/` | 5,426 | 712.8 MB |
| `dev/log/self-authored/` | 12 | 108 KB |
| `docs/case-studies/` (whole tree) | — | 263 MB |
| `.git` object store | — | 430 MB |
| working tree (excluding `.git`) | — | ~690 MB of which `dev/log` is ~70% |

Issue #1085 §1.4 said captured logs and case-study raw data dominate the
repository by volume; the measured split confirms it. The issue's "fresh
clone below 150 MB" leg is therefore **not met today and cannot be met by
drafting**: the bytes are already in git history, so only the move (plus,
eventually, history rewrite or a fresh start) brings a clone down. What this
change delivers is the machinery that makes the number move in one
direction from here.

## Requirements

| Requirement | Status |
|---|---|
| R1088-1 — a Links Notation index with sha256, size and URL per artifact group | delivered: `docs/evidence/index.lino` (two measured groups, `pending-move` URLs) |
| R1088-2 — the case-study prose stays in this repository | delivered: only raw captures are indexed; the three prose shapes are the gate's exempt paths |
| R1088-3 — CI check: a pull request adds at most 2,000 non-source lines outside `docs/case-studies/issue-*/{README,requirements,solution-plan}.md` | delivered: `scripts/check-evidence-lines.rs` |
| R1088-4 — every index entry resolves and its hash matches, checked by a test that reads the index | delivered: `scripts/move-evidence.rs --check` (hash-per-group) and its self-tests; a `rust/tests/unit` pin is listed in the integration manifest |
| R1088-5 — `git ls-files dev/log docs/case-studies \| wc -l` below 2,000 | drafted as a gate that arms itself: `check-evidence-lines.rs --files` enforces it exactly when no index row is `pending-move` |
| R1088-6 — fresh clone below 150 MB | open: history already carries the bytes; measured, not hidden (see above) |
| R1088-7 — land #1072 (git worktree instead of archive copies) | open: separate issue, tracked as the dependency it is |

## Deliverables

| Artifact | Role |
|---|---|
| `docs/evidence/index.lino` | The "public knowledge as cache" record: per group, the file count, byte count, content-addressed aggregate sha256, and the URL the group moves to. Digest method documented in the header: sha256 over every sorted (relative-path, file-sha256) pair under the group, so any single file change moves the group digest. |
| `scripts/move-evidence.rs` | The move, idempotent and honest: `--check` verifies every group's digest (exit 1 on mismatch or on a `pending-move` row whose files are gone); `--write` recomputes digests, copies the trees into `../formal-ai-evidence`, rewrites the index URLs, and `git rm -r --cached` the moved paths so the removal is staged for review. If the evidence repository is not cloned beside this one it exits 1 with the reason: creating `link-assistant/formal-ai-evidence` is the maintainer's outward action, not a script's. Dependency-free sha256 (FIPS 180-4) with the `abc` test vector pinned in its self-tests. |
| `scripts/check-evidence-lines.rs` | The CI gate (R1088-3) and the arming file-count gate (R1088-5). `--files` mode counts tracked evidence files and enforces the sub-2,000 limit only once no index row is `pending-move` — the count is the ratchet, so the gate turns on at exactly the commit that completes the move, never before. |
| `.gitignore` | `/dev/log/` added and the old `!dev/log/**/ci-logs/` carve-outs retired: tracked files stay tracked (git never ignores a tracked file), but nothing new may grow under `dev/log`. |

## How to run

```sh
rust-script scripts/move-evidence.rs --check          # digests match the index
rust-script scripts/check-evidence-lines.rs           # diff budget vs origin/main
rust-script scripts/check-evidence-lines.rs --files   # tracked evidence file count
```

## Residuals (deliberately open)

1. **Creating `link-assistant/formal-ai-evidence`** — an outward action
   (repository creation) the maintainer takes; `move-evidence.rs --write`
   then does the rest and stages both sides.
2. **The physical `git rm` of the 5,438 files** — lands as its own pull
   request once (1) exists, so the removal diff is reviewable alone.
3. **Fresh-clone size (R1088-6)** — history rewrite territory; recorded in
   this case study rather than pretended.
4. **Wiring the two gates into CI** — `.github/workflows` edits are gated
   behind the issue #1168/#1169 GUARD; recorded as pending rows in
   `docs/integration-manifest.md`.
