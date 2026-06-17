# Issue 488 Case Study: Deep Thinking Preview

## Inputs

- GitHub issue: https://github.com/link-assistant/formal-ai/issues/488
- Pull request: https://github.com/link-assistant/formal-ai/pull/489
- Raw GitHub data:
  - `raw-data/issue-488.json`
  - `raw-data/issue-488-comments.json`
  - `raw-data/pr-489.json`
  - `raw-data/pr-489-review-comments.json`
  - `raw-data/pr-489-conversation-comments.json`
  - `raw-data/pr-489-reviews.json`
- Code search snapshots:
  - `raw-data/github-code-search-thinking-steps.json`
  - `raw-data/github-code-search-diagnosticsSteps.json`
  - `raw-data/recent-merged-prs-thinking-diagnostics.json`
- Online research: `raw-data/online-research.md`
- Verification screenshot: `screenshots/thinking-preview-expanded.png`

## Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R488-1 | Show thinking while the model is working. | The pending assistant bubble now renders the same `ThinkingPreview` surface with a localized working step. |
| R488-2 | Show the last thinking paragraph by default. | Completed assistant messages render a collapsed preview whose current paragraph is the latest human-readable structured step. |
| R488-3 | Show part of the previous thinking paragraph with a fade. | The collapsed preview shows the previous step in a clipped, masked line above the current step. |
| R488-4 | Provide an expand button. | `ThinkingPreview` toggles between collapsed current/previous view and the full ordered list. |
| R488-5 | Use human-readable target-user language instead of meta labels. | `buildThinkingPreviewSteps` maps structured step ids such as `match_rule` and `deformalize` into localized summary templates. |
| R488-6 | Preserve raw diagnostics for debugging. | Existing `.thinking-steps`, `.diagnostics-steps`, evidence, tool calls, and report traces remain gated by diagnostics mode. |
| R488-7 | Let users control thinking granularity. | Settings now include a localized `Thinking detail` select (`brief`, `standard`, `detailed`) that reshapes previews without changing raw diagnostics. |
| R488-8 | Compile a case study and use existing components/libraries. | This directory preserves raw data and research; the implementation reuses the app's existing React renderer, i18n catalog, and structured solver steps. |
| R488-9 | Apply thinking beyond UI-only state to solver/API logic. | `ThinkingStep` is now first-class solver output derived from `EventLog`, serialized through Links Notation, Chat Completions, Responses, and consumed by the desktop HTTP UI path. |
| R488-10 | Thinking steps must be **concrete by default** (maintainer feedback). | A shared `naturalize_thinking_step(step, detail)` turns each meta-language event into a concrete English sentence that names the real content — the prompt, the detected language, the chosen route, the computed `expr = result`, the looked-up entity, the composed answer — instead of a generic category label. The core `ThinkingStep` carries this as a `summary` field so every surface renders the same concrete reasoning. |
| R488-11 | Apply to the **broadest possible** set of surfaces, not only the browser UI. | The same concrete reasoning is surfaced on the CLI (`formal-ai chat --thinking`), the OpenAI-compatible Chat Completions / Responses APIs, the Anthropic Messages API, the browser worker/app, and the Telegram bot (via the platform's native `<blockquote expandable>`). |
| R488-12 | Change architecture where it makes the result perfect (maintainer feedback). | Thinking is promoted to a first-class concern in its own `src/thinking.rs` module (model + naturalizer), re-exported via `crate::engine`, rather than living inside `engine.rs`. This keeps each file within the per-file line budget and makes "thinking" architectural rather than an engine implementation detail. |

## Subtasks

- Solver/API logic: derive ordered thinking metadata from `EventLog`; attach it to every finalized `SymbolicAnswer`; preserve raw evidence/diagnostics; expose the same metadata on OpenAI-compatible chat and responses outputs.
- User presentation: render collapsed/expanded localized thinking previews in the browser; support detail granularity; keep diagnostics mode as the raw maintainer view; retain the case-study evidence.

## Root Cause

Assistant messages already carried UI-local `thinkingSteps`, but `Message` only rendered them when diagnostics mode was enabled. Those strings were raw debug labels such as `match_rule: greeting`, which are useful for maintainers but do not satisfy a user-facing thinking UI. The pending assistant bubble also had no thinking surface; it only displayed the generic working text.

The follow-up PR review identified a second gap: the first pass was mostly a browser projection. The Rust solver returned evidence links and Links Notation, but did not expose a reusable, structured thinking model for API/desktop/browser surfaces.

## Solution

- Added `ThinkingStep` as a structured solver output with stable id, order, canonical step key, detail, level, source event, and optional parent id.
- Added `EventLog::thinking_steps()` to project raw solver events into user-facing thinking metadata while keeping raw event evidence and diagnostics intact.
- Attached thinking steps to common solver finalizers, composed synthesis answers, diagnostic answers, and policy/tool refusal answers.
- Serialized thinking metadata through answer Links Notation plus OpenAI-compatible Chat Completions and Responses response objects.
- Updated the desktop HTTP chat path to prefer server-supplied `message.thinking_steps` before falling back to legacy synthetic desktop steps.
- Added `buildThinkingPreviewSteps` and `naturalizeThinkingStep` in `src/web/app.js` to convert existing structured trace events into localized summaries.
- Added `ThinkingPreview`, a collapsed/expanded React component that shows the previous faded step and the current step by default.
- Added a `Thinking detail` preference so the same structured trace can render as brief, standard, or detailed user-facing steps.
- Added light/dark CSS for the preview without changing the existing diagnostics panels.
- Added i18n catalog entries and checker coverage for English, Russian, Chinese, and Hindi.
- Added `tests/source/source_tests/event_log/tests.rs`, OpenAI compatibility assertions, and `tests/e2e/tests/issue-488.spec.js` coverage for the solver projection, API transport, and browser preview.

### Concrete-by-default round (maintainer feedback)

Maintainer feedback after the first round: *"Thinking steps should be more concrete
by default. Apply each requirement to the entire code base... broadest possible sense.
It is ok to change architecture to make it perfect."* The root cause (see
`../../analysis/issue-488-todo.md` and `experiments/`) was that the concreteness lived
in the data but was destroyed at projection + render time: the Rust projection emitted
noisy meta-language identifiers, and the browser naturalizer mapped a step *kind* to a
generic template while **discarding `detail`**. The follow-up work:

