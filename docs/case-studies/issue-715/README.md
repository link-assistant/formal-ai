# Issue 715: contextual code changes must mutate the workspace

## Result

Formal AI now treats the latest client-side write as the active mutable
artifact. A follow-up code request reads the current bytes through the Agent
CLI, executes an ordered normal (Markov) string-rewrite program, and writes only
the completed result. It no longer answers with stale prose or rewrites a
server-side copy that the user cannot see.

The public rewrite representation implements rule priority, leftmost matching,
restart from rule zero, terminal substitutions, and empty-string creation and
deletion. This is the computationally universal normal-algorithm model. The
network-facing executor adds a 100,000-substitution bound and refuses to write a
partial result on exhaustion; universality describes the representation, not an
unbounded promise for one HTTP request.

Natural-language compilation is structural rather than phrase-list based. One
explicit quoted value updates the prior output literal; ordered quoted old/new
pairs compile to ordered rules. Empty and Unicode slots are preserved across
English, Russian, Hindi, and Chinese prose. A fenced slot can hold a complete
file, so the mechanism is not tied to a programming language or extension.

## Reproduction and durable evidence

The original screenshot shows OpenCode returning a Rust snippet inline and then
ignoring the requested output change. The authoritative issue payload and
validated PNG are in [`raw-data`](raw-data/). Two independently discovered red
states are preserved:

- [`reproducer-red.log`](test-logs/reproducer-red.log): no workspace tool call
  for the original issue flow.
- [`markov-red.log`](test-logs/markov-red.log): creation reversed its operands,
  deletion became an unrelated output edit, and a second ordered rule was lost.
- [`transport-quotes-red.log`](test-logs/transport-quotes-red.log): the first
  real replay exposed OpenCode's outer transport quotes being mistaken for
  program output.

Run the focused suites:

```sh
cargo test --test unit issue_715 -- --nocapture
cargo test --test issue_715 -- --nocapture
cargo test --test issue_715_rewrite_variations -- --nocapture
cargo test --test integration issue_716_agentic_execution -- --nocapture
```

Run the actual external CLI replay:

```sh
cargo build --bin formal-ai
experiments/agent_cli_e2e/run_issue_715_opencode.sh
```

The four OpenCode turns create, compile, and run `main.rs`; replace `Hello,
world!` with `Hello 2`; create a leading line with an empty-pattern rule; and
delete that line with an empty-replacement rule. The retained run completed 14
OpenAI-compatible chat rounds. Its JSONL, server trace, and final compiling file
are in [`opencode-run`](opencode-run/), and the summary is
[`opencode-replay.log`](test-logs/opencode-replay.log).

The built-in Formal AI Agent CLI also executed the auto-learning task in three
turns: it wrote the derived report and read it back through `run_command`. See
[`formal-ai-agent-learning-session.json`](formal-ai-agent-learning-session.json)
and [`formal-ai-agent-learning.log`](test-logs/formal-ai-agent-learning.log).

## Requirements and evidence

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Mutate the real Agent CLI workspace | Capability routing emits client `read` and `write` calls; the server never directly edits the client's filesystem. | Four-turn OpenCode replay and four shared HTTP-protocol integrations |
| Generate and execute programs | Initial creation delegates to the typed `ExecutionRecipe`, which writes source and invokes the catalog's check/run commands. | Ten-language issue-716 suite and OpenCode `write -> rustc -> ./main` transcript |
| Preserve current state | The newest prior write identifies the artifact, but current bytes always come from the newest read result after the latest user turn. | Read-before-write assertions and changed OpenCode file |
| Support normal-algorithm substitutions | Public ordered rules use first-applicable priority, the leftmost match, restart, and terminal rules. | Priority, restart, state-symbol unary increment, and trace tests |
| Support creation and deletion | Empty pattern is creation; empty replacement is deletion. Empty slots survive literal extraction. | Direct algebra tests, 40 NL variants, and real turns 3-4 |
| Prevent runaway or partial edits | Each execution has a step bound. A cyclic program returns `StepLimit` without a write call. | `a -> b`, `b -> a` regression |
| Avoid language/extension gates | The active prior write may be any path; rewrite data is UTF-8 text rather than a source-language AST. | Ten catalog languages, `notes.custom-format`, and four natural languages |
| Support multiple rules | Every ordered old/new pair becomes a rule; evaluation restarts at the highest-priority rule after every non-terminal substitution. | `Hello -> Hi`, `world -> team` integration and cyclic safety case |
| Make execution auditable | Final responses render target, bound, each pattern/replacement/terminal flag, halt reason, applied rule, and byte offset in Links Notation. | Unit assertions and retained OpenCode JSONL/server trace |
| Learn without autonomous promotion | Persisted failures and linked amendments feed the production associative-learning adapter; the derived report remains `awaiting_human_review`. | Learning derivation test and built-in Agent CLI session |
| Accept broad NL variation without hardcoded multilingual cues | The compiler consumes ordered structural literal slots independently of surrounding prose. | Ten creation/deletion phrasings in each of English, Russian, Hindi, and Chinese |

