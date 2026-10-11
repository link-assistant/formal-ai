Part of the E127 umbrella (#1183): the "full context of repository" the fact checker, the coder and Q&A reason over.

## Evidence

`formal-ai github-logs` collects issue, PR, review and Actions evidence into a case-study directory as files; nothing turns Git history, issues, pull requests, comments, reviews and CI runs into links the solver can query. Questions like "which issue asked for the self-hosting release gate and who changed its threshold" cannot be answered.

Probe (0.347.0): "In the formal-ai repository, what does the function evaluate_calculation do?" → "I am formal-ai, a deterministic symbolic AI implementation …" (a self-description; the function is `rust/src/calculation.rs:422`).

Verified against `origin/main` (2026-09-29):
- `rust/src/calculation.rs:425` is `pub fn evaluate_calculation(expression: &str) -> Result<CalculationEvaluation, ArithmeticError>`. Line 422 is the first line of its three-line `///` doc comment, not the `pub fn` line itself — the original claim is correct in substance (the function lives in that 422-428 block) but the signature itself is 3 lines lower, at 425.
- `rust/src/github_logs.rs` (427 lines) is real and matches the claim exactly: `github_log_capture_plan` builds a `Vec<GithubLogCapture>` of `gh` CLI invocations (`gh repo view`, `gh issue list/view`, `gh pr list/view/diff`, `gh api .../comments`, `gh api .../reviews`, `gh run list/view`) and `collect_github_logs` executes them, writing each result as a raw JSON/diff file plus a `manifest.json` (`GithubLogManifest`) under `docs/case-studies/github-logs/raw-data` by default. It shells out per-call and writes files; it builds no queryable structure and does not touch `rust/src/memory.rs` or the memory query language at all. Confirmed correct.

## What to build

- An importer that formalizes, incrementally: commits (author, time, message, trailers, changed paths and symbols via the meta-language parse), issues and PRs (title, body formalized into requirements, labels, state changes, links between them), comments and reviews (formalized statements with author and time), Actions runs (workflow, conclusion, failing step, error text).
- Queries through the memory query language: blame at the level of meanings ("which commit introduced the claim X"), requirement lineage ("which issue asked for Y and which PR delivered it"), CI history of a test.
- Used by E144 (#1179) (evidence), E135 (#1170) (self-coding reads its own history), and Q&A ("why is Z like this?").

## How to test

On this repository: the lineage query for `check-self-development-release.rs` returns the issues and PRs that introduced and changed it; `formal-ai chat` answers "why does the self-development status fail?" with the threshold, the commit range and the rule's source issue.

## Requirements

- **R1180-1** — A commit importer walks `git log` on the working repository (no GitHub API needed for commits) and formalizes each commit's author, committer date, subject, body, trailers (`Refs #NNN`, `Closes #NNN`, `Co-Authored-By:`), and the paths its tree changed, into one formal record per commit.
- **R1180-2** — For every changed `.rs` path in a commit, the importer runs the existing Rust census `crate::agentic_coding::self_ast::ast_census` (`self_ast.rs:148`, built on `meta_language::LinkNetwork`) on the pre-image and post-image blobs (for `.js`/`.ts` paths: `es_meta::parse_document`, `rust/src/es_meta.rs:325`) (via `git show <sha>^:<path>` / `git show <sha>:<path>`) and records which top-level items (functions, structs, impls) differ, not just which file paths changed.
- **R1180-3** — An issue/PR importer reuses `github_logs`'s existing `gh`-backed capture (`rust/src/github_logs.rs`: `GithubLogCollectorConfig`, `github_log_capture_plan`, `collect_github_logs`) — either by reading the JSON files it already writes under `docs/case-studies/github-logs/raw-data/` or by driving the same `gh issue/pr/api` command shapes directly — and formalizes issue/PR title, body, labels, state transitions, and issue↔PR/commit links (`Refs #NNN`, `Closes #NNN`, `Merge pull request #NNN`).
- **R1180-4** — A comments/reviews importer formalizes each `gh api .../comments` and `gh api .../reviews` entry as one record with author, body, and timestamp, linked to its parent issue or PR number.
- **R1180-5** — A CI-run importer formalizes each Actions run (`gh run list/view` JSON: `databaseId`, `workflowName`, `conclusion`, `headSha`, `jobs`) into one record per run, carrying the failing step and error text when the run failed.
- **R1180-6** — Every importer function is incremental: given a stored cursor, it re-imports only commits/events newer than the watermark, and running it twice back-to-back with no new history produces zero new records (idempotent).
- **R1180-7** — All five record kinds are formalized as `MemoryEvent` values (`rust/src/memory.rs:66`) — no new table or schema — with `kind` set to `"commit" | "issue" | "pull_request" | "review" | "ci_run"`, so they are queryable through the *existing* memory query language (`rust/src/memory_query_language/`) without any change to `MemoryField`, `expect_memory_table`, or the SQL/GraphQL grammar.
- **R1180-8** — The lineage query for `scripts/check-self-development-release.rs` returns exactly the commits `git log --follow` lists for that path (six on 2026-09-29: `88d368a5b` introduce, `02f590106` edition move, `96f405e0d`, `aa6a5d085`, `9efeb735c`, `33451d012`), each with its issue and pull request.

## Design

### Storage location

Mirrors the existing `data/meta/<topic>/` convention for generated formal artifacts (e.g. `data/meta/self-ast/`, `data/meta/requirement-status-ledger/`, `data/meta/merge-conflict-policy.lino`) and the exact on-disk shape `rust/src/memory.rs` already documents (`demo_memory` Links Notation, one `event "<id>"` node per record, loaded with `MemoryStore::load_from_file` at `rust/src/memory.rs:318`, written with `MemoryStore::save_to_file` at `rust/src/memory.rs:351`):

- `<memory dir>/repository-history/<owner>-<repo>/events.lino` — the formalized `MemoryStore`, in the user's memory directory (the same place `formal-ai memory` exports from), not committed to the repository; only the worked-example excerpt is committed under the case study.
- `<memory dir>/repository-history/<owner>-<repo>/cursor.lino` — the incremental watermark (see below), same Links Notation shape, one node per stream.

```
repository_history
  event "commit:88d368a5b86ad37d05b0c41b2ec5f6b1c941d15b"
    kind "commit"
    role "author"
    content "fix(ci): resolve diagnostic failures"
    sentAt "2026-09-08T00:00:00Z"
    conversationId "issue-1014"
    evidence "path:scripts/check-self-development-release.rs"
    evidence "pr:1015"
  event "issue:1014"
    kind "issue"
    content "<formalized issue body as requirement statements>"
    sentAt "2026-09-07T00:00:00Z"
    evidence "pr:1015"
  event "ci_run:36266423193"
    kind "ci_run"
    content "Coverage lane: budget exceeded"
    sentAt "2026-09-26T21:00:00Z"
    conversationId "issue-1149"
```

### New/modified files

- `rust/src/repository_history_import.rs` **[NEW]** — verified nothing with this name exists (`git ls-tree -r origin/main --name-only | grep repository_history_import` is empty). Houses:
  - `pub fn import_commits(repo_root: &Path, since_sha: Option<&str>) -> Result<Vec<MemoryEvent>, RepositoryHistoryImportError>` — runs `git log <since_sha>..HEAD --format=...` (or full history when `since_sha` is `None`), and for each commit calls `formalize_commit`.
  - `fn formalize_commit(raw: &RawCommit, changed_symbols: &[SymbolChange]) -> MemoryEvent` — maps to a `MemoryEvent` as shown above; `evidence` carries one entry per changed path and one per changed symbol name.
  - `fn diff_symbols(repo_root: &Path, sha: &str, path: &str) -> Result<Vec<SymbolChange>, RepositoryHistoryImportError>` — calls `self_ast::ast_census` on the blob before and after the commit for each changed `.rs` path (`es_meta::parse_document` for `.js`/`.ts`) and diffs the top-level items.
  - `pub fn import_issues_and_pulls(logs_dir: &Path, since_updated_at: Option<&str>) -> Result<Vec<MemoryEvent>, RepositoryHistoryImportError>` — reads the `issue-*.json`, `pr-*.json`, `*-comments.json`, `*-reviews.json` files `github_logs::collect_github_logs` (`rust/src/github_logs.rs`) already knows how to produce, or invokes the same `gh` command shapes (`ISSUE_LIST_FIELDS`, `PR_LIST_FIELDS`, `RUN_LIST_FIELDS` constants in `github_logs.rs` already select `updatedAt`/`createdAt`, so the incremental filter needs no new field).
  - `pub fn import_ci_runs(logs_dir: &Path, since_run_id: Option<u64>) -> Result<Vec<MemoryEvent>, RepositoryHistoryImportError>` — same source, filtered on `databaseId > since_run_id`.
  - `pub fn write_repository_history(events: Vec<MemoryEvent>, store_path: &Path) -> io::Result<()>` — loads the existing store with `MemoryStore::load_from_file` (or starts empty if absent), appends, and calls `store.save_to_file(store_path)` (both already public in `rust/src/memory.rs`).
- `rust/src/cli_repository_history.rs` **[NEW]** — a `formal-ai repository-history import|query` subcommand, following the `Plan`/`Collect` pattern already used by `rust/src/cli_github_logs.rs`.
- `rust/src/lib.rs`, `rust/src/main.rs` **[MODIFIED]** — register the new module and CLI subcommand, mirroring how `github_logs`/`cli_github_logs` are wired in today (confirmed via `git grep github_logs origin/main -- rust/src/lib.rs rust/src/main.rs`).

### Incremental cursor

A `RepositoryHistoryCursor` struct persisted at `<memory dir>/repository-history/<owner>-<repo>/cursor.lino`:

```
repository_history_cursor
  last_commit_sha "88d368a5b86ad37d05b0c41b2ec5f6b1c941d15b"
  last_issue_pr_updated_at "2026-09-26T21:00:00Z"
  last_ci_run_database_id "36266423193"
```

- Commits: next run passes `last_commit_sha` as the lower bound of `git log <sha>..HEAD`; `git` itself guarantees no reprocessing.
- Issues/PRs: next run keeps only records whose `updatedAt` (already a captured field per `ISSUE_LIST_FIELDS`/`PR_LIST_FIELDS` in `github_logs.rs`) is strictly newer than `last_issue_pr_updated_at`.
- Actions runs: next run keeps only `databaseId > last_ci_run_database_id` (the field `RUN_LIST_FIELDS` in `github_logs.rs` already captures).
- After a successful import, `write_repository_history` also rewrites `cursor.lino` with the new watermarks in the same call, so a crash between "events written" and "cursor advanced" only risks re-importing the last batch (append is idempotent on `MemoryEvent.id`, so duplicates are detected and skipped by id, not silently doubled).

### Queries via the memory query language

No schema change is required: `rust/src/memory_query_language/mod.rs` already restricts every query to the `memory`/`memoryevent(s)` table (`expect_memory_table`, `sql.rs:337`) over the fixed 15-field `MemoryField` set (`mod.rs:171-206`) that `MemoryEvent` already provides — `kind`, `content`, `sentAt`, `conversationId`, `evidence`, etc. — so `compile_memory_query`/`execute_memory_query` (`rust/src/memory_query_language/mod.rs:615`, `execution.rs:24`) work unmodified once the store contains repository-history events.

Worked example, grounded in real history (`git log --follow scripts/check-self-development-release.rs` on `origin/main`):
- Introduced by commit `88d368a5b` ("fix(ci): resolve diagnostic failures", `Refs #1014`), merged via **PR #1015** (`Merge pull request #1015 from link-assistant/issue-1014-fa915643117e`, commit `d15d9658f`).
- Changed by commit `96f405e0d` ("fix(ci): bound how long a release may stay silently deferred", `Closes #1064`), merged via **PR #1065**.
- Changed by commit `aa6a5d085` ("fix(ci): remove the release deferral budget", `Refs #1066`), merged via **PR #1067**.
- Also touched by `02f590106` (edition 2024 move), `9efeb735c` ("unblock self-authoring and relax the floor after a day") and `33451d012` ("let the floor answer the question a pull request can answer"); the query must return these too.

1. **Blame at the level of meanings** — "which commit introduced the claim that deferring work is forbidden":
   ```sql
   SELECT id, content, sentAt FROM memory
   WHERE kind = 'commit' AND content CONTAINS 'Deferring work is forbidden in this repository'
   ORDER BY sentAt ASC LIMIT 1
   ```
   Expected: `id = "commit:aa6a5d085eff5ce5126e477e1dbf9f60d2db85cd"` (its body opens with exactly that sentence), linked via `evidence "pr:1067"` and `evidence "issue:1066"`.

2. **Requirement lineage** — "which issue asked for `check-self-development-release.rs` and which PR delivered it":
   ```sql
   SELECT id, conversationId, evidence FROM memory
   WHERE kind = 'commit' AND evidence CONTAINS 'path:scripts/check-self-development-release.rs'
   ORDER BY sentAt ASC
   ```
   Expected: six rows in time order, the first three being `commit:88d368a5b8...` (`issue-1014`, `pr:1015`), then `02f590106`, `commit:96f405e0d4...` (`issue-1064`, `pr:1065`), `commit:aa6a5d085e...` (`issue-1066`, `pr:1067`), then `9efeb735c` and `33451d012` with their issues.

3. **CI history of a test** — "how has `regenerate-self-ast-census` fared in Actions":
   ```sql
   SELECT id, content, sentAt FROM memory
   WHERE kind = 'ci_run' AND content CONTAINS 'regenerate-self-ast-census'
   ORDER BY sentAt DESC LIMIT 20
   ```
   Expected: one row per run of `.github/workflows/regenerate-self-ast-census.yml` (confirmed to exist), each carrying its conclusion and, on failure, the failing job/step text captured from the `jobs` field `RUN_VIEW_FIELDS` already selects in `github_logs.rs`.

## Tests

- `rust/tests/unit/issue_1180_repository_history.rs` **[NEW]** (registered in `rust/tests/unit/mod.rs`):
  - `commit_formalization_extracts_trailers_and_changed_symbols` — runs the importer against a small fixture repo (a `tempfile` git init with 2-3 commits touching a `.rs` file) and asserts `MemoryEvent.conversation_id`, `evidence` paths, and `evidence` symbol names match the fixture's known diff.
  - `incremental_import_is_idempotent` — imports twice with no new commits between runs; asserts zero new events and an unchanged `cursor.lino`.
  - `incremental_import_only_advances_past_the_watermark` — seeds a cursor mid-history, imports, and asserts only commits after the watermark appear.
  - `issue_and_pr_import_links_commit_to_source_issue` — feeds the importer a fixture `issue-1014.json`/`pr-1015.json` pair shaped like `github_logs.rs`'s real JSON output and asserts the resulting `MemoryEvent`s cross-reference by number.
  - `on_this_repo_check_self_development_release_lineage` — an integration test run against `origin/main`'s actual history (or a pinned shallow mirror) asserting the memory-query-language lineage query above returns `issue-1014`/`pr-1015`, `issue-1064`/`pr-1065`, `issue-1066`/`pr-1067` for `scripts/check-self-development-release.rs` — this is the exact scenario the issue's "How to test" section promises.
- Run: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1180_repository_history` and, unchanged, `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit memory_query_language`.

## Definition of done

- `docs/requirements/issue-1180-repository-history-formalization.md` created following the shard convention in `docs/requirements/README.md` (one file per issue, `## Issue #1180 ...` heading, `R1180-N` rows with status/evidence columns), then `rust-script scripts/assemble-requirements.rs --write` run so `REQUIREMENTS.md` regenerates from it.
- A row for each `R1180-N` added to `docs/requirements-traceability.md`, following its existing table shape (requirement id, delivery commit, automated test, manual confirmation status — honestly `not yet confirmed` where true).
- `docs/case-studies/issue-1180/` created holding the worked lineage example above (the three real commits/PRs for `check-self-development-release.rs`) as durable evidence, the same way `docs/case-studies/issue-914/` anchors the requirements-traceability audit.
- A changelog fragment added under `changelog.d/` (filename pattern `YYYYMMDD_HHMMSS_issue-1180-<subject>.md`, `bump: minor` front matter, `### Added` section), following the existing fragment shown in `changelog.d/20260926_215107_issue-1149-coverage-budget.md`.
- `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green, including the new `repository_history_import` test module and the existing `memory_query_language` suite untouched (no schema change).

## Depends on / blocks

- Depends on / part of **#1183** (E127 umbrella) — this issue is one formal-context input the umbrella collects.
- Blocks **#1179** (E144) — the fact checker needs repository-history links as evidence before it can cite "which issue asked for X."
- Blocks **#1170** (E135) — self-coding needs to query its own commit/issue history before it can reason about why prior code exists.
- Blocks the repository-Q&A class of #1171 (E136): the probe "what does `evaluate_calculation` do?" is answered from the code plus this history.
- Three-roots parity: the importer is translated with `formal-ai translate --from rust --to js|ts --input rust/src/repository_history_import.rs --write`.
- Reuses, does not block: `rust/src/github_logs.rs` / **`formal-ai github-logs`** stays as the raw-capture tool; this issue's importer consumes its output (or its command shapes) rather than replacing it.

