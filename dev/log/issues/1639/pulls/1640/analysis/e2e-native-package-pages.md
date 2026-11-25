# Main run 45636781871: E2E, native observations, package verify, Pages deploy

The per-job logs are in `../ci-logs/cicd-jobs/`. Line numbers below refer to those files.
The prebuilt release binary from the same run (`/tmp/ci-art-1639/bin/formal-ai`) was used for every
reproduction. Nothing was rebuilt locally.

The PR lane (#1191) is `fast-pr-lint` plus `native-type-check`. `fast-pr-lint` runs only
`node scripts/check-*.mjs`; the cheap entries live in `scripts/check-fast-governance.mjs`, which
allows 60s in total and 15s per check. Neither job runs `node --test`. So the JS regression tests
added below catch these failures locally and in the main-only lanes, but not before merge, unless one
of the proposals in "Narrowest pre-merge check" is adopted.

## 1. E2E computer-use-generalization: `notes_count_and_pack explicit plan differs from local synthesis`

- **Symptom.** `E2E_..._computer-use-generalization_.log` line 1351 has the node `Error: notes_count_and_pack
  explicit plan differs from local synthesis`. Lines 1354, 1366 and 1370 then show the
  `::error title=issue #707 ...::local reference rejected for notes_count_and_pack` exit.
- **Reproduction.** Run `/tmp/ci-art-1639/bin/formal-ai --silent computer-use --agent-mode --confirm-effects --from
  <corpus> --prompt <prompt>` and then `... --plan --prompt <prompt>`. In the outcome,
  `outcome.plan.steps[].primitive` is the serde variant name (`"FsWrite"`), but the explicit plan and
  `outcome.steps[].primitive` use the dotted wire name (`"fs.write"`). The two arrays never compare equal.
- **Introducing commit.** 66169b8fa, in PR #1614, which merged as 38fb0750d. That commit made the harness
  compare the explicit plan against `outcome.plan.steps`. E2E was skipped on main until run 45490077033
  (1c109025c), so this is the first main run that exercised the change.
- **Root cause.** The harness compares the two naming schemes directly.
  `verify_issue_707_generalization.mjs` expects variant names in `expected.jsonl` and maps them through
  its `PRIMITIVES` table.
- **Fix.** `experiments/agent_cli_e2e/run_issue_707_generalization.sh` now compares the dotted explicit
  names against `outcome.steps[].primitive`, and checks that the executed steps and the plan steps have the
  same length. It still writes the variant-name `steps` from `outcome.plan.steps`, as the verifier expects.
- **Verification.** The reference-derivation loop, run with the prebuilt binary, produced all 12
  `expected.jsonl` entries. Every step maps through the verifier's `PRIMITIVES` table, with 0 unmapped.
  The record and replay phases need the real `agent` CLI and were **not** run locally.
- **Narrowest pre-merge check.** The comparison is inside an inline `node -e` in a shell harness, so no
  static check can see it. The narrowest gate is to move the `node -e` body into
  `experiments/agent_cli_e2e/issue_707_reference.mjs`, with a committed outcome fixture of one case, and
  call it from a fast-governance entry. This step is proposed, not done.

## 2. Main E2E agent-CLI job: exit 1 at line 2431

- **Symptom.** In the main E2E log, line 2430 is `!! expected ≥5 chat completions, got 0 (loop stalled?)`
  and line 2431 is `Process completed with exit code 1`.
- **Reproduction.** Start `formal-ai serve` with `FORMAL_AI_TRACE_REQUESTS=1` alone, and the server log
  contains no `[trace] POST` header lines (0 POSTs). Add `FORMAL_AI_LOG_LEVEL=trace` and the same
  exchange shows 2 POSTs.
- **Introducing commit.** ec6ade5cd (#1592), merged as 851e15863. It made the request header a
  `trace`-level event, while `FORMAL_AI_LOG_LEVEL` defaults to `debug`. The first failing run was 44168670977.
- **Root cause.** The harnesses enabled the request trace but not the level that emits it.
- **Fix.** `FORMAL_AI_LOG_LEVEL=trace` now appears next to every `FORMAL_AI_TRACE_REQUESTS=1`. That covers
  24 `experiments/agent_cli_e2e/*.sh` files plus `experiments/issue_714_agentic_mode/run_report_e2e.sh`;
  the docker variant passes it with `-e`. A new test in `experiments/agent_cli_e2e/server-log-evidence.test.mjs`
  scans every harness launch and fails if the level is missing.
- **Verification.** The POST count went from 0 to 2 with the prebuilt binary, and the scan test passes.
  This job stopped at its first assertion, so **the later steps in the E2E job have never run on
  main since 851e15863. They may surface further failures.**
- **Narrowest pre-merge check.** The scan test is static and under 50 ms. As a fast-governance entry, it
  would have rejected #1592's harness/server mismatch on the PR.

## 3. native-response-observations: selected 28, accepted 27

- **Symptom.** Line 2159 of the native-observations log is `{"selected":28,"accepted":27,...}` and line
  2160 is the exit 1.
- **Reproduction.** In the observations artifact, the missing case
  `issue_823_recursive_learning.rs::the_live_solver_recalls_an_approved_lesson_instead_of_rederiving_it`
  wrote its records under child PID 3343, not under the parent PID 3341 that the capture harness bound.
- **Introducing commit.** 15ae736ab, in PR #1637. It moved that test's solver calls into an
  `offline_replay::in_child*` re-execution.
- **Root cause.** The capture binds `native-<pid>.jsonl` to the process it launched, so records written
  by a re-executed child are never found.
- **Fix.** The case was removed from `data/meta/native-response-capture-cases.json`. A test in
  `rust/tests/web/native-test-module-graph.test.mjs` now fails if any committed case body calls
  `offline_replay::in_*(`.
- **Verification.** The guard test passes with the change and fails if the case is restored.
- **Narrowest pre-merge check.** The same static scan, run as a fast-governance entry, would have
  flagged PR #1637, because it touched a selected case.

## 4. Auto_Release verify-package: `aider issue report failed`

- **Symptom.** In `Auto_Release_auto_verify-package.log`, lines 4038-4087 show `aider issue report failed`.
  The report menu is printed on **one line**, followed by
  `AssertionError ... null !== 0 at aider-report.mjs:35:10`; a `null` exit code means the deadline killed
  the process. Line 4469 is `Process completed with exit code 1`. Aider was the only client that
  failed in this run.
- **Reproduction.** Run `formal-ai serve` with `FORMAL_AI_AGENT_MODE=1` and ask "Report issue". The server
  answers with the question, then `"\n"`, then every option joined by two spaces:
  `1. Harness log — …  2. Server log — …  3. GitHub issue …`. The scripted turn `/^3\. GitHub issue/`
  never matches, so the script waits until its deadline.
- **Introducing commit.** 8711d61f2, a merge dated 2026-11-21. Both of its parents (23a78fd4d and
  97fe40bc1) put each option on its own line; the merge resolution switched to `"  "` in
  `rust/src/agentic_coding/report_issue.rs` (around line 252) and in `js/agentic/report_issue.mjs`
  (`.join('  ')`, around line 171). It reached main through PR #1581 (3888187b7).
  - verify-package passed on 45053448033, 45118482226 and 45275490842.
  - It did not run on 45463430673.
  - It timed out on 45490077033.
  - On 45549993600 several clients failed; 8404d0aa8 fixed that port reuse.
  - On 45636781871 only Aider failed.
- **Root cause.** The scripted Aider turns anchored each choice at the start of a line, but the menu now
  prints every choice on a single line. The server's own numbered-reply parsing
  (`starts_with("{question}\n1. ")`) is unaffected.
- **Fix.** The turns moved to the new file `experiments/issue_1609_client_reports/aider-report-turns.mjs`,
  and each choice is now matched anywhere on a line (`(?:^|\s)3\. GitHub issue\b`).
  `aider-report.mjs` imports them. On a deadline kill, its assertion now names the line it was still waiting for.
  `rust/tests/web/issue-1609-client-reports.test.mjs` holds the same matchers to the server's real
  destination, scope and submission text.
- **Verification.**
  - Against the live prebuilt server, the old regex fails on turns 1 and 2, and the new one matches
    all three. The server accepted `3` and then `1`, and emitted
    `formal-ai report create-issue --session '…' --source both …`.
  - `issue-1609-client-reports.test.mjs` reports 20 tests: 18 pass, 0 fail, 2 skipped. With the old
    `^3\.` regex restored temporarily, it fails with `… answers no line of …`.
  - `default-branch-build-policy.test.mjs` passes 197/197.
  - A real Aider run was **not** performed locally.
- **Narrowest pre-merge check.** The test is already in
  `data/benchmarks/fast-pull-request-javascript-tests.json`, but that list is only an optional local
  inventory. Either run that one file in `fast-pr-lint`, or add a fast-governance entry that renders the
  menu (this requires the agentic host to be installed, because calling `planTextReportFlow` alone throws
  `agentic_host_not_installed`) and applies `aiderReportTurns`.

## 5. Deploy Web App to GitHub Pages: HTTP 500 from `POST /pages/deployments`

- **Symptom.** In `Deploy_Web_App_to_GitHub_Pages.log`, lines 3-4 show artifact_id 15040052161 and
  source_sha 513e4d9542a181cc4925105f44ad6e3648974e27. Lines 25-27 show three 500s, at 16:15:23Z,
  16:15:35Z and 16:16:00Z, with request IDs
  0849:365FDC:701366A:16013FAD:6B07099A, 0848:183DA4:52DE51B:103920E0:6B0709A5 and
  0848:183DA4:52E9190:103B35C0:6B0709BB.
- **Payload comparison.** Upstream `actions/deploy-pages@v5` is tag sha 368f825. Its `createPagesDeployment` sends
  `{artifact_id, pages_build_version: GITHUB_SHA, oidc_token}`, where `oidc_token` comes from
  `core.getIDToken()` with no audience, and adds `preview` only when it is set. Our
  `.github/actions/deploy-pages/deploy.mjs` sends exactly that shape: a numeric `artifact_id`, the source
  commit SHA, and the OIDC token. `rust/tests/web/pages-deployment.test.mjs` (35/35 pass) already pins it.
  Artifact 15040052161 is the run's own `github-pages` artifact (33 MB). The Pages config is
  `build_type: workflow` and public.
- **Timeline** (from the read-only `gh api` deployments list):
  - Deployment 8013639816, from 966f697fd in run 45275490842, used upstream v5 with the commit SHA. It succeeded.
  - f8718c44e (2026-11-20) sent a synthetic sha256 build version. Deployments 8042452580 (run 45490077033)
    and 8049330367 (run 45549993600) then failed with **404**. That was our payload.
  - 14054acac (2026-11-25) restored the commit SHA. Deployment 8062791916 (run 45636781871) then failed
    with **500**.
- **Root cause: a GitHub outage, not our payload.** githubstatus incident `qn8mfv6q5hlx`, "Investigating
  degraded experiences across GitHub products", was critical from 15:55:19Z to 16:52:16Z on 2026-11-25.
  It covered API Requests, Actions and Pages, among other components.
  - 16:11:14Z: "Pages is experiencing degraded availability".
  - 16:14:11Z: "high database load".
  - 16:28:29Z: Pages mitigated.

  All three 500s fall inside the degraded window. The full record is in
  `githubstatus-incident-qn8mfv6q5hlx.json`.
- **Fix.** None is needed in the payload. Re-run the deploy job. Optionally, lengthen the retry backoff:
  it is now 3 attempts at 10s and 20s, about 35s in total, against a degradation that lasted 17 minutes.
- **Verification.** The payload matches upstream and the 500s match the outage; the evidence above
  supports both. The 404 period was our own regression, but only a live POST reveals it, so no pre-merge
  check could catch it. The existing payload-shape test is the closest guard.

## W3. Generalization upload step: `No files were found`

- **Symptom.** Line 1436 of the generalization job log is
  `##[warning]No files were found with the provided path: /tmp/formal-ai-serve-*.log …`. The step sets
  `EVIDENCE_DIR=/tmp/formal-ai-issue-707-generalization-evidence`, and the upload glob
  `/tmp/formal-ai-issue-*-evidence` in `.github/workflows/agent-cli-e2e.yml` matches it. But the
  directory was still empty: `expected.jsonl` is copied only after the reference loop, and `fail()`
  exits through the EXIT trap, which ran `rm -rf "$WORKDIR"` first.
- **Fix.** `run_issue_707_generalization.sh` now checks the exit status in `cleanup()`. On a non-zero
  exit it waits for the server to stop, then copies the workdir's `*.log`, `*.err`, `*.local.json`,
  `*.txt` (explicit plans), `*.jsonl` and `opencode.json` into `$EVIDENCE_DIR/failure/`, keeping the
  relative paths. The corpus and the `memory.lino` stores are left out.
- **Verification.** With a stub `BIN` that fails, the script exits 1 and leaves
  `failure/{expected.jsonl, notes_count_and_pack.local.err, notes_count_and_pack.local.json, publisher.log}`.
  No stray fixture publisher is left running. The workflow needs no change.
