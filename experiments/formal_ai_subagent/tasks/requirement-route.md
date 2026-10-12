TASK (tag REQ-ROUTE): finish R1188-U20 by making requirement extraction answerable in chat, in both roots, generically.

## Already done (LEAD, commit ab7e0664a)
- `js/agentic/crate/requirement_extraction.mjs` + `rust/src/agentic_coding/requirement_extraction.rs` (`extractRequirements` / `extract_requirements`).
- Vocabulary in `data/seed/meanings-requirement-extraction.lino`.
- Benchmark `scripts/measure-requirement-extraction.mjs` + gate `check_requirement_extraction` + ratchet `data/meta/text-capability-ratchet.lino`.

## Goal
"List the requirements of this issue: <text>" (and the same request in ru/hi/zh/es, with the text pasted after a colon, a line break or in quotes) answers with a short localized intro plus one requirement per line, exactly the list `extractRequirements` returns. The answer comes through the browser worker (`js/worker/`) and the native solver (`rust/src/`), with byte-identical text.

## Design (generalize; prefer data over code)
- Do NOT add a new compiled handler file or a `try_*` dispatch entry. The debt ratchets `handler_files` (47) and `try_dispatch_entries` (33) in `data/meta/debt-ratchet.lino` may not rise.
- Express the route as a seed handler rule in `data/seed/handler-rules.lino`, interpreted by `rust/src/rule_interpreter/` and its JS twin `js/worker/formal_ai_worker_handler_rules.js`.
- Add ONE generic value source to the rule language, e.g. `value requirements transform requirement_list`. It applies a named pure text transform to the request's free-text payload (the same payload `free_text_payload` / `textTransformFreeTextPayload` extracts for summarization).
  - The transform registry maps a name to a function. Its first entry is `requirement_list`, which calls the extractor.
  - Future transforms (summaries, translations) reuse the same source.
  - The rule declines (falls through) when the payload is missing or yields no requirements.
- Recognition is a new seeded role, e.g. `requirement_listing_action`, with surfaces in en/ru/hi/zh/es ("list the requirements", "extract requirements", "what are the requirements", "перечисли требования", "выпиши требования", "आवश्यकताएँ सूचीबद्ध करें", "列出需求", "列出要求", "enumera los requisitos", ...). It goes in `data/seed/meanings-requirement-extraction.lino`, which you may append to.
  - Run `python3 scripts/generate-role-registry.py` and mirror `data/seed/roles.lino`.
- The response intent is `requirement_listing`, with five-language rows in the right `data/seed/multilingual-responses-*.lino` file (for example `-summarization.lino` or `-text-transform.lino`) and a definition record in `data/seed/meanings-response-intents-handlers.lino`.
  - The response-language debt (`node scripts/generate-response-parity-debt.mjs --check`) must not rise.
- Register the handler wherever a rule-only handler is registered. Follow `github_repository_traffic` or `conversation_control` through:
  - `data/seed/handler-precedence.lino` (before `summarization_text`, so a requirement-listing request is not summarized);
  - `data/seed/capability-routing.lino` and `meanings-routing-vocabulary.lino`;
  - `data/meta/handler-migration-ledger.lino`;
  - the worker registry permutation in `js/worker/formal_ai_worker_20.js`;
  - `rust/src/meta_method_dispatch.rs`;
  - `docs/diagrams/solver-handlers.md`, if it lists handlers.
- Run the existing routing probes that cover summarization or text transforms (JS only). Make sure summarization prompts still summarize.

## Tests
- A JS test in `rust/tests/web/` (e.g. `requirement-listing-route.test.mjs`) that pins exact answers for an English, a Russian and a Chinese prompt through the worker solver, plus one decline (no payload).
- A Rust twin under `rust/tests/unit/`, registered in `rust/tests/unit/mod.rs`. Check it with rustfmt only.

## Requirement row
Update R1188-U20 in `docs/requirements/issue-1188-user-requirements.md`: remove "Missing: a chat route…" and name the route and tests. Then regenerate:
`node scripts/assemble-requirements.mjs --write && node scripts/generate-requirement-status.mjs --write && node scripts/render-status.mjs --write && node scripts/check-requirement-status.mjs`

## Rules
- **No cargo and no rust-script locally.** Check Rust with `rustfmt --edition 2024 --check` only.
- **Minimal local tests:** only the ones next to your change.
- **Disk:** about 20 GB free; no repository copies.
- **Size:** files at most 1500 lines; Rust at most 1000; mind `data/meta/worker-line-budget/*.lino`. A ceiling may rise only with a recorded reason, and only if unavoidable; prefer a new small worker module.
- **Readable:** multi-line code; lines at most 300 code characters.
- **After touching `js/`:** run `node scripts/translate-es.mjs --write`. If you touch a module in the JS→Rust translation scope, run `node scripts/translate-js-rust.mjs --write --fetch <module>`. The translated count must not fall.
- **Gates:** `node experiments/formal_ai_subagent/local-gates.mjs --only <gates>`, using `check_debt_ratchet_js_twin`, `check_file_size_js_twin`, `check_readable_code`, `check_hardcoded_language_js_twin`, `check_worker_line_budget_js_twin`, `check_response_parity_debt`, `check_seed_registry_js_twin`, and `python3 scripts/check-closure-audit.py`.
- **Formal AI:** use it for small edits (`node experiments/js_dogfood/drive.mjs`); ledger rows T520–T539 in `docs/case-studies/pull-request-1188/formal-ai-dogfood.md`; failures as new gaps in `gaps.md`.
- **Claims:** claim your files in `claims.md`.
- **Others:** TEXT-CAPABILITY is working in parallel. Do not touch its `text_formalization` / `dependency_summarization` / `round_trip_translation` / `page_formalization` files or its seed `text-formalization.lino`. CIFIX-LOOP commits CI fixes.
- **Do not commit;** LEAD integrates.
- **Report:** files changed, tests run, and what still differs between the roots.
