# Upstream reports filed from issue #1081

Issue #1081 asks to find every false positive, false negative, warning and
error in this repository's CI/CD, to compare **all** files against the five
`link-foundation/*-ai-driven-development-pipeline-template` repositories, and
to file an issue upstream wherever the same defect exists in a template. The
task description added that each report must carry a reproducible example, a
workaround and a code-level fix.

Issue #1079 ran the same comparison one round earlier and filed eight reports
(`../../../1079/pulls/1080/upstream-reports/README.md`). Nothing below repeats
one of those. Three of the five defects here were found by comparing in the
direction #1079 did not finish -- template feature by template feature, asking
which template is *missing* a control the others have, rather than only
looking for this repository's bugs upstream. The fourth is a defect in a
template's own invariant check: a gate that both rejects correct
configurations and passes unbounded ones. The fifth is the one control none of
the five has, found by working through `CI-CD-BEST-PRACTICES.md` principle by
principle rather than by diffing the templates against each other.

Everything below was reproduced locally before filing, at the template commits
snapshotted in `../references/templates/*-template.HEAD`. Transcripts and
measurement tables are in `../analysis/`.

| # | Defect | Filed | Body |
|---|---|---|---|
| 1 | `actions/checkout` leaves the job's `GITHUB_TOKEN` in `.git/config` as `http.extraheader` for every later step, on 25 of 27 checkouts (js), 9 of 11 (php) and 12 of 13 (csharp), in jobs that never push. The templates already ship the audit that reports it -- zizmor's `artipacked` -- but every finding on a checkout that merely *omits* the input is Low confidence, and all five zizmor jobs set `min-confidence: medium`, so the gate is green by construction | [js#179](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/179), [php#8](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/8), [csharp#54](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/54) | `templates-checkouts-persist-credentials.md` |
| 2 | `release.yml` caps every job with `timeout-minutes` (7 of 7 php, 8 of 8 csharp) and no job observes the outcome. GitHub reports a job killed by its cap as **cancelled**, not failed, so the one failure mode `timeout-minutes` exists to catch is the one these two pipelines cannot report: the run goes grey, not red. The other three templates ship a terminal `pipeline-status` job over `scripts/check-pipeline-status.sh` | [php#9](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/9), [csharp#55](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/55) | `templates-missing-pipeline-status.md` |
| 3 | `scripts/run-with-budget-warning.sh` does not exist in the php and csharp templates and no step is wrapped (measured: rust 4 wrapped steps, js 6, python 4, php 0, csharp 0), so the job cap is the only timeout those pipelines have -- and by defect 2 the cap cannot fail a job. The other half of the same gap: report 2 is the missing observer, this is the missing deadline | [php#10](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/10), [csharp#56](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/56) | `templates-missing-execution-budgets.md` |
| 4 | `getStepBudgetSeconds` in `tests/ci-timeouts.test.js` -- the invariant that keeps every step budget expiring before its job cap -- returns both kinds of wrong answer. It sums budgets across steps whose `if:` conditions are mutually exclusive, so alternatives that never run in the same job are counted as a sequence (**false positive**: `test` measures 600s where the true worst case is 300s, 30s of apparent headroom against 330s of real headroom); and its regex matches only literal integers, so the `"$VAR"` budget form used by the rust and python templates is dropped without a word (**false negative**: a 900s budget equal to the whole 15-minute cap passes) | [js#180](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/180) | `js-template-budget-invariant-false-results.md` (patch: `js-template-budget-condition-grouping.patch`) |
| 5 | No template checks that it can publish before it builds. Every publishing credential is exercised for the first time by the step that uses it, after 30-85 capped minutes of `lint`/`test`/`build`: rust `CARGO_TOKEN`/`DOCKERHUB_TOKEN` at 50 minutes, js `NPM_TOKEN` at 30, python `DOCKERHUB_TOKEN` at 85, csharp `NUGET_API_KEY` at 55, and php's Packagist registration at 55 -- *after* the version tag has already been pushed. Principle 16 of `CI-CD-BEST-PRACTICES.md` ("Prove You Can Publish Before You Build") is unimplemented in all five, and the pre-push signal they do have is a login, which on Docker Hub answers a `pull,push` scope request with **200** and a pull-only `access` claim | [rust#167](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/167), [js#181](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/181), [python#77](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/77), [php#11](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/11), [csharp#57](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/57) | `templates-no-release-preflight.md` |

## Which templates each defect affects

Checked rather than assumed. Every count below is a measurement over the
snapshotted trees; the transcripts are `../analysis/template-artipacked-sweep.md`
and `../analysis/template-dimensions.md`.

| | rust | js | python | php | csharp |
|---|---|---|---|---|---|
| `actions/checkout` sites | 26 | 27 | 18 | 11 | 13 |
| of those, `persist-credentials: false` | 26 | **2 (defect 1)** | 18 | **2 (defect 1)** | **1 (defect 1)** |
| `artipacked` findings, auditor run directly | 2 | **25** | 1 | **9** | **12** |
| ... reported by the template's own zizmor job | 0 | **0** | 0 | **0** | n/a (no zizmor job, [csharp#53](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/53)) |
| every remaining finding's confidence | Low | Low | Low | Low | Low |
| exceptions carry a comment naming the push | yes | no | yes | no | no |
| ships `scripts/check-pipeline-status.sh` | yes | yes | yes | **no (defect 2)** | **no (defect 2)** |
| terminal `pipeline-status` job in `release.yml` | yes | yes | yes | **no (defect 2)** | **no (defect 2)** |
| jobs capped with `timeout-minutes` | all | all | all | 7 of 7 | 8 of 8 |
| ships `scripts/run-with-budget-warning.sh` | yes | yes | yes | **no (defect 3)** | **no (defect 3)** |
| steps wrapped in a budget | 4 | 6 | 4 | **0 (defect 3)** | **0 (defect 3)** |
| enforces budget < job cap in a test | no | **yes (defect 4)** | no | no | no |
| credentials probed before the build (`release-preflight`) | **no (defect 5)** | **no (defect 5)** | **no (defect 5)** | **no (defect 5)** | **no (defect 5)** |
| capped build minutes before the first credential use | 50 | 30 | 85 | 55 | 55 |

The `artipacked` rows are the argument for defect 1 being a gate defect rather
than a style preference: the number the auditor produces and the number the
pipeline reports differ by 25, 9 and 12, and the reason is one line of
configuration, not the audit's opinion. rust's 2 and python's 1 are jobs that
genuinely push; both templates already annotate them.

The last row is why defect 4 is filed against js alone. It is the only template
that checks the budget-vs-cap invariant at all -- the defect is a cost of being
ahead, and the two templates whose budget syntax the check cannot read (rust
and python, which pass the budget as `"$VAR"`) would inherit the false negative
the moment they adopt it.

## Cross-references filed on the issues themselves

- php#9 ↔ php#10 and csharp#55 ↔ csharp#56: the missing observer and the
  missing deadline are independent fixes that compose into one gap, so each
  issue names the other and says which half it is.
- js#180 carries the verified patch as a comment
  ([#180 comment](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/180#issuecomment-5571258697)),
  so the fix can be applied without transcribing it out of the body.

## Corrections to the #1079 round

`../../../1079/pulls/1080/analysis/artipacked-sweep.md` states that "all five
`link-foundation/*` templates set `persist-credentials: false`". That claim is
**refuted** by the measurement in `../analysis/template-artipacked-sweep.md`:
it was a presence-of-string check, not a coverage check. The string is present
in all five trees; the *coverage* is 26/26, 2/27, 18/18, 2/11 and 1/13. Defect
1 above is the corrected finding.
