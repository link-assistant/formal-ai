# Issue 498 Case Study

Status: **delivered and driven by the Agent CLI + Formal AI**, not deferred.
Issue [#498](https://github.com/link-assistant/formal-ai/issues/498) asks Formal
AI to learn from popular Google searches visible in Google Trends, especially the
top requests for the U.S. Trends page in Russian UI mode.

The implemented slice turns a Google Trends RSS snapshot into a deterministic
test catalog: the top 10 topics, two prompt variations in every supported
language, and one Formal AI answer for every generated prompt. The live fetch is
kept in the refresh tool path; tests consume the checked-in seed so CI remains
offline and reproducible.

## Source material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/498>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/640>
- Raw GitHub and Google Trends data: [`raw-data/`](raw-data/)
- Requirements decomposition: [`requirements.md`](requirements.md)
- Per-requirement solution plan: [`solution-plans.md`](solution-plans.md)
- Online research notes: [`raw-data/online-research.md`](raw-data/online-research.md)
- Committed Agent CLI session:
  [`agent-cli-session-google-trends.json`](agent-cli-session-google-trends.json)
- Captured end-to-end Agent CLI run:
  [`agent-cli-e2e-run.log`](agent-cli-e2e-run.log)
- Generated catalog artifact:
  [`../../../data/meta/google-trends-catalog.lino`](../../../data/meta/google-trends-catalog.lino)
- Deterministic Trends seed:
  [`../../../data/seed/google-trends-snapshot.lino`](../../../data/seed/google-trends-snapshot.lino)

## 1. Collected Data

- Issue snapshot: `raw-data/issue-498.json`.
- Issue comments: `raw-data/issue-498-comments.json`.
- Prepared PR snapshot: `raw-data/pr-640.json`.
- PR conversation, review-comment, and review snapshots:
  `raw-data/pr-640-comments.json`, `raw-data/pr-640-review-comments.json`, and
  `raw-data/pr-640-reviews.json`.
- Google Trends RSS snapshot:
  `raw-data/google-trends-us-rss.xml`, collected from
  `https://trends.google.com/trending/rss?geo=US&hl=ru`.
- Online research notes: `raw-data/online-research.md`.
- The committed Agent CLI session and transcript record the exact `write_file`,
  verification command, and final answer for the generated catalog.

No issue screenshots were present, so there were no image attachments to
download or verify.

## 2. Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R498-1 | Preserve issue data, PR data, online research, requirements, solution plans, and raw Trends evidence under `docs/case-studies/issue-498`. | This directory stores all raw GitHub snapshots, the raw Trends RSS XML, this README, requirements, solution plans, research notes, and the Agent CLI transcript/session. |
| R498-2 | Convert the Google Trends top 10 searches into a durable Formal AI training/test artifact. | `src/google_trends_catalog.rs` parses a checked-in Trends snapshot and caps the catalog with `GOOGLE_TRENDS_TOP_LIMIT = 10`; `data/meta/google-trends-catalog.lino` is the reviewable artifact. |
| R498-3 | Generate variations in every supported language. | The catalog expands each topic across `supported_languages()` (`en`, `ru`, `hi`, `zh`) with two prompt variants per language. |
| R498-4 | Answer every generated prompt through Formal AI rather than only storing prompts. | `google_trends_catalog()` delegates each prompt to `FormalAiEngine::answer` and records intent, confidence, answer text, and evidence links. |
| R498-5 | Keep live Google Trends access reproducible and safe for CI. | The RSS parser and refresh examples can update `data/seed/google-trends-snapshot.lino`; tests use the checked-in seed without network access. |
| R498-6 | Drive the solution through Formal AI's own Agent CLI. | The `google_trends_catalog` agentic recipe writes the catalog, verifies it with a sandboxed compact read, and returns the generated document. |
| R498-7 | Pin the Agent CLI session byte-for-byte. | `tests/unit/issue_498_google_trends_catalog.rs` compares a fresh `run_agentic_task(GOOGLE_TRENDS_CATALOG_TASK)` session with `agent-cli-session-google-trends.json`. |
| R498-8 | Keep the documentation and implementation contract test-covered. | `tests/unit/docs_requirements_issue_498.rs` protects this case-study evidence, and the issue-specific tests protect parser, catalog, recipe routing, and artifact freshness. |

## 3. Root Cause

Before this change, Formal AI could answer a user-supplied prompt but had no
first-class path from current search demand into reviewable prompts and answers.
Issue #498 named Google Trends as the source, and the maintainer clarified that
the result should become automated scripts/tools that collect data and add test
cases across all supported languages.

The missing abstraction was a catalog boundary:

- collect the top searches from Google Trends;
- store the live snapshot as a small seed instead of relying on a changing
  network feed in CI;
- expand each trend into multilingual prompt variants;
- answer every prompt through the normal symbolic engine; and
- commit the generated artifact and Agent CLI session so future changes are
  reviewable.

## 4. Implemented Design

`src/google_trends_catalog.rs` owns the Trends conversion:

- `parse_google_trends_rss` parses the RSS feed shape used by
  `https://trends.google.com/trending/rss?geo=US&hl=ru`.
- `render_google_trends_snapshot_lino` renders a parsed feed as
  `data/seed/google-trends-snapshot.lino`.
- `google_trends_catalog()` loads that seed, keeps the top 10 topics, generates
  two prompts per supported language, and answers each prompt through
  `FormalAiEngine`.

`src/agentic_coding/google_trends_catalog.rs` owns the agentic recipe:

- `is_google_trends_catalog_task` routes differently worded Trends catalog
  requests without colliding with the question-catalog recipe.
- `render_document` emits the answered catalog as Links Notation.
- The planner walks `write_file -> run_command -> final`; the run step uses the
  sandboxed `python3` allowlist to print the line count and first 12 lines rather
  than dumping the full catalog through the tool result.

The snapshot collected on 2026-07-08 records these top 10 U.S. Trends queries:

1. `julián andrés quiñones`
2. `blue jays`
3. `valli geiger`
4. `grok 4.5`
5. `london city lionesses`
6. `alexia putellas`
7. `phil regan`
8. `connor bedard`
9. `fire pits`
10. `brandy norwood`

## 5. Prior Art And Existing Components

| Component | Relevance | Decision |
| --- | --- | --- |
| Google Trends page and RSS feed | Provides the issue's source of popular current searches and a simple top-search feed shape. | Use RSS for the first implementation because it is easy to snapshot and parse without a browser. |
| Google Trends API alpha | Google now documents a programmatic API alpha, but access is limited and designed for approved testers. | Record it as the future production-grade direction; do not make CI depend on alpha access. |
| pytrends | Popular unofficial automation library for Google Trends, but it warns that it can break when Google changes backend behavior. | Record as prior art; avoid adding it as a runtime dependency for this deterministic Rust implementation. |
| Existing `FormalAiEngine` | Already answers prompts with trace evidence. | Reused for every generated prompt; no parallel answer path was added. |
| Existing Agent CLI recipe pattern | Issue #527 already demonstrated byte-for-byte generated catalogs driven by the in-repo agentic CLI. | Mirror the pattern with a Google Trends-specific recipe and pinned session. |

## 6. Verification

Reproducing tests were added before implementation:

```sh
cargo test --test unit issue_498 -- --nocapture
```

Before the implementation, those tests failed to compile because the
`google_trends_catalog` API, agentic recipe, and generated artifacts did not
exist.

After the implementation, focused coverage verifies:

- RSS parsing into ranked topics with traffic, pub date, and news references;
- checked-in catalog coverage for the top 10 topics, all supported languages,
  two variations per language, and one answer per prompt;
- byte-for-byte freshness of `data/meta/google-trends-catalog.lino`;
- recipe routing and planner behavior without colliding with the question
  catalog;
- a fresh Agent CLI run matching the committed
  `agent-cli-session-google-trends.json`; and
- this documentation/evidence contract.

To refresh the Trends seed from the live feed:

```sh
curl -sL 'https://trends.google.com/trending/rss?geo=US&hl=ru' \
  | cargo run --example issue_498_parse_google_trends_rss \
  > data/seed/google-trends-snapshot.lino
```

Then regenerate the catalog and Agent CLI session:

```sh
cargo run --example dump_google_trends_catalog > data/meta/google-trends-catalog.lino
cargo run --bin formal-ai -- agent \
  --task "Convert the Google Trends top 10 searches into multilingual Formal AI test prompts, include two request variations in every supported language, answer each request, and record the Google Trends catalog in Links Notation." \
  --transcript \
  --session-json docs/case-studies/issue-498/agent-cli-session-google-trends.json \
  > docs/case-studies/issue-498/agent-cli-e2e-run.log
```
