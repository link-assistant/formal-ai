# Reasoning/thinking step emission map (R8)

> Source: code investigation of the Rust + worker + web pipeline on 2026-06-20.

## Data model is already complete
`src/thinking.rs` `ThinkingStep` (L19-75) already carries everything R8 needs:
- `level`: `"high"` (universal-algorithm phases) or `"detailed"` (mechanical sub-steps).
- `parent_id: Option<String>`: hierarchy — composite parents with rolled-up children (e.g. a `compute` parent with `compute_engine`/`compute_expression`/`compute_steps` children).
- `summary`: an auto-computed, concrete, human-readable English sentence (issue #488's "meta-language description" layer) via `naturalize_thinking_step(step, detail)`.

`src/event_log.rs::curate_thinking_event` (L347-406) assigns the level per step kind and folds the calculator trace into one composite parent + children (`CalculationCluster`, L260-330). `thinking_steps_for_answer` (L116-172) de-dupes and assigns `order` + `parent_id`.

The browser worker mirrors this: `HIGH_LEVEL_THINKING_STEPS` set + `thinkingStepLevel()` + `withThinkingLevels()` (`src/web/formal_ai_worker.js` ~L37894-37948) tag every step with a level before returning.

## Root causes of the R8 symptoms
1. **Raw symbolic syntax leaks into the human view.** `naturalizeThinkingStep()` in `src/web/app.js` (~L2502-2518) renders the `formalize`/`formalize_resolved` steps by interpolating the raw Links-notation tuple (`message.thinkingStep.formalizeTuple` → e.g. `(@USER OP:express ?…)`). R8 explicitly wants "no special syntax" in the default reasoning view. The tuple belongs only in diagnostics mode.
2. **Default detalization is maxed out.** `PREFERENCE_DEFAULTS.thinkingDetailLevel = "detailed"` (app.js L1015) shows every mechanical sub-step by default. R8 wants the 50% midpoint ("standard") to show only high-level steps by default.
3. **`summary` is under-used in the UI.** The worker steps do not always carry the `summary` sentence, and `naturalizeThinkingStep()` does not prefer it, so the UI re-derives text from `step` + raw `detail` instead of the concrete sentence.

## Fix direction (R8)
- Default `thinkingDetailLevel` to `"standard"` so by default only high-level steps render (`filterThinkingEntriesForDetail` already keeps `level==="high"` + last for standard).
- In `naturalizeThinkingStep()`, prefer a clean human projection and **stop surfacing raw tuples** outside diagnostics mode; keep injected data/terms but no `@USER`/`OP:`/`?id` syntax.
- Keep "detailed" fully human-readable (data and terms allowed, no special syntax).
