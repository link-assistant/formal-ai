Part of the E127 umbrella (#1183). The architect's words are quoted there.

## Gap

Reasoning can only draw on 16 registered sources through 6 source-specific extractors (`data/seed/sources-registry.lino`, `rust/src/concept_lookup.rs:272`). Web search exists (`rust/src/web_search_core.rs`, `rust/src/source_research.rs`) but its results are "observations" that do not become formal knowledge the solver can use. `rust/src/document_formats.rs` already converts documents through `meta_language::LinkNetwork::reconstruct_text_as`, yet no path fetches an arbitrary page and hands its structure to reasoning.

Line anchors (origin/main `d209aac64`): `concept_lookup.rs:265` `read_glosses` (the extractor dispatcher), `document_formats.rs:186-187`, `source_fetch.rs:172-178` (`SourceCapture`).

## What to build

1. **One generic page formalizer.** Any fetched resource (HTML, Markdown, plain text, JSON, PDF text, source files) → `LinkNetwork` via meta-language document parsing: headings, paragraphs, lists, tables, and every code block as a code node tagged with its language (from fence/class/heuristics). Keep the six bespoke extractors only as faster paths whose output is checked to equal the generic formalizer's on the same bytes.
2. **Search as the entry point for any need.** A need with no registered source goes to web search (existing providers + RRF fusion), fetches the top results through the capture boundary (`source_fetch` cache with hash and timestamp), formalizes each page, and ranks the resulting statements. Trust is a score (publisher domain ownership, official docs, license, agreement across pages, the existing `primacy` field), used first, never as a whitelist.
3. **Formal API over formalized pages.** Queries over the formalized pages go through the existing memory query language: "code blocks on `kotlinlang.org` whose text prints a string", "the command in the paragraph that mentions compiling". Results are links, not strings.
4. **Rediscovery record.** Every formalized page stores the procedure that found it (query, rank, URL, hash), so cached knowledge can be dropped under storage pressure and recovered (2026-09-14 note).

## How to test

- Fixture: captured copies of `https://kotlinlang.org/docs/command-line.html`, a Rust Book page and a Scala docs page → formalized networks contain the code blocks with correct language tags; the six bespoke extractors agree with the generic formalizer on their fixtures.
- Online test (gated like the existing `--online` runs): a need "how to compile a Kotlin program from the command line" resolves to the `kotlinc … -include-runtime -d …` command from the official page, with the URL and hash in the trace.

## Already-verified anchors to build from (confirmed on origin/main)

- `rust/src/concept_lookup.rs:244` — `fn sense_language`; line 265 — `fn read_glosses(extractor: &str, bytes: &[u8], surface: &str)` dispatcher matching on `"wiktionary_entry_v1"`, `"wordnet_sense_v1"`, `"mediawiki_summary_v1"`, `"wikidata_entity_v1"`, falling back to `projection_glosses` for cached `.lino` projections.
- `data/seed/sources-registry.lino` — 16 `source <slug>` blocks, 6 distinct `extractor` values. Shape confirmed: `source <slug>` / `name` / `kind` / `service_group` / `default_enabled` / `need_kinds (...)` / `extractor <slug>_v1` / `primacy <kind>` with nested `upstream`/`basis` / `api "..."` / `api_language ...` / `license_name`/`license_url` / `cache_path data/cache/<slug>/` / `note "..."`.
- `rust/src/web_search_core.rs` — `no_std`+`alloc`: `WEB_SEARCH_RRF_K: u32 = 60`, `WEB_SEARCH_CONCURRENCY_PER_CATEGORY: u32 = 5`, `WEB_SEARCH_PROVIDER_LIMIT: u32 = 10`, `enum ProviderCategory { Search, Knowledge, Papers, Code }`, `struct ProviderSpec { id, label, category, cors_readable, default_for_category }`.
- `rust/src/document_formats.rs:186-187` — `LinkNetwork::parse(source_text, source, ParseConfiguration::default())` then `network.reconstruct_text_as(target, ParseConfiguration::default())`.
- `rust/src/source_fetch.rs:172-179` — `pub struct SourceCapture { source_url: String, fetched_at: String, sha256: String, cached: bool, bytes: Vec<u8> }` with accessor methods and `trace_payload()`. `CachedSourceClient`/`CurlSourceTransport`/`SourceCapture`/`SourceTransport` are the fetch/cache boundary.
- `meta-language` crate: version `0.58.2` (`rust/Cargo.toml`). `LinkNetwork::parse(source, label, ParseConfiguration::default())` is the document-parsing entry point used in `document_formats.rs`, `agentic_coding/self_ast.rs`, and `coding/cst.rs`. No HTML-to-structure path currently exists — only source-code parse and document format conversion.
- `rust/src/meta_translate.rs` — any-direction translation pivot (`Rust ↔ Meta`, `JS ↔ Meta`, `TS ↔ Meta`); JS/TS parity for this issue is delivered through `meta_translate.rs` projecting formalized page networks into ES-compatible representations, consistent with the 2026-09-24 three-roots doctrine.

---

## Requirements

R1. A single `formalize_page(bytes: &[u8], mime_hint: Option<&str>) -> LinkNetwork` function in `rust/src/page_formalizer.rs` (new) produces a `LinkNetwork` from any HTML, Markdown, plain-text, JSON, or PDF-text payload.

R2. Every code block in a formalized page carries a language tag derived from the fence annotation, HTML class, or a heuristic fallback; an unrecognized fence produces the tag `"unknown"` rather than an empty tag.

R3. For each of the six bespoke extractors, `formalize_page` on the same fixture bytes yields every statement the bespoke extractor yields (generic ⊇ bespoke), so each bespoke extractor becomes a faster path that can be deleted without losing knowledge.

R4. A need whose `need_kinds` has no match in `data/seed/sources-registry.lino` routes to web search via the existing `web_search_core.rs` RRF pipeline rather than returning empty.

R5. Each web-search result is fetched through `CachedSourceClient` (existing `source_fetch.rs`), the `sha256` and `fetched_at` fields are recorded, and the bytes are passed to `formalize_page`.

R6. The formalized `LinkNetwork` for each fetched page is stored in the solver's working memory under a key that encodes the URL and SHA-256, so a second solve for the same URL reuses the network without a re-fetch.

R7. A trust score in [0, 1] is attached to each formalized page, computed from features whose weights live in data (new `data/seed/source-trust-weights.lino`, never in code): the page is on the subject's official website (Wikidata P856 of the formalized subject, e.g. kotlinlang.org for Kotlin), it is served over HTTPS, the registry `primacy` of a matching registered source (`first_hand_record`, `self_published`, `editorial_synthesis`, `citation`), an open license, and agreement of the extracted statement across independently fetched pages. The score ranks sources; it never excludes one.

R8. The memory query language resolves "code blocks on `<domain>` whose text contains `<term>`" and "the command in the paragraph that mentions `<phrase>`" against formalized-page networks and returns `Link` values, not strings.

R9. Every formalized page record stores the rediscovery procedure (query text, rank at time of fetch, URL, SHA-256, timestamp) as a `rediscovery` node so the record can be dropped under storage pressure and re-fetched deterministically.

R10. Three-roots parity: the JavaScript and TypeScript roots get the formalizer by translating `rust/src/page_formalizer.rs` with `formal-ai translate` (rust → js, rust → ts; both legs are live per `meta_translate.rs:95-108`), committing the outputs under `js/` and `ts/`, and extending the parity fixture `data/parity/cross-runtime-synthesis.json` with formalization cases that all three roots must answer identically. The browser worker (`js/worker/formal_ai_worker.js`) calls the JS root.

---

## Design

### New file

**`rust/src/page_formalizer.rs`** (new)

```rust
use meta_language::{LinkNetwork, ParseConfiguration};
use crate::source_fetch::SourceCapture;

/// Mime categories the formalizer recognises; `Unknown` falls back to plain-text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageMime { Html, Markdown, PlainText, Json, PdfText, Unknown }

impl PageMime {
    pub fn from_hint(hint: Option<&str>) -> Self { /* … */ }
}

/// Convert raw fetched bytes into a `LinkNetwork`.
///
/// Headings → section links; paragraphs → statement links; lists → list links;
/// tables → table links; fenced code blocks → code links with a `language` field.
pub fn formalize_page(bytes: &[u8], mime: PageMime) -> LinkNetwork { /* … */ }

/// Attach a trust score and rediscovery procedure to a just-formalized network.
pub fn annotate_capture(network: &mut LinkNetwork, capture: &SourceCapture,
    query: &str, rank: u32, trust: u8) { /* … */ }
```

Language tag resolution order: fence annotation → HTML `class="language-<x>"` → heuristic (shebang, extension in URL) → `"unknown"`.

### Files to modify

| Path | Change |
|---|---|
| `rust/src/concept_lookup.rs` | After the `read_glosses` dispatcher (line 265): add a `formalize_page` fallback branch for extractor value `"generic_page_v1"`. |
| `rust/src/source_research.rs` | After search results are ranked: call `formalize_page` on each `SourceCapture`, store the network, attach the rediscovery record. |
| `rust/src/document_formats.rs` | Expose `formalize_page` as a `DocumentConversion` source when `source_format == "html"` or `"markdown"`. |
| `rust/src/memory_query_language/mod.rs` | Add query predicates `code_blocks_on(domain, term)` and `command_mentioning(phrase)` that match against formalized-page node types. |

### Data file snippet (Links Notation style — matches `data/seed/sources-registry.lino`)

```lino
sources_registry
  source web_page
    name "Generic Web Page"
    kind web_document
    service_group external_open
    default_enabled true
    need_kinds (procedure concept how_to code)
    extractor generic_page_v1
    primacy self_published
      upstream "the page's own publisher"
      basis "A page on the subject's official website (Wikidata P856) is the publisher's own statement about its product."
    cache_path data/cache/web_page/
    note "Fallback source for any URL not covered by a bespoke extractor. Fetched through CachedSourceClient; bytes formalized via formalize_page."
```

### JS/TS parity

Per R10: `formal-ai translate --from rust --to js --input rust/src/page_formalizer.rs --write` and the same with `--to ts` produce the other two roots; the translated files are committed and the parity fixture pins identical outputs. `js/wasm-worker/` (which `#[path]`-includes `no_std` Rust modules such as `arithmetic.rs`) may also include the module if it is kept `no_std` + `alloc`, but the translated JS/TS root is the doctrine's requirement.

---

## Tests

New test modules are registered in `rust/tests/unit/mod.rs` by adding one `mod` line each, following the existing style (e.g., `mod issue_1138_prerequisite_need;`):

```
mod issue_1163_page_formalizer;
mod issue_1163_search_entry_point;
mod issue_1163_memory_query;
mod issue_1163_live_fetch;
```

**`rust/tests/unit/issue_1163_page_formalizer.rs`**
- `test_html_code_block_language_tag`: given a minimal HTML fixture with ` ```kotlin `, asserts the formalized network contains exactly one code link with `language = "kotlin"`.
- `test_unknown_fence_tag`: an unfenced code block produces `language = "unknown"`.
- `test_generic_formalizer_covers_bespoke_extractors`: for each of the six extractors, on its fixture under `rust/tests/fixtures/issue-1163/<extractor>/` (captured from the existing `data/cache/<source>/` entries), asserts every statement of the bespoke output is present in `formalize_page`'s output.
- Fixtures: captured HTML of `https://kotlinlang.org/docs/command-line.html`, one Rust Book page, one Scala docs page — stored as `rust/tests/fixtures/issue-1163/kotlinlang-command-line.html`, `rust-book-page.html`, `scala-docs-page.html`.

**`rust/tests/unit/issue_1163_search_entry_point.rs`**
- `test_no_source_match_routes_to_web_search`: a need with `need_kinds` absent from `data/seed/sources-registry.lino` triggers the web-search path and returns a non-empty result list when the mock search provider is loaded.
- `test_capture_stores_sha256_and_timestamp`: after a formalize call the stored record includes non-empty `sha256` and `fetched_at` fields.

**`rust/tests/unit/issue_1163_memory_query.rs`**
- `test_code_blocks_on_domain_query`: a formalized network for `kotlinlang.org` matches the predicate `code_blocks_on("kotlinlang.org", "kotlinc")`.
- `test_command_mentioning_query`: a network containing a paragraph with "compiling" plus a code block matches `command_mentioning("compiling")` and returns a `Link`, not a `String`.

**`rust/tests/unit/issue_1163_live_fetch.rs`**
Gated identically to `rust/tests/unit/issue_991_how_to_synthesis.rs`:

```rust
fn live_fetch_requested() -> bool {
    matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[test]
fn live_kotlin_command_resolves_to_kotlinc() {
    if !live_fetch_requested() { return; }
    // fetches https://kotlinlang.org/docs/command-line.html,
    // formalizes, queries for code blocks mentioning "kotlinc",
    // asserts the result contains "-include-runtime" and the URL+hash in trace.
}
```

**Run commands:**
```
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1163_page_formalizer
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1163_search_entry_point
RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1163_memory_query
FORMAL_AI_LIVE_FETCH=1 RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1163_live_fetch
```

---

## Definition of done

- [ ] `docs/requirements/issue-1163-internet-as-formal-knowledge.md` exists with the table format from `docs/requirements/issue-1138-prerequisite-discovery.md` (`| ID | Requirement | Status / evidence |`), one row per R1–R10, all rows showing evidence of automated test coverage.
- [ ] `REQUIREMENTS.md` regenerated: `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/assemble-requirements.rs --write`.
- [ ] `docs/requirements-traceability.md` has one row per R1–R10 with columns `| ID | Shard | Delivered | Automated test | Manual confirmation |`.
- [ ] `docs/case-studies/issue-1163/` exists with: `plans/00-formalizer-design.md`, `agent-cli-evidence/kotlinlang-formalization/README.md` (showing the formalized network), `self-use/` (a solve trace where the solver uses a formalized page to answer a code question).
- [ ] Changelog fragment `changelog.d/<timestamp>_issue-1163-internet-formal-knowledge.md` (repository root, same front-matter format as the existing fragments: `bump: minor`, `### Added`).
- [ ] CI gate green: `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust`.

---

## Depends on / blocks

**This issue depends on nothing in #1154–#1183** — `source_fetch.rs`, `web_search_core.rs`, `document_formats.rs`, and `meta_translate.rs` are all already on `main`. E128 is the foundational web-knowledge layer.

**E128 blocks:**
- **#1164 E129** (code examples decompose/recompose) — decomposition requires formalized code blocks, which E128 provides.
- **#1165 E130** (discovery on production path) — production discovery routes through the generic page formalizer introduced here.
- **#1168 E133** (latest versions) — version discovery from official pages requires fetching and formalizing those pages.
- **#1170 E135** (self-coding) — self-coding consults formalized documentation pages for API signatures and usage examples.

#1166 E131, #1167 E132 do not strictly require E128 to land first and may proceed in parallel.

