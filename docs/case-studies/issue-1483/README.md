# Issue #1483: from inspection to a checked edit

Issue [#1483](https://github.com/link-assistant/formal-ai/issues/1483) reports that a
work-item session read evidence and then stopped (`planned_not_executed`) without an
edit, a check or a named reason. This folder holds the evidence for the fix; the
offline tests that guard it are listed at the end.

Every result below says how it was produced. **Nothing here is a hosted Hive Mind
`solve` run against GitHub**; see [What is not done here](#what-is-not-done-here).

## The ten planned-not-executed fixtures

The comment of 2026-11-10 names ten Hive Mind drafts whose sessions ended
`planned_not_executed`. Their issue titles and bodies were read through the REST API
on 2026-11-15 and frozen in
[`data/benchmarks/fixtures/work-item-1483/issues.json`](../../../data/benchmarks/fixtures/work-item-1483/issues.json).
[`experiments/issue-1483/session-outcome.mjs`](../../../experiments/issue-1483/session-outcome.mjs)
runs each one to its end. It uses the ordinary Hive Mind prompt
([`hive-mind-prompt.txt`](../../../data/benchmarks/fixtures/work-item-1483/hive-mind-prompt.txt)),
this checkout's planner and a disposable git worktree, and executes every tool call
for real. The `gh` read is answered from the fixture; network fetches are answered
as unavailable.

```
node experiments/issue-1483/session-outcome.mjs all          # ~7 minutes, all ten at HEAD
node experiments/issue-1483/session-outcome.mjs 1455 --at 3c592625d
```

Output: [`fixture-replay/session-outcomes.log`](fixture-replay/session-outcomes.log),
plus one full record per issue in `fixture-replay/issue-<n>.json` (calls, report,
machine record, diff).

| Issue | Draft | Ends with | Named gap / remaining work (from the record) |
|---|---|---|---|
| 1235 | 1236 | gap | `RequestedInterfaceAbsent`: `create-interface:assessLegalContext` |
| 1330 | 1331 | gap | `RequestedInterfaceAbsent`: `create-interface:sameMeaning`, `create-interface:same_meaning` |
| 1364 | 1369 | gap | `UnresolvedReferences`: the referenced paths do not exist in the checkout |
| 1395 | 1397 | gap | `AttachmentEvidenceUnavailable`: the screenshots are listed, not described |
| 1396 | 1398 | gap | `AttachmentEvidenceUnavailable` |
| 1438 | 1439 | gap | `ExternalEvidenceUnavailable`: the linked page could not be fetched |
| 1446 | 1448 | gap | `NoChangeRecipe`: `change-recipe-unavailable:rust/src/solver.rs`, … |
| 1455 | 1456 | **edit + check** | guide edit to `docs/associative-tech-stack.md`; its checks pass. Partly done (see below) |
| 1483 | 1495 | gap | `NoChangeRecipe`: `change-recipe-unavailable:data/benchmarks/self-coding-ci-observations.json`, … |
| 1548 | 1549 | gap | `RelatedEvidenceUnread`: the issue only links related pages |

No session ends in "repository work not executed" any more. Each one either edits
and checks, or names the narrowest gap it leaves. Every report is in English, the
language of the issue bodies. Issues 1330 and 1396 used to be reported in Russian;
the report now applies the planner's prose-language rule from #1557 (`workItemLanguage`
/ `work_item_language` in `general_planner`).

## #1455 on its pre-fix checkout

#1455 asks for documentation of the command-stream, web-capture and web-search
integrations. It was fixed by PR #1456; the checkout before that fix is `3c592625d`.

### 1. Planner replay

`session-outcome.mjs 1455 --at 3c592625d` (last two lines of the log above, and
[`hive-replay-1455.log`](hive-replay-1455.log) /
[`hive-replay-1455-record.json`](hive-replay-1455-record.json)). The session:

- reads `rust/Cargo.toml`, `package.json`, `js/web-search-component-entry.js` and the guide;
- appends manifest rows and call excerpts to `docs/associative-tech-stack.md`;
- reads the file back;
- stops at `UsefulCandidateReadbackVerified`, reported as **Partly done**.

### 2. Hive Mind `--tool agent --model formal-ai` leg, run locally

[`experiments/issue-1483/hive-agent-local.mjs`](../../../experiments/issue-1483/hive-agent-local.mjs);
artifacts are in [`hive-agent-1455/`](hive-agent-1455/).

What is real:

- Hive Mind 2.39.10's own prompt builders (`agent.prompts.lib.mjs`), `mapModelToId`,
  `buildAgentArgs` and `resolveFormalAiToolExecution`;
- the native `agent` CLI (`--model formalai/formal-ai`);
- this branch's JavaScript Formal AI server (0.355.34).

What is replaced: only `solve`'s GitHub orchestration (clone, branch, draft PR,
push, comments). The workspace is a local clone at `3c592625d`, and `gh issue view`
is answered from the frozen fixture.

