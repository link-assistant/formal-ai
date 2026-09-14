# Hive Mind × Formal AI, 2026-09-13: three Hello World runs, zero solutions

Three `solve --model formal-ai` tasks ran on 2026-09-13 against fresh
`konard/test-hello-world-*` repositories, one per Hive Mind tool. None produced
a Hello World. This document is the root-cause analysis for all three, the
replay that reproduces every Formal AI mechanism offline, and the fixes proposed
on each side. It is the reference for formal-ai issue #1133 (Formal AI fixes)
and hive-mind issue #2247 (Hive Mind fixes).

| run | tool | pull request | outcome |
| --- | --- | --- | --- |
| Kotlin | `--tool claude` (Claude Code) | [019fb330-fa49…/pull/2](https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/pull/2) | 547 identical `mcp__playwright__browser_click` calls, then `Prompt is too long`; nothing written |
| Scala | `--tool agent` (Agent CLI) | [019fb330-00e1…/pull/2](https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2) | `Main.scala` written, `scala: not found`, reported as *"Created and verified"*, never committed; 5 byte-identical restarts; PR marked ready anyway |
| Rust | `--tool codex` (Codex) | [019fb331-c107…/pull/2](https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2) | issue fetched through the operator's ChatGPT GitHub connector, raw JSON echoed as the final answer, PR marked ready with zero changes; 5 identical until-mergeable restarts |

Versions in the logs: `solve v2.22.0` (2026-09-07) on every run, although
hive-mind v2.28.1 had been released at 16:59Z the same day, before the first run
started at 18:50Z. Formal AI backend 0.349.2 (the serving sidecar), the version
this analysis replays against.

## Sources

Every log Hive Mind attached to the three pull requests on 2026-09-13:

