# Architecture: Formalization, Temperature, and Symbolic Probability

Part of the [architecture overview](../../ARCHITECTURE.md) (§5–§6.1). Section
numbers match the overview's table of contents; paths are relative to the
repository root.

## 5. Formalization

Formalization converts free-form text into typed link references. The
target shape:

```text
formalization
  subject_q   "wikidata:Q14660"     # noun phrase -> Q-id
  predicate_p "wikidata:P31"        # verb phrase -> P-id
  object_q    "wikidata:Q170978"
  source_text "is a sorting algorithm"
  language    "en"
```

Where a P/Q-id does not yet resolve, the formalizer falls back to a
`wikipedia:` or `wiktionary:` URL. The fallback chain is:

1. Wikidata item / property (fully formal, language-independent).
2. Wikipedia article (per-language; bridges to Wikidata through `Q-id` if
   one exists for the article).
3. Wiktionary entry (per-language; useful for verbs and idioms that
   Wikidata does not model).
4. Raw text in `as_is` only, with a `formalization_unresolved` flag.

The formalizer is deliberately allowed to emit **multiple** interpretations
per phrase. Selection happens in step 6.

The current implementation has two cooperating formalization layers:

- `rust/src/concepts.rs` handles seed concept lookup through explicit aliases and
  context hints.
- `rust/src/translation/formalization.rs` handles arbitrary prompt fragments with
  a deterministic multilingual label table, concept-seed Q-id reuse, scored
  Wikidata P/Q anchors, and explicit Wikipedia/Wiktionary/raw fallbacks.

`rust/src/solver.rs` records the selected formalization as `formalization:*`
events before local search, including typed links such as
`formalization:predicate_p:wikidata:P31`,
`formalization:subject_q:wikidata:Q89`, and
`formalization_unresolved:<surface>` for later translation-gap handling.

---

## 6. Temperature-Based Interpretation Selection

Each candidate formalization carries a `score` field. The solver normalizes
the scores by a softmax controlled by the same temperature knob a neural
network would use:

```text
P(c_i) = exp(score_i / T) / Σ exp(score_j / T)
```

- `T = 0`  → deterministic; the highest-scored candidate always wins.
- `T = 1`  → maximum configured exploration across the scored candidates.

The temperature is sourced from `SolverConfig`. `rust/src/translation/selection.rs`
normalizes 0..1000 formalization scores to 0.0..1.0, applies a stable
softmax, and uses a content-hash-seeded draw whenever the solver must guess
under ambiguity. This keeps the same prompt + same config deterministic.

If the top-two probabilities are within ε (configurable through
`SolverConfig.questioning_rigor`), the solver:

- if the configuration permits guessing, samples from the softmax distribution
  with the impulse hash as a seed and records a
  `policy:guessed_under_ambiguity` event so the trace is honest;
- otherwise, emits a clarifying-question intent (the smallest question that
  separates the candidates) and stops the pipeline until the user replies.

The seeded-from-impulse-hash draw in `rust/src/translation/selection.rs` keeps
guessing deterministic per prompt, so the same input + same config produces the
same answer.

---

## 6.1 Symbolic Probability Evidence

Issue #279 adds a narrow probabilistic layer without changing the project's
non-neural boundary. `rust/src/probability.rs` stores evidence as ordinary
append-only records:

```text
probability_evidence
  id "probability_..."
  target "formalization:subject=wikidata:Q89 predicate=wikidata:P279 object=wikidata:Q3314483"
  observation "taxonomy_context_prefers_subclass"
  weight "1.000000"
  model "bayesian_evidence"
  provenance "source:seed:test"
  recorded_at "2026-05-26T00:00:00Z"
```

The current supported models are deliberately small:

- `bayesian_evidence` adds independent symbolic evidence weights to a
  candidate's prior score before temperature softmax.
- `markov_transition` applies a weight only when the previous symbolic state
  matches `ProbabilityRankingConfig::markov_from`.

