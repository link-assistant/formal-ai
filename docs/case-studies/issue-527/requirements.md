# Issue 527 Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R527-1 | Preserve issue data, PR data, online research, requirements, and solution planning under `docs/case-studies/issue-527`. | This case-study directory stores raw GitHub snapshots, `online-research.md`, this requirement table, and `solution-plans.md`. |
| R527-2 | Generate a lazy infinite iterator of candidate questions ordered from smallest to largest word count. | `QuestionGenerator` starts at one-word candidates, advances through all accepted candidates at that length, then moves to two words, three words, and so on without precomputing the sequence. |
| R527-3 | Rank the candidate vocabulary by frequency evidence from multiple corpus scores and support the issue's 10%, 5%, 2.5%, ... vocabulary-tier policy. | `QuestionWord::from_corpus_scores` records average frequency evidence; `QuestionGenerationConfig` defaults to the issue #527 percentage policy and exposes `with_all_ranked_words` for exhaustive local tests or experiments. |
| R527-4 | Distinguish grammatically correct questions from fragments and ungrammatical sequences. | Each `GeneratedQuestion` carries `QuestionGrammarClass::{Fragment, Grammatical, Ungrammatical}`. |
| R527-5 | Split grammatically correct questions into logically meaningful versus open-slot/non-meaningful cases. | Each `GeneratedQuestion` carries `LogicalMeaningClass::{Meaningful, OpenSlot, NotMeaningful}` and a combined `GeneratedQuestionClass`. |
| R527-6 | Make "what counts as a question" configurable instead of hardcoding one acceptance rule. | `QuestionAcceptance` lets callers request any question-like candidate, grammatical candidates, or only grammatical and logically meaningful candidates. |
| R527-7 | Provide a way to answer generated questions with the best existing Formal AI answer path. | `generated_question_answers(config)` yields `GeneratedQuestionAnswer` values by delegating each question to `FormalAiEngine::answer`. |
| R527-8 | Study popular question datasets, frequency resources, grammar frameworks, and question/answer generation prior art before choosing the first implementation slice. | `raw-data/online-research.md` records current top-question sources, Wordfreq, Universal Dependencies, question generation surveys, and answer generation survey notes. |
| R527-9 | Execute the task by driving Formal AI through its own Agent CLI, not by hand-editing files. | The `question_catalog` recipe (`src/agentic_coding/question_catalog.rs`) walks `write_file → run_command (cat verify) → final`; `data/meta/question-catalog.lino` is byte-for-byte what the driver writes, asserted under test. |
| R527-10 | Enumerate, classify, and answer smallest-first in one reviewable catalog. | `QuestionCatalog` records the smallest-first four-way classification and answers the grammatical-and-meaningful questions with the deterministic engine, rendered as Links Notation. |
| R527-11 | Wire answered questions into auto-learning without changing solver behaviour behind a human's back. | `QuestionCatalog::answer_for` is a case/whitespace-insensitive recall table over the answered questions; it never mutates the human-gated learning ledger. |
| R527-12 | Keep the whole loop reproducible. | `run_agentic_task(QUESTION_CATALOG_TASK)` regenerates the committed Agent CLI session byte-for-byte, pinned by `tests/unit/issue_527_question_catalog.rs`. |

## Generality Boundary

The vocabulary, template sources, and grammar/meaning classifiers are pluggable
by design, not deferred: the generator ranks whatever `QuestionWord` evidence it
is given, the recipe renders whatever the generator produces, and the labels are
shaped so a Universal-Dependencies-style parser can replace the heuristics
without changing any caller — the iterator, the answer stream, the agentic
recipe, and the recall table all keep their contracts. Larger corpora and
stronger classifiers extend the *data and the classifier*, never the general
routing logic.
