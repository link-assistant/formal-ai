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

## Scope Boundary

The implementation does not claim to enumerate every natural-language question
from every language immediately. It provides the bounded, lazy, configurable
core needed to do that work safely: consumers can plug in larger vocabularies,
future template sources, and stronger grammar/meaning classifiers while keeping
the iterator and answer-stream contract stable.
