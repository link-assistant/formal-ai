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

## Scope, and the two open blockers

The issue declares itself blocked by [#702](https://github.com/link-assistant/formal-ai/issues/702)
(retrieval planning) and [#843](https://github.com/link-assistant/formal-ai/issues/843)
(real HTTP fetching), both still open. Waiting for them would have blocked the
five deliverables that need no network at all, so this change draws the seam at
the fetch:

- `SourceProvider` is a one-method trait (`fetch(url) -> Option<FetchedSource>`).
  The gathering loop, its depth bound, its fixpoint test, and its
  content-addressed cache are shipped and tested against it.
- `#843` will supply an HTTP implementation of that trait. Nothing in this change
  performs network I/O, so the whole pipeline is testable and byte-for-byte
  reproducible today.
- The merge target is the existing `world_model::Context`, not a new engine, so
  the JTMS fixpoint and the relative-meta-logic evidence rules apply unchanged.

## What was built

| Module | Deliverable |
| --- | --- |
| `src/summarization/dedup.rs` | Statement-level merge: a `StatementSignature` over content terms and polarity, one `MergedStatement` per fact, a `MergeLink` per absorbed sentence, a `Contradiction` per asserted/denied pair, and `DedupReport::split` to undo a merge. |
| `src/summarization/importance.rs` | Evidence-weighted ranking: the kind prior blended with observed frequency (how many sources assert it) and stance (at what tier, and who denies it). |
| `src/summarization/gathering.rs` | The unmet-difference loop: fetch a seed, read what it supplies, follow what it links, stop at the depth bound, at the fixpoint, or when nothing is missing. `SourceCache` stores bodies content-addressed with one entry per URL. |
| `src/summarization/recheck.rs` | The fact-checking gate: each ranked fact is re-assessed from its evidence and gets a `Verdict`; unsupported facts are withheld from the rendered summary but stay in the context. |
| `src/summarization/context.rs` | The translation layer: ranked facts → `world_model::Context`, with `variant:`/`source:` receipt links and mutual `Contradicts` edges. |
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

## Determinism

There is no neural inference anywhere in the path (`NON-GOALS.md:7`), and the
result does not depend on the order sources arrived in: signatures are
content-addressed, ranking breaks ties on the signature key, and posteriors are
rounded to `TRUTH_VALUE_DECIMALS`, so two runs agree to the last digit
(`the_merge_is_deterministic_and_independent_of_source_order`, which compares
probabilities bit-for-bit).

## Self-hosting evidence

Every commit that implements the merge is hand-authored, so none of them carries
a `Formal-AI-Session` trailer: `CONTRIBUTING.md` forbids attaching those trailers
to human work and says plainly that "an honest 0% release is valid". Keeping that
honest is what makes the differential ratchet in `Self-Hosting Evidence Check`
fall on this branch, and the answer issue #839 established is to let Formal AI
author release work of its own here rather than to relabel ours.

[`experiments/issue-844-self-hosting-evidence/run.sh`](../../../experiments/issue-844-self-hosting-evidence/run.sh)
is a thin wrapper over #839's harness — it only picks the axes and the output
directory, because that harness already accepts `OUT`, `ONLY`, `LOG` and `PORT`
so a branch can record its own sessions without rewriting transcripts that are
already committed. Three real Agent CLI sessions (Agent CLI → local Formal AI
server, private empty memory, no dreaming) ran against *this* branch's tree, and
[`self-hosting-evidence/`](self-hosting-evidence/) holds what they produced:

| Session | Recipe | Formal AI's artifacts |
| --- | --- | --- |
| `ses_0676fd278ffeuv3bsN1LHMgrcn` | source-to-links (#558) | `self-source-links.lino`, plus `whole-repository-projection-0{1,2,3}.lino` — all 305 owned modules translated to Links Notation and back, each round-trip proven byte-for-byte, `src/summarization/{dedup,importance,gathering,context,identifier}.rs` among them |
| `ses_0676f331effe6WXVXKl1214qa9` | CST/AST census (#538/#673) | `self-ast.lino`, and the whole-workspace rendering of the same `ast_census` under `data/meta/self-ast/**` |
| `ses_0676bc5a4ffe0RbQFKLUvNTh78` | grounded self-explanation (#558) | `how-formal-ai-works.lino`, resolved against this branch's own source manifest (`source_tree_8a22e53f3473e9d1`) |

None of it is prose about the work: each artifact is a deterministic function of
the source tree this branch leaves behind, so re-running the script reproduces
every `.lino` byte-for-byte. The `.jsonl` transcripts and `.log` server traces are
the excluded evidence bundle
(`scripts/self-hosting-metric.rs::CAPTURED_ARTIFACT_EXTENSIONS`) that binds each
artifact to the session id that authored it.

## Traceability

[`requirements.md`](requirements.md) maps R844-01…R844-10 to named regression
tests, and `tests/unit/docs_requirements_issue_844.rs` parses that table: a
requirement whose test does not exist fails the build. Test logs are in
[`test-logs/`](test-logs/); the issue and pull-request conversations at the time
of implementation, plus both blockers, are preserved in
[`raw-data/`](raw-data/).
