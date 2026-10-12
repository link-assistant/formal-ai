TASK (tag CHAT-ROUTES): deliver the chat half of R1188-U18, R1188-U19 and R1188-U21. The text capabilities are measured, but chat does not use them yet.

1. **Summarize (R1188-U21).**
   - `rust/src/solver_handlers/summarization_request.rs` and its browser twin `trySummarizationText` in `js/worker/formal_ai_worker_text_transform.js` still answer with the R197 weight-ranked selection.
   - Answer with the dependency summarizer instead: `summarize_by_dependency` in `rust/src/summarization/dependency.rs`, twin `js/agentic/crate/dependency_summarization.mjs`. The browser worker cannot import node ESM, so its twin must run as worker code. Generate it, or load the projection the same way the worker loads its other shared modules; read how `js/worker-modules.js` lists them.
   - Keep the declines: no payload, and one statement.
   - Keep the evidence trace: every kept and dropped statement, and the duplicates removed.
   - Re-pin `rust/tests/unit/issue_1174_text_transform.rs` and the browser pins to the new outputs. The two roots must agree byte for byte; add a browser test that asserts the same outputs.
2. **Formalize a fetched page (R1188-U18).**
   - Add a route: "formalize <url>" or "formalize this page: <url>", in all five languages. The words go in seed lexemes of a new meaning, not in code.
   - The route fetches the page through the existing URL-fetch path (it obeys the network policy and the source cache) and answers with the statements of `rust/src/formalization/page.rs` (twin `js/agentic/crate/page_formalization.mjs`).
   - Offline, or when the fetch is refused, it says so.
   - Pin it with a fixture page from `data/benchmarks/web-formalization/` and no live network.
3. **Translate with the round-trip choice (R1188-U19).** Where the chat translation route picks among several surfaces, use the round-trip choice of `rust/src/translation/round_trip.rs` (twin `js/agentic/crate/round_trip_translation.mjs`). Re-pin what changes in both roots.
4. **Update the rows.** Update R1188-U18, R1188-U19 and R1188-U21 in `docs/requirements/issue-1188-user-requirements.md` with what changed. Then run the requirement pipeline:

   ```
   node scripts/assemble-requirements.mjs --write && node scripts/generate-requirement-status.mjs --write && node scripts/render-status.mjs --write && node scripts/check-requirement-status.mjs
   ```

Rules: the same as `tasks/generalize.md`.
- No cargo or rust-script locally.
- Size limits; seed mirrors; closure audit grounding; `translate-es` after JS edits; `translate-js-rust --write --fetch` for modules in scope.
- Run only local tests next to your change, plus the gates for what you touched.
- Claim files in `claims.md`.
- Use Formal AI for single-line edits and log them as T705–T739.
- Don't commit. LEAD integrates.
