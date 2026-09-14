## Issue #527 Question Generation Requirements

Issue [#527](https://github.com/link-assistant/formal-ai/issues/527) asks for a
system that can generate possible questions from shortest to longest, use
frequency-ranked words, distinguish grammatical and logically meaningful
questions, make the criteria configurable, and answer the generated questions.

| ID | Requirement | Status |
| --- | --- | --- |
| R527-1 | Issue data, online research, requirements, and solution planning must be compiled under `docs/case-studies/issue-527`. | Implemented by `docs/case-studies/issue-527/{README,requirements,solution-plans}.md`, `raw-data/online-research.md`, and raw GitHub snapshots. |
| R527-2 | Candidate questions must be generated lazily from one-word questions to two-word questions and onward, without materializing the infinite search space. | Implemented by `QuestionGenerator`, an iterator over `GeneratedQuestion` values ordered by `word_count`. |
| R527-3 | The vocabulary must be rankable from multiple corpus-frequency observations and support the issue's 10%, 5%, 2.5%, ... ranked-word policy. | Implemented by `QuestionWord::from_corpus_scores`, `QuestionGenerationConfig`, and the default issue #527 percentage policy; tests can opt into `with_all_ranked_words()` for exhaustive tiny-vocabulary checks. |
| R527-4 | Generated candidates must distinguish grammatical questions from fragments and ungrammatical sequences. | Implemented by `QuestionGrammarClass::{Fragment, Grammatical, Ungrammatical}` on every `GeneratedQuestion`. |
| R527-5 | Grammatical questions must be split into logically meaningful versus open-slot candidates. | Implemented by `LogicalMeaningClass` and the combined `GeneratedQuestionClass`, including `GrammaticalAndMeaningful`. |
| R527-6 | What counts as a question must be configurable. | Implemented by `QuestionAcceptance::{AnyQuestionLike, Grammatical, GrammaticalAndMeaningful}`. |
| R527-7 | Generated questions must be answerable through the existing Formal AI answer path. | Implemented by `generated_question_answers`, which delegates every accepted question to `FormalAiEngine::answer` and preserves normal trace evidence. |
| R527-8 | Popular question demand, word-frequency resources, grammar frameworks, and question/answer generation prior art must be researched before selecting the first implementation slice. | Implemented by `docs/case-studies/issue-527/raw-data/online-research.md`, covering Exploding Topics, Ahrefs, Wordfreq, Universal Dependencies, question generation, and answer generation sources. |
| R527-9 | The task must be executed by driving Formal AI through its own Agent CLI, not by hand-editing files. | Implemented by the `question_catalog` agentic recipe (`src/agentic_coding/question_catalog.rs`), which the deterministic planner walks `write_file → run_command → final`; `data/meta/question-catalog.lino` is byte-for-byte what the driver writes, asserted by `tests/unit/issue_527_question_catalog.rs`. |
| R527-10 | The generated questions must be enumerated, classified, and answered smallest-first in one reviewable catalog. | Implemented by `QuestionCatalog`, which records the smallest-first four-way classification and answers the grammatical-and-meaningful questions with the deterministic engine, rendered as Links Notation. |
| R527-11 | Answered questions must feed auto-learning without silently changing solver behaviour. | Implemented by `QuestionCatalog::answer_for`, a case/whitespace-insensitive recall table over the answered questions that never mutates the human-gated learning ledger. |
| R527-12 | The whole agentic loop must be reproducible. | Implemented by pinning `run_agentic_task(QUESTION_CATALOG_TASK)` byte-for-byte against `docs/case-studies/issue-527/agent-cli-session-question-catalog.json` in `tests/unit/issue_527_question_catalog.rs`. |
