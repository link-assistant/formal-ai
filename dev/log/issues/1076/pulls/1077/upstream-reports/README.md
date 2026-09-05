# Upstream reports filed from issue #1076

The issue asked to reuse the four
`link-foundation/*-ai-driven-development-pipeline-template` repositories as the
reference for CI/CD best practice, and the task description asked that defects
belonging to another project be reported there with a reproducible example, a
workaround and a code-level fix. Comparing this repository's pipeline against
the templates surfaced three defects in the templates themselves, and fixing
this pull request's own red build surfaced two more in `links-notation`, the
parser every `.lino` gate in this repository is validated with.

All five were reproduced locally before filing; the scripts and transcripts are
in `experiments/issue-1076/`.

| # | Defect | Filed | Body |
|---|---|---|---|
| 1 | `run-with-budget-warning.sh` counts poll iterations rather than elapsed time; a fractional `BUDGET_POLL_SECONDS` is an arithmetic error that leaves `elapsed` at 0, so the budget never expires | [rust#153](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/153) | `rust-template-budget-poll.md` |
| 2 | `run-with-budget-warning.sh` tracks liveness on the wrapper subshell, not the command, so a process that ignores SIGTERM is never escalated to SIGKILL and outlives the step | [js#164](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/164) | `js-template-termination-path.md` |
| 3 | `cache-to: type=gha,mode=max` with no `scope=` writes to buildx's default `buildkit` scope, so builds overwrite each other and the export crowds the shared 10 GB quota | [rust#154](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/154), [js#165](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/165), [python#66](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/66) | `templates-unscoped-gha-cache.md` |
| 4 | Links Notation has no comment syntax, so a `#` prose line is an ordinary link and one bare colon in it (`... a commit can break: two of the tests`) makes the whole file unparseable | [links-notation#301](https://github.com/link-foundation/links-notation/issues/301) | `links-notation-no-comment-syntax.md` |
| 5 | The Rust parser reports failures by `Debug`-printing the `nom` error -- no line, no column, and the entire unconsumed remainder as the payload -- while the JavaScript port of the same version reports `line`, `column` and what it expected | [links-notation#302](https://github.com/link-foundation/links-notation/issues/302) | `links-notation-rust-error-position.md` |

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
assumption. None of them was filed, because none reproduces upstream.

| Defect here | Checked | Result |
| --- | --- | --- |
| **D14** — an unquoted workflow `name:` containing ` #` is truncated at the YAML comment | every `name:` in all four templates' `.github/workflows/` and `.github/actions/` | none contains an unquoted ` #`; the templates do not embed issue references in names |
| **D15/D16** — a step-level `timeout-minutes:` larger than 70% of the job cap it must fire under | every step-level cap in all four templates | `rust` and `php` declare none; `js` (20 m under 30 m) and `python` (40 m under 60 m) are both at 66%, inside the share |
| **D10b** — `actionlint` run as a bare binary silently skips every `run:` block, because it delegates those checks to a ShellCheck that is not installed | how each template invokes actionlint (`.github/workflows/workflows.yml` in all four) | all four already use the container form — `uses: docker://rhysd/actionlint:1.7.7` — which bundles ShellCheck. The defect was this repository running the binary instead; the fix here is the templates' own shape, one version newer |

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

## Defects 4 and 5: what this repository did about them

Both were found the hard way. `data/meta/ci-gates/check-job-headroom.lino`, a
gate file added by this pull request, carried the comment `holds is the part a
commit *can* break: two of the tests parse the`, and
`data_files::lino_data_files_are_parseable_human_readable_and_bounded` failed on
it -- turning `Test (ubuntu-latest / full)` red on an edit that changed nothing
but prose. Finding the character cost a line-by-line bisection, because the
message (defect 5) named the file and then printed several hundred characters
of its own content.

- Defect 4 is worked around by writing ` -- ` instead of `: ` in `.lino` prose.
  52 checked-in `.lino` files carry `#` paragraphs, so this is a trap rather
  than a one-off; `lino_location::every_checked_in_gate_file_still_parses_line_by_line`
  holds the gate registry against it.
- Defect 5 is worked around by `lino_location::first_unparseable_lino_line`,
  which re-parses each line on its own and names the first that fails. The
  failing test now reports the path, the line number and the offending text
  before the original error, so the next writer reads the answer instead of
  bisecting.
  `experiments/issue-1076/repro-lino-comment-colon.sh` reproduces both defects
  against the Rust and the JavaScript implementation.
