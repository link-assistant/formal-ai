---
bump: patch
---

### Fixed
- **Issue #37 — Russian and Chinese conversation-summary phrases not recognised.** The `try_summarize_conversation` handler now matches Russian phrasings (`о чём мы разговаривали`, `о чём мы говорили`, `резюме беседы`, `резюмируй разговор`, bare `резюме`) and the Chinese shorthand (`总结`), as well as the English `summarize this conversation` and bare `summarize`, so users asking "О чём мы разговаривали?" receive the conversation summary instead of the unknown-intent response.