## Root causes

The original failure and the deeper review exposed six distinct boundary bugs:

1. Code generation could produce symbolic prose without converting the
   established catalog program into an Agent CLI workspace write.
2. A contextual follow-up omitted the path and old text, while planning ignored
   the prior write that already established both artifact identity and content.
3. Recipe progress could scan earlier turns, so an old tool result could make a
   new operation appear finished.
4. The first mutation VM compiled one non-empty replacement. It dropped
   zero-length literal slots, could not express an ordered normal algorithm, and
   limited active artifacts to catalog source extensions.
5. A write could be planned from remembered bytes rather than the client's
   current read result.
6. OpenCode's positional `run` transport wraps user content in quotes. The typed
   creation path initially interpreted those framing quotes as requested output;
   only the real CLI replay exposed this integration defect.

## Design

`RewriteProgram` is an ordered list of `RewriteRule` values. On each step the
executor selects the first rule whose pattern occurs, replaces its leftmost
occurrence, and restarts selection at rule zero. A terminal rule stops
immediately. An empty pattern matches byte offset zero, and an empty replacement
removes the matched sequence. Every step records its rule index and byte offset.

The planner treats tool history as identity metadata, not as trusted current
content. It first requests the active path through the advertised read
capability, decodes known OpenCode read envelopes, executes the immutable
program over those returned bytes, and emits a complete write only after a safe
halt. A `StepLimit` outcome produces an explanation and no mutation.

Initial creation with a command capability stays in the typed execution path
introduced by issue 716. That path writes source and runs its declared commands.
Write-only harnesses retain catalog creation, while subsequent contextual edits
use the same normal-rewrite path in every OpenAI-compatible protocol adapter.

The issue-715 learning memory records the observed stale response, single-rule
VM, lost empty slots, and extension gate. The existing associative-memory
adapter ranks those observations together with linked amendments. Promotion is
explicitly gated on algebraic laws, multilingual slots, and real Agent CLI E2E
evidence, so running auto-learning cannot silently change production behavior.

## Scope and honest boundary

Normal algorithms can represent any computable string transformation, and this
implementation exposes that model directly. A fixed 100,000-step production run
is intentionally not an unbounded universal machine. Callers can also request a
transformation whose bound is insufficient or whose program does not halt; that
case is observable and non-mutating.

The deterministic natural-language compiler does not claim to infer every
underspecified semantic refactor. It reliably compiles explicit literal slots,
including complete fenced file contents and any number of ordered pairs. A
higher-level agent can ground an arbitrary semantic change by supplying the
desired complete source through the normal tool contract. Requests without
workspace tools continue through the prose solver.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-07-13 | Issues 680 and 681 established explicit file-intent routing and write-before-read ordering. |
| 2026-07-14 | Issue 712 reported regressions in those explicit routers. |
| 2026-07-15 | Issue 715 reported the multi-turn code-artifact failure with an OpenCode screenshot. |
| 2026-07-15 | The mandatory self-coding wrapper was attempted; its model allowlist rejected `formal-ai`, and the failure was preserved and reported on the issue. |
| 2026-07-15 | The initial artifact planner and real two-turn OpenCode reproducer were added. |
| 2026-07-16 | New review feedback required general normal-algorithm creation, deletion, multi-rule execution, auto-learning, and deeper Agent CLI evidence. |
| 2026-07-16 | Three red algebra regressions replaced the private single-replacement VM with the public bounded normal-algorithm executor. |
| 2026-07-16 | The first four-turn OpenCode attempt found transport-quote corruption; a dedicated red test and fix preceded the successful 14-round replay. |

## Data inventory

- `raw-data/issue-715.json` and `issue-715-comments.json`: issue description and
  discussion snapshots.
- `raw-data/issue-715.png`: downloaded attachment; its PNG signature was
  verified before visual inspection.
- `raw-data/issue-{680,681,712,714,716}.json`: adjacent routing and agentic-mode
  reports used for regression analysis.
- `raw-data/pr-727.json`: prepared-PR metadata snapshot.
- `test-logs/`: red reproductions, local checks, self-coding attempts, built-in
  Agent CLI evidence, and the external replay summary.
- `research.md`: current external contracts and primary research consulted
  during design.
