# Issue 840: grounded action recipe

Issue [#840](https://github.com/link-assistant/formal-ai/issues/840) combines
three apparently different failures—local discovery (#838), a definition and
follow-up (#827), and a comparison followed by a report request (#826)—under
one procedural defect. Formal AI took one over-built action, treated its first
result as conclusive, and skipped verification or synthesis.

The implementation models a topic-neutral eight-step recipe in
[`grounded-action-recipe.lino`](../../../data/meta/grounded-action-recipe.lino):
declare meanings, recognize the route, extract typed slots, take the smallest
action, observe its result, widen only when justified, synthesize scoped
evidence, and preserve runtime parity.

## Evidence inventory

Authenticated GitHub API snapshots are retained under [`raw-data/`](raw-data/):

- `issue-840.json` and `issue-840-comments.json`;
- the complete seed reports and comments in `seed-issue-{838,827,826}*.json`;
- `pr-850.json`, PR conversation comments, inline review comments, and reviews.

The issue, seed reports, and comments contain no image attachments, so there
was no screenshot to download or validate. The failure is behavioral rather
than visual.

Other raw, reproducible evidence is kept beside this README:

- [`agent-cli-e2e/`](agent-cli-e2e/) contains native Agent CLI streams, Formal
  AI traces, dialog JSONL, the fake-`gh` invocation, and isolated memory for all
  four reported journeys;
- [`self-hosting/`](self-hosting/) contains the Agent-CLI-authored smallest
  leaf and its five-leaf decomposition;
- [`self-hosting-projection/`](self-hosting-projection/) contains the current
  whole-repository source-to-links projection and focused agentic-coding
  self-AST;
- [`differential-gate.log`](differential-gate.log) is the real release-server
  run of the machine-enforced 24-node/reference comparison.

The machine-readable reference baseline and checker live in
[`experiments/issue_840_reference_agents/`](../../../experiments/issue_840_reference_agents/).
The measured prose transcript summary remains in `findings.txt`.

## Timeline

- 2026-07-22: #826 and #827 recorded the comparison/report and
  definition/follow-up failures.
- 2026-07-24: #838 recorded the Desktop-routing failure and the PEM-file false
  positive. #840 consolidated the three reports and defined the grounded-action
  standard.
- 2026-07-24: eight reference-agent attempts were measured on the exact Desktop
  fixture. Six completed successfully, one stopped too early, and one was
  inconclusive because of a permission gate. A transient Laguna provider 429
  was explicitly reclassified as inconclusive while retrying; the eventual run
  completed and became the strongest reference.
- 2026-07-24: the four-level task ladder measured the pre-change v0.303.0
  baseline at 8/24.
- 2026-07-24–25: the branch added meanings-driven routing, stateful local
  discovery, definition/follow-up and comparison planning, the topic-neutral
  recipe, browser parity, real-client/TUI journeys, and self-authorship
  evidence. The first prepared automation session ended during a service
  transport failure; its complete 61 MB log was recovered and audited before
  continuing.
- 2026-07-26: current `origin/main` (v0.304.1) was merged and report-flow
  conflicts were resolved in favor of #839’s current six-section body and real
  session handling while preserving #840’s one-action-per-destination flow.
- 2026-07-26: three new red tests reproduced provider-denial leakage,
  all-fetches-failed fallback, and short-page furniture leakage. Fixing the
  short-page path exposed and then fixed a separate decimal sentence-splitting
  regression.
- 2026-07-26: a generated multilingual property suite and deterministic
  reference differential gate were added. The missing Russian report journey
  was also driven through the real Agent CLI and captured.
- 2026-07-26: Formal AI refreshed the merged tree’s 300-module exhaustive
  source-to-links projection and 42-module focused agentic-coding self-AST.

## Root causes

### Local routing was structurally asymmetric

Web search was represented by first-class seed meanings and roles; local path
discovery depended on surface conditions. A generic verb such as “Search”
could outweigh “desktop”, and adding or dropping a possessive changed the
route. The new local action, scope, and requested-kind meanings make local
discovery a peer of web search. The lexicon supplies the language-specific
surfaces; Rust consumes roles rather than embedding per-language phrase lists.

### Discovery was a guessed command, not an observed procedure

The prior implementation packed five guessed globs into one `find` and stopped
at its first match with `-print -quit`. It had no state representing an empty
observation, no widening transition, and no type verification. The replacement
planner advances only from recorded tool results:

```text
exact name → stable substring → bounded inventory → scoped conclusion
```

A command error terminates with an honest failure; only an empty successful
observation widens. Directory/file kind remains a constraint throughout, and a
near match is named rather than silently substituted.

### Web results were accepted before evidence validation

Fetch errors could be treated as content, all failed fetches could fall back to
search snippets, and short pages bypassed formalization entirely. That allowed
provider-denial text and navigation furniture to become answers. Tool-result
failure is now a multilingual seed role, failure detection is meanings-driven,
and any attempted-but-unsuccessful fetch set produces a scoped no-content
answer. Every successful page takes the same formalize/rank/synthesize path,
including short pages.

Removing the short-page shortcut revealed that sentence splitting treated the
period in `19.5` as a sentence boundary. The shared source and its mirrored
test-source implementation now preserve periods between ASCII digits.

### Follow-up, comparison, and report intent lacked the same loop

The definition follow-up lost its antecedent, comparison did not decompose its
two operands, and the Russian report surface did not reach the report flow.
Antecedent extraction now binds only the prior topic, comparison plans the two
lookups independently, and report intent is lexicon-driven across supported
languages. The current report implementation from #839 lowers each selected
destination separately and uses `formal-ai report body` for the final GitHub
transaction.

## Prior art and existing mechanisms

The reference-agent experiment is the most direct prior art because every
agent saw the identical fixture:

- Laguna tried the literal request, observed no match, widened, listed the
  parent, verified the directory, and named the mismatch in five steps.
- Codex demonstrated recovery after an empty narrow search.
- Claude demonstrated explicit discrepancy naming.
- Simple two-level `ls` strategies from Claude and Nemotron also beat the old
  compound command.

The implementation reuses repository mechanisms instead of adding a new
dependency: seed lexicon roles for multilingual intent, the existing agentic
planner/tool-result loop for state, the formalizer and evidence ranker for web
synthesis, the meta-recipe specification pattern for architecture drift, and
command-stream/xterm frame deduplication for real TUI verification. The recent
#839 report work on `main` supplied the current report-body/session contract.

## Requirement ledger

| Issue #840 requirement | Implementation and executable evidence |
| --- | --- |
| Model local filesystem search with `ROLE_*` constants symmetric with web search. | `data/seed/meanings-local-search.lino`, `src/seed/roles/intent.rs`, and `grounded_action_recipe_roles_are_live_seed_contracts`. |
| Route by meaning; possessives and articles must not flip the route. | `local_scope_dominates_search_verb_and_possessive_variations` plus the fixed and generated multilingual suites. |
| Cover 10–20 variations per supported language. | Four benchmark files contain 14 cases each (56 total) in English, Russian, Hindi, and Chinese. |
| Generate property cases rather than remembering only reported strings. | `generated_local_routing_property_holds_for_twelve_prompts_per_language` derives 48 Cartesian cases from live lexicon roles. |
| Let location nouns dominate generic search verbs; do not fall to unknown. | `local_path_discovery_benchmark_routes_every_case_to_find`, the Rust planner tests, and 8/8 browser-worker parity cases. |
| Use one inspectable action per step; forbid `;`, `&&`, and `-print -quit`. | Unit command assertions and the executed `assert_simple_command` integration fixture. |
| Widen after an empty result before claiming absence. | `empty_exact_local_lookup_widens_instead_of_claiming_absence` and the live fixture transcript. |
| State the scope of a true absence claim. | `absence_requires_exact_substring_and_bounded_inventory_observations` and ladder task `838.L4.d`. |
| Name fuzzy discrepancies rather than silently substituting. | `exact_observation_widens_then_reports_the_verified_directory_and_discrepancy`. |
| Honor requested file/folder kind and reject the PEM decoy. | The real Desktop fixture, `differently_typed_near_match_is_named_instead_of_reported_as_absent`, and the negative differential control. |
| Synthesize fetched evidence; strip titles and page furniture. | `short_fetched_pages_are_synthesized_instead_of_repeated_verbatim` and the #827 Agent CLI evidence. |
| Never answer from failed fetches. | `a_plain_text_provider_denial_is_not_used_as_research_evidence` and `research_reports_no_content_when_every_fetch_attempt_fails`. |
| Resolve definition follow-ups against their antecedent. | Same-turn, later-turn, and bare-follow-up unit tests plus the real #827 Agent CLI journey. |
| Decompose and attempt both sides of a comparison. | `comparison_is_decomposed_before_open_web_research` plus the real two-search #826 Agent CLI journey. |
| Recognize report intent in every supported language. | Seed-role tests and the exact `Зарепорти баг` four-round Agent CLI report journey with fake `gh`. |
| Narrate informatively in the user’s language without command leakage. | `src/agentic_coding/narration.rs`, multilingual response meanings, real client logs, and TUI transcript assertions. |
| Encode a topic-neutral recipe and pin it to source. | `data/meta/grounded-action-recipe.lino` and six specification tests under `grounded_action_meta_algorithm.rs`. |
| Preserve Rust/browser runtime parity. | Canonical browser seed hydration and `grounded_action_browser_parity_executes`. |
| Verify all reported cases through Formal AI’s own Agent CLI. | `run_issue_840.sh` executes local discovery, definition/follow-up, comparison, and Russian reporting; logs are committed here. |
| Exercise the real PEM-decoy filesystem. | `tests/integration/issue_840_grounded_action.rs`, the task ladder, and four native local clients create and query the real directory/file layout. |
| Gate regressions against reference agents in CI. | `run_differential_gate.sh` reruns the full ladder, then checks recovery, command complexity/count, type, discrepancy, scope, and decoy rejection against `baseline.json`. |
| Treat reference quota exhaustion as inconclusive. | The baseline policy and checker exclude inconclusive observations; the provider-429 transition is explicitly preserved. |
| Verify dialog structure at TUI level with frame deduplication. | The OpenCode PTY capture feeds command-stream frames through xterm, deduplicates wraps, and validates user → tool → result → final transitions. |
| Preserve a reproducible self-application slice. | Five smallest leaves are enumerated; one is Agent-CLI-authored (20%), with exact session evidence and byte-for-byte reproduction. |

## Reproduction and measured outcome

The 24-node task ladder decomposes #838, #827, and #826 through four levels.
On the pre-change v0.303.0 baseline it passed 8/24 nodes. Against the current
release binary it passes all 24:

```text
L1: 3/3
L2: 6/6
L3: 7/7
L4: 8/8

#838 local discovery: 10/10
#827 definition/follow-up: 7/7
#826 comparison/report: 7/7
```

The differential gate reports:

```text
issue #840 differential gate: PASS
(24/24 ladder nodes; 2 commands vs laguna-s-2.1-free's 5)
```

Run the deterministic core and release-server checks with:

```sh
cargo test --test unit issue_840::
cargo test --test integration issue_840 -- --test-threads=1
cargo build --release
experiments/issue_840_reference_agents/run_differential_gate.sh
```

Run the native clients and self-application evidence with:

```sh
ARTIFACT_DIR="$PWD/docs/case-studies/issue-840/agent-cli-e2e" \
  experiments/agent_cli_e2e/run_issue_840.sh
experiments/issue_840_self_authoring/run.sh
experiments/issue_840_self_hosting_projection/run.sh
```

## Self-application and residual uncertainty

Formal AI drove Agent CLI to author and verify one of the five reviewed
smallest leaves. Session `ses_069af4151ffep7T8HoP6ObsuBY` produced
[`grounded-action-authored-invariant.lino`](self-hosting/grounded-action-authored-invariant.lino);
the raw stream and server trace are retained beside it. The decomposition
records exactly one Formal-AI-authored leaf out of five, or 20%.

Fresh session `ses_06225d5bdffe90ToKcvaY0uedg` projected the merged tree:
300/300 owned source modules round-trip byte-for-byte through the links/meta
representation, the canonical self-AST census covers 301 documents, and the
focused issue subsystem contains 42 `agentic_coding` modules. The session,
projection shards, manifest, trace, and focused AST are all committed.

The remaining uncertainty is intentionally scoped. Local absence means “not
found after exact, substring, and bounded inventory checks in the requested
location,” never “does not exist anywhere.” Web synthesis is bounded by
successfully fetched evidence. Live third-party model quotas are
nondeterministic, so CI compares against recorded outcomes and treats quota or
permission exhaustion as inconclusive rather than inventing a pass or failure.
