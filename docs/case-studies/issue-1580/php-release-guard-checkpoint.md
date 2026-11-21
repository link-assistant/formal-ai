# Pinned PHP automatic-release guard

This checkpoint supports [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
It is an offline reproduction of the pinned workflow expression, with a candidate
condition, rather than an observed hosted release or an upstream repair.

The PHP template default head remains `3a034b27a56067971ca74dd4c74c7707056b215d`.
The complete 511-line release workflow was read. Its `auto-release` job declares
lint, test, build and preflight dependencies but explicitly checks only preflight
success. The status functions in its condition replace GitHub's implicit
`success()` prerequisite. An unsuccessful quality check therefore does not stop
that guard from admitting publication on a noncancelled main push.

The unmodified workflow is retained with its original license, Git blob
`d383bce663bc9c7de1513b951131c937864d7676` and SHA-256
`b1ea5a8b7a2c103e2bad5a92a36229ea56397642fb48db733af0db6d3dfd3899`.
The bounded expression model uses actual dependency names and result strings;
it translates the hyphenated preflight property to JavaScript bracket syntax.

The regression exercises all nine combinations of lint/test/build with failure,
cancelled and skipped results. Every combination is admitted by the original
guard and blocked by a disposable candidate requiring all three checks to
succeed. Preflight failures remain blocked; successful release remains admitted.
Pull-request, dispatch, wrong-branch and workflow-cancellation controls remain
blocked. The unchanged manual-release condition already requires all four
prerequisites to succeed. A further control checks implicit success gating and
shows why deleting only `always()` while retaining `cancelled()` is insufficient.

The focused and maintained entrypoint runs, original rejected gate selector,
corrected four-gate run, exact commands and hashes are retained under
`experiments/issue-1580/evidence/checks/php-release-guard/`. The final control is
verified in five focused and 43 maintained cases, without skips or failures. No PHP installation, dependency resolution or
native build is needed for this workflow-expression observation.

The fresh upstream inventory contains 26 issue/PR entries and eight pull requests.
The full bodies and feedback of release hardening PR #13 and latest merged PR #24
were read, including conversation, inline and review endpoints. Existing #26
reports failed Git observations and is separate. The distinct release-guard
report is filed as [PHP template #27](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/27),
with immutable executable evidence at `fda677ef9f25692b7b3823556f1349839a71f7a3`.
Reporting does not establish an upstream repair.

All 608 repository blobs are hash verified. The path taxonomy tags 92 root or
nonhistorical files (13,759 lines) and 516 historical or experimental files.
This taxonomy is not a CI activity boundary: the complete 159-line
`templates.yml` currently installs, tests, builds and audits four captured trees
under `docs/case-studies/issue-1/data/templates/`. Those source trees are current
CI inputs despite their historical paths. The conservative subtree inventory is
retained in `ci-input-classification.json`. The existing conservative reference
graph already contains all 221 files in those four captured subtrees; its explicit
working-directory context unknowns remain. Full dependency and semantic review
remain unfinished. The earlier path-only active/historical wording was incorrect
and is preserved in the supplied-helper archive. The PHP tree and broader issue
remain unfinished. Supplied scripts, conditions and reasoning receive zero
independent Formal AI authorship credit.