The prompt carries only the issue URL; the script fails if the task text appears in
it. No completed solution is supplied.

- **First run** ([`first/`](hive-agent-1455/first/)):
  - **Partly done**, stopped at `UsefulCandidateReadbackVerified`.
  - Checks `task-claims`, `source-freshness-and-exact-postimage`,
    `source-anchored-facts` and `preimage-preserved` passed.
  - Still open: `independent-integration-semantics`, `consumer-discovery` and two
    `*-unverified` items. Requirements 5–7 (boundaries, the wider ecosystem, the
    inventory in another repository) are listed as not yet verified.
  - Diff: [`first/workspace.diff`](hive-agent-1455/first/workspace.diff).
- **Repeat run on the committed first diff** ([`repeat/`](hive-agent-1455/repeat/)):
  - **Partly done already**, stopped at `VerifiedNoOp`; `source-bound-noop` passed.
  - No file changed.

### 3. Independent check of the guide change

[`experiments/issue-1483/check-1455-guide-change.mjs`](../../../experiments/issue-1483/check-1455-guide-change.mjs)
shares no code with Formal AI:

- it reads the pre-fix commit with `git show`;
- it parses `rust/Cargo.toml` line by line and `package.json` with `JSON.parse`;
- it requires every documented row to match its manifest's version, section, target
  and options;
- it requires every "Imported X from P is called in F" claim to match a real import,
  with the quoted call present verbatim in F;
- it requires the diff to only append to the guide;
- it requires the three components named in the issue to be documented.

```
node experiments/issue-1483/check-1455-guide-change.mjs --at 3c592625d \
  --diff docs/case-studies/issue-1483/hive-agent-1455/first/workspace.diff
```

Result: **11/11 checks passed**
([`first/independent-check.txt`](hive-agent-1455/first/independent-check.txt)).
A tampered diff fails three of the checks.

## Claude and Codex adapters, without an LLM service

[`experiments/issue-1483/hive-prompt-sessions.mjs`](../../../experiments/issue-1483/hive-prompt-sessions.mjs)
→ [`hive-prompt-sessions.log`](hive-prompt-sessions.log). The same controller is
driven through the Anthropic Messages adapter (Claude) and the OpenAI Responses
adapter (Codex). It uses the ordinary Hive Mind system and user prompts
([`hive-claude-*.txt`, `hive-codex-*.txt`](../../../data/benchmarks/fixtures/work-item-1483/)),
which carry only the issue URL.

| Adapter | First run | Repeat |
|---|---|---|
| Claude | `gh issue view` → Read → Edit → Read → independent `node` check: `UsefulChangeVerified`, `src/ceiling.mjs` changed | `VerifiedNoOp`, no change |
| Codex | same sequence, `UsefulChangeVerified` | `VerifiedNoOp`, no change |

The report says **Partly done**. The issue title, "Ceiling check excludes the
maximum", restates the defect without a bound contract, so it stays an open
requirement rather than being claimed. When the same repair is given as a direct
task, the report is **Done** and every requirement is verified.

## Tasks selected before execution

These tasks were chosen before execution and are fixed in the tests.

- **A small production logic repair.** "Repair the upper-bound check in
  `src/ceiling.mjs` so the maximum is included…"
  - Result: an edit, then a Run-tool check of samples derived from the requested
    bound against the original signature. Ends `UsefulChangeVerified` (Done).
  - The replay is a verified no-op.
- **A useful regression-test task.** "Add a regression test in
  `test/ceiling.test.mjs` for the upper bound…"
  - Over a correct source: the test is written, plain `node --test` runs it and it
    passes, and the replay is a verified no-op.
  - Over a source that contradicts the bound: `IndependentCheckFailed`.
