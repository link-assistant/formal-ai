# Issue #960 — Codifying and enforcing three recorded-but-unenforced conventions

Issue: <https://github.com/link-assistant/formal-ai/issues/960>
Pull request: <https://github.com/link-assistant/formal-ai/pull/975>

## 1. Collected Data

Raw GitHub payloads are under `raw-data/github/`:

| File | Contents |
| --- | --- |
| `issue.json` | Issue #960 as filed. |
| `issue-comments.json` | Comment thread on the issue (empty at the time of the fix). |
| `pull-222-comment.json` | The R222-1 source comment on PR #222 (2026-05-22). |
| `pull-234-comment.json` | The R234-2 / R234-4 source comment on PR #234 (2026-05-24). |
| `pull-request.json` | PR #975, the single pull request for this issue. |

`raw-data/cache-budget-run.txt` is the real output of the new
`scripts/check-cache-budget.rs` on this repository, not a synthesized sample.

## 2. Timeline

- **2026-05-22** — PR [#222](https://github.com/link-assistant/formal-ai/pull/222#issuecomment-4513844358):
  "we should cache not more than 128 the most frequently used words... But each
  .lino file, cannot be larger than 1500 lines, so there should be parts of
  .lino files." The `.lino` cap was implemented; the 128 became a constant.
- **2026-05-24** — PR [#234](https://github.com/link-assistant/formal-ai/pull/234#issuecomment-4528554549):
  "I don't see in tests exact examples of answers, only contains and not
  contains... we need a test or CI/CD rule that will guarantee it." And, in the
  same comment: "we should use proper `Fixes <url>` syntax... Word `Addresses`
  is not recognized by GitHub as explicit link to the issue."
  Both were applied once, to that pull request, and to nothing after it.
- **2026-08-06** — Issue #960 filed after an audit found all three conventions
  recorded and none enforced: `check-file-size.rs` excluded
  `data/cache/wikidata/`, `MAX_SEED_RECORDS_PER_BUCKET` had no reader that could
  fail, and CONTRIBUTING.md / the PR template said nothing about linking.

## 3. Requirements

| ID | Requirement (issue wording) |
| --- | --- |
| R960-1 | Include `data/cache/wikidata/` in the `.lino` line-count gate, or state explicitly, with reason, why it is exempt. |
| R960-2 | Add active enforcement of `MAX_SEED_RECORDS_PER_BUCKET = 128` — a CI check or test that fails if any cache bucket exceeds it. |
| R960-3 | Fix `data/cache/wikidata/entity` to ≤128 records, or bucket it. |
| R960-4 | A CI script checking conversational/behavioural tests for the exact-answer style, failing on loose-only tests. |
| R960-5 | A check confirming a PR body contains `Fixes #N` / `Fixes <url>`, not "Addresses". |
| R960-6 | CONTRIBUTING.md / `.github/pull_request_template.md` sections codifying the linking rule and `docs/case-studies/pull-request-{id}`. |

## 4. Root Causes

**RC1 — A number in a comment is not a gate.** `MAX_SEED_RECORDS_PER_BUCKET`
(`src/translation/cache.rs`) was read by no check. The value drifted from 128 to
406 in `data/cache/wikidata/entity` with nothing to notice. The general failure
mode is recording a convention in the place it is *described* rather than in the
place it is *violated*.

**RC2 — "Generated" was treated as a reason to be exempt.**
`check-file-size.rs` skipped `data/cache/wikidata/` because the directory is
written by a tool. But a generator is precisely what can breach a cap at
machine speed; generated-but-committed is the reason to be measured, not to be
excused.

**RC3 — A one-time practice looks like a convention.** The exact-answer test
style and the `Fixes <url>` rule were each honoured in the PR where they were
requested. Nothing carried them to the next contributor, and CONTRIBUTING.md —
the file whose whole purpose is to carry conventions forward — did not mention
either.

## 5. The 128-Record Cap Versus Total Closure

The issue asks for `data/cache/wikidata/entity` to be fixed to ≤128 records "or
bucketed". Measurement says neither is the right move for three specific
buckets, so this deviation is stated openly rather than quietly implemented.

The repository has a *total reference closure* gate
(`scripts/audit-total-closure.py`, `tests/unit/total_closure.rs`): every bare
token in `data/seed/` must resolve to a meaning, a role, a cached lemma, or a
Wikidata id. Cached records also reference each other (`L3412.lino` →
`Q4833830`), so the closure is recursive. Every record in those buckets exists
because something points at it.

Measured on this repository:

| Bucket | Records | Orphans (nothing references them) |
| --- | ---: | ---: |
| `data/cache/wikidata/entity` | 406 | 0 |
| `data/cache/wordnet/en` | 332 | 0 |
| `data/cache/wiktionary/en` | 243 | 0 |

Trimming to 128 would therefore delete data the closure gate requires and turn
one red build into another. Sharding into `entity-a/`, `entity-b/`… would keep
the same bytes in the same repository while rewriting roughly ten reader sites
(`cli_import`, `ground-meanings.rs`, `ground-entity-names.py`,
`ground-release-timelines.py`, the backfill example, the
`audit-total-closure.py` glob, several tests, and the `data/overrides` mirror) —
motion without a lighter repository.

The resolution: a **hard** 128-record cap for every non-exempt bucket, plus an
explicit `CLOSURE_DRIVEN_BUCKETS` list naming these three with a written reason.
The exemption is not free — it buys a stricter invariant. For exempt buckets the
check fails if *any* record becomes an orphan, which is the actual thing the
128 was protecting against (unbounded, unjustified caching), and emits a
permanent warning so the overflow never becomes invisible.

## 6. Prior Art In This Repository

- **Burn-down allowlist ratchet** — `scripts/check-hardcoded-language.rs` with
  `scripts/hardcoded-language-allowlist.txt` (issue #659). New debt is blocked,
  stale rows must be pruned, `--write` regenerates. `check-tests-as-docs.rs`
  reuses this shape exactly: 398 loose-only tests exist today, and the number
  can only fall.
- **Warning band beside a hard limit** — `WORKFLOW_YAML_LIMIT` in
  `check-file-size.rs` (issue #812). `check-cache-budget.rs` uses the same
  two-tier reporting for its exempt buckets.
- **Constant read out of the library by the gate** — prevents the gate and the
  code it guards from drifting apart, as they had here.

## 7. Implemented Fix

| Requirement | Change |
| --- | --- |
| R960-1 | `scripts/check-file-size.rs`: `EXCLUDE_PATH_FRAGMENTS` reduced to `dev/log/`; `tests/unit/data_files.rs`: blanket `cache` exemption removed. The largest cached file, `data/cache/wikidata/lexeme/L3302.lino`, is 1347 lines, so the cap binds without a single split being needed today. |
| R960-2 | `scripts/check-cache-budget.rs`, wired into `.github/workflows/release.yml`. Records are counted by file stem (`Q1860.json` + `Q1860.lino` = 1). The cap is parsed from `src/translation/cache.rs`. |
| R960-3 | Documented exemption plus the no-orphan invariant described in §5. |
| R960-4 | `scripts/check-tests-as-docs.rs` + `scripts/tests-as-docs-allowlist.txt` (398 rows). `tests/unit/assistant_name.rs` converted as the worked exemplar. |
| R960-5 | `scripts/check-pull-request-link.rs`, run on every pull request with `PR_BODY`. |
| R960-6 | CONTRIBUTING.md § *Project Conventions* rules 12–16, § *Pull Request Process* steps 5–6, and `.github/pull_request_template.md`. |

## 8. Verification

- `rust-script --test scripts/check-cache-budget.rs` — 9 tests, including a
  bucket one record over the cap failing, a bucket exactly at the cap passing,
  an exempt bucket warning on overflow but failing on an orphan, and a test
  asserting the script's cap equals the library constant.
- `rust-script --test scripts/check-tests-as-docs.rs` — 9 tests, including
  `loose_only_test_is_flagged`, `exact_answer_assertion_passes`, and
  `commented_out_exact_assertion_does_not_count`.
- `rust-script --test scripts/check-pull-request-link.rs` — 9 tests, including
  `addresses_is_rejected` and
  `prose_mentioning_fix_without_a_reference_does_not_count_as_a_link`.
- Manual demonstration of the issue's clause (c), reproducible from the
  fixtures kept in `experiments/`:

  ```console
  $ rust-script scripts/check-pull-request-link.rs experiments/pr-body-loose.md
    The description links no issue with a GitHub closing keyword (...).
    line 3: `addresses #N` is not recognised by GitHub and will not close the issue on merge.
  exit=1
  $ rust-script scripts/check-pull-request-link.rs experiments/pr-body-exact.md
  Description closes its issue with a recognised GitHub keyword
  exit=0
  ```

- Ratchet demonstrated end to end: converting `tests/unit/assistant_name.rs` to
  exact answers made the check fail with "Stale allowlist rows (these tests are
  explicit now — prune them)" until `--write` regenerated the list, 399 → 398.
- `tests/unit/docs_requirements_issue_960.rs` asserts the documentation, the PR
  template, and the CI wiring all exist, so this case study cannot rot away from
  the code.
