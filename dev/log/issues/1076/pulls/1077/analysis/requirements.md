# Issue #1076 — every requirement, enumerated

Source: <https://github.com/link-assistant/formal-ai/issues/1076> (author `konard`,
opened 2026-09-05T09:36:12Z, label `bug`, no comments as of this writing) plus the
instructions carried in the pull-request task description.

The issue body is short; most of its weight is in the title and in four sentences
after the run table. Each is broken out below with the evidence that decides
whether it is met. `R` = requirement from the issue, `T` = requirement from the
task description accompanying it.

---

## R1 — "Check for all false positives, false negatives, warnings and errors in CI/CD and fix them all"

The title, and the only requirement stated as an imperative over the whole system
rather than over a specific artifact. It names four distinct classes, and they are
not synonyms:

| Class | Meaning here | Found |
| --- | --- | --- |
| **False negative** | CI is green while something is wrong | D1, D3, D8, D10b, D16 |
| **False positive** | CI is red or noisy while nothing is wrong | D12, D13, D15, D17 — all four introduced by this PR's own fixes and caught before merge; D4 was investigated in the existing pipeline and withdrawn |
| **Error** | a job that fails or is cancelled | D1 (Coverage, run 33955786082) |
| **Warning** | an annotation nobody acts on | D2, D3, D14 |

The distinction matters for D1 specifically. The `Coverage` run in the issue's
table is *not* a failure — it is `cancelled`, because a job that exceeds
`timeout-minutes` is cancelled rather than failed, and a cancelled job is not a
red check. So D1 is simultaneously an error (the job did not do its work) and a
false negative (the pipeline did not report that it had not). See README §3.

**Status: addressed.** Defect register in README §4, D1–D17 with D4 withdrawn.

## R2 — "Use all the best practices from CI/CD templates"

Qualified in the issue by "(check full file tree to compare for all GitHub
workflow and CI/CD scripts file)" — a *file-tree-wide* comparison, not a spot
check — against four named templates:

- `link-foundation/rust-ai-driven-development-pipeline-template`
- `link-foundation/js-ai-driven-development-pipeline-template`
- `link-foundation/python-ai-driven-development-pipeline-template`
- `link-foundation/php-ai-driven-development-pipeline-template`

**Status: comparison complete** (`analysis/template-diffs/`, 30 diffs across four
templates, inventory in `file-inventory.txt`). Findings applied: D10 (zizmor
config + `workflows.yml` audit job, the one template file with no counterpart
here).

## R3 — "if the same issue is found in template report issue also in templates"

A conditional obligation: defects that also exist upstream must be reported
upstream, not merely fixed here.

**Status: three defects reported, five issues filed** across the four templates.
The full index, including which template each defect affects and how it was
reproduced, is `upstream-reports/README.md`; the bodies as filed are the three
`*.md` files beside it. All three were found by comparing this repository
against the templates, not the other way round:

1. `run-with-budget-warning.sh` in the rust template counts loop iterations
   instead of reading `$SECONDS`, so a non-integer `BUDGET_POLL_SECONDS` makes
   the budget never fire. Reproduced: the template's script exits 0 after the
   full 10s command; this repository's exits 124 at 2s.
   → [rust#153](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/153)
2. The js template tracks liveness on the wrapper subshell rather than the
   command, so a process that ignores SIGTERM is never escalated to SIGKILL and
   outlives the step. Reproduced: js leaves one survivor after 3s; rust and
   python leave none.
   → [js#164](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/164)
3. `cache-to: type=gha,mode=max` with no `scope=` lets one image build evict
   every other consumer from the shared 10 GB cache quota. Measured here: 48
   `buildkit-blob-*` entries = 4.91 GB = 42.9% of quota. Present at four sites
   across three templates, so it was filed against each.
   → [rust#154](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/154),
   [js#165](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/165),
   [python#66](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/66)

Two candidates were checked against the templates and **not** filed, because the
defect turned out to be local rather than shared: the `kill "${VAR:-0}"`
self-signalling shape (D12) appears nowhere in the four templates' `kill` sites,
and zizmor's default `inputs: .` (D13) is harmless in a repository that does not
archive other projects' workflows, which none of the templates does.

## R4 — "We should compare all files, so we don't have more CI/CD errors in the future"

Restates R2 and adds the *purpose*: prevention, not just repair. Read together
with R1 this is what makes tests part of the deliverable rather than optional —
a fix with no test is a fix that regresses silently.

**Status: addressed.** `tests/unit/ci-cd/issue_1076.rs`, eleven tests, each
pinning one measured property rather than the shape of the current file. Two of
them exist because the audit built for D5 found defects nothing else was looking
for: `no_declared_name_is_truncated_by_an_unquoted_comment` (D14, four workflow
names silently cut at a YAML comment) and
`every_step_level_timeout_can_fire_before_its_job_cap` (D16, the budget
mechanism issue #1017's sweep does not read).

## R5 — "Follow the CI/CD best practices collected in hive-mind `docs/CI-CD-BEST-PRACTICES.md`"

Fifteen numbered principles. Compliance audited one by one in
`analysis/best-practices-audit.md`; the document itself is archived at
`references/hive-mind-CI-CD-BEST-PRACTICES.md` so this analysis stays readable
against the version that was current when the issue was filed.

## R6 — "plan and execute everything in this single pull request … until each and every requirement is fully addressed"

Scope constraint: one pull request (#1077), no follow-up issues used as an escape
hatch for work that belongs here.

---

## Requirements from the task description

| # | Requirement | Where satisfied |
| --- | --- | --- |
| T1 | Download all logs and collected data into `dev/log/issues/1076/pulls/1077` | this folder: `runs/`, `ci-logs/`, `annotations/`, `analysis/`, `references/` |
| T2 | Deep analysis: timeline, root cause per problem, solution plans | README §2 (timeline), §3 (causal chain), §4 (register), `analysis/solution-plans.md` |
| T3 | Search online for additional facts, and for existing components that solve the same problem | `analysis/online-research.md` |
| T4 | If data is insufficient for a root cause, add debug output and a verbose mode, default off | README §7. `scripts/report-runner-capacity.sh` and the cache-outcome step in `.github/actions/cache-cargo-registry`, both gated on `FORMAL_AI_CI_VERBOSE`; the script prints nothing and exits 0 when unset |
| T5 | Report issues to other projects where the defect is theirs, with reproducible examples, workarounds and code-level fix suggestions | `upstream-reports/` (see R3) |
| T6 | Apply each requirement across the *entire* codebase, not just the first site | the seven-site cargo-cache migration, the five-site docker cache scoping, the four-site YAML name quoting (D14) and the two-site release Docker budget (D15, `auto-release` *and* `manual-release`); verified by sweeps over every workflow rather than by inspection |

---

## What is deliberately not in scope

Recorded so the omissions are visible rather than silent:

- **The 34 `self-repository` zizmor findings.** GitHub's `uses: $/...` syntax is
  two months old, and 21 of the 34 sites are in `release.yml` on paths that only
  execute when a release is published — which a pull request's CI cannot
  exercise. Suppressed with the reasoning recorded in `.github/zizmor.yml`, the
  same call the rust template makes for its own single instance.
- **The `dangerous-triggers` finding on `desktop-release.yml`.** Analysed and
  found not to be the insecure shape of the pattern; reasoning recorded at the
  site rather than silently filtered. See the header comment in that file.