- **Beyond the supported subset.**
  - An unsupported expression stays an explicit gap with no edit.
  - A hosted negative task stays unresolved even when its oracle passes.

## Acceptance criteria → offline tests

All tests are under `rust/tests/web/` (Node) unless they are Rust unit tests; none
needs a network or an LLM.

| Criterion | Test |
|---|---|
| Original failure: a pre-fix #1455 item yields useful partial docs, and a repeat writes nothing | `repository-documentation.test.mjs`: "original pre-fix work item produces useful partial documentation and a verified no-write repeat" |
| Held-out variants reject unsupported claims | `repository-documentation.test.mjs`: "a version claim the manifest contradicts is rejected before any edit"; "renamed paths and an unrelated component use observed declarations and aliased calls"; "invalid manifest evidence and negated documentation instructions never write" |
| Production repair, plus a verified no-op replay | `work-item-boundary-route.test.mjs`: "a Run tool executes the contract-derived check; a replay is a verified no-op" (Rust twin in `unit/agentic-coding/work_item_boundary_candidate_routes.rs`) |
| Useful regression-test task | `work-item-boundary-route.test.mjs`: "a regression test over a source that meets the requested bound is written, passes on its own and replays as a no-op"; "… that contradicts the requested bound fails its check" |
| Beyond the subset stays honest | `work-item-boundary-route.test.mjs`: "an unsupported expression stays an explicit gap with no physical edit"; `work-item-hosted-effects.test.mjs`: "an expected unsupported negative case stays unresolved even when its oracle passes" |
| Claude and Codex adapters, direct and with the ordinary Hive prompts | `work-item-adapter-sessions.test.mjs` |
| A diagnostic-only first attempt fails useful-change acceptance | `hive-mind-draft-workflow.test.mjs`: "observed plan/log/bootstrap-only drafts are gaps; legitimate docs and removals remain candidates" |
| Independent check, concrete diff, authorship boundary | the record fields `checks`, `authored` and `noOp`, and the diff, asserted in the tests above; `work-item-owned-manifest-checker.test.mjs` and `work-item-source-quotation-checker.test.mjs` |
| Named gaps for the ten fixtures | `work-item-evidence.test.mjs`: "an interface the requirements ask for and no read source defines is the named gap", "… leaves the files no change recipe covers", "a work item that links only related pages names each as unresolved evidence", "the attachments of issues 1395 and 1396 are the named gap, never described", "the linked document of issue 1438 that cannot be fetched is the named gap" (frozen issue bodies); the full ten-issue replay takes about 7 minutes, so it stays an experiment |
| Report language | `work-item-report-outcome.test.mjs` and `unit/agentic-coding/work_item_report_outcome.rs`: issues 1330 and 1396 are reported in English, a Russian issue in Russian |

## What is not done here

- **A hosted Hive Mind `solve` run against GitHub.** The issue's first criterion asks
  for #1455 to be reproduced through ordinary `solve --tool agent --model formal-ai`.
  - Reproduced here: the agent leg of that command (Hive's prompt, model mapping,
    argv and Formal AI runtime, plus the real `agent` CLI) and the planner replay.
  - Not reproduced: `solve`'s own GitHub orchestration. It creates branches, pull
    requests and comments in a live repository, and
    `.github/workflows/work-item-qualification.yml` (the hosted qualification for
    agent, Claude and Codex) runs only from `main`.
  - Therefore the hosted run, and the separate hosted qualification, are **pending
    until this pull request is merged** and are not claimed as done.
- **A path in brackets in a pasted issue body refuses reads.**
  - When the text of #1364 is given as the user's own message, rather than through
    the Hive prompt, every read is refused (`ReadPolicyRefused`).
  - The cause is the fail-closed read-request rule from
    `balanced-read-path-containers.test.mjs`: a path in brackets that is not a
    complete read request counts as a pending read condition. The sentence here is
    "… moved on to the actual request (`Main.jar`)".
  - Hive sessions are not affected, because the user message is the URL-only Hive
    prompt and the body arrives as a tool result. The replay above ends at
    `UnresolvedReferences`.
  - The rule is left unchanged; reproduce with
    `node experiments/issue-1483/read-policy-1364.mjs`.
- **#1455 is partly done, not done.** Its requirements 5–7 and the integration
  semantics are left open by name.
