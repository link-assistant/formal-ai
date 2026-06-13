# Case study — Issue #449: Is arXiv:2605.00940 useful for formal-ai?

> **Verdict: yes.** The paper's distinctive mechanisms map cleanly onto
> formal-ai's associative, deterministic, non-neural stack, and the most
> valuable ones (an explicit *evidence count* kept separate from utility, a
> *counted-utility* decision policy, and *utility/count thresholds* for
> under-evidenced transitions) have been ported into `src/probability.rs` in
> this pull request. None of them require neural-network inference; all of them
> are additive and backward-compatible.

- **Issue:** [#449](https://github.com/link-assistant/formal-ai/issues/449) —
  "Can https://arxiv.org/abs/2605.00940 paper be useful in our project?"
- **Pull request:** [#450](https://github.com/link-assistant/formal-ai/pull/450)
- **Paper:** Anton Kolonin, *Interpretable experiential learning based on state
  history and global feedback*, `arXiv:2605.00940v1 [cs.LG]`, 1 May 2026.

This folder is the compiled evidence base for the analysis. The verbatim
external captures live under [`raw-data/`](./raw-data/) (issue/PR JSON
snapshots, the paper PDF, the extracted full text, and online-research notes);
this README is the analysis built on top of them.

---

## 1. The paper at a glance

The paper introduces a learning model that is **interpretable by
construction** rather than interpretable after the fact. Its core ideas:

- **Behaviour is a transition graph between sets of states.** There are no
  hidden or latent variables; every node is an interpretable situation built
  from observable "objects, events and properties" by a *state transformer*.
- **Every transition carries two numbers: a utility `U` and an evidence count
  `C`.** `U` is how good the transition turned out to be; `C` is how many times
  it has been observed. They are tracked separately on purpose.
- **Learning uses global feedback.** At the end of an episode the outcome is
  applied to *every* transition that was traversed, in one shot — unlike deep
  Q-learning, which nudges weights gradually with per-step gradients.
- **Decisions are an argmax over stored transitions.** With the *counted-utility*
  switch `CU = False` the next state is `argmax_s U`; with `CU = True` it is
  `argmax_s U·C`, so a frequently confirmed transition can be preferred over a
  rarely seen but high-utility one. When the current situation is not matched
  exactly, a **cosine-similarity fallback** reuses the nearest stored situation
  above a similarity threshold `SS`.
- **It runs on low-end hardware in real time** while staying competitive with
  deep-RL baselines, because the "model" is just a graph database of counts and
  utilities, not a trained network.

The architecture is three layers: **state transformer → state learning layer
(an in-memory graph database) → decision-making layer.** The key
hyperparameters (with the paper's defaults) are `CS=2`, `LM=2`, `SR=True`,
`CU=False`, `EA=False`, `SC=2`, `SS=0.9`, `TU=0`, `TC=1`. See
[`raw-data/online-research.md`](./raw-data/online-research.md) for the full
hyperparameter table, the Breakout baseline figures (model 196 @ ~41M frames
vs. DQN 168, Rainbow-IQN 176, human 31), the author's background, and the
reference implementation (`github.com/aigents/pygents`).

---

## 2. Why this fits formal-ai (the central finding)

formal-ai is a **deterministic, symbolic** assistant. Its hard constraint is
the "no neural-network inference" boundary stated at the top of
`src/probability.rs` and in `ARCHITECTURE.md`. The associative technological
stack it uses is:

- **Links / doublets** as the storage primitive (Links Data Store style).
- **Links Notation (`.lino`)** as the serialization format.
- An **append-only event log** plus a **link-store projection**.
- **Deterministic, replayable computation** over those links.

The paper's model is, structurally, *already* an associative store:

| Paper concept | formal-ai associative analogue |
| --- | --- |
| Transition graph between state sets | Markov-style transition records (`ProbabilityModel::MarkovTransition`) keyed by `transition_from` |
| State learning layer = in-memory graph DB | The append-only `ProbabilityStore` projected into the link store / event log |
| Utility `U` on a transition | Accumulated weight: `ProbabilityStore::target_weight` |
| Evidence count `C` on a transition | **New:** `ProbabilityStore::target_evidence_count` |
| Decision `argmax U` / `argmax U·C` | Deterministic ranking in `rank_probability_candidates` (temperature `0.0` ⇒ argmax) |
| Counted-utility switch `CU` | **New:** `ProbabilityRankingConfig::counted_utility` |
| Transition thresholds `TU` / `TC` | **New:** `min_transition_utility` / `min_transition_count` |
| Global feedback (episode-wide update) | Append-only evidence records replayed into the log (one outcome ⇒ many records) |
| Interpretability of each decision | Every `RankedProbabilityCandidate` exposes prior, `evidence_weight` (`U`), `evidence_count` (`C`), posterior |

The match is not superficial: the paper's whole pitch — *interpretable,
count-and-utility-based, deterministic, runs without a GPU* — is the same design
philosophy formal-ai already commits to. What the existing module was missing,
before this PR, was the **separation of count from utility** and the
**decision-policy knobs** built on top of it. That gap is exactly what we
closed.

---

## 3. Requirements extracted from the issue

The issue body is short but contains several distinct, testable requirements.
Enumerated atomically (`R0`…`R10`):

| # | Requirement (from the issue text) |
| --- | --- |
| **R0** | Decide **whether** the paper can be useful in this project. |
| **R1** | If it can, **apply all best practices from the paper** — but using **our associative technological stack** (no neural inference). |
| **R2** | **Collect data** related to the issue into this repository. |
| **R3** | **Compile that data** into the `./docs/case-studies/issue-{id}` folder (here: `issue-449`). |
| **R4** | Use the data to do a **deep case-study analysis**. |
| **R5** | **Search online** for additional facts and data. |
| **R6** | Produce a **list of each and all requirements** from the issue. |
| **R7** | **Propose possible solutions and solution plans for each requirement.** |
| **R8** | **Check known existing components/libraries** that solve a similar problem or can help. |
| **R9** | **Plan and execute everything in this single pull request** (#450). |
| **R10** | Continue **until each and every requirement is fully addressed** and everything is done. |

---

## 4. Per-requirement solution plans and status

Each requirement is listed with the plan taken and its status in this PR.

### R0 — Is it useful? → **Resolved: yes**
**Plan:** read the full paper, map every mechanism to the associative stack,
and judge usefulness against the non-neural constraint.
**Outcome:** Useful. The transition-graph-with-utility-and-count model is a
direct fit; see §2. The judgement is backed by a concrete, merged-quality
implementation rather than an opinion.

### R1 — Apply best practices on our stack → **Resolved**
**Plan:** port the paper's *distinctive, stack-compatible* practices and
explicitly decline the ones that would violate the non-neural boundary or add
unjustified scope.

Ported (in `src/probability.rs`):
1. **Evidence count `C` separate from utility `U`** — `target_evidence_count`.
2. **Counted-utility decision policy `CU` (`argmax U·C`)** — `counted_utility`.
3. **Transition utility/count thresholds `TU`/`TC`** — `min_transition_utility`
   / `min_transition_count`, which withhold an under-evidenced transition's
   learned evidence so the candidate falls back to its structural prior.
4. **Local interpretability of every decision** — `RankedProbabilityCandidate`
   now exposes `evidence_count` next to `evidence_weight`, so each ranked option
   carries both the utility and the number of observations behind it.

Consciously **not** ported, with rationale (documented here so the decision is
auditable):
- **State transformer / Atari pixel pipeline** — formal-ai's "states" are
  symbolic formalization candidates, not game frames; the transformer stage is
  already covered by the existing translation layer.
- **Cosine-similarity `SS` fallback** — formal-ai already has a deterministic
  similarity/guessing path in `src/translation/selection.rs`. A symbolic cosine
  fallback is a reasonable *future* extension, but adding it now would change
  the guess path's determinism without a requirement driving it. It is listed
  as future work rather than silently bundled.
- **Episode-level reinforcement loop / reward backprop** — the append-only
  evidence model already expresses "one outcome updates many records"; a full
  RL training loop is out of scope for a symbolic assistant and would need its
  own issue.

### R2 / R3 — Collect & compile data → **Resolved**
**Plan:** capture every external artefact into `docs/case-studies/issue-449/`.
**Outcome:** [`raw-data/`](./raw-data/) contains:
- `issue-449.json`, `issue-449-comments.json`, `pr-450.json` — GitHub snapshots.
- `paper-2605.00940.pdf` — the paper.
- `paper-2605.00940-extracted.txt` — full extracted text (used for the figures).
- `online-research.md` — author, related work, baselines, reference impl.

### R4 — Deep analysis → **Resolved**
**Plan:** write the analysis on top of the compiled data.
**Outcome:** this README (§1–§7) plus the supporting notes.

### R5 — Search online → **Resolved**
**Plan:** corroborate the paper's claims and find the author/related work and
any reusable implementation.
**Outcome:** captured in [`raw-data/online-research.md`](./raw-data/online-research.md)
— author affiliations (Aigents / SingularityNET / Novosibirsk State
University), the lineage paper (AGI 2021, Springer LNCS), the Breakout
baselines, and the `aigents/pygents` reference implementation, each with source
URLs.

### R6 — List all requirements → **Resolved**
**Plan & outcome:** §3 above (`R0`…`R10`).

### R7 — Solutions & plans per requirement → **Resolved**
**Plan & outcome:** this section (§4), one entry per requirement.

### R8 — Existing components/libraries survey → **Resolved**
**Plan & outcome:** §5 below.

### R9 / R10 — One PR, fully done → **Resolved in this PR**
**Plan:** land the data, the analysis, the code, the tests, the architecture
docs, and a changelog fragment together on `issue-449-2895c0132690` / PR #450,
and run the full quality gates. See §6 (what shipped) and §7 (verification).

---

## 5. Existing components / libraries survey

The issue asks us to "check known existing components/libraries that solve a
similar problem or can help in solutions". The relevant landscape:

**Inside this repository (preferred — reused, not reinvented):**
- **`src/probability.rs`** already implemented Bayesian-evidence and
  Markov-transition records with a deterministic softmax ranking. This is the
  natural home for the paper's mechanisms, so the work was an *extension* of an
  existing component rather than a new subsystem.
- **`src/event_log.rs` + `src/link_store.rs`** provide the append-only log and
  link-store projection that play the role of the paper's "in-memory graph
  database" — already associative, already deterministic.
- **`src/translation/selection.rs`** already provides deterministic selection,
  margin-based clarification, and a seeded "guess under ambiguity" path — the
  symbolic counterpart of the paper's similarity fallback.

**The paper's own reference implementation:**
- **`github.com/aigents/pygents`** (Python) — the author's machine-learning
  experiments for the Aigents project. Useful as a conceptual reference for the
  algorithm, but it is Python and research-grade; it is **not** a dependency
  candidate for a deterministic Rust assistant. We treated it as documentation.

**External Rust crates considered (and why we did not add one):**
- **`petgraph`** — a general graph library. The transition graph here is just
  keyed append-only records; a full graph crate would add a dependency without
  buying anything the link store does not already give us.
- **`ndarray` / `sprs`** — would help only if/when the cosine-similarity `SS`
  fallback is implemented as real vector math. Not needed for the mechanisms
  shipped in this PR; noted as future-work tooling.
- **doublets-rs / Links Data Store** — the associative substrate the project is
  built around; the probability records already serialize to Links Notation and
  replay into the link store, so the paper's "graph DB" requirement is satisfied
  by the substrate we already use.

**Conclusion:** the best "library" for this problem was the project's own
associative stack. The change adds **zero new dependencies**.

---

## 6. What shipped in this pull request

All code changes are additive and backward-compatible. Defaults reproduce the
exact prior behaviour, which also equals the paper's recommended baseline
(`CU=False`, `TU=0`, `TC=1`).

**`src/probability.rs`**
- `ProbabilityStore::target_evidence_count(target, offline, markov_from)` —
  counts the append-only observations supporting a target, using the *same*
  offline and Markov-state filters as `target_weight`, so utility `U` and count
  `C` always describe the same evidence subset.
- `ProbabilityRankingConfig` gains three fields:
  - `counted_utility: bool` (the paper's `CU`) — when `true`, the contribution
    is `U·C` instead of `U`.
  - `min_transition_utility: Option<f32>` (the paper's `TU`).
  - `min_transition_count: Option<usize>` (the paper's `TC`).
- `RankedProbabilityCandidate` gains `evidence_count: usize`, surfaced next to
  `evidence_weight` so every ranked option is locally interpretable.
- `rank_probability_candidates` now reads the count, applies the `TU`/`TC`
  gate (an under-evidenced transition has its evidence withheld and falls back
  to its structural prior), then forms `prior + (CU ? U·C : U)`.
- `count_to_f32` helper — a saturating `usize → f32` that preserves `0 ⇒ 0.0`
  (kept distinct from the existing `usize_to_f32`, which clamps to `≥ 1` for
  softmax-denominator safety).

**Call sites updated for the new fields (behaviour unchanged):**
`src/solver_synthesis.rs` and `src/translation/selection.rs` spread
`..ProbabilityRankingConfig::default()` into their existing config literals.

**Tests — `tests/unit/specification/probabilistic_reasoning.rs` (+6):**
- `evidence_count_is_tracked_separately_from_accumulated_utility`
- `default_ranking_config_preserves_additive_posterior`
- `counted_utility_prefers_frequently_confirmed_transition`
  (A: `U=0.9, C=1` vs B: `U=0.8, C=2` → A wins under `CU=false`, B wins under
  `CU=true` because `0.8·2 = 1.6 > 0.9`)
- `transition_count_threshold_withholds_under_evidenced_evidence`
- `transition_utility_threshold_withholds_low_utility_evidence`
- `markov_evidence_count_respects_transition_state`

**Mirror crate kept in sync:** `tests/source/probability.rs`,
`tests/source/translation/selection.rs`, and `tests/source/solver_synthesis.rs`
mirror their `src/` counterparts (the `source` crate compiles a copy of `src/`
for private-function tests).

**Docs:** this case study, plus an `ARCHITECTURE.md` §6.1 update describing the
evidence-count / counted-utility / threshold mechanisms.

**Changelog:** a `changelog.d/` fragment (`bump: minor`, since the public
ranking API gains fields).

---

## 7. Verification

- `cargo test` — full unit suite and the `source` mirror suite pass, including
  the 6 new probability tests (13 probability tests total).
- `cargo fmt --check` — clean.
- `cargo clippy --all-targets --all-features` — no warnings (pedantic + nursery
  are denied in CI).
- `rust-script scripts/check-file-size.rs` — `src/probability.rs` stays under
  the 1000-line limit.

The new behaviour is proven by tests rather than asserted: the counted-utility
test demonstrates a decision *reversal* (A vs B) driven solely by the `CU`
switch, and the threshold tests demonstrate that an under-evidenced transition
is withheld and the candidate falls back to its prior.

---

## 8. Future work (explicitly out of scope here)

These are reasonable follow-ups, each deserving its own issue rather than being
silently bundled:

1. **Symbolic cosine-similarity `SS` fallback** for inexact state matches,
   integrated with the existing deterministic guess path.
2. **Context size `CS` > 2** — multi-step transition keys (n-gram states).
3. **A worked end-to-end example/benchmark** exercising counted-utility on a
   sequential decision task, mirroring the paper's Breakout evaluation in a
   symbolic domain.

---

## 9. References

- Paper (landing): <https://arxiv.org/abs/2605.00940>
- Paper (full-text HTML): <https://arxiv.org/html/2605.00940v1>
- Lineage paper (AGI 2021, Springer LNCS):
  <https://link.springer.com/chapter/10.1007/978-3-030-93758-4_12>
- Reference implementation: <https://github.com/aigents/pygents>
- Author profile: <https://github.com/akolonin>,
  <https://www.researchgate.net/profile/Anton-Kolonin>
- Local evidence base: [`raw-data/`](./raw-data/) in this folder.
