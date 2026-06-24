# Issue 559 Options Comparison

The PR feedback said: "if there are multiple options and directions we can
implement them all to compare." This document enumerates the major design
decisions, gives 2–4 concrete options each with pros / cons / cost / risk and a
recommendation, and — where it is feasible and cheap — specifies a **comparison
harness** so competing options can be run side by side behind a `SolverConfig`
knob and judged by tests and benchmarks rather than by opinion.

The guiding instruction is to "prefer as general and universal solutions and
decisions as possible," so each recommendation is justified against generality,
not just expedience.

Cost legend: **S** ≈ one focused PR step; **M** ≈ a few steps; **L** ≈ a phase or
more.

## Decision 1 — How to represent the problem frame

The explicit, link-serializable meaning record (`ProblemFrame`, mapping to the
formalized impulse — see [alignment.md](alignment.md) C1).

| Option | Summary | Pros | Cons | Cost | Risk |
| --- | --- | --- | --- | --- | --- |
| 1A Extend `IntentFormalization` in place | Add frame fields/events to the existing struct (`src/intent_formalization.rs:48`) and emit a `.lino` trace | Minimal churn; reuses caching, routing, parity; honors R157 | Struct grows; risk of a "god struct" | S–M | Low |
| 1B New `ProblemFrame` Rust type wrapping `IntentFormalization` | A new struct composes the existing one + needs/units/evidence | Clean separation; clear new schema | Two carriers to keep in sync; more parity surface | M | Medium |
| 1C Frame as a `meta-language` network from day one | Store the frame directly as a `meta-language` link network | Furthest toward algorithm-as-data (R24) | Upstream dependency surface; heavier; premature | L | High |

**Recommendation: 1A first, evolving toward 1B only if the struct gets unwieldy.**
1A is the most general *and* the cheapest: it makes the existing meaning record
explicit without a parallel ontology (resolves C1), keeps Rust↔JS parity small,
and emits a `.lino` trace that is already a link representation (R311
`format_lino_record`). 1C stays the long-term target (R24) but is gated behind
the `meta-language` round-trip phase in
[upstream-dependency-audit.md](upstream-dependency-audit.md); adopting it now
would violate "no upstream blocker before this phase."

**Comparison harness:** not a runtime A/B — these are sequential refactors. The
guard is that the emitted `.lino` frame trace is identical before/after 1A→1B
(snapshot test), so the representation change is provably behavior-neutral.

## Decision 2 — How method selection works (the core of issue 559)

This replaces the residual of ROADMAP Pillar 20 (`SPECIALIZED_HANDLERS` as a
precedence table behind the formalized router).

| Option | Summary | Pros | Cons | Cost | Risk |
| --- | --- | --- | --- | --- | --- |
| 2A Keep Rust table, add data metadata | Annotate handlers with `.lino` metadata but keep `ordered_handler_names` as the selector | Lowest risk; fully compatible | Selection still lives in Rust; does not close Pillar 20 | S | Low |
| 2B Data-described registry, Rust executes | A `.lino` method/skill registry holds preconditions/evidence/validation/cost; a generic selector reads it; handlers stay as executable hooks | Closes Pillar 20; general; honors R103/R97; algorithm-as-data on-ramp | Must reproduce the exact 50-handler order initially | M–L | Medium |
| 2C Full registry + learned ranking | As 2B but selection scores are tuned from benchmark feedback | Most adaptive | Tuning risks nondeterminism; needs the self-improvement gate | L | High |

**Recommendation implemented in PR #560: 2B, with 2C gated behind the
self-improvement gate.** 2B is the general solution the issue asks for: selection
is data the system can inspect and the solver executes specialized handlers
through the registry-backed `meta_method_dispatch::try_dispatch` path, while
handlers remain callable Rust hooks (NG2). 2C only after deterministic replay and
benchmarks can prove learned ranking never regresses.

**Selection trace (the auditability surface):** add a `SolverConfig` knob
`selection_mode ∈ {off, record}` (added to config first). The live solver path
uses the registry-backed executor as the sole dispatch authority; `selection_mode`
controls the optional audit artifact. In `record` mode the solver names, for every
atomic leaf, the method the registry resolves (or `unresolved`), making the
dispatch auditable per request. An interim parity certificate first proved the
registry resolved the whole route corpus at zero contradictions against the legacy
mapper; with that proof in hand R344 removed the mapper and the certificate
scaffolding outright, and the closure invariant now holds directly against the
live registry.

## Decision 3 — Recursion direction

