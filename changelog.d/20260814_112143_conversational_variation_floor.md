---
bump: minor
---

### Added
- Enforce at least five distinct wording variations per conversational test case in every advertised language (en, ru, hi, zh) with the `check:variation-floor` CI gate, backed by a recorded corpus whose every prompt is answered by the engine and whose every record shows the exact answer that wording produces.
- Join incremental Agent-CLI execution and auto-learning into one evidence-preserving lifecycle: attempt the whole task, split only after failure, compose passing leaves, retry the parent, and feed every recorded session to proposal-only learning behind human review.

### Fixed
- Answer small talk in full in Hindi and Chinese. The question-necessity pass could not find a sentence boundary in a script that does not space its sentences or that ends them with a danda, and its requirement cues covered the English follow-up questions only, so `धन्यवाद`, `谢谢` and `你好吗` answered with an empty string and the Russian and Hindi wellbeing answers lost their closing sentence. A question the answer quotes as an example — in corner brackets or parentheses — is no longer read as a question the answer asks.
- Normalize variation prompts identically in Node and Rust with NFKC plus Unicode category filtering, so fullwidth compatibility characters deduplicate while Hindi combining marks remain meaningful.
