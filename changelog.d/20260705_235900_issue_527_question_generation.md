### Added

- Added a lazy, configurable issue #527 question-generation API with grammar and meaning classification plus an answer stream through `FormalAiEngine`.
- Added the `question_catalog` agentic recipe (the eleventh) that drives Formal AI through its own Agent CLI to enumerate questions smallest-first, classify them, answer the meaningful ones, and record the reviewable catalog in Links Notation (`data/meta/question-catalog.lino`), grounded in the `data/seed/question-generation-lexicon.lino` frequency-tier vocabulary. Answered questions form a case/whitespace-insensitive recall table (`QuestionCatalog::answer_for`) that never mutates the human-gated learning ledger.
