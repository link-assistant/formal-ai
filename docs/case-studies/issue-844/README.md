# Issue #844: Merging Many Sources Into One Context

Issue [#844](https://github.com/link-assistant/formal-ai/issues/844) asks for six
things at once: statement-level deduplication over the links network,
evidence-weighted importance, recursive source gathering with a recheck before
presenting, a merge target that is a *context, not a list*, and a new bottom rung
on the summarization ladder — a single identifier.

The unifying claim is that a summary of many sources is not a shorter list of
sentences. It is a small links network: one node per fact, a probability on each
node, a `Contradicts` edge wherever the sources disagree, and a retractable link
recording every sentence that was folded into which fact. Everything else in the
issue follows from that container.

## From blocker seam to production composition

The first draft stopped at a mock `SourceProvider` seam because issue
[#702](https://github.com/link-assistant/formal-ai/issues/702) (retrieval
planning) and issue [#843](https://github.com/link-assistant/formal-ai/issues/843)
(exact HTTP capture) were open. Both blockers are now closed. Issue
[#845](https://github.com/link-assistant/formal-ai/issues/845) also supplied the
named, disproof-first `FactChecker`. This branch therefore retains the mock seam
for small unit fixtures but no longer presents it as production behavior.

`execute_multi_source_summary` is the production composition:

1. `CachedSourceClient` captures exact response bytes, URL, timestamp, and
   SHA-256; live access is opt-in and the same call replays offline.
2. A classifier that receives that exact `SourceCapture` extracts text, trust
   tier, supplied attributes, and outgoing links. A failed capture remains a
   diagnostic and contributes no statement or evidence.
3. The shared recursive kernel gathers until the unmet difference is empty, a
   citation fixpoint is reached, or the explicit depth bound stops it.
4. The observations are deduplicated into one `Context` with an explicit
   `FormalSystem`, reversible provenance links, and contradiction edges.
5. `FactChecker` audits that named context before presentation. Unsupported
   statements remain inspectable in the context but are withheld.
6. The operation renders a deterministic human-gated learning proposal from the
   captures, merge receipts, contradictions, and fact-check audit. It never
   promotes itself into durable memory.

## What was built

| Module | Deliverable |
| --- | --- |
| `src/summarization/dedup.rs` | Statement-level merge: a `StatementSignature` over content terms and polarity, one `MergedStatement` per fact, a `MergeLink` per absorbed sentence, a `Contradiction` per asserted/denied pair, and `DedupReport::split` to undo a merge. |
| `src/summarization/importance.rs` | Evidence-weighted ranking: the kind prior blended with observed frequency (how many sources assert it) and stance (at what tier, and who denies it). |
| `src/summarization/gathering.rs` | The shared unmet-difference traversal plus the production adapter over exact `SourceCapture`s. `SourceProvider`/`SourceCache` remain fixture seams; `CachedSourceClient` is the real capture boundary. |
| `src/summarization/recheck.rs` | A compatibility preflight for callers that already have observations but no captures. It is deliberately not described as the production fact checker. |
| `src/summarization/context.rs` | The translation layer: ranked facts → a named `world_model::Context`, with `variant:`/`source:` receipt links and mutual `Contradicts` edges. |
| `src/summarization/pipeline.rs` | The production composition: exact gathering → named context → `FactChecker` audit → checked presentation → deterministic review-gated learning proposal. |
| `src/summarization/identifier.rs` | The identifier rung: `to_identifier` under a `NamingConvention` and an `IdentifierBudget`, rejecting reserved words and honouring the length budget. |

## Conservative on purpose

`NON-GOALS.md:39` forbids over-merging, and the merge takes that literally: it
compares content terms after dropping seed-known function words, and it does
**not** stem. So `"The crate is published"` and `"The crates are published"`
stay two facts, which is pinned by
`inflected_wordings_stay_separate_because_the_merge_does_not_stem`. The same
reasoning keeps subject pronouns out of the function-word list: dropping them
would conflate `"I broke it"` with `"you broke it"`. A missed merge is a
recoverable loss of concision; a wrong merge is a fabricated fact.

## Three defects the acceptance suite found outside the new code

The tests were written before the implementation, and four of the first failures
were real defects — three of them in code that predates this issue.

1. **`SourceCache` lost per-URL provenance.** Bodies were keyed by digest only,
   so an unoriginal mirror of a first-party page inherited the first party's
   `SourceTier` and its probability. Fixed by keeping content-addressed bodies
   *and* one `CacheEntry` per URL.
2. **A claim and its denial could both read as probable.** `Context::recalculate`
   is a Jacobi iteration over a frozen per-pass snapshot. When both sides of a
   contradiction carry evidence whose support saturates, the update is the exact
   swap `x ← 1 - x`: it oscillates forever, and the pass bound returned whichever
   half of the oscillation the last pass landed on — reporting *both* a
   first-party claim and a first-party denial as probable. `recalculate` now
   remembers visited states and collapses an exactly-repeated cycle to its mean,
   verifying that the mean is a fixpoint before claiming convergence. Two
   original sources that flatly disagree now settle at `0.5`, which is the honest
   reading. See `a_saturated_mutual_contradiction_settles_at_maximal_uncertainty`.
3. **`formalize` split sentences inside a token.** Every `.` ended a sentence, so
   `crates.io.` became two, and the fragment `io.` was ranked as a well-evidenced
   "fact" asserted by two sources. A `.` glued to an alphanumeric is now treated
   as inside a token; other terminators stay eager.

A fourth failure was a design property rather than a bug (the no-stemming rule
above), and it was pinned as its own regression test instead of being papered
over by loosening the merge.

One more behaviour needed a knob: `StatementKind::Install.is_boilerplate()` is
true, so the project-summary path drops install commands. In a merged thread
whose *question* is "how do I install this", the install command is the answer.
`SummarizationConfig::keeping_boilerplate()` lets the evidence weight, not the
sentence kind, decide.

## The worked example

`cargo run --example issue_844_statement_merge` runs the issue's Stack Overflow
case end to end over a fixed thread: a question, three answers, and a first-party
page that cites the question back (a citation cycle). The full output is kept in
[`test-logs/example-output.txt`](test-logs/example-output.txt); the shape of it:

```
stop depth=2 converged=true depth_bound=false open=[license]
cached: 5 urls, 5 distinct bodies

=== replay from the warm cache ===
no fetch lines above means the provider was never called; byte-identical trace: true

=== merged facts, ranked by evidence ===
 26  p=1.000  Install it with cargo install formal-ai.  [asserted by 3 of 5 sources]
 26  p=0.940  How do I install formal-ai?  [asserted by 1 of 5 sources]
 23  p=0.800  The crate is published on crates.io.  [asserted by 1 of 5 sources, denied by 1]
 23  p=0.300  The crate is not published on crates.io.  [asserted by 1 of 5 sources, denied by 1]

=== disagreements, reported rather than resolved ===
  "The crate is published on crates.io" (asserted by 1 of 5 sources, denied by 1) contradicts "The crate is not published on crates.io" (asserted by 1 of 5 sources, denied by 1)

=== the ladder, all the way down ===
Full: Install it with cargo install formal-ai. How do I install formal-ai? The crate is published on crates.io (disputed: asserted by 1 of 5 sources, denied by 1).
Standard: Install it with cargo install formal-ai. How do I install formal-ai?
Short: Install it with cargo install formal-ai.
Topic: Install it with cargo install
Identifier: install_cargo_install
```

Three properties are worth reading off that output:

- the recursive gather stopped by **fixpoint** (`converged=true`), not by hitting
  the depth bound, even though the sources form a cycle — and it says honestly
  that `license` was never satisfied;
- the second gather produced a **byte-identical** trace with zero provider calls,
  so a cached run replays exactly;
- the contradiction is **kept**: both sides remain in the context at
  `truth:0.615341` and `truth:0.231810` with mutual `contradicts:` edges, and the
  denial is reported as a disagreement rather than dropped or averaged into the
  claim. The unoriginal denial contributes no support of its own, which is why
  the claim stays the more probable side.

The label rungs come from the top-ranked fact, because the ranking is the merge's
total order — asking `to_topic` to pick a maximum among equal weights would name
the context after whichever tied fact happened to sort last.

`cargo run --example issue_844_captured_pipeline` exercises the production
boundary rather than the fixture seam. Its transport returns exact bytes; the
classifier extracts text and links from each `SourceCapture`; the operation
merges into the `captured_parser_reports` formal system, runs `FactChecker`,
renders both prose and the identifier rung, and emits the human-gated learning
proposal. It then creates an offline client over the warm cache and asserts that
the checked summary and complete learning proposal are byte-identical with zero
new transport requests.

## Determinism

There is no neural inference anywhere in the path (`NON-GOALS.md:7`), and the
result does not depend on the order sources arrived in: signatures are
content-addressed, ranking breaks ties on the signature key, and posteriors are
rounded to `TRUTH_VALUE_DECIMALS`, so two runs agree to the last digit
(`the_merge_is_deterministic_and_independent_of_source_order`, which compares
probabilities bit-for-bit). The production regression goes further: it compares
the recursive trace, checked summary, named-context audit, and learning proposal
between a live fixture execution and offline cache replay.

## Wording is data, traces are machine records

The two kinds of line in that output are governed by different rules
(`docs/design/no-hardcoded-natural-language.md`, R379):

- `asserted by 3 of 5 sources`, `, denied by 1` and the `(disputed: …)` wrapper
  are prose a reader sees, so they live in
  [`data/seed/multilingual-responses-summarization.lino`](../../../data/seed/multilingual-responses-summarization.lino)
  — one record per intent per language (en, ru, hi, zh) — and are rendered by
  `summarization::vocabulary::rendered_response`. The Rust source carries only
  intent slugs and placeholder names, and `to_statements_in` /
  `ImportanceScore::evidence_summary_in` take the language from the caller
  (`the_evidence_wording_comes_from_the_seed_for_every_supported_language`);
- `fetch url=… depth=… digest=…` and `verdict=… sources=…` are machine records,
  not sentences: every whitespace-separated field is a `name=value` pair with a
  slug on the left, which is what the trace assertions in
  `tests/unit/issue_844_statement_merge.rs` pin. They are never translated.

## Self-hosting evidence

The implementation remains honestly human-authored. One smallest leaf of this
same issue task was authored by Formal AI through the external Agent CLI:
session `ses_050f9a572ffefpehWRjysug6cv` planned the requested literal change,
used the client-owned write tool to create
`multi-source-summary-honesty-invariant.lino`, used the shell tool to verify it,
and reported completion in four chat rounds.

The canonical
[`data/meta/multi-source-summary-honesty-invariant.lino`](../../../data/meta/multi-source-summary-honesty-invariant.lino)
is byte-for-byte equal to the generated artifact under
[`self-hosting-authorship/`](self-hosting-authorship/). That directory also
contains the raw Agent CLI log, Formal AI server trace, and reviewed
decomposition. Four implementation leaves are human-authored; the invariant is
Agent-CLI-authored: one of five leaves, or 20%. The reproducible harness is
[`experiments/issue_844_self_authoring/run.sh`](../../../experiments/issue_844_self_authoring/run.sh).

After the final source tree was fixed, a separate real Agent CLI self-AST run
(`ses_050ca60f5ffeXLhV1fH0Q2l0qW`) regenerated the exhaustive 343-document
census through the same deterministic AST engine. Its client stream, server
trace, focused artifact, and session record are under
[`self-hosting-census/`](self-hosting-census/). That final-source evidence also
advances issue #842's complete workspace-census ratchet; it is not counted as a
second same-task implementation leaf.

[`experiments/issue-844-self-hosting-evidence/run.sh`](../../../experiments/issue-844-self-hosting-evidence/run.sh)
is a thin wrapper over #839's harness — it only picks the axes and the output
directory, because that harness already accepts `OUT`, `ONLY`, `LOG` and `PORT`
so a branch can record its own sessions without rewriting committed transcripts.
The three earlier sessions under [`self-hosting-evidence/`](self-hosting-evidence/)
remain useful whole-tree self-model evidence, but they are not counted as
same-task authorship:

| Session | Recipe | Formal AI's artifacts |
| --- | --- | --- |
| `ses_0676fd278ffeuv3bsN1LHMgrcn` | source-to-links (#558) | `self-source-links.lino`, plus `whole-repository-projection-0{1,2,3}.lino` — all 305 owned modules translated to Links Notation and back, each round-trip proven byte-for-byte, `src/summarization/{dedup,importance,gathering,context,identifier}.rs` among them |
| `ses_0676f331effe6WXVXKl1214qa9` | CST/AST census (#538/#673) | `self-ast.lino`, and the whole-workspace rendering of the same `ast_census` under `data/meta/self-ast/**` |
| `ses_0676bc5a4ffe0RbQFKLUvNTh78` | grounded self-explanation (#558) | `how-formal-ai-works.lino`, resolved against this branch's own source manifest (`source_tree_8a22e53f3473e9d1`) |

None of it is prose about the work: each artifact is a deterministic function of
the source tree captured by its session. The `.jsonl` transcripts and `.log`
server traces are the excluded evidence bundle
(`scripts/self-hosting-metric.rs::CAPTURED_ARTIFACT_EXTENSIONS`) that binds each
artifact to the session id that authored it.

## Traceability

[`requirements.md`](requirements.md) maps R844-01…R844-14 to named regression
tests, and `tests/unit/docs_requirements_issue_844.rs` parses that table: a
requirement whose test does not exist fails the build. Test logs are in
[`test-logs/`](test-logs/); the issue and pull-request conversations at the time
of implementation, plus the former blockers, are preserved in [`raw-data/`](raw-data/).
