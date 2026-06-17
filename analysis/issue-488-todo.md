# Issue 488 Todo: Deep Thinking (concrete-by-default round)

Source issue: https://github.com/link-assistant/formal-ai/issues/488
Pull request: https://github.com/link-assistant/formal-ai/pull/489

## Why this round exists

The first round shipped the UI shell (collapsed preview, fade, expand, a
granularity setting, a `ThinkingStep` model, API plumbing). Maintainer feedback:

- "Thinking steps should be more concrete by default."
- "Make sure it is not only the UI, but fully applied to all the logic."
- "Apply each requirement to the entire code base... broadest possible sense."
- "It is ok to change architecture to make it perfect."

Root cause found by inspection (see `experiments/`):

1. `EventLog::thinking_steps()` (Rust) is a noisy 1:1 projection of raw solver
   events. It surfaces meta-language identifiers (`miss impulse_...`, the full
   `intent_formalization` blob, `((8 / 100) * (50 USD))`, `accepted_without_extra_constraints`).
2. The JS worker emits cleaner curated steps but with raw `detail`.
3. `naturalizeThinkingStep` in `app.js` maps the step *kind* to a generic
   localized template and **discards `detail`**, so the UI shows
   "Verified the local behavior rule." instead of "Computed 8% of $50 = 4 USD."

The concreteness exists in the data but is destroyed at projection + render time.

## Requirements extracted from the issue (each must be addressed)

- R1 Collapsed preview shows the latest step; while working it animates.
- R2 Second-to-last step is half shown with a gradient fade.
- R3 Expand reveals all ordered human-readable reasoning steps.
- R4 Self-questions, requests, and decisions are surfaced as steps.
- R5 Every smallest step is naturalized from meta-language to human-readable text.
- R6 Collapsed by default (steps run at lightning speed).
- R7 Pipeline: reasoning step -> meta-language description -> target user language.
- R8 Architecture produces visible thinking for *all* tasks/surfaces, not only UI.
- R9 Steps split to smallest thoughts; too-small steps omitted by default.
- R10 Configurable granularity; min granularity shows only high-level steps.
- R11 Recursively composite / fractal steps (parent/child).
- R12 Generalizes to the whole class of tasks (universal problem-solving algorithm).
- R13 Case study compiled in `docs/case-studies/issue-488` (with research + plans).
- R14 Everything in one PR (#489).

## Architecture change (maintainer: "It is ok to change architecture to make it perfect.")

- [x] Promote thinking to a first-class concern: extract the `ThinkingStep` model
      and the shared naturalizer into a dedicated `src/thinking.rs` module (re-exported
      via `crate::engine`), instead of burying them inside `engine.rs`. This keeps each
      file within the per-file line budget and makes "thinking" architectural, not an
      engine implementation detail. Mirror is `tests/source/thinking.rs` (byte-identical).

## Subtask 1 — Concrete naturalization in the CORE and every non-UI surface

- [x] Add a concrete `summary` (meta-language description) field to the core
      `ThinkingStep` model so non-UI surfaces get concreteness for free (R5, R7, R8).
- [x] Add a single shared `naturalize_thinking_step(step, detail)` function in the
      core that surfaces real content: prompt text, detected language, route/intent,
      computed `expr = result`, looked-up entity, invoked tool, answer preview (R5, R7).
- [x] Rework `EventLog::thinking_steps()` to emit CURATED, CONCRETE steps:
      introduce specific kinds (`compute`, `lookup_fact`, ...), clean `detail`,
      drop pure-noise events, de-duplicate consecutive repeats (R5, R9).
- [x] Assign `level` so the universal-algorithm phases are `high` and internals are
      `detailed`, so min granularity == high-level direction only (R10, R12).
- [x] Preserve the raw diagnostics trace for maintainers (unchanged evidence links).
- [x] Surface concrete thinking on the CLI (`formal-ai chat --thinking`) in text mode (R8).
- [x] Carry `summary` through OpenAI-compatible chat/responses + Anthropic outputs (R7, R8).
- [x] Surface concrete thinking on the Telegram bot via the platform's native
      `<blockquote expandable>` (collapsed by default, expands on tap), budget-guarded
      so the answer still leads and the reply stays within Telegram's 4096-char limit (R8).
- [x] Make refusal/policy thinking steps concrete too (R5).
- [x] Mirror EVERY core change into `src/web/formal_ai_worker.js` (summary + curated steps).
- [x] Mirror EVERY `src/*.rs` edit into `tests/source/*.rs` (keep appended `mod tests;`).

## Subtask 2 — Concrete, localized, composite presentation (UI) + tests + docs

- [x] Rewrite `naturalizeThinkingStep` in `app.js` to surface concrete `detail`
      (interpolate cleaned params; fall back to core `summary`) (R5, R7).
- [x] Add/extend i18n templates for en, ru, hi, zh with concrete interpolation (R7).
- [x] Keep collapsed-latest + faded-previous + expand-all behavior intact (R1, R2, R3, R6).
- [x] Keep the granularity setting meaningful against the new levels (R9, R10).
- [x] Render composite parent/child relationships where present (R11).
- [x] Verify concrete output across task classes: greeting, calculation (en/ru),
      translation, coding, QA/lookup, unknown (R12).
- [x] Update the case study in `docs/case-studies/issue-488` with the new design (R13).
- [x] Add a `changelog.d/` fragment and bump the package version for release.
- [x] Update/extend tests: source mirror tests, `openai_compatibility.rs`,
      e2e `issue-488.spec.js`; add concreteness assertions (e.g. compute summary
      contains the computed result) (R5).
- [x] Run full local verification (fmt, clippy, cargo test, e2e checks, file-size).
- [ ] Merge latest default branch, refresh PR description, `gh pr ready 489`.

## Design boundary (documented, not a gap)

Two-stage pipeline (R7): reasoning step → English meta-language `summary` → target
user language. Localization into the user's language is a **UI** concern (the browser's
`naturalizeThinkingStep` uses the i18n catalog). Non-UI surfaces (CLI `--thinking`, the
OpenAI/Anthropic APIs, and the Telegram bot) render the English `summary` as-is — they
have no per-user locale catalog. This is the intended architecture, so "concrete by
default" holds on every surface while localization stays where the catalog lives.
