# Issue #1073 contradiction audit

The issue asks for the audit itself as a deliverable: enumerate what defines
reasoning today, flag what contradicts the standard, update each flagged place,
and leave a regression behind. This file is the enumeration and the disposition
of each finding. Every row was located by reading the named file, not inferred.

## 1. What defines how reasoning is performed today

| Place | What it defines |
| --- | --- |
| `src/meta_core.rs` | The pipeline every prompt walks: formalization, work-unit decomposition, downward reasoning, upward construction, method selection, evidence join, skill ledger — and, since this issue, the reasoning-standard audit. |
| `src/meta_reasoning.rs` | The *why* attached to each recursive work unit, in both directions. |
| `src/meta_construction.rs` | Which recursive directions are reasoned about at all (`RecursionMode`). |
| `src/selection.rs` | Whether the per-leaf method-selection trace is recorded (`SelectionMode`). |
| `src/skill_ledger.rs` | Whether what a pass learned is distilled and recorded (`SkillMode`). |
| `src/solver.rs` (`SolverConfig`) | The caller-facing surface where all of the above are chosen, plus `FORMAL_AI_*` environment overrides via `meta_core::apply_env_modes`. |
| `src/fact_checking.rs` | Recursive refutation of a statement and of its symbolic negation. With `src/proof_engine/presenter.rs` (which presents a request as a claim to be refuted) and `src/web_engine_core.rs` (which reports an arithmetic claim as `Unrefuted`), this was the whole of the refutation-first machinery, and all three are reached only by the routes that own them. |
| `src/thinking.rs`, `src/thinking_prose.rs` | How a trace is narrated to a human; seed-data prose, no judgment. |
| `src/seed/sources.rs`, `data/seed/sources-registry.lino` | Which external sources are trusted and at which tier. |
| `src/web_search_fusion_core.rs` | How retrieved sources are weighed against each other during fusion. |
| `src/relative_meta_logic.rs` | The `SourceTier` ladder itself and what each rung means. |
| `data/meta/recursive-core-recipe.lino` | The core algorithm as data, pinned to the source by `tests/unit/specification/recursive_core_recipe.rs`. |
| `docs/meta-algorithm.md`, `docs/philosophy.md` | The prose statements of the same. |
| `examples/*.rs` | Runnable demonstrations of each stage. |

## 2. Trust assumed rather than derived

**Flagged.** `src/seed/sources.rs::tier_from_seed` read a hand-written
`source_tier` out of `data/seed/sources-registry.lino` and mapped it with
`_ => SourceTier::IndependentCorroboration`. Nothing derived the tier, and four
of the thirteen sources (`wikidata`, `wiktionary`, `wordnet`, `wikipedia`)
declared no tier at all and silently took the fallback.

**Updated.** Each source declares a `primacy` chain — the hops between it and
the primary record, each naming its upstream and the basis for the claim, which
in every case is the site's own policy. `SourceRecord::tier` is now
`primacy.derive_tier()`; `tier_from_seed` is deleted; the hand-written value
survives only as `SourceRecord::asserted_tier`, which a test compares against
the derivation wherever it is declared. The four sources that declared nothing
now derive `independent_corroboration` — the same value the fallback produced,
but with `DerivationReason::NamedUpstreamChain` and a named upstream behind it
instead of a `_` arm.

**Left unchanged, deliberately:** `canonical_tier` in
`src/web_search_fusion_core.rs` still normalizes an unrecognized tier string to
`independent_corroboration`. That function parses a *wire field* arriving from
the browser worker, where the alternative to a default is dropping the source;
the tier it receives is now the derived one, because the registry is where it
comes from.

## 3. Single-source conclusions and confirmation-seeking

**Flagged.** `src/fact_checking.rs`, `src/proof_engine/presenter.rs` and
`src/web_engine_core.rs` attempt refutation, each only for the statements its
own route hands it. No obligation existed over conclusions in general, and
nothing anywhere required a *variety* of refutations: three restatements of one
doubt counted as three attempts.

**Updated.** The `refutation_variety` gate applies to every conclusion in an
episode, and counts distinct mechanisms across at least two of the three axes
(mechanism, source, denied assumption). Below the threshold the verdict is
`not_confirmed_not_refuted` with the blockers named. Corroboration has a floor
too: `instruction_formalization` requires at least two distinct sources before a
gathered instruction step counts as corroborated, since a single source cannot
corroborate itself.

## 4. Depth conditional on difficulty or prompting

**Flagged.** Three defaults made depth opt-in:

| Knob | Was | Now |
| --- | --- | --- |
| `RecursionMode` | `Down` — upward construction only on request | `Both` |
| `SelectionMode` | `Off` — no method-selection trace | `Record` |
| `SkillMode` | `Off` — nothing distilled from the pass | `Accumulate` |

Each was reachable through `SolverConfig` and through a `FORMAL_AI_*`
environment variable, which is precisely "conditional on explicit prompting".
The narrow modes still exist for deliberately quietening a trace; they are no
longer the resting state.

**Not a knob at all:** the reasoning-standard audit. `record_meta_core` calls
`reasoning_standard::record_reasoning_standard` with no condition in front of
it, and `the_meta_core_runs_the_audit_with_no_mode_in_front_of_it` in
`tests/unit/specification/reasoning_standard_meta_algorithm.rs` fails if one is
ever added.

**Also flagged, and this is what makes the trivial request covered:** an audit
that only ran when there was something to audit would be conditional on task
difficulty. `open_episode` opens an episode from the formalization alone, so
every gate reports `NotTriggered` naming the trigger that was false. The
obligations are enumerated identically on the trivial request and the hard one;
only which of them fire differs.

## 5. Regression derived from the reference dialog

`data/meta/reasoning-standard-reference-episode.lino` encodes the reference
session's reasoning — its observations, claims, sources, refutations and actions
— with the project-specific tooling, absolute paths and session management the
issue asks us to drop left out. It clears all seven gates with verdict
`confirmed`, and each gate is shown to fail under a mutation that removes the
behaviour it enforces (`tests/unit/issue_1073_reasoning_standard.rs`).
