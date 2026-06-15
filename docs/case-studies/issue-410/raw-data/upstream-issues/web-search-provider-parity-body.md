FormalAI issue link: https://github.com/link-assistant/formal-ai/issues/410

During the FormalAI issue 410 case study, I compared FormalAI's current `src/web_search_core.rs` provider contract with `web-search` HEAD.

Current gaps:

- FormalAI's live default plan is `duckduckgo`, `internet-archive`, `wikipedia`, `wikidata`, `wiktionary`, `wikinews`.
- `web-search` currently reports default providers as `duckduckgo`, `google`, `bing`, `wikipedia`.
- The registry metadata marks Google as `defaultForCategory: true` and DuckDuckGo as `defaultForCategory: false`, while FormalAI currently treats DuckDuckGo as the default search provider.
- `web-search` does not yet include several FormalAI provider IDs, including `internet-archive`, `wiktionary`, `wikinews`, `openlibrary`, `semantic-scholar`, `europepmc`, `doaj`, `dbpedia`, the dictionary providers, `yandex`, and the non-GitHub code hosts (`gitlab`, `codeberg`, `gitee`, `bitbucket`, `gitflic`).

Why this blocks FormalAI:

Replacing FormalAI's current in-repo registry with `web-search` right now would remove existing provider coverage and would change the default search semantics. FormalAI issue 410 asks to use `web-search` as the component, but it also asks to make sure the component has all supported/required features first.

Requested acceptance criteria:

- Make `web-search` a superset of FormalAI's current provider IDs or provide a documented compatibility map for provider IDs that intentionally differ.
- Align default search semantics with FormalAI's DuckDuckGo-first behavior, or document a migration plan that FormalAI can opt into explicitly.
- Include FormalAI's current live default plan: DuckDuckGo, Internet Archive, Wikipedia, Wikidata, Wiktionary, and Wikinews.
- Add provider-registry tests in JavaScript and Rust that assert FormalAI-compatible IDs, categories, CORS/fetchability metadata, and defaults.
- Keep the optional `web-capture` provider namespace, but make clear which providers are native search providers and which are delegated through capture.

Evidence captured in FormalAI PR 414 case-study raw data:

- `docs/case-studies/issue-410/raw-data/formal-ai/web_search_core.rs`
- `docs/case-studies/issue-410/raw-data/web-search/provider-registry.json`
- `docs/case-studies/issue-410/raw-data/web-search/provider-code-index.txt`