Both models operate on symbolic target IDs, not neural logits. The selector
still produces a deterministic `FormalizationSelection`: the same prompt, same
probability store, same config, and same impulse hash produce the same selected
candidate. Evidence records can carry `source_url`, `fetched_at`, `sha256`, and
`cached` fields; offline mode ignores live-only evidence, preserves cached
source provenance, and emits `policy:offline` for skipped live evidence.

The solver exposes `UniversalSolver::solve_with_probability_store` for callers
that have a probability store. The default `FormalAiEngine::answer` path uses
an empty store, so existing deterministic behavior is unchanged until evidence
is explicitly supplied.

### Evidence count and counted-utility ranking (issue #449)

Issue #449 ports the interpretable, non-neural mechanisms from Kolonin's
"Interpretable Experiential Learning" (arXiv:2605.00940) onto this same
associative layer. The paper models behaviour as a transition graph where every
transition carries both a **utility `U`** and an **evidence count `C`**; the
existing store already accumulated `U` (`target_weight`) but folded the count
into it. The additions keep the two separate without leaving the non-neural
boundary:

- `ProbabilityStore::target_evidence_count` returns the count `C` of append-only
  observations supporting a target, using the same offline and Markov-state
  filters as `target_weight`, so `U` and `C` always describe the same evidence
  subset.
- `ProbabilityRankingConfig` gains three opt-in fields mirroring the paper's
  decision-policy hyperparameters: `counted_utility` (`CU` — rank by
  `argmax(U·C)` instead of `argmax(U)`), `min_transition_utility` (`TU`), and
  `min_transition_count` (`TC`). A transition below `TU`/`TC` is treated as
  under-evidenced: its learned evidence is withheld and the candidate falls back
  to its structural prior.
- `RankedProbabilityCandidate` exposes `evidence_count` and `similarity` next to
  `evidence_weight`, so each ranked option is locally interpretable — it carries
  the utility, the number of observations behind it, and how the evidence was
  matched.

The defaults (`counted_utility = false`, both thresholds `None`) reproduce the
prior additive behavior, which equals the paper's recommended `CU=False`,
`TU=0`, `TC=1` baseline; existing callers are unaffected until they opt in.

#### Similarity fallback (`SS`)

The paper falls back to the **closest** stored state when no exact transition
matches, gated by a cosine-similarity floor `SS`. The same idea is ported
symbolically:

- `symbolic_cosine_similarity` computes a deterministic bag-of-words cosine over
  the alphanumeric tokens of two target IDs — no embeddings, no neural logits.
- `ProbabilityStore::nearest_similar_evidence` finds the highest-similarity
  stored target (above the `similarity_threshold` floor, ties broken by target
  name) and lends its evidence, scaled by the similarity, to a candidate that
  has no exact evidence of its own.
- The fallback only fires when the candidate's direct evidence count is zero,
  and the borrowed evidence still passes the `TU`/`TC` gate, so an
  under-evidenced neighbour cannot smuggle weight past the thresholds.

#### Episode-wide global feedback

`ProbabilityStore::reinforce_transition_path` mirrors the paper's one-shot,
episode-wide reward: given an ordered state path and a reward, it appends one
`markov_transition` observation per adjacent pair, so a whole successful episode
reinforces every transition it traversed in a single append-only pass. The
records replay deterministically through the event log and link-store projection
like any other evidence.

#### Generalized decision policy

The `CU`/`TU`/`TC`/`SS` knobs are grouped into a single
`ProbabilityDecisionPolicy` value. `SolverConfig::probability_policy` threads it
through every selection use case — `select_formalization_candidate_with_policy`
(the formalization selector) and `try_synthesize_from_sub_results` (the
synthesis ranker) — via `ProbabilityRankingConfig::with_decision_policy`. The
default policy is the paper's baseline, so the store-only entry points delegate
with it and existing surfaces are byte-for-byte unaffected until a caller opts
in. The full analysis is archived in `docs/case-studies/issue-449/`.
