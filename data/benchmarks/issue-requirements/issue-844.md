**Blocked by #702** (symbolic world models — contexts/formal systems) and **#843** (real fetched content with honest provenance).

Target behaviour, stated as a test we should be able to pass: point the system at a top Stack Overflow question and ask it to summarize. It should recursively gather the question, its answers, and the linked material; **merge all that text; deduplicate facts and statements** (not sentences — the same fact stated five different ways is one fact); rank what remains by how often and how authoritatively it is asserted; recheck the survivors; and present the important ones, shorter. All by logical reasoning — no neural inference, per `NON-GOALS.md:7`.

## What already exists (and is good)

`src/summarization/` is a real deterministic three-stage pipeline — formalize → summarize → deformalize — and it already has two of the things this issue needs:

- **Levels.** `SummarizationMode` (`mod.rs:174-190`): `Topic` (1–5 words), `Short` (~20%), `Standard` (~50%), `Full` (100%), `Expand` (~200%), with a numeric ladder at `:198-206`. `generate_chat_title` (`:35`) and `to_topic` (`:468`) already make "the ultimate summarization is a title" a first-class operation.
- **Recursion, bounded.** `summarize_repository_resource` recurses through a directory tree, demoting one rung per level via `one_step_shorter` (`:208-218`), with `Topic` as the fixed point. `examples/issue_563_folder_summary.rs` demonstrates it.
- **Determinism.** `mod.rs:4-6`: *"intentionally deterministic … No neural model or external API is consulted."*

The architecture is right. Three specific capabilities are missing.

## Gap 1 — no deduplication of facts or statements

`grep -E 'dedup|duplicate|distinct|HashSet|BTreeSet' src/summarization/**` returns **zero hits**. Two identical sentences from two sources both survive `summarize()` (`mod.rs:394-406`). Merging N sources today means concatenating N restatements of the same fact.

What is needed is dedup at the level of **statements**, not strings — recognising that "FBS means the seller stores the goods" and "under FBS, storage is the seller's responsibility" are one fact. That is a semantic-equivalence judgement, and it must be made symbolically: via the meaning lexicon and links network, not string similarity.

Two hard constraints from the project's own documents:

- `NON-GOALS.md:39` — *"Duplicate names should not be forced into one meaning when evidence shows different concepts."*
- `VISION.md:176` — *"If the same name points to two different meanings, the system should split them into separate concepts and record why."*

So dedup must be **conservative and reversible**: merge only when the links network supports equivalence, record the merge as a link with its justification, and split back when evidence later distinguishes the two. A merge that cannot be explained is a bug.

## Gap 2 — importance is a static table, not observed frequency

Ranking today is `filtered.sort_by_key(|stmt| Reverse(stmt.weight))` (`mod.rs:399`) over per-kind constants (`mod.rs:380-386`): `Feature => 70, UseCase => 65, Misc => 30, Example => 15, Install => 10`.

That is a reasonable prior for a README. It cannot express "this fact appears in nine of eleven sources" or "this is asserted by the accepted answer and contradicted by one comment." There is no term frequency, no occurrence counting, no salience anywhere in the module.

`GOALS.md:80` already asks for the missing ingredient:

> Persist meta-language expressions with usage (read) and change (write) counting derived from incoming and outgoing links, so frequently used or changed knowledge persists longer.

Frequency should come from **link counts in the network** — how many sources assert this statement, how many dependents rely on it — not from counting words in text. Combine that observed evidence with the existing static prior rather than replacing it.

## Gap 3 — no recursion over *sources*, only over *file trees*

Recursion today walks a directory. What this issue needs is recursion over research: fetch, extract statements, find what is unresolved, fetch again, stop when the unresolved set stops shrinking.

That loop is already designed and already has no driver — `src/option_network.rs:33-39`, with `unmet()` (`:510`) and `open_attributes()` (`:520`) generating the next queries, and `observe(candidate)` (`:436`) waiting for a caller that does not exist because nothing fetches. **#843 is the prerequisite.** Once content arrives, this loop is the natural driver, and it already terminates on a shrinking difference rather than a fixed turn count.

Recursion must be bounded and deterministic: a depth cap, a fixpoint condition (no new statements), and every fetch content-addressed so replay is byte-identical.

## Why #702 blocks this

Summarizing is not source-independent. "The answer is X" is true relative to a version, a platform, a question as asked. Merging statements from eleven sources without a context to merge *into* produces a pile of assertions with no frame in which they are jointly true — which is how a summary ends up containing two contradictory claims and no signal that they conflict.

#702 provides exactly the missing substrate: `Context` as a links network of dependent statements (`world_model.rs:212-227`), `merge_from` for combining contexts, `Stance::Supports`/`Stance::Contradicts` dependency edges (`world_model.rs:59-66`), and `recalculate` as a JTMS fixpoint (`:299-301`). Summarization should merge sources **as contexts**, so contradictions surface as `Contradicts` edges rather than as adjacent sentences, and so every retained statement carries the probability the world model computed for it.

Note that `WorldModel` is currently library-only — every `WorldModel::new()` call site is in `tests/unit/issue_649_world_model.rs`. Nothing in `src/solver*.rs` uses it. That wiring is #702's job, not this issue's.

## Requested scope

1. **Statement-level dedup** over the links network, conservative, with each merge recorded as an explainable link and reversible on new evidence.
2. **Evidence-weighted importance**: combine the existing static kind-prior with observed link frequency and source stance, so "most frequently asserted" and "most depended upon" are expressible.
3. **Recursive source gathering** driven by `option_network`'s unmet-difference loop, bounded by depth and a fixpoint, every fetch cached and content-addressed.
4. **Recheck before presenting**: surviving statements pass through the fact-checking path (companion issue) so a summary never presents a claim the system could have refuted.
5. **Merge into a context**, not into a list, so contradictions become edges and every statement carries a probability.
6. **Extend the level ladder downward**: today's shortest rung is `Topic` (1–5 words of prose). Add an identifier rung — function name, variable name, commit subject — where the output must satisfy syntactic constraints (an identifier, a length budget, a naming convention). This is the "ultimate summarization" case and it is genuinely different from prose truncation: `summarize(fn body) -> fn name` has to respect the target language's naming rules.

## Acceptance

- [ ] Summarizing N sources asserting the same fact in different words yields that fact once, with a link recording the merge and its justification.
- [ ] A merge that later proves to conflate two meanings can be split, per `VISION.md:176`.
- [ ] Ranking reflects observed frequency and source stance, not only the static kind table; a fact asserted by nine sources outranks one asserted by one, all else equal.
- [ ] Contradictory claims across sources surface as `Contradicts` edges and are reported as disagreement, never silently dropped or silently averaged.
- [ ] Recursive gathering terminates by fixpoint, respects a depth bound, and replays byte-identically from cache.
- [ ] The Stack Overflow case works end to end: question + answers + linked material → deduplicated, ranked, rechecked, shortened, with every retained statement traceable to its sources.
- [ ] An identifier-level summarization rung exists and produces valid identifiers under a length budget.
- [ ] No neural inference anywhere in the path (`NON-GOALS.md:7`); determinism preserved (`GOALS.md:54`).

## Dependencies

- **Blocked by #702** — contexts/formal systems as the merge target and probability substrate.
- **Blocked by #843** — real fetched content with honest provenance; there is nothing to summarize until pages actually arrive in Rust.
- **Pairs with** the fact-checking issue — step 4 above is that issue's entry point.
- **Related to #840 §3.5** — "synthesize, never echo source furniture" is the answer-side symptom of the missing merge/dedup stage; #827 is a live example (three correct sources fetched, page titles emitted).

