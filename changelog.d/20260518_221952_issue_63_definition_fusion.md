### Added
- Added deterministic cross-language definition fusion for prompts like `Merge Wikipedia definitions of IIR`, combining localized seed/Wikipedia definition blocks for the same concept anchor with source-language and citation evidence. Fixes issue #63.
- Added `SolverConfig::definition_fusion_by_default`, `FORMAL_AI_DEFINITION_FUSION`, `formal-ai chat --definition-fusion auto`, and a persisted browser Settings control so plain prompts like `What is IIR?` can opt into the same fusion path.
- Expanded definition-fusion coverage with 15 self-explanatory prompt examples across IIR, color, KISS, Links theory, and Telegram Ads, plus a negative unknown-term case.