The feedback asks for *both* decomposition-first and construction-first reasoning
"at the same time" (R19, R20).

| Option | Summary | Pros | Cons | Cost | Risk |
| --- | --- | --- | --- | --- | --- |
| 3A Decomposition-first only | Split until atomic, then solve leaves | Simple; matches existing `decompose` | Misses ready-made compositions; can over-split | M | Medium |
| 3B Construction-first only | Search components/skills/cache, compose upward | Reuses skills; fast when a composition exists | Stalls on novel tasks with no ready parts | M | Medium |
| 3C Bidirectional with a meeting point | Run both passes; downward split proposes units, upward search tries to satisfy each unit from registry/cache/stdlib; stop when a unit is satisfied or atomic | Most general; matches the feedback exactly; each pass covers the other's blind spot | More moving parts; needs a clear termination rule | L | Medium |

**Recommendation: 3C.** It is the only option that satisfies R20 literally and is
the most general. Termination is bounded by `SolverConfig::max_decomposition_depth`
plus a new `atomicity_policy` knob (added to config first): a unit stops when it
is atomic (a single method/library/function/skill call) or when the upward search
already satisfies it. See [recursive-core.md](recursive-core.md) for the
downward/upward pseudo-code grounded in `solver.rs:411-653`.

**Comparison harness:** the same recursive engine runs all three by a knob
`recursion_mode ∈ {down, up, both}`. Benchmarks record solution found / depth /
units created / wall-cost per mode so the team can see, per task family, whether
`both` actually beats the single-direction baselines before `both` becomes
default.

## Decision 4 — Evidence pipeline: where crawl/extract runs

Search + RRF already exist (CR4). The missing stages are crawl, extract, compare,
hypothesize, and live non-CORS providers.

| Option | Summary | Pros | Cons | Cost | Risk |
| --- | --- | --- | --- | --- | --- |
| 4A Browser-worker crawl only | Extend `tryFetch` (`formal_ai_worker.js`) to crawl reranked pages | Reuses the real search engine; no new process | CORS blocks Google/Bing/Brave; 2000-byte truncation | M | Medium |
| 4B Desktop fetch seam for non-CORS | Route non-CORS providers + crawl through `desktop/lib/tool-router.cjs:92` `httpFetch` | Unlocks Google/Bing/Brave; full page bodies | Desktop-only; must mirror to browser where possible | M | Medium |
| 4C Rust-native fetch | Add a real HTTP client in Rust behind `offline` | Canonical implementation; testable | Heavy; conflicts with `no_std` web core; large parity surface | L | High |
| 4D Symbolic-only (status quo) | Keep descriptive `try_web_search`/`try_http_fetch` | Zero risk; deterministic offline | No fresh data; fails R10/R22 | S | Low (but unmet need) |

**Recommendation: 4A + 4B together, with the pipeline defined once in Rust as
data/policy and executed by whichever runtime is available.** The general design
keeps the *pipeline definition* canonical (a `.lino` evidence recipe + Rust
policy) while execution is delegated to the browser worker (4A) for CORS-readable
providers and the desktop fetch seam (4B) for non-CORS providers. Offline mode
(`FORMAL_AI_OFFLINE`) short-circuits to cache, preserving determinism (R67). 4C
stays a possible future once a Rust HTTP client is justified; 4D is the offline
fallback, not the answer.

**Comparison harness:** an evidence-quality benchmark scores answers with the
pipeline on vs off (and CORS-only vs CORS+desktop) on a fixed question set, using
cached fixtures so CI stays deterministic. This quantifies the marginal value of
non-CORS providers and crawl before they become default-on.

## Decision 5 — Migration strategy

| Option | Summary | Pros | Cons | Cost | Risk |
| --- | --- | --- | --- | --- | --- |
| 5A Big-bang replace dispatch | Swap `SPECIALIZED_HANDLERS` for the registry in one PR | Fast; no dual code path | High blast radius; hard to review; risks regressions across 50 handlers | M | High |
| 5B Registry-backed path, prove parity, then retire the mapper | Land registry-backed dispatch, prove a corpus-wide parity certificate against the old mapper, then remove the legacy authority and the audit scaffolding outright | Empirical; the certificate de-risks the swap before it happens, then leaves a single authority with no obsolete-by-example code | Two-phase: the audit scaffolding exists transiently before removal | M–L | Low |
| 5C Handler-by-handler | Migrate one handler family at a time | Smallest steps | Long tail; inconsistent intermediate state; 50 families | L | Medium |

