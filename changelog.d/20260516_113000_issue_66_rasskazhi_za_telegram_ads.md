---
bump: patch
---

### Added
- `pattern_concept_prefix_rasskazhi_za` and `pattern_concept_prefix_rasskazhi_mne_za` prompt patterns in `data/seed/prompt-patterns.lino` so the colloquial Russian prefix «расскажи за» triggers `concept_lookup` instead of falling through to unknown-intent
- `concept_telegram_ads` entry in `data/seed/concepts.lino` with English and Russian localisations, aliases, and an official source so queries about Telegram Ads are answered from the knowledge base

### Fixed
- Issue #66: «Расскажи за Telegram Ads» now resolves to `concept_lookup` and returns a factual summary about Telegram Ads instead of the generic «я пока не знаю символьного правила» fallback
