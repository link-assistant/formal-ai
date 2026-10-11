# Issue #1180 (E145) — the repository's history as formal context

Issue [#1180](https://github.com/link-assistant/formal-ai/issues/1180), part
of the E127 umbrella [#1183](https://github.com/link-assistant/formal-ai/issues/1183).
The requirement shard is
`docs/requirements/issue-1180-repository-history-formalization.md`; the raw
issue (title, body, comments, creation time) is
[`raw-data/issue-1180.json`](raw-data/issue-1180.json). The issue has no
comments, so every requirement below comes from its body.

## Timeline

| When (UTC unless noted) | Event |
| --- | --- |
| 2026-08-15 → 2026-09-12 | `scripts/check-self-development-release.rs` is introduced by `88d368a5b` and changed by five later commits — the lineage the issue uses as its acceptance example ([`raw-data/check-self-development-release-lineage.txt`](raw-data/check-self-development-release-lineage.txt), produced by `git log --follow`). |
| 2026-09-29 09:16 | Issue opened. Probe on 0.347.0: "In the formal-ai repository, what does the function `evaluate_calculation` do?" answers with formal-ai's self-description instead of the function at `rust/src/calculation.rs:425`. |
| 2026-09-29 09:21 | Umbrella #1183 opened; #1180 is its "full context of repository" input. |
| 2026-09-30 11:12 | PR [#1188](https://github.com/link-assistant/formal-ai/pull/1188) (`qa-reasoning-coding-bulk-fixes`) opened for the bulk batch that carries this work. |
| 2026-09-30 (+07:00) | `d81d549e0` — `rust/src/history_context.rs`, `data/seed/history-formalization.lino` (with its `rust/embedded/` mirror) and `rust/tests/unit/issue_1180_history_context.rs`. |
| 2026-09-30 (+07:00) | `c80c498a8` — the 1,280-line module split into `history_context/{commits,cursor,github}.rs` under the file-size cap. |
| 2026-10-07 (+07:00) | `46d45c9e1` — the full-suite repair pass finds that `git log` was handed the record format as a bare argument; it now passes `--format=<format>` (changelog line in `changelog.d/20261007_120000_full-suite-repairs.md`). |

## Requirements

The issue numbers its own requirements R1180-1 … R1180-8. Three more are
stated in prose and are tracked here so none is lost:

| ID | Source in the issue | Requirement (short) | Status |
| --- | --- | --- | --- |
| R1180-1 | Requirements | commit importer: author, date, subject, body, trailers, changed paths | partial — `Co-Authored-By:` not formalized as evidence |
| R1180-2 | Requirements | per-path syntax diff with `ast_census` / `es_meta` | partial — node-kind and token-count deltas, not item names |
| R1180-3 | Requirements | issue/PR importer over `github_logs` output, with links | partial — body not formalized into requirements; current state only, no transitions |
| R1180-4 | Requirements | comments/reviews, one record each, linked to the parent | implemented |
| R1180-5 | Requirements | CI runs with failing step and error text | implemented |
| R1180-6 | Requirements | incremental by watermark, idempotent re-run | implemented |
| R1180-7 | Requirements | `MemoryEvent` records, existing query language, no schema change | implemented |
| R1180-8 | Requirements / How to test | lineage of `check-self-development-release.rs` = six commits with issue and PR | implemented for the six commits and the introducing commit's issue/PR; later commits' links unasserted |
| R1180-9 | Design (`cli_repository_history.rs`) | `formal-ai repository-history import / query` | pending |
| R1180-10 | How to test / Depends on (#1171) | `formal-ai chat` answers "why does the self-development status fail?" from this history | pending |
| R1180-11 | Depends on (three-roots parity) | JS/TS twins of the importer | pending |

Definition-of-done items that are process rather than behaviour: this shard
and case study (done here), traceability rows (drafted for the maintainer, see
below), a `changelog.d/` fragment with `### Added` (not written yet — only the
`--format=` fix line exists), and a green `run-ci-gates.rs --stage rust`
(not re-run for this write-up).

## Root causes

1. **Capture without formalization.** `rust/src/github_logs.rs` shells out to
   `gh` and writes JSON and diff files plus a `manifest.json`. Nothing read
   those files back into the memory store, so the evidence existed on disk
   but no query could reach it.
2. **No commit source at all.** Git history was never read by the solver; the
   lineage of a file was answerable only by a person running `git log`.
3. **Routing, for the probe.** The `evaluate_calculation` question matched the
   self-description route because no handler owned repository questions — a
   separate gap (R1180-10, and the `repository_qa` class of #1171) that a
   history store alone does not close.
4. **The `--format` slip (found after delivery).** The first importer built
   `git log --no-show-signature <format> --name-only HEAD`; git reads a bare
   argument as a revision, so every real import failed with an ambiguous
   argument while the parser unit tests — which feed `parse_log_output` text
   directly — still passed. The fixture-repository tests are what exercise
   the real command line.

## Prior art and existing pieces reused

- `git log --follow`, `git show <rev>:<path>` and `git blame` — the plumbing
  the commit importer drives; no libgit2 binding was added.
- `formal-ai github-logs` (`rust/src/github_logs.rs`) stays the raw capture
  tool; its `ISSUE_LIST_FIELDS`/`PR_LIST_FIELDS`/`RUN_LIST_FIELDS` already
  select `updatedAt`/`databaseId`, which is what made watermarks possible
  without new fields.
- `MemoryStore` load/save (`rust/src/memory.rs`) and the memory query
  language (`rust/src/memory_query_language/`) — reused unchanged.
- `agentic_coding::self_ast::ast_census` and `es_meta::extract` for the
  syntax deltas.
- Outside the repository: `git log -S`/`-G` (the pickaxe) is git's own
  content-level blame and is the closest existing tool to "which commit
  introduced claim X"; GitHub's GraphQL `timelineItems` connection carries the
  issue/PR state transitions R1180-3 still lacks. Neither formalizes meanings,
  so the design keeps everything in the existing links store rather than a
  separate index.

## What was built

- `rust/src/history_context.rs` — `HistoryRules` loaded from
  `data/seed/history-formalization.lino` (kind spellings, id prefixes, roles,
  `Refs`/`Closes` trailer patterns, the merge-subject pattern, evidence
  prefixes, which census applies to which suffix), with
  `HistoryRules::defaults()` equal to the parsed seed.
- `history_context/commits.rs` — `import_commits`, `import_commits_for_path`
  (adds `--follow`), `formalize_commit`, `diff_symbols`, and the merge lookup
  (`git log --merges --ancestry-path`) that attaches `pr:N` to a commit
  delivered through a pull request.
- `history_context/github.rs` — `import_issues_and_pulls` (issues, PRs,
  comments, reviews from the `github-logs` JSON files) and `import_ci_runs`
  with the failing-step text.
- `history_context/cursor.rs` — `RepositoryHistoryCursor`,
  `import_incremental`, `write_repository_history` (deduplicates on event id),
  and `store_paths` (`<memory dir>/repository-history/<owner>-<repo>/`).

## Verification

`rust/tests/unit/issue_1180_history_context.rs` (registered in
`rust/tests/unit/mod.rs`):

| Test | Pins |
| --- | --- |
| `commit_formalization_extracts_trailers_and_changed_symbols` | R1180-1, R1180-2 on a fixture repository |
| `incremental_import_is_idempotent` | R1180-6 |
| `incremental_import_only_advances_past_the_watermark` | R1180-6 |
| `issue_and_pr_import_links_commit_to_source_issue` | R1180-3, R1180-4, R1180-7 (runs the lineage query) |
| `ci_run_import_carries_failing_steps` | R1180-5, run-id watermark |
| `rules_seed_round_trips_through_the_parser` | seed = compiled defaults |
| `on_this_repo_check_self_development_release_lineage` | R1180-8 on this repository's own history (skips on a shallow clone) |

These tests were not re-run while writing this case study; the status column
reflects what the test bodies assert, read against the code on
`qa-reasoning-coding-bulk-fixes`.

## Residuals

R1180-1 (`Co-Authored-By:` evidence), R1180-2 (named items rather than kind
counts), R1180-3 (requirement formalization of bodies, state history),
R1180-9 (CLI), R1180-10 (chat wiring — the part that makes the issue's probe
pass), R1180-11 (JS/TS twins), and the changelog fragment.