**Recommendation implemented in PR #560: 5B, carried through to its end state.**
A corpus-wide parity certificate first proved the registry resolved the entire
route vocabulary as a behavior-preserving replacement (zero contradictions); with
that proof in hand the legacy `specialized_handler_name` mapper and the
`dispatch_parity` audit scaffolding were **removed outright**, leaving the registry
as the sole dispatch authority. The certificate's closure invariant survives
directly against the live registry (`method_registry.rs` corpus-closure test), and
`selection` records what that one authority resolves per request. 5C is no longer
needed for the existing handler set; bespoke metadata can still be added later
without reintroducing a direct dispatch loop.

## Decision 6 — Skill/method registry storage and grounding

| Option | Summary | Pros | Cons | Cost | Risk |
| --- | --- | --- | --- | --- | --- |
| 6A `.lino` registry under `data/seed/` or `data/meta/` | Store method/skill entries as seed `.lino`, embedded via `include_str!` | Reuses existing data architecture (R15); embedded; total-closure-checked | Must author closed; size caps apply | M | Low |
| 6B New `data/registry/` directory | Dedicated directory for registry data | Clear home | New embedding wiring; new audit scope | M | Medium |
| 6C `meta-language` store | Registry as a `meta-language` link store | Closest to algorithm-as-data | Upstream surface; premature (see audit) | L | High |

**Recommendation: 6A.** Reusing `data/seed/`/`data/meta/` honors R15 (preserve
the data architecture) and the embedding path (`src/seed/embedded.rs`), and it
inherits the total-closure and no-hardcoded-NL gates for free. Grounding follows
the existing recipe pattern: a single test parameterized over all
`data/meta/*-recipe.lino` (resolves C5), so adding the general recipe
automatically adds grounding.

## Decision 7 — Algorithm-as-data representation (round trip)

The long-term R24 target: the general algorithm is data that can become code and
back.

| Option | Summary | Pros | Cons | Cost | Risk |
| --- | --- | --- | --- | --- | --- |
| 7A `.lino` recipe + grounding test (now) | The algorithm is described in `.lino` and asserted against live source | Available today; no upstream blocker; matches #444/#468 | Description↔code link is by test, not by execution | M | Low |
| 7B `meta-language` round trip (later) | Parse code to a link network and reconstruct it losslessly | True round trip; source spans; snapshots | Needs the `meta-language` phase; bigger | L | Medium |
| 7C Executable-from-data (much later) | The `.lino` recipe is directly interpreted at runtime | Maximal algorithm-as-data | Large; needs an interpreter + the self-improvement gate | L+ | High |

**Recommendation: 7A now, 7B as the named next phase, 7C as the horizon.** This
sequencing matches both the upstream audit (no blocker for 7A; 7B is a later
phase) and the self-modification boundary (C3): 7C cannot precede the proposal
gate. It keeps the most general target (7C) explicit without letting it block
near-term, behavior-preserving progress.

## Cross-Cutting: every option respects these invariants

Independent of which option is chosen, all of them must:

- add any new control knob to `SolverConfig` first (`NON-GOALS.md`);
- keep the 11-step loop shape unbranched by domain (`GOALS.md`);
- pass total reference closure, no-hardcoded-NL, traceability, loop-event
  compatibility, recipe grounding, and cross-runtime parity (see
  [alignment.md](alignment.md) C6);
- keep specialized handlers callable during migration (NG2);
- represent new structures as doublet links (`VISION.md:44`).

## Summary Of Recommendations

| Decision | Recommended option | Comparison harness? |
| --- | --- | --- |
| 1 Frame representation | 1A → 1B | Snapshot equality of `.lino` trace |
| 2 Method selection | 2A → 2B (2C gated) | Trace — `selection_mode={off,record}`; corpus-closure invariant |
| 3 Recursion direction | 3C bidirectional | Yes — `recursion_mode={down,up,both}` |
| 4 Evidence execution | 4A + 4B | Yes — pipeline on/off + CORS vs CORS+desktop benchmark |
| 5 Migration strategy | 5B prove-then-retire | Parity certificate (interim) → corpus-closure test |
| 6 Registry storage | 6A `data/seed`/`data/meta` | Recipe grounding test |
| 7 Algorithm-as-data | 7A → 7B → 7C | Grounding test now; round-trip test later |

The three runtime comparison harnesses (Decisions 2, 3, 4) are the concrete
realization of "implement them all to compare": each is a `SolverConfig` knob
with a benchmark, so the choice between directions is settled by measured parity
and quality, not by assertion.
