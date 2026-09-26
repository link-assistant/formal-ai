# Formal AI coverage-budget verification sessions (issue #1149)

External Agent CLI, local `formal-ai/formal-ai` model at
`formal-ai/0.351.0`, 2026-09-27. The server binary is
`/tmp/formal-ai-target-incr/debug/formal-ai`, the same build verified for
PR #1144, reused for PR #1146 and PR #1148; PR #1150 touches only a
workflow file and a changelog fragment, so the served model behavior is
unchanged. No hosted model participated.

## Context

Main run 36269286808 (merge of PR #1146, `9e673a1dc`) burned the full
4200-second coverage budget with every test passing and no hanger, while
the identical tree on branch run 36266423193 (`f3948e0c7`) finished the
same lane green in 2574s the same day — a contended host, not a slower
suite. PR #1150 raises `TEST_BUDGET_SECONDS` to 5400 together with the
job timeout to 135 minutes, keeping the 66.7% budget-to-cap ratio
issue #1017 enforces, and extends the job's history comment with the
event. The sessions below had the local Formal AI model read the fixed
`.github/workflows/coverage.yml`.

The file was copied to each session workspace's root as `coverage.yml`
(the same relocation-to-workspace pattern the #1148 sessions used for
`scripts/*.rs`): a first batch of three sessions prompted with the
in-repo path `.github/workflows/coverage.yml` never issued a read — the
workspace file listing hides dot-directories, so the model could not
locate the file — and that batch was replaced and is not counted.

## Sessions

All three sessions opened the file ("Let me open coverage.yml and read
what it says.") and returned its contents, which carry every marker of
the fix: `TEST_BUDGET_SECONDS: 5400`, `timeout-minutes: 135`, the
`36269286808` run id, and the `66.7%` ratio sentence.

1. `ses_f204a0c83ffeyyxp0r8kBAeBXm` — "Read the file coverage.yml in
   this workspace and quote the value assigned to TEST_BUDGET_SECONDS
   in it."
2. `ses_f2049fc7fffeJtJpEepy6SBe6t` — "Read the file coverage.yml in
   this workspace and quote the timeout-minutes value of the Code
   Coverage job."
3. `ses_f2049ee68ffeEoj2304IXqGxXy` — "Read the file coverage.yml in
   this workspace and quote the comment sentence that mentions run
   36269286808."

All sessions ended `hasError: false`; none produced a prose synthesis
beyond the presented contents, so the mechanical confirmation is the
human-side verification recorded in issue #1149 and PR #1150: the
workflow passes `actionlint` locally, `rust-script
scripts/check-changelog-fragment.rs` passes on the committed diff, and
the branch's own Coverage run exercises the new budget in CI.
