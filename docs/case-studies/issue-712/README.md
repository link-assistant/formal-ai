# Issue 712 case study — tool intent as semantic frames

Issue [#712](https://github.com/link-assistant/formal-ai/issues/712) reported a
routing matrix in v0.289.0: tools were advertised by the client, but small
wording changes prevented URL fetch, web search, file edit, and declarative file
write calls. The maintainer's follow-up on
[PR #719](https://github.com/link-assistant/formal-ai/pull/719#issuecomment-4982573187)
rejected an initial phrase-by-phrase patch and required a deeper solution aligned
with the vision, roadmap, associative auto-learning, and Formal AI via Agent CLI.

## Reproduction

`tests/issue_712.rs` preserves every failing prompt from the issue. The first
pre-fix run is in [`red-regression.log`](red-regression.log). The suite also
contains paraphrases that were not in the report; these prevent the implementation
from passing by memorizing the issue sentences. `tests/integration/issue_712_intent_routing.rs`
boots the real server and verifies Chat Completions, Responses, and Gemini.

The original self-coding sessions are preserved in the three `agent-*.jsonl`
files in this directory. The current implementation adds a second, independent
Agent CLI task: Formal AI ingests the persisted routing observations through the
associative-memory adapter, ranks observations and semantic amendments, writes
`tool-routing-learning-report.lino`, verifies it, and leaves promotion explicitly
`awaiting_human_review`. The runner is
[`experiments/agent_cli_e2e/run_issue_712_learning.sh`](../../../experiments/agent_cli_e2e/run_issue_712_learning.sh).

## Root cause and architecture

The problem was a mismatch between semantic knowledge and executable argument
shape, not four missing aliases. See [`root-cause.md`](root-cause.md) for the
traced paths and [`requirements.md`](requirements.md) for the full contract.

The resulting routing model is:

| Capability | Seed-defined semantic evidence | Typed argument evidence |
| --- | --- | --- |
| Fetch | HTTP-fetch or URL-navigation action | one normalized HTTP(S) URL |
| Search | search action, or source-grounded/current-source frame | non-empty topic extracted through a topic connective or imperative slot |
| Edit | edit action | safe file target plus old/new replacement spans |
| Write | write action/target/content roles | safe file target plus non-empty literal payload |

Natural-language surfaces remain in `data/seed`, including `google` as a search
action and `contents` as a write-content lead. Rust and the browser worker apply
the same structural algorithms in `web_search_intent.rs` and the worker modules
`formal_ai_worker_16.js` / `formal_ai_worker_17.js`. The three reported search
sentences are no longer explicit templates. File-write classification now calls
the exact parser that composes the executable write plan, eliminating a separate
English/Russian verb gate that could disagree with execution.

## Auto-learning boundary

`data/meta/issue-712-routing-learning.lino` is an append-only observation and
amendment network. `agentic_coding::routing_learning` passes it through the
production `AssociativeMemory` pipeline, preserving evidence links and ranking
expressions by reads, writes, incoming links, and outgoing links. Its output is a
review artifact, not an automatic seed mutation. Promotion is gated on both the
reported matrix and unseen-paraphrase benchmark. This uses learning to retain and
prioritize a reusable rule—seed roles plus argument shape—without allowing a
single incident to silently rewrite routing policy.

## Verification evidence

- `tests/issue_712.rs`: reported matrix, unseen search paraphrases, multilingual
  write frames, and a `webhook` boundary negative.
- `tests/integration/issue_712_intent_routing.rs`: production HTTP surfaces and
  the whole capability matrix.
- `tests/unit/issue_712_routing_learning.rs`: derived (not canned) associative
  ranking, human-review gate, and in-repo three-turn Agent CLI execution.
- `.github/workflows/release.yml`: real external Agent CLI declarative-write E2E.
- [`green-regression.log`](green-regression.log) and
  [`live-agent-cli-e2e.log`](live-agent-cli-e2e.log): original local evidence.

No new dependency is introduced. The fix composes the seed lexicon, existing
intent router, general planner, associative persistence, and document recipe.
