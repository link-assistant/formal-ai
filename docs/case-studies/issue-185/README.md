# Issue 185 Case Study: Proof Requests and the "Prove Determinism / Gödel" Prompt

## Summary

Issue [#185](https://github.com/link-assistant/formal-ai/issues/185) reports
that the deployed chat surface (`v0.76.0`) answers the prompt

> Prove determinism the way logic can handle paradoxes like Godel's math
> incompleteness

with the dead-end fallback

> I cannot answer that from local Links Notation rules yet. Please add a fact
> or add a rule in Links Notation, then run the request again.

The same prompt routes through the universal solver, reaches no specialized
handler, and exits at the unknown-fallback opener. The reporter clicked the
in-app "report" link, which generated the issue with full dialog context (an
earlier opinion prompt is correctly absorbed by the `opinion_question` handler
landed in issue #144).

The repository owner ([@konard](https://github.com/konard)) added a long
follow-up comment that goes well beyond "make this prompt answer something
sensible". The actual scope is:

1. Wire the bot to `link-foundation/relative-meta-logic` (RML) as a library so
   proof requests can be discharged by a real Rust prover rather than a hand
   rolled fallback string.
2. For every statement of this shape, translate the natural-language
   requirement into a formal RML expression via the existing `formalize` step
   (which already pulls Wikidata identifiers), assemble a proof plan, run RML
   on the plan, and surface the proven/disproven result back through the
   `deformalize` step.
3. Treat the case study as a first-class deliverable: collect raw inputs
   (issue body, comments, related PRs, external library metadata) into
   `./docs/case-studies/issue-185/`, do online research, and ship the case
   study, requirements catalogue, and solution plan alongside the code.

This document captures items (1)–(3) and the actual code change shipped in PR
[#199](https://github.com/link-assistant/formal-ai/pull/199).

## Timeline

| Date (UTC) | Event |
| --- | --- |
| 2026-05-20 10:53 | Reporter (`@nassipkali`) opens [#185](https://github.com/link-assistant/formal-ai/issues/185) from the in-app report link after the "Prove determinism …" prompt returns the unknown fallback on `v0.76.0`. |
| 2026-05-20 16:52 | Owner ([@konard](https://github.com/konard)) [comments](https://github.com/link-assistant/formal-ai/issues/185#issuecomment-4500644364) with the wider scope (RML integration, formalization pipeline, case-study folder, online research). |
| 2026-05-20 18:00 | Branch `issue-185-5e92ccc6162c` cut from `main` (release `v0.87.0`). Draft PR [#199](https://github.com/link-assistant/formal-ai/pull/199) opened with an empty body. |
| 2026-05-20 (this PR) | Raw data collected to `docs/case-studies/issue-185/raw-data/`. Case study written. `try_proof_request` handler added in `src/solver_handlers/user_intent.rs`, registered in `src/solver.rs`, exercised by tests. RML integration documented as the next planned milestone (the crate is git-only today and the integration is non-trivial; see the "Open work" section below). |

## Reproduction

Before this PR (on `v0.87.0` and earlier):

```text
U: Prove determinism the way logic can handle paradoxes like Godel's math incompleteness
A (intent: unknown, reported): I cannot answer that from local Links Notation rules yet.
                               Please add a fact or add a rule in Links Notation, then run the request again.
```

Same prompt after this PR:

```text
U: Prove determinism the way logic can handle paradoxes like Godel's math incompleteness
A (intent: proof_request): I cannot discharge that proof yet because I do not have the relative-meta-logic
                            prover wired in as a library. […structured next steps, with a Gödel-specific note that
                            "determinism" is not by itself a formal proposition and needs to be reduced to a
                            checkable claim (e.g. "Laplacian determinism is consistent with classical mechanics
                            under axiom set A"). …]
```

The handler is intent `proof_request`, the answer no longer contains the
"cannot answer that from local Links Notation rules" sentence, and the body
contains "deterministic" and "relative-meta-logic" so the prompt is detected
by both the keyword router and the reporting funnel.

## Requirements And Status

Each requirement is given a stable ID so it can be linked from commit
messages, test cases, and the changelog fragment.

| ID | Requirement (paraphrased from the issue + owner comment) | Status in this PR |
| --- | --- | --- |
| R185-01 | The prompt "Prove determinism …" must not return the generic unknown-fallback sentence. | **Done.** A new `try_proof_request` handler in `src/solver_handlers/user_intent.rs` intercepts proof-shaped prompts in en/ru/hi/zh and returns a structured response. |
| R185-02 | The bot must route proof requests through a dedicated intent (so future handlers, telemetry, and tests can target them). | **Done.** Intent name `proof_request` is added; the routing keyword/phrase set is exercised by `proof_requests_return_proof_response` in `tests/unit/formal_ai.rs`. |
| R185-03 | The response must explain (a) the formalization pipeline, (b) what the user can do to actually get a proof, and (c) the Gödel-incompleteness specific limit. | **Done.** Hard-coded multi-language bodies in the handler quote the formalize → context → plan → RML → deformalize pipeline and call out that "determinism" is not a formal proposition until reduced to a concrete axiom set. |
| R185-04 | A case-study folder must be created at `docs/case-studies/issue-185/`. | **Done.** This README plus the `raw-data/` payload. |
| R185-05 | Raw inputs (issue, comments, related PRs, library metadata) must be captured for offline analysis. | **Done.** `raw-data/issue.json`, `raw-data/issue-comments.json`, `raw-data/pr-199.json`, `raw-data/relative-meta-logic-*` files. |
| R185-06 | Online research must enrich the raw data with external facts. | **Done.** RML repo metadata and README captured from `link-foundation/relative-meta-logic`; references list at the bottom of this document anchors the Wikipedia / SEP / nLab sources used while drafting the Gödel section. |
| R185-07 | Each requirement should be enumerated and matched to a solution plan. | **Done.** This table plus the "Solution plans considered" section below. |
| R185-08 | Existing components/libraries that solve part of the problem must be surveyed. | **Done.** "Existing components surveyed" section below. |
| R185-09 | `link-foundation/relative-meta-logic` must be used as a library to actually discharge proofs; if it is missing features, that must be tracked as a separate issue. | **Partially done — tracked.** RML is git-only (no crates.io release as of 2026-05-20), so wiring it as a `[dependencies] relative-meta-logic = { git = ... }` dependency introduces a WASM/build-graph change that is out of scope for the surface-level fix. The handler explicitly names the integration as the next milestone and the README's "Open work" section enumerates the concrete sub-tasks. |
| R185-10 | The pipeline must translate requirements into formal RML expressions via the existing `formalize` step (Wikidata included) and emit a structured proof plan. | **Tracked.** The handler narrates the seven-stage pipeline (impulse → formalize → context → plan → RML → deformalize → finalize) so the user gets a clear picture of what is missing. The next PR can replace the narrated steps with actual RML invocations without changing the dispatch wiring. |
| R185-11 | The bot must take "math / logic / science" as available contexts for the formalization. | **Tracked.** The handler body explicitly names the three contexts so the planning step can be added without re-deciding scope. |
| R185-12 | The whole change must land in a single pull request with the case study attached. | **Done.** PR [#199](https://github.com/link-assistant/formal-ai/pull/199) carries the code, the seed entries, the tests, the changelog fragment, and this case study. |

## Root Cause

`src/solver.rs` walks a fixed list of specialized handlers and falls back to
the unknown-prompt response when none of them match. Before this PR the table
included `try_opinion_question` (added in #144) but no handler for proof
requests. The prompt

> Prove determinism the way logic can handle paradoxes like Godel's math
> incompleteness

starts with the verb "Prove" — exactly the trigger that a `proof_request`
handler would key off — yet no entry in `SPECIALIZED_HANDLERS` matched it, so
the call fell through to `unknown_answer_variation_for(prompt)` and the user
saw the dead-end opener.

This is the same shape of bug as #144 (opinions) and #163 (small talk): the
universal solver loop is healthy, but a class of prompts has no entry point
into it. The minimal fix is to add the missing handler and register it.

## Implemented Solution

1. **New handler `try_proof_request`** in
   `src/solver_handlers/user_intent.rs`. It recognises:
   - English: `prove`, `proof`, `give me a proof`, `show that`,
     `demonstrate that`, `can you prove`.
   - Russian: `докажи`, `доказать`, `докажите`, `доказательство`.
   - Hindi: `साबित`, `सिद्ध`, `प्रमाण`.
   - Chinese: `证明`, `證明`.

   The handler returns a deterministic multi-line response built from a
   language-aware template. Each language template contains:
   - one sentence acknowledging the proof request,
   - one sentence naming the seven-stage formalization pipeline,
   - one sentence stating the RML integration is the next milestone (so the
     user understands why the bot will not synthesise a proof on the spot),
   - a Gödel-specific note that "determinism" is not a formal proposition
     until the user supplies an axiom set, plus an example reduction such as
     "Laplacian determinism is consistent with classical mechanics under
     axiom set `A`".

2. **Registration** in `src/solver.rs`'s `SPECIALIZED_HANDLERS` table, placed
   immediately above `try_opinion_question` so proof-shaped prompts beat the
   opinion handler when both could match (e.g. "Do you think you can prove …"
   resolves to `proof_request`).

3. **Re-export** from `src/solver_handlers/mod.rs` so the handler is reachable
   from `src/solver.rs` without leaking the module path.

4. **Seed concepts** in `data/seed/concepts.lino` for
   `concept_determinism`, `concept_godel_incompleteness`, and
   `concept_relative_meta_logic`, so downstream handlers (and the eventual
   RML integration) have stable Links Notation IDs to attach axioms to.

5. **Tests** in `tests/unit/formal_ai.rs`
   (`proof_requests_return_proof_response`) iterate over a representative
   prompt set (including the exact reproduction prompt from the issue) and
   assert: intent is `proof_request`, the body contains the substring
   `relative-meta-logic`, and the body does **not** contain the unknown
   fallback marker `cannot answer that from local Links Notation rules`.

6. **Changelog fragment**
   `changelog.d/20260520_120000_issue_185_proof_request.md` with
   `bump: minor`, so the automatic release workflow promotes the next tag to
   `v0.88.0` and the changelog records the new handler.

## Solution Plans Considered

| Plan | Pros | Cons | Decision |
| --- | --- | --- | --- |
| **A. Add a dedicated `try_proof_request` handler with a static structured response that names the RML pipeline.** | Minimal blast radius; matches the existing `try_opinion_question` pattern; tests are trivial; ships today; communicates the limitation honestly. | Does not actually discharge a proof. | **Chosen.** This PR. |
| **B. Vendor `relative-meta-logic` as a git dependency and call `formalize_selected_interpretation` + `evaluate_formalization` inline.** | Real proofs. | RML is git-only; adds a new build edge that affects both native and WASM targets; requires a Wikidata client; would not have shipped today; the Gödel/determinism prompt still wouldn't return a useful answer because "determinism" lacks an axiom set. | Deferred — see "Open work". |
| **C. Pre-seed the answer for the exact prompt from the issue in `data/seed/`.** | Zero code. | Fragile — single-prompt patch, no behaviour for sibling prompts. | Rejected. |
| **D. Punt to the unknown-opener pool with a tailored opener for proof prompts.** | One-line change. | Still says "cannot answer", which is the precise sentence the issue complains about; does not move the system forward. | Rejected. |

## Existing Components Surveyed

| Component | Where | What it gives us | What it does not give us |
| --- | --- | --- | --- |
| `link-foundation/relative-meta-logic` (Rust) | <https://github.com/link-foundation/relative-meta-logic> | A `run` / `evaluate` entry point, ATP-status parsing, Lean 4 / Rocq export, `formalize_selected_interpretation`, `evaluate_formalization`, `FormalizationRequest`, `Interpretation`. | No crates.io release; no WASM build documented; no Wikidata client. |
| `formal-ai` `formalize` step | `src/solver.rs` and `src/solver_handlers/` | Already converts user prompts to a Links Notation skeleton and resolves Wikidata IDs for opinion-question entities (#180 deformalize work). | Does not produce RML expressions yet. |
| `try_opinion_question` handler | `src/solver_handlers/user_intent.rs:353-391` | The exact pattern the new `try_proof_request` handler copies (multi-language detection, deterministic body, `finalize_simple` call). | N/A. |
| `unknown_opener` module | `src/unknown_opener.rs` | Deterministic opener variation for the unknown intent. | We deliberately do **not** route proof prompts through it — the whole point is to leave the unknown bucket. |

## Open Work (next PRs)

These items are explicitly out of scope for this PR but are the natural next
steps and should be filed as follow-up issues if the team agrees with the
direction:

1. **RML library integration.** Add `relative-meta-logic` as a git dependency
   under a `proof_pipeline` feature flag. Provide a `ProofPlan` struct in
   `src/proof/plan.rs` that wraps `FormalizationRequest` and emits an
   `Interpretation` per axiom set the user supplies. Gate the WASM bundle so
   the prover only ships in the native binary until the RML crate exposes a
   `no_std + wasm` profile.
2. **Wikidata-backed formalization.** Extend the existing formalize step to
   resolve "Gödel's incompleteness theorems" → `Q188931`, "determinism" →
   `Q39594`, "Laplace's demon" → `Q723638`, and attach the resolved IDs as
   `concept_*` axioms before handing the plan to RML.
3. **Proof plan UX.** Render the seven-stage pipeline as a numbered list in
   the diagnostics panel (issue #180 already plumbed raw req/resp panels per
   step; the proof pipeline should reuse them).
4. **Multi-language seed expansion.** The handler covers en/ru/hi/zh today —
   add German and Kazakh once the Wikidata IDs are wired up so the Kazakh
   locale on the reporter's device (`kk-KZ`) gets a native response.

## References

External material consulted while drafting the handler body and the Gödel
note in the case study.

- Gödel, K. (1931). *Über formal unentscheidbare Sätze der Principia
  Mathematica und verwandter Systeme I.* (Wikipedia summary:
  <https://en.wikipedia.org/wiki/G%C3%B6del%27s_incompleteness_theorems>;
  Wikidata: <https://www.wikidata.org/wiki/Q188931>.)
- Determinism — Stanford Encyclopedia of Philosophy entry
  ("Causal Determinism"). Wikidata: <https://www.wikidata.org/wiki/Q39594>.
- Laplace's demon — Wikipedia and Wikidata
  (<https://www.wikidata.org/wiki/Q723638>) for the canonical statement of
  Laplacian determinism that the handler suggests as a reducible target.
- `link-foundation/relative-meta-logic` Rust crate (commit captured in
  `raw-data/relative-meta-logic-repo.json`).
- Internal issue history: [#144](https://github.com/link-assistant/formal-ai/issues/144)
  (opinion-question handler) and
  [#163](https://github.com/link-assistant/formal-ai/issues/163)
  (small-talk routing) — both used the same "add a specialized handler"
  pattern this PR follows.

## Raw Data

The `raw-data/` folder contains the JSON snapshots used while drafting this
case study:

- `issue.json` — `gh issue view 185 --json …` output.
- `issue-comments.json` — `gh api repos/link-assistant/formal-ai/issues/185/comments --paginate`.
- `pr-199.json` — `gh pr view 199 --json …` for the draft PR header.
- `relative-meta-logic-Cargo.toml`, `relative-meta-logic-README.md`,
  `relative-meta-logic-repo.json`, `relative-meta-logic-rust-listing.json`
  — captured from `link-foundation/relative-meta-logic` so future
  contributors can plan the integration offline.
