# Issue 712 requirements

| ID | Requirement | Evidence |
| --- | --- | --- |
| R712-01 | Every reported URL-fetch phrasing selects the advertised fetch tool and preserves the URL. | `tests/issue_712.rs::failed_web_fetch_phrasings_route_by_url_intent` |
| R712-02 | Every reported web-search phrasing selects the advertised search tool with a non-empty subject. | `failed_web_search_phrasings_route_by_research_intent` |
| R712-03 | Update, modify, patch, rewrite, substitute, and refactor replacement requests select edit and preserve path/old/new. | `failed_edit_phrasings_route_by_replacement_shape` |
| R712-04 | Declarative `new file` plus `contents` selects write, never read. | unit and HTTP integration regressions |
| R712-05 | Routing must use verb class plus typed argument shape, not whole-sentence templates. | semantic-frame parsers and unseen-paraphrase tests |
| R712-06 | A tool call is emitted only for a capability advertised by the client. | `intent_router` capability lookup and matrix tests |
| R712-07 | Natural language must live in the Links Notation seed, not Rust verb tables. | seed roles; removal of `WRITE_VERBS` |
| R712-08 | Shared algorithms must accept English, Russian, Hindi, and Chinese seed forms. | multilingual edit and write tests |
| R712-09 | OpenAI Chat Completions, OpenAI Responses, and Gemini adapters must agree. | `tests/integration/issue_712_intent_routing.rs` |
| R712-10 | The browser worker and native solver must implement the same search frames. | `formal_ai_worker_16.js`, `formal_ai_worker_17.js`, Rust handler |
| R712-11 | A failing regression must precede the fix, and unseen variants must prevent overfitting. | red log and `unseen_web_search_paraphrases_route_by_semantic_frame` |
| R712-12 | Formal AI must execute the task through Agent CLI, including a real external CLI run. | release workflow, external runner, committed evidence |
| R712-13 | Auto-learning must derive a persistent ranked artifact and remain human-review gated. | routing-learning memory/module/tests |
| R712-14 | The work must incorporate current `main`, issue comments, all PR comment types, and the maintainer follow-up in PR #719. | merge commit and this case study |
