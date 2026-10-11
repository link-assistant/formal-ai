TASK (tag SPEC-PARITY): close the browser worker's specification gaps (R1188-U29, JavaScript first). The native engine passes its specification suite; the browser worker is the JavaScript root and must answer the same.

The measure:
- `node scripts/check-specification-in-javascript.mjs --list` (gate `check-specification-in-javascript`) carries the Rust specification tests of the simple shape to JavaScript (`scripts/lib/rust-specification-cases.mjs`) and asks the worker each one.
- On 2026-10-09, 136 of 1169 tests are carried and 102 pass.
- The 34 that fail are listed with what the worker answered in `data/meta/specification-javascript-gaps.lino`.

Work progressively, the whole first and then sharper: one general mechanism per class of gap, never a case per prompt.

1. **Evidence links.** The native answer's `evidence_links` come from `build_evidence_links` in `rust/src/event_log.rs`: `prompt:<stable id>` first, then one typed link per logged event (`intent:`, `impulse:`, `candidate:`, `validation:`, `policy:*`, `agent_mode:*`, `source:`, `search:external`, `response:*` …). The worker returns its own `trace:*` links instead.
   - Give the worker the same builder, as a twin under `js/agentic/crate/` or `js/worker/`, read from the same event kinds. Keep the worker's existing links where tests rely on them.
   - Each worker answer then carries the native links for the events it logs.
   - This should close the `prompt:`, `intent:`, `impulse:`, `response:` and `candidate:` gaps, and give the policy gaps a place to land.
2. **Policies and refusals.** Gaps: chat-bounded autonomy, destructive actions, agent time budget, cache flush, add-only history, inappropriate content, the `tool_call_refused` intent, `skill_gap`.
   - Find each native rule (a seeded role or policy, not code) and make the worker read the same seed.
   - The words belong in seed lexemes in all five languages.
3. **Routing differences.** Gaps: `project_lookup` taking identity, concept and http_fetch prompts; the agent workspace task read as text manipulation; the project-lookup sentences; the unknown-language fallback sentence.
   - Compare the two routes' precedence (the seed `precedence` data both roots read) and fix the worker's reading, not the prompts.
4. After each class, run `node scripts/check-specification-in-javascript.mjs --write`. The passing count must rise and never fall. Also run the worker's own web tests next to the change.
5. **Rows.** Update R1188-U29 in `docs/requirements/issue-1188-user-requirements.md` with the new counts, then run the requirement pipeline and `node scripts/render-progressive-plan.mjs --write`.

Rules: the same as `tasks/generalize.md`.
- No cargo, rustc or rust-script locally. The Rust side should not need to change. If it must, write it carefully by hand for CI to compile.
- After JS edits, run `node scripts/translate-es.mjs --write`. Seed edits need their `rust/embedded` mirrors.
- `scripts/translate-js-rust.mjs` belongs to TRANSLATE: name any js/agentic/crate module you add, and LEAD will have it regenerated.
- Claim your files in `claims.md`.
- Use Formal AI from JavaScript (`experiments/js_dogfood/drive.mjs`) for single-line edits, and log them as T810-T839.
- Don't commit. LEAD integrates.
