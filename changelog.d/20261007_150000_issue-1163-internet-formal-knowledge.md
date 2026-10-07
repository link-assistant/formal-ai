---
bump: minor
---

### Added
- Web search pages become formal knowledge (issue #1163): source research stores every captured page's formalized network in the solver's working memory under its URL and SHA-256 and reuses it on a second pass, and its learning proposal carries one `formalized_page_statement` record per statement.
- The memory query language resolves "code blocks on `<domain>` whose text contains `<term>`" and "the command in the paragraph that mentions `<phrase>`" from `page_query` templates in `data/seed/page-formalization-rules.lino`; the second answers only for paragraphs that themselves mention the phrase, with the command each one introduces.
- A need whose kind no registered source declares can be researched through web search (`research_unmatched_need`), and the sources registry gains a `web_page` row whose `generic_page_v1` extractor reads any fetched page through the generic formalizer in the concept-lookup dispatcher.
- The browser worker gets the generic page formalizer as a JavaScript twin (`js/worker/formal_ai_worker_web_formalize.js`): mime hints and sniffing, the HTML walker, Markdown, JSON and plain-text blocks, seed-driven language tags, the URL-and-hash page store, both page queries and the trust score, with node tests over the same fixtures as the Rust tests.