- Kotlin: [solution draft log](https://gist.githubusercontent.com/konard/6c75a862747bb2a99ae7169a3050307d/raw/41cd7f99cca7ec1c0ac5025bacf15aec155936d6/tmp-hive-mind-log-upload-3zDdgh-sanitized.log.txt) (7.4 MB)
- Scala: [draft](https://gist.githubusercontent.com/konard/97b2139c89465d6b3679db3b62a82539/raw/065780ad1512257d351c76459216051e5400c54f/tmp-hive-mind-log-upload-tLzEYP-sanitized.log.txt), restarts [1](https://gist.githubusercontent.com/konard/28e0697c85dc6fd880746990f56e21c9/raw/fbd82a87350fcfcb4cb6d0bd82ef83c700f7df30/tmp-hive-mind-log-upload-bE2eti-sanitized.log.txt) [2](https://gist.githubusercontent.com/konard/b30ba302c6b62c09a0add97e089eb678/raw/42c76991c2954e721c390150d4b1458e93da9dbc/tmp-hive-mind-log-upload-CznALT-sanitized.log.txt) [3](https://gist.githubusercontent.com/konard/80e334c24a192777ffd8a4dd0728c17a/raw/bef30de2a3b1318c9ca6858f94a5e2e281d2e256/tmp-hive-mind-log-upload-UNgg5v-sanitized.log.txt) [4](https://gist.githubusercontent.com/konard/8ff4b0150537d8dd85eb0a6e0d19de06/raw/2a92d97c46358a9d57465875535c9936ce8d4f1d/tmp-hive-mind-log-upload-zYNqhK-sanitized.log.txt) [5](https://gist.githubusercontent.com/konard/d2cabb3f700a8dc1b8207c46c2507102/raw/d7c38df7f4ad520b0c4dae1ebf48947c9abffb59/tmp-hive-mind-log-upload-ZctRYF-sanitized.log.txt), [final failure](https://gist.githubusercontent.com/konard/fea22085db837838403c07645b954632/raw/22dece57ed53733d00dadf7261fb26eefb515b58/tmp-hive-mind-log-upload-dWq9FR-sanitized.log.txt)
- Rust: [draft](https://gist.githubusercontent.com/konard/04ab1d3f93cb95e3cc7ee532b66eb5f3/raw/72a6fcbd55fdae0f04751758edb88336c28c3a69/tmp-hive-mind-log-upload-OZdDre-sanitized.log.txt), until-mergeable restarts [1](https://gist.githubusercontent.com/konard/b9356b5918d7fed1c9bb1b92976cef5c/raw/931e88b7f22f92c4901ed3a130ab467a15973a51/tmp-hive-mind-log-upload-o1aN5G-sanitized.log.txt) [2](https://gist.githubusercontent.com/konard/ea9ce87ae4e2f8758c1d272e058bce7c/raw/09b055fb8fd01547f08a3ba7d7ab64fccf54fb0a/tmp-hive-mind-log-upload-M3voG4-sanitized.log.txt) [3](https://gist.githubusercontent.com/konard/00521a1ac0cb9dc8e6376c1403b3595f/raw/6b0bd49b6d4be96b99a3094406bc99b5f54b55bd/tmp-hive-mind-log-upload-Bt0H6i-sanitized.log.txt) [4](https://gist.githubusercontent.com/konard/eda5778457f566027ae16aaffead02da/raw/9766dd48c8c2e18e796c6ff5209bdb3b631c043d/tmp-hive-mind-log-upload-vNL1HS-sanitized.log.txt) [5](https://gist.githubusercontent.com/konard/031120801daae06159554dbf4f03b7f0/raw/7f7a0309bbe0b604e36080aa2df87ea8fba7de54/tmp-hive-mind-log-upload-SZzxnl-sanitized.log.txt), [final failure](https://gist.githubusercontent.com/konard/45dd8844d6367f0afe4a6548acda67f6/raw/4282dc1c6e585feeb1b4ee3f0ca865c40b8b19dc/tmp-hive-mind-log-upload-XPgkIQ-sanitized.log.txt)

The offline replay, its output, and the log excerpts each claim below rests on
are in one gist: <https://gist.github.com/konard/36ddb270dc7f10e000360da6add6f287>.
The replay is `examples/replay_hive_mind_1133.rs` in this repository; run it with
`cargo run --release --example replay_hive_mind_1133`.

## What the prompt asked for

Hive Mind's formal-ai prompt (`src/formal-ai-prompt.lib.mjs`) is five lines:

```
Resolve the GitHub issue at <issue URL> in this repository.
Keep the solution on branch <branch>.
Update the pull request at <PR URL>.

Implement and verify the solution before reporting completion.
Proceed.
```

The issue asks for a Hello World file that prints `Hello, World!`, with
comments, idioms, run instructions, and a GitHub Actions workflow. Formal AI
routes this as a *repository work item* (`compose_repository_work_plan`,
`src/agentic_coding/general_planner.rs`): fetch the issue, then plan from what
the issue says. Everything below happens after that decision, which is correct.

## Formal AI root causes

Each item states what the log shows, why the code does it, the replay line that
reproduces it without Hive Mind, and the fix.

### F1. A browser *click* tool is chosen to fetch a URL, and the failure can never be recorded

**Observed (Kotlin).** Every one of 547 assistant turns called
`mcp__playwright__browser_click` with input `{"target": ""}`; Playwright answered
`Unexpected token "" while parsing css selector ""`; the next turn made the
same call. Claude Code's context filled up and Anthropic's API returned
`Prompt is too long`. `WebFetch` was advertised the whole time and never used.

**Mechanism.**

1. `classify_tool` (`src/agentic_coding/capability_router.rs:125-129`) maps any
   tool whose name contains `fetch`, `open`, `browse`, `get_url` or `read_url` to
   `Capability::Fetch`. `mcp__playwright__browser_click` contains `browse`.
2. `tool_for` (`capability_router.rs:23-35`) prefers an `mcp__` tool for Search
   and Fetch before consulting the seed aliases, so the click tool beats
   `WebFetch`.
3. The planner emits `{"format":"text","url":…}`. `response_arguments_for_tool`
   (`src/protocol_responses.rs:74`) projects arguments onto the tool's schema:
   `url` is not a property of `browser_click`, its only required property is
   `target`, and a missing required string defaults to `""`. The harness sees
   `{"target": ""}`.
4. On the next turn `Progress::scan` (`src/agentic_coding/progress.rs:82-88`)
   records the attempted fetch by reading `url` out of the *echoed* tool call
   (`fetch_call_url`, `progress.rs:280`). There is no `url` any more, so
   `attempted_fetches` stays empty, the guard in
   `plan_general_change_step` (`general_execution.rs:119-126`) sees the target
   as never attempted, and the same call is planned again. Nothing bounds the
   loop.

**Replay.** `CLAUDE (Kotlin run) tool set`: six consecutive
`mcp__playwright__browser_click({"format":"text","url":…})` plans when the
harness echoes `{"target":""}`.

**Fix.**

- `classify_tool`: a browser-automation tool acts on a page the client is
  driving; it retrieves nothing. Names under `browser_` or carrying an
  interaction marker (`click`, `hover`, `drag`, `snapshot`, `screenshot`,
  `press_key`, `fill_form`, `select_option`, `handle_dialog`, `file_upload`)
  classify as no capability at all (`is_browser_interaction_tool`).
- `tool_for(Search | Fetch)` ranks candidates (`research_tool_rank`):
  client-executed aliases (`WebFetch`, `webfetch`) first, then client-scoped
  `mcp__` tools, then remote-scoped `mcp__` tools, then the provider-hosted
  tools (`web_search`, `web_fetch`) whose result never returns through the
  CLI — which is the ordering issue #781 wanted and the old rule approximated
  by putting every `mcp__` name first.
- `Progress::attempted_fetch_of(url)`: a fetch attempt the harness echoed
  without a `url` counts as the attempt on the one URL the plan asked for.
- Loop stop (`planner::stop_repeated_failure`): a tool that has failed twice
  this turn with the same report is not planned a third time; the failure
  report replaces the call. It is applied to whatever any route planned.

### F2. A failed verification command is reported as "Created and verified"

**Observed (Scala).** Formal AI wrote `Main.scala`, ran `scalac Main.scala`
(`/bin/sh: 1: scalac: not found`) and `scala Main` (`/bin/sh: 1: scala: not
found`), then answered:

> Created and verified `Main.scala` through the agentic CLI harness. …
> Actual tool output: `/bin/sh: 1: scala: not found`

**Mechanism.** The Agent CLI's bash tool returns the process text as the tool
message and keeps the status in `metadata.exit: 127`, which the model never
sees. `StepFailure::from_result` (`src/agentic_coding/command_reroute.rs:250`)
looks for an `Exit Code:` label first, then `reports_failure_in_prose`
(`command_reroute.rs:267`), whose markers are `command timed out`, `permission
denied`, `no such file or directory`, and non-empty `error:`/`failed:` fields.
`not found` is not among them, so the run counts as a success and
`ExecutionRecipe::final_answer` (`command_reroute.rs:87`) prints its fixed
"Created and verified" sentence — with the failing output quoted underneath.

`tool_result.rs` already has the seed-backed failure lexicon
(`looks_like_error`, role `tool_result_failure_signal`, which includes
`not found` from `data/seed/meanings-tool-access.lino:144`) and uses it for
exactly this harness (`tool_result.rs:244-252`). The recipe path does not.

**Replay.** `AGENT (Scala run) tool set`, step 4: `FINAL: Created and verified
… /bin/sh: 1: scala: not found`.

**Fix.**

- `StepFailure::from_result` uses `tool_result::failure_message` (the same
  verdict the narration path uses), so an exit-less harness result is judged by
  the seed lexicon, and `command not found` / `not found` / `No such file` all
  mean failure.
- `final_answer` is never a claim: it says *created and verified* only when
  every verification command succeeded; otherwise it says which command failed
  and with what, and that the artifact is unverified. The wording comes from
  `data/seed/` like the other outcomes (`general_plan_completed`,
  `general_plan_unverified`).
- No separate toolchain precheck: the compile step *is* the check, and once
  its failure is reported honestly the answer names the missing command
  (`scalac Main.scala` … `not found`), which is the precise failure class Hive
  Mind needs for H9. A `command -v` step before every recipe would add a call
  to every run to learn what the first command already says.

### F3. Nothing is ever committed or pushed, and "commit them" is answered with a web search

**Observed (Scala).** After the final answer the branch still had
`?? Main.scala`. Hive Mind's auto-restart prompt —

```
🔄 Auto-restart: resume the previous session and handle its uncommitted changes.
Uncommitted files (1):
?? Main.scala
…
Please review these changes and commit them with an appropriate commit message.
```

— produced five byte-identical sessions. In the replay the resumed session plans
`websearch("🔄 Auto-restart: resume the previous session and handle its
uncommitted changes")` and then `FINAL: ok`.

**Mechanism.** The repository-work-item recipe ends at verification: no step
maps the request's *"Keep the solution on branch X"* / *"Update the pull request
at URL"* to `git add`/`git commit`/`git push`. Separately, a request the planner
does not recognise falls through to web research (`web_research.rs:281,364`),
which is the same fall-through issue #1115 measured for an additive edit
instruction. A request that names local files, a branch, or a commit is never a
web question.

**Fix.**

- A *Commit* stage ends the work-item recipe (`command_reroute.rs` with
  `git_commit.rs`): after every verification command succeeded, `git add -A &&
  git commit -q -m 'feat: add <path>' -m 'Resolves <issue URL>' && git push -q
  origin <branch> && git rev-parse HEAD`, the branch read from the request
  after the seeded branch cue (`Keep the solution on branch …`, `в ветке …`,
  `分支 …`) or `HEAD` when none is named. The final answer quotes the push
  output with the hash. It runs whether or not the prompt says "commit": a
  request that names the pull request is a request to land the result there.
- The issue's workflow requirement is honoured the same way: when the fetched
  work item asks for a GitHub Actions workflow (seeded `ci_workflow_request`),
  the recipe gains `.github/workflows/run.yml` as a supporting file running
  the verified commands, written and committed with the program
  (`ci_workflow.rs`).
- A request to commit (`commit them`, `закоммить изменения`, `提交这些更改`, …;
  seeded `git_commit_request`) is a stage-commit-push step whose subject is
  read from the `git status --porcelain` lines the request quotes
  (`feat: add Main.scala`), so a resumed session completes instead of
  restarting.
- An instruction that edits a named file is never a web question
  (`positional_edit::names_local_edit`): when no edit route composes it, the
  planner declines rather than searching for the sentence. This closes #1115's
  route as well; #1115's additive shape itself now composes (F6).

### F4. A remote GitHub connector's JSON is taken as the issue text and echoed as the answer

**Observed (Rust).** Codex advertised the operator's ChatGPT GitHub connector
(`mcp__codex_apps__github_fetch`, origin `https://chatgpt.com`, `auth_mode=
Chatgpt`). Formal AI fetched the issue through it. The result was an MCP
envelope `{"content":[{"type":"text","text":"Action completed."}],
"structuredContent":{"content":"{\"url\":…,\"title\":\"Implement Hello World
in Rust\",\"body\":…}"}}`. Formal AI's final answer was:

> The `_fetch` command completed. Output: ```json { "content": "{\"url\":… ```

No file was written. Hive Mind then converted the PR to ready (H2).

**Mechanism.** Two defects stack. First, the repository work-item route is
gated on `tool_for(tool_names, Capability::Write)` (`planner.rs`,
`plan_settled_routes`), and Codex advertises no write alias — its `apply_patch`
classifies as Edit — so for Codex the route never ran at all; the fetch was
planned by the web-research route, which reads any URL in the request, and
the turn after it fell to result narration (`tool_result.rs`, intent
`tool_result_completed`), which quotes whatever the tool returned as the
assistant's message. Second, even where the route runs, `tool_for(Fetch)`
would take the `mcp__` connector (F1, point 2) although it acts in a remote
service and the checkout has a `shell` that can run `gh issue view`; and
`normalize` reduces the MCP envelope to `content[0].text`, `Action completed.`,
dropping the `structuredContent` that held the issue.

**Replay.** `CODEX (Rust run) tool set`: `FINAL: The
mcp__codex_apps__github_fetch command completed. Output: Action completed.`

**Fix.**

- Gate file-creating routes on *any* workspace creation tool
  (`capability_router::workspace_creation_tool`: the write alias or a patch
  tool), so Codex reaches the work-item route like every other client.
- Read the work item through the client workspace when the only fetch tool
  is remote-scoped or there is none: `gh issue view <url> --json title --jq
  .title && echo && gh issue view <url> --json body --jq .body` through
  `Capability::Run` (`general_execution::plan_work_item_read`; the wording
  lives in `data/meta/work-item-steps.lino` with the commit and workflow
  templates). Hive Mind's task containers have `gh` authenticated.
  `codex_apps` joins the remote namespaces in
  `data/seed/tool-resource-scopes.lino`, since every ChatGPT connector acts on
  chatgpt.com.
- Read MCP envelopes whole: when `structuredContent` (or a JSON string with
  `title`/`body`) is present and longer than the `content` text, it is the
  result, and a GitHub issue record reads as `title + body`.
- With the route reached and the page read, the fetch is never narrated: the
  recipe runs, or the answer is the *planned, not executed* record.

### F5. (latent) A harness "text to summarize" envelope would be executed as the task

**Observed.** The Agent CLI's title/summary calls in the Scala run went to
`opencode/big-pickle`, 12 requests, every one HTTP 400 *"OpenCode's free tier
can only be used in OpenCode"*. They did not reach Formal AI; this is H5 on the
Hive Mind side and the corrected premise of #1127 on ours.

**Replay.** When the same envelope *is* handed to Formal AI with tools (`AGENT
summarize call driven`), Formal AI unwraps nothing and runs the whole task
again — fetch, write, `scalac`, `scala` — inside a summarization request.
Without tools it answers with the plan record.

**Fix.** The `The following is the text to summarize: <text>…</text>` envelope
is a protocol marker the CLI emits verbatim, so it is matched as one
(`harness_envelope.rs`) before any route runs: the answer is the quoted text's
leading sentence and no tool is planned. (The title-generation sibling carries
its marker in the system prompt, not the user turn, and is not reachable from
the user text alone; with H5 fixed neither call is made.) `planner.rs` already
special-cases Agent's own compaction envelope; this is the same family.

### F6. Open issues that are the same defects, folded into the same fix

| issue | relation |
| --- | --- |
| #1115 additive edit routed to web search | same fall-through as F3's restart prompt; fixed by the no-Search-for-local-work rule and an additive edit composer |
| #1116 `\n` written literally, reported as success | same "claim without check" class as F2; the artifact check must compare bytes after unescaping |
| #1117 task-contract parser keeps quotes | script defect in `resolve-formal-ai-task.sh`; fixed alongside |
| #1118 self-authoring cannot push `.github/workflows/` | option 2 (detect early, fail with the exact reason) lands here; the token itself is konard's |
| #1127 "Agent CLI ignores provider config" | premise was wrong: the failing calls are the title/summary calls to `big-pickle`; `--no-summarize-session --no-generate-title` (already in `author-change-with-formal-ai.sh`) is the fix, and the issue is closed with the correction |
| #1130 docs-only contribution cannot move the share | the Formal AI-authored parts of this fix land under `src/`/`tests/`, which the metric counts |

## Hive Mind root causes

These are reported in hive-mind #2247 with the same evidence. They are listed
here because "works perfectly" needs both sides and the order matters: H1
decides which Formal AI and which Hive Mind the next run even executes.

| id | defect | evidence | fix |
| --- | --- | --- | --- |
| H1 | Tasks run `solve v2.22.0` although v2.28.1 was released before they started; the image tag `latest` (`hive-mind-image.lib.mjs`) is never refreshed and the version is not surfaced in the PR comment | first line of all three logs | pull/verify the image version at task start; print `solve`, image digest, and formal-ai backend version in the *AI Work Session Started* comment; fail loudly when `solve` is older than the minimum the sidecar requires |
| H2 | `ensurePullRequestIsReady(reason: 'solution draft verified')` (`solve.results.lib.mjs:818`) runs without checking `changeStats.hasChanges` (the guard at `:775` covers a different branch), so a PR with zero changes is marked ready and *until-mergeable* restarts begin | Rust log 19:26:53Z `Converting PR: To ready for review (solution draft verified)`; the PR has one commit, Hive Mind's own | require `hasChanges` **and** a non-empty diff against the base before converting; otherwise keep draft and post *no changes were produced* |
| H3 | Auto-restart repeats a byte-identical session five times; nothing compares the new session with the previous one | Scala restarts 1–5 and Rust restarts 1–5 have identical tool sequences and final text | stop after the second identical outcome (same final message hash, same `git status`), post one *no progress* comment with both logs |
| H4 | No circuit breaker on the Claude runner: 547 identical failing tool calls run to context exhaustion; `playwright` is attached by default (`MINIMAL_MCP_SERVERS`) though the formal-ai flow never needs a browser | Kotlin log | break after N identical `(tool, input, is_error)` triples (N=3) and fail the session with that reason; do not attach playwright for `--model formal-ai` |
| H5 | `buildAgentArgs` (`agent-command.lib.mjs`) omits `--no-summarize-session --no-generate-title`; each Agent CLI run makes 12 calls to `opencode/big-pickle` that all fail with HTTP 400 | Scala draft log, `service: session.summary`, `modelID: big-pickle` | pass both flags for every `--tool agent` run (they are mandatory for formal-ai; harmless elsewhere) — same as formal-ai#1127's correction |
| H6 | The formal-ai prompt has no commit/push contract | `formal-ai-prompt.lib.mjs` | add the line *Commit the changes and push them to the branch before reporting completion.* (Formal AI will commit without it after F3, but the prompt should say what the contract is) |
| H7 | `CODEX_HOME` is seeded from the operator's `~/.codex` (`formal-ai-runtime.lib.mjs:349-363`), so the task inherits the operator's ChatGPT `codex_apps` GitHub connector; the formal-ai run fetched the issue through chatgpt.com with the operator's identity | Rust log `mcp_server_origin=https://chatgpt.com auth_mode="Chatgpt"` | seed only `auth.json` and a minimal `config.toml` with no MCP servers for formal-ai routed tasks (the single-credential principle of #2227 / formal-ai#1075) |
| H8 | The working-session-summary comment posts the raw request/response JSON (≈10 KB) six times per PR | Scala PR comments 19:13:37Z … 19:17:16Z | collapse into `<details>`, cap at a few lines, link the gist for the rest |
| H9 | The task image (`ghcr.io/link-foundation/box:2.10.2`) has no Scala toolchain while the test-repo generator picks Scala | `/bin/sh: 1: scalac: not found` | either install Scala in box or drop Scala from `create-test-repo`'s language pool until it is; publish the languages the image supports as a list both sides read |
| H10 | The failure comment classifies the Kotlin run as `Prompt is too long`, the last symptom, not the loop | Kotlin PR *Solution Draft Failed* | classify from the tool-call history: *identical tool call repeated N times* |

## What "works perfectly" means, as an acceptance test

For each of the three tools, a fresh `test-hello-world` task must end with:

1. the issue text read from the repository's own GitHub (`gh`/`WebFetch`), no
   browser, no third-party connector;
2. the program file written and the toolchain checked; when the toolchain is
   missing the final answer says so and the PR stays draft;
3. the program compiled and run, `Hello, World!` observed in the run output;
4. a GitHub Actions workflow file written (the issue's requirement 6);
5. `git commit` and `git push` to the task branch, the hash in the final answer;
6. the PR converted to ready only after 5, and only when the diff is non-empty;
7. no restart when nothing changed between sessions; one log per session, no
   raw JSON in comments.

Formal AI's side (1–5) is covered by the replay example turned into unit tests
under `tests/unit/`, one per mechanism above, in the same pull request as the
fixes. Hive Mind's side (1, 6, 7 and the runtime items) is hive-mind #2247.
