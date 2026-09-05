# Upstream reports filed from issue #1076

The issue asked to reuse the four
`link-foundation/*-ai-driven-development-pipeline-template` repositories as the
reference for CI/CD best practice, and the task description asked that defects
belonging to another project be reported there with a reproducible example, a
workaround and a code-level fix. Comparing this repository's pipeline against
the templates surfaced three defects in the templates themselves.

All three were reproduced locally before filing; the scripts and transcripts are
in `experiments/issue-1076/`.

| # | Defect | Filed | Body |
|---|---|---|---|
| 1 | `run-with-budget-warning.sh` counts poll iterations rather than elapsed time; a fractional `BUDGET_POLL_SECONDS` is an arithmetic error that leaves `elapsed` at 0, so the budget never expires | [rust#153](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/153) | `rust-template-budget-poll.md` |
| 2 | `run-with-budget-warning.sh` tracks liveness on the wrapper subshell, not the command, so a process that ignores SIGTERM is never escalated to SIGKILL and outlives the step | [js#164](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/164) | `js-template-termination-path.md` |
| 3 | `cache-to: type=gha,mode=max` with no `scope=` writes to buildx's default `buildkit` scope, so builds overwrite each other and the export crowds the shared 10 GB quota | [rust#154](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/154), [js#165](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/165), [python#66](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/66) | `templates-unscoped-gha-cache.md` |

## Which templates each defect affects

Checked rather than assumed -- the three budget wrappers differ:

| | rust | js | python | php |
|---|---|---|---|---|
| has `scripts/run-with-budget-warning.sh` | yes | yes | yes | no |
| deadline clock | `elapsed += POLL_SECONDS` (**defect 1**) | `SECONDS=0` (correct) | `started=$SECONDS` (correct) | n/a |
| grace-loop clock | `waited += 1` (correct) | `waited += poll_seconds` (**defect 2b**) | `grace_deadline=$((SECONDS + grace))` (correct) | n/a |
| SIGKILL escalation | correct | **defect 2a** | correct | n/a |
| unscoped `cache-to: type=gha` | 2 sites (**defect 3**) | 1 site (**defect 3**) | 1 of 2 sites (**defect 3**) | no container build |

Measured with `experiments/issue-1076/cases2.sh` (2s budget, 3s grace, child
ignores SIGTERM): js exits after 3s leaving 1 survivor; rust waits the full 5s
and python 6s, both leaving 0.

## Defects found here that the templates do **not** have

Checked in the same direction, so the absence is a measurement rather than an
assumption. Neither was filed, because neither reproduces upstream.

| Defect here | Checked | Result |
| --- | --- | --- |
| **D14** — an unquoted workflow `name:` containing ` #` is truncated at the YAML comment | every `name:` in all four templates' `.github/workflows/` and `.github/actions/` | none contains an unquoted ` #`; the templates do not embed issue references in names |
| **D15/D16** — a step-level `timeout-minutes:` larger than 70% of the job cap it must fire under | every step-level cap in all four templates | `rust` and `php` declare none; `js` (20 m under 30 m) and `python` (40 m under 60 m) are both at 66%, inside the share |

## What this repository did about the same defects

- Defect 1 does not apply: `scripts/run-with-budget-warning.sh` here already
  reads `$SECONDS` (`started=$SECONDS`, `elapsed=$((SECONDS - started))`) and
  enforces correctly at `BUDGET_POLL_SECONDS=0.5`.
- Defect 2 does not apply: the command is backgrounded directly, so `kill -0`
  tracks the command rather than a wrapper.
- Defect 3 **did** apply and is fixed in this PR: all five `cache-from`/`cache-to`
  sites in `release.yml` now carry `scope=docker-image`, and
  `issue_1076::container_build_caches_are_bounded_and_scoped` fails the build if
  a new unscoped export appears.
