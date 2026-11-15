# Issue #1559: a system prompt of one session became a learned standing requirement in another repository's session

Issue: <https://github.com/link-assistant/formal-ai/issues/1559>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1566>

## 1. Collected data

| Path | Content |
| --- | --- |
| `raw-data/issue-1559.json`, `issue-1559-comments.json` | The issue as read on 2026-11-15 (no comments) |
| `../../../rust/tests/fixtures/issue-1559/scala-solver-prompt.txt` | The solver prompt of the Scala test session (`konard/test-hello-world-019fb330-…`, branch `issue-1-1f3e3886bcb8`), the text quoted after `Learned standing requirement (you):` |
| `raw-data/reproduce-before-fix-main-58a229dda.txt` | `experiments/issue-1559-reproduce.mjs` run on `main` at `58a229dda` |
| `raw-data/reproduce-after-fix.txt` | The same script on the branch of this pull request |
| `raw-data/experiment-russian.txt` | `experiments/issue-1559-russian.mjs`: a user's "Always answer in Russian." is learned and applied for that user only |
| `raw-data/experiment-generalize.txt`, `experiment-generalize-verified.txt` | A statement made by two users stays scoped without a replayed task, and becomes one generalized amendment with one |
| `raw-data/hive-mind-formal-ai-sidecar-excerpt.mjs.txt` | How Hive Mind runs Formal AI on a host, from `link-assistant/hive-mind` at `318abfd9` |

## 2. Reproduction

`node experiments/issue-1559-reproduce.mjs <checkout>` records the Scala
solver prompt as one agent session (a system message plus the prompt delivered
as the first user message, with a `bash` tool), runs one dreaming pass over
the same memory file, and renders the agentic final answer of a second session
whose prompt is the same text for the Rust repository and branch.

Before the fix (`main` at `58a229dda`):

```text
stored amendments: [ { topic: 'topic=issue', rule_chars: 13710 } ]
summary starts: "Summary: done.\n\nLearned standing requirement (issue): You are an AI issue solver using @link-assistant/agent. …"
summary carries the Scala repository: true
summary carries the Scala branch: true
```

The topic is `issue` on `main` and `you` at `70935ad8e`, where the issue read
the source, because the stop-word list changed in between. The mechanism is
the same: the whole prompt (13,710 characters) is the rule.

After the fix:

```text
stored amendments: []
summary starts: "Summary: done."
summary carries the Scala repository: false
summary carries the Scala branch: false
```

## 3. Root causes

