# Delivery verification — requirements chunk #156–#310 (136 items)

Verified 2026-08-04 against the working tree at `de61602f` (read-only: grep/read/jq/gh only).
Full per-item records: `verified-156-310.ndjson`.

## Verdict counts

| Verdict | Count |
|---|---|
| DELIVERED | 125 |
| PARTIAL | 10 |
| OBSOLETE | 1 |
| NOT-DELIVERED | 0 |
| UNVERIFIABLE-LOCALLY | 0 |

Every claimed-delivered item received at least one concrete probe (file:line, seed entry, or pinning test); none were rubber-stamped.

## Standing-scope enforcement checks (special attention items)

- **Language-parity guard — ALIVE and in CI.** `tests/e2e/package.json` defines `check:language-parity` (`check-language-change-parity.mjs`), `check:intent-coverage` (`check-multilingual-intent-coverage.mjs`), `check:i18n`; all three run in `.github/workflows/release.yml:445–458`.
- **Translation-arc numeric rules — mostly enforced, one gap.**
  - Cache-only-raw-API-responses: enforced by design (`src/translation/cache.rs` `CachedHttpClient`; raw JSON in `data/cache/wikidata/{entity,property,lexeme}/`; legacy derived caches removed in #398).
  - `.lino ≤ 1500` lines: enforced by `scripts/check-file-size.rs` (CI `release.yml:390`) and `tests/unit/data_files.rs` — **but** `check-file-size.rs:57` excludes `data/cache/wikidata/` from the gate (files currently comply anyway; max is 1347 lines).
  - **≤128 most-frequent words/entities/properties: NOT enforced.** `MAX_SEED_RECORDS_PER_BUCKET = 128` exists only as a constant (`src/translation/cache.rs:70`); `data/cache/wikidata/entity/` holds **394** entities today. → new issue proposed (R222-1).
  - Human-readable, never base64: enforced by `tests/unit/data_files.rs:13,121` (`…human_readable_and_bounded`, `no_codepoint_byte_dumps`); zero base64 hits in `data/cache`.
- **#171 iframe-policy "silent drop" — actually DELIVERED later.** The audit narrative's "most clear-cut dropped requirement" is wrong for the current tree: `src/web/worker/formal_ai_worker_18.js:83` implements `detectFramePolicy()` via an external CORS-free frame-policy service (parses X-Frame-Options + CSP frame-ancestors), it runs before every iframe preview, and e2e tests pin exactly one policy request per navigation prompt (`tests/e2e/tests/multilingual.spec.js:1083–1175`).
- **Three unfiled upstream issues:**
  - #164 calculator currency: **moot** — currency delegation works (`link-calculator = 0.20.3`, exact prompt pinned in `tests/unit/specification/calculator_delegation.rs:171`), and konard himself had already filed currency issues upstream (link-assistant/calculator #54, #123, both pre-dating #164). No missing feature remained to report.
  - #185 RML-as-library: **still unfiled.** RML is absent from Cargo.toml; `src/relative_meta_logic.rs` re-models it in-repo. `gh` search of link-foundation/relative-meta-logic shows no issue about library usability from formal-ai. → new issue proposed.
  - #209 RML wasm-to-wasm: **still unfiled**, and no wasm proof compilation exists in `src/proof_engine/`. → new issue proposed.

## PARTIAL + untracked → proposed new issues (needs_issue = 5)

1. **R185-1** — File the promised upstream issue on link-foundation/relative-meta-logic about consuming it as a Rust library from formal-ai (from #185).
2. **R209-1** — File the promised relative-meta-logic wasm compilation feature request or record why it is technically impossible (from #209). *(1 and 2 could be one combined RML filing issue.)*
3. **R222-1** — Enforce the 128-records-per-bucket cache cap and include `data/cache` in the `.lino` line gate (from #222). Entity bucket is at 394/128 today.
4. **R234-2** — Add the promised CI rule enforcing tests-as-docs exact-answer style (from #234). Style is practiced (e.g. `tests/unit/assistant_name.rs`) but nothing enforces it.
5. **R234-4** — Codify the `Fixes <issue-url>` PR-linking and pull-request case-study conventions in CONTRIBUTING (from #234). Neither CONTRIBUTING.md nor the PR template mentions issue linking.

## PARTIAL but already tracked

- **R180-7** (double tests / 100% coverage) → **#895** coverage ratchet.
- **R224-1** (answer any research question) → **#873** research-then-answer principle (gap evidenced by open #720/#722/#872).
- **R226-2** (learn whole class, never memoize a pair) → **#922** E75 method learning; seeded one-concept fixes (#286/#288 → `concepts.lino`) show the doctrine still gets bypassed.
- **R244-2** (minimum core solving any problem) → **#918** E71 minimal-core boundary audit.
- **R245-9** (verified ability to answer/manipulate anything) → **#923** E76 formal-reasoning coverage growth.

## OBSOLETE

- **R195-2** (use link-foundation/start `--isolation docker`): superseded by the E11/E26 isolation model konard directed in #256/#303 — delivered as `src/agent.rs` bounded workspace execution + `tests/unit/specification/agent_isolation.rs`. The underlying goal (execute/test code before returning it) is met; the `start`-specific mechanism was never adopted.

## Hand-check list (runtime verification recommended, not blocking)

- **R171-2**: confirm the production frame-policy check endpoint is reachable from the deployed GitHub Pages app (e2e mocks it via route interception).
- **R180-5**: open web-app diagnostics and confirm raw HTTP exchanges render expandably for a live web search (`formatHttpExchangeAsLinks`, `main.jsx:5795`).
- **R304-1**: run the external-benchmark suite to confirm pass ratios match the recorded ratchet floors (`data/benchmarks/external-results.lino`).

## Surprising discoveries

1. **The #171 frame-policy requirement was NOT silently dropped** — a real detection mechanism with an external service and e2e tests exists (see above). The 2026-07 audit narrative should be corrected.
2. **The report-URL decoding script (#159) exists**: `scripts/decode-github-issue-url.rs` — another item the narrative listed as "no delivery evidence".
3. **wikiHow integration (#172) was delivered**: `src/solver_handler_how.rs:50` builds wikiHow API candidates; `wikihow_candidate` events in `src/event_log.rs:615`.
4. **Raw-HTTP diagnostics (#180) shipped**: worker ring buffer (`recordWebSearchDiagnostic`) + Links-Notation projection in the diagnostics panel.
5. **The 128-record cache cap is the one numeric translation-arc rule that quietly rotted**: constant documented, bucket at 394 entities, gate deliberately excludes the cache dir (`check-file-size.rs:57` `EXCLUDE_PATH_FRAGMENTS`).
6. **Doctrine tension confirmed in-tree**: #286/#288 fixes are seeded `concepts.lino` entries — exactly the memoization pattern konard bans; the class-learning epic (#922) is the right tracker.
7. The standing process constitution (case studies, debug tracing, upstream filings, one-PR-per-issue) **is codified** in CONTRIBUTING.md rules 7–10, with 218 case-study directories — but the `Fixes` linking rule from #234 never made it into any process doc.
