# Plan 00 — the generic page formalizer

Issue #1163 asks that any fetched page become formal knowledge: links that
keep the page's structure, its code with the language the page declares, and
the provenance (URL, fetch time, SHA-256) of every statement. This plan records
the design the branch implements and the limits it knowingly keeps.

## Pipeline

1. **Capture.** Every byte goes through `CachedSourceClient`
   (`rust/src/source_fetch.rs`). Offline it only replays the capture cache; a
   miss is an error, never a guess. Each capture carries `source_url`,
   `fetched_at` and `sha256`, and records a `source:http` trace event
   (`URL fetched_at=… sha256=… cached=…`).
2. **Mime resolution.** The transport's content type, else sniffing (`<html`,
   a Markdown fence or heading, a JSON object), else plain text.
3. **Walk.** One walker per format turns the page into ordered blocks:
   `heading`, `paragraph`, `list_item`, `code_block`, `table_row`, `link`.
   HTML is walked by tag, Markdown by line, JSON by key path, text by
   paragraph. Navigation, scripts and styles are skipped by the seed's
   wrapper and skip lists, not by code.
4. **Language tags.** A code block's language is the first of: the
   `language-`/`lang-` class of `<pre>`, of its `<code>`, or of the enclosing
   container; a seed code attribute (`data-lang`, kotlinlang); a Markdown
   fence. Prefixes and attributes are seed data
   (`data/seed/page-formalization-rules.lino`); an undeclared language is
   `unknown`, never inferred.
5. **Nesting.** A list item or definition term that contains block elements
   (kotlinlang puts each numbered step's command inside the `<li>`) is
   stepped into, so the step's paragraph and its code block become siblings in
   reading order.
6. **Links.** Blocks become link records anchored to the page's URL; each
   statement keeps the page SHA-256. `FormalizedPageStore` indexes pages by
   URL and answers `code_blocks_on(url)` and `command_mentioning(phrase)` —
   the code block that the first paragraph mentioning the phrase introduces.
7. **Trust.** `web_formalize_trust.rs` scores a page by the seed's source
   trust weights (official documentation over forums) so competing statements
   can be ranked; scores never delete a statement.

The browser twin is `js/worker/formal_ai_worker_web_formalize.js`; both read
the same seed file and are pinned by the same captures.

## Evidence the design rests on

- Byte-for-byte captures fetched 2026-10-07 under
  `rust/tests/fixtures/coding-discovery/captured/` (kotlinlang command line,
  Rust Book page and source, Scala 3 book, Kotlin tour, go.dev, Swift book
  source), pinned by SHA-256 and language tags in
  `rust/tests/unit/issue_1163_1164_captured_pages.rs` and
  `rust/tests/web/issue-1163-1164-captured-pages.test.mjs`.
- The captures forced two design changes the trimmed fixtures had hidden:
  kotlinlang's `data-lang` attribute on a `<div>` inside `<li>` (point 4 and
  5), and Scala/Swift `<pre>` blocks whose language class sits on the
  container (point 4).

## Known limits

- go.dev declares no language on its blocks, so they stay `unknown`; the
  decomposer is told the language by the caller instead.
- Indentation inside code blocks is not preserved by the HTML walker (the
  decomposed programs in `docs/case-studies/issue-1164/decomposed-example/`
  show it).
- The Agent CLI research path does not use the formalizer yet
  (`../agent-cli-evidence/kotlinlang-formalization/README.md`): it answers from
  `webfetch` plain text and drops the code block a how-to question needs.