1. **Role.** Learning skipped only `assistant` and `system` events. A client
   that sends its system prompt as the first user message ("many clients send
   them as the first user message", in the issue's words) had that prompt
   recorded with role `user`.
2. **Size.** The whole content of a matching event became the rule.
3. **Topic.** The topic was the first word not on a short English stop-word
   list, so `You are an AI issue solver` gave `you`. `you` occurs in almost
   every prompt, so the amendment matched almost every later answer.
4. **Scope.** Recorded exchanges carried no scope, and every amendment in the
   store applied to every request served from it.
5. **Publication.** The agentic final answer, which Hive Mind posts as the
   working-session summary of a pull request, had amendments appended to its
   text.
6. **Shared memory on the host.** Traced in section 4.

## 4. Why two sessions shared memory on the Hive Mind host (step 6)

- Hive Mind runs **one** Formal AI sidecar per host for every
  `/solve --model formal-ai` task. In `src/formal-ai-sidecar.lib.mjs` the
  container is `hive-mind-formal-ai`, and "Concurrent `/solve --model
  formal-ai` runs share one sidecar. Each task holds a named lease; the
  sidecar is stopped only after the last lease is released".
- The memory is the named volume `hive-mind-formal-ai-memory` mounted at
  `/home/box/.formal-ai` and "never removed": it is kept "between tasks and
  across container replacement" on purpose.
- The sidecar image runs `formal-ai serve --agent-mode` with
  `FORMAL_AI_MEMORY_PATH=/home/box/.formal-ai/memory.lino`
  (`scripts/build-hive-original-delivery-image.mjs`, `docker/compose.slim.yml`).
  Every request it answers is recorded in that one file (`FORMAL_AI_RECORD_CHAT`
  defaults to on).
- `start_core_dreaming` (`rust/src/dreaming_runtime.rs`) runs a dreaming pass
  on the same file once the server has been idle for 60 seconds
  (`DEFAULT_IDLE_SECONDS`). Any gap of a minute between two tasks, or before
  the next task when the sidecar is started again, is enough.

The Scala and the Rust test sessions differed in the client tool, not in the
server: both reached the same sidecar and wrote to the same `memory.lino`. The
Scala prompt was recorded, a dreaming pass in an idle minute learned it as an
amendment with topic `you`, and every later answer of the Rust session, whose
prompt also contains "you", had it appended.

**Isolation chosen.** The store file stays shared, because Hive Mind relies on
that file being kept across tasks. Formal AI now isolates sessions inside it:

- every exchange of an agent session (a request with tools) is recorded with
  `scope:repository=<host/owner/repo>` and `scope:session=<id>`, where the id
  is derived from the session's first user message;
- every request reads the store through `events_in_scope` /
  `eventsInScope`, which drops events and amendments of any other scope.

Sharing is opt-in for the operator with
`FORMAL_AI_SHARE_STANDING_REQUIREMENTS=1`. A request without a user or session
scope, such as a person's chat with their own local store, keeps the previous
behaviour: what it teaches applies everywhere in that store.

## 5. The fix, step by step

| Step of the issue | Change | Verified by |
| --- | --- | --- |
| 1. No requirements from system or developer instructions, however delivered | When an exchange is received it is recorded with role `instruction` if the prompt opens like a system prompt (`instruction-opener` entries of `data/meta/dreaming-lexicon.lino` in en/ru/hi/zh), or if it is the first user message of an agent session that has no system message. Learning skips `system`, `developer`, `instruction` and `assistant`. | JS and Rust tests "replaying the two recorded prompts…" (recorded role is `instruction`, no amendment) |
| 2. Bound a rule | `bounded_statement`/`boundedStatement` take one sentence with a requirement cue. The sentence is rejected if it is over `rule-maximum-characters` (240, data), if it opens like an instruction, or if it names one task: a URL into a repository, an `owner/repo` slug, a path, a hash-like branch or commit, `#N`, or `issue`/`pr`/… followed by a number. Amendments already in the store that break a bound are not applied (`retained_amendments`). | "the solver prompt opens like a session instruction…", "the stored amendment of the issue is never applied" |
| 3. Topic from the statement's subject | `statement_topic` skips pronouns, modal verbs and function words (`topic_stopword`, en/ru/hi/zh) and reads unspaced scripts (Chinese) by the longest known stop-word prefix. `is_valid_topic` rejects a pronoun topic everywhere. | "pronouns and function words are never topics in en, ru, hi and zh" |
| 4. Scope amendments | An amendment carries its source events' scope (`scope:user=…`, `scope:repository=…`, `scope:session=…`) and applies only where every pair matches. If the same statement is made in two or more scopes, it becomes one unscoped amendment marked `scope:generalized`, but only when replaying recorded tasks with it covers at least one of them; otherwise the scoped copies stay. | "a requirement stated inside one agent session stays in that session", "a statement made in two scopes stays scoped until replay verifies…", "…verified by replay becomes one generalized amendment" |
| 5. No amendments in published final answers | The agentic `final` answer is the plan's answer only. Applied amendments go to `thinking_steps` (`match_rule`, detail `standing_requirement:<topic>`, source `meta_algorithm_amendment:<id>`) and `reasoning`/`reasoning_content` in Chat Completions, and to `evidence_links` in Responses. | "an agentic final answer lists applied amendments in its trace, never in its text", `memory_maintenance::learned_amendment_changes_the_agentic_final_answer` |
| 6. Trace and isolate | Section 4 | "sharing standing requirements between agent sessions is an explicit opt-in" |
| 7. Tests in both runtimes | `rust/tests/web/issue-1559-standing-requirement-scope.test.mjs` (JS) and `rust/tests/unit/issue_1559_standing_requirement_scope.rs` (Rust) | CI |

## 6. Acceptance

| Criterion | Evidence |
| --- | --- |
| Replaying the two recorded prompts in order produces a second summary with no text of the first prompt | Both tests replay the Scala prompt and the Rust prompt through the HTTP handler with one dreaming pass in between. The store also holds the amendment found on the host (topic `you`, the whole prompt). The second answer and the agentic summary contain neither the Scala repository, nor its branch, nor `Learned standing requirement`. |
| No stored amendment has a pronoun as its topic | Every learned amendment in the tests goes through `assert_bounded_amendments` / `assertBoundedAmendments` (`is_valid_topic`); the pronoun table covers en/ru/hi/zh. |
| No stored rule is longer than the limit or contains a branch or repository name of one task | The same assertion checks `rule-maximum-characters` and `contains_task_identifier`, plus the literal repository and branch names of both sessions. |
| "Always answer in Russian" is still learned and applied in that user's later conversations | The test "always answer in Russian is learned and applied in that user's later conversations only" (both runtimes) and `rust/examples/scoped-standing-requirements.mjs`: alice's later answers carry it, bob's and an anonymous user's do not. |

## 7. Limits

- A user scope needs the client to declare the user (OpenAI `user`,
  Anthropic `metadata.user_id`). A requirement stated without a user or session
  scope still applies to the whole store it is recorded in, as before.
- The session id is derived from the first user message of the conversation,
  because the protocols carry no session id. Two sessions that open with
  byte-identical first messages share a session scope. If the prompt names an
  issue or pull request URL, the repository scope still tells them apart.