- Promoted thinking to a first-class concern in a dedicated `src/thinking.rs` module
  (the `ThinkingStep` model plus the shared naturalizer), re-exported through
  `crate::engine`. This is the architecture change the maintainer authorized; it keeps
  every file within the repository's per-file line budget. The mirror is the
  byte-identical `tests/source/thinking.rs`.
- Added `naturalize_thinking_step(step, detail)` — the deterministic
  `(step, detail) -> concrete English sentence` translator that is the single source of
  truth for every surface. `ThinkingStep::new` calls it to populate the `summary` field,
  so concreteness is produced once and reused everywhere.
- Reworked `EventLog::thinking_steps()` into a curated projection: specific step kinds,
  cleaned `detail`, dropped pure-noise events, and `level` assignment so the
  universal-algorithm phases are `high` and internals are `detailed` (min granularity
  shows only high-level direction).
- Made the policy/refusal step concrete (it now names the policy that applied).
- Surfaced the concrete reasoning on the **Telegram** bot through the platform's native
  `<blockquote expandable>` (Bot API 7.0+): collapsed by default, expands on tap. The
  blockquote is appended after the answer and is budget-guarded — dropped entirely if
  answer + thinking + trace would exceed Telegram's 4096-character limit — so the answer
  always leads and the message never overflows. Each step sentence is HTML-escaped for
  the HTML parse mode. See `examples/issue_488_telegram_thinking.rs` and
  `raw-data/telegram-thinking-example.log`.
- Mirrored the concreteness into the browser: `src/web/formal_ai_worker.js` emits the
  curated projection with levels, and `naturalizeThinkingStep` in `src/web/app.js`
  interpolates the concrete `detail` (and additionally localizes it).

#### Design boundary: meta-language vs. localization

The pipeline (R7) is `reasoning step -> English meta-language summary -> target user
language`. The final localization stage is a **UI** concern: the browser's
`naturalizeThinkingStep` translates the summary using the i18n catalog. Non-UI surfaces
(CLI `--thinking`, the OpenAI/Anthropic APIs, and the Telegram bot) render the English
`summary` as-is, because they have no per-user locale catalog. This is the intended
architecture — "concrete by default" holds on every surface, while localization stays
where the catalog lives.

## Verification

Pre-fix reproduction:

```sh
npm --prefix tests/e2e run test:local -- --grep "Issue #488"
```

Result: failed because `[data-testid="thinking-preview"]` was not found. Saved in `raw-data/repro-before-e2e.log`.

Post-fix checks:

```sh
npm --prefix tests/e2e run check:i18n
npm --prefix tests/e2e run check:web-tdz
npm --prefix tests/e2e run test:local -- tests/issue-488.spec.js
cargo test --test source thinking_steps_project_events_to_canonical_user_steps
cargo test --test unit openai_compatibility
npm --prefix tests/e2e run test:local -- --grep "diagnostics toggle shows trace"
rust-script scripts/check-file-size.rs
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
```

Result: all passed. Logs are saved under `raw-data/`.

### Concrete-by-default round

```sh
cargo fmt --all -- --check
RUSTFLAGS=-Dwarnings cargo clippy --all-targets --all-features
cargo test                                 # 1467 passing (integration 40, source 424, unit 1003)
cargo test --test unit semantic_grounding
bun run build:web && git diff --exit-code -- src/web/vendor.bundle.js src/web/ocr.bundle.js
npm --prefix tests/e2e run check:i18n
npm --prefix tests/e2e run check:language-parity
npm --prefix tests/e2e run check:language-test-coverage
npm --prefix tests/e2e run check:intent-coverage
npm --prefix tests/e2e run check:web-tdz
npm run vscode:test
cargo run --example print_thinking_steps        # core: concrete summaries + fractal sub-steps + levels
cargo run --example issue_488_telegram_thinking # Telegram: answer-first + expandable blockquote + trace
```

Result: all passed. `print_thinking_steps` shows the concrete `summary` for every
step, the `high`/`detailed` levels, and the recursively composite (`↳`) sub-steps for
the calculator; `issue_488_telegram_thinking` shows the answer leading, the concrete
reasoning inside a native `<blockquote expandable>` (HTML-escaped), and the `/trace`
footer, all within Telegram's 4096-character limit. The Telegram capture is saved in
`raw-data/telegram-thinking-example.log`.

## Follow-Up

The current worker returns structured steps when an answer completes; the pending preview therefore uses a neutral localized working step. Streaming true intermediate step deltas during a long-running worker request would require a worker progress event protocol and a stateful per-turn progress buffer. The solver/API model added here is compatible with that future protocol because each step already has stable order, detail level, and source-event metadata.
