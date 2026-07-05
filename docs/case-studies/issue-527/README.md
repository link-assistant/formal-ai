# Issue 527 Case Study

Issue [#527](https://github.com/link-assistant/formal-ai/issues/527) asks for a
system that can generate possible questions from shortest to longest, rank the
word space by frequency, distinguish grammatical and meaningful questions, make
the acceptance criteria configurable, and answer the generated questions.

## 1. Collected Data

- Issue snapshot: `raw-data/issue-527.json`.
- Issue comments: `raw-data/issue-527-comments.json` (empty at collection time).
- Prepared PR snapshot: `raw-data/pr-638.json`.
- PR conversation, review-comment, and review snapshots:
  `raw-data/pr-638-comments.json`, `raw-data/pr-638-review-comments.json`, and
  `raw-data/pr-638-reviews.json` (all empty at collection time).
- Online prior art and source notes: `raw-data/online-research.md`.

No issue screenshots were present, so there were no image attachments to
download or verify.

## 2. Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R527-1 | Preserve research, requirements, raw data, and solution planning under this case-study directory. | This README plus `requirements.md`, `solution-plans.md`, and `raw-data/` keep the evidence together. |
| R527-2 | Generate questions from one word to two words and onward as a lazy sequence. | `QuestionGenerator` is an iterator that advances by word count and never materializes the full sequence. |
| R527-3 | Use frequency-ranked words, with the issue's 10%, 5%, 2.5%, ... policy available. | `QuestionGenerationConfig` ranks `QuestionWord` values and applies the issue #527 percentage policy by default. |
| R527-4 | Distinguish grammatical, fragmentary, and ungrammatical candidates. | `GeneratedQuestion.grammar` stores `QuestionGrammarClass`. |
| R527-5 | Split grammatical questions into logically meaningful versus open-slot candidates. | `GeneratedQuestion.logical_meaning` and `GeneratedQuestion.class` store the semantic classification. |
| R527-6 | Make the question criteria configurable. | `QuestionAcceptance` selects any question-like, grammatical, or grammatical-and-meaningful candidates. |
| R527-7 | Answer generated questions through the existing Formal AI engine. | `generated_question_answers` delegates every question to `FormalAiEngine::answer`. |
| R527-8 | Research popular question demand, word-frequency data, grammar frameworks, and question/answer generation. | `raw-data/online-research.md` records the online research and design implications. |

## 3. Root Cause

Before this PR, Formal AI could answer a prompt that a user supplied, but there
was no first-class representation for the inverse problem: systematically
constructing prompts/questions for the engine to answer. The project also had no
configured boundary for the key policy questions raised in the issue:

- which vocabulary slice is in scope for a given word count;
- whether fragments such as `what?` should be kept or filtered;
- whether a grammatical but incomplete question should be treated differently
  from a logically meaningful one; and
- how generated questions should be routed back through the normal solver.

The missing abstraction was a lazy candidate stream with classification metadata.
Without that, the only alternatives were either a finite hardcoded list of
questions or an impossible attempt to build the infinite set in memory.

## 4. Implemented Design

`src/question_generation.rs` adds the public issue #527 surface:

- `QuestionWord` records a word surface plus averaged frequency evidence from
  one or more corpus scores.
- `QuestionGenerationConfig` ranks those words, applies the default issue #527
  frequency-tier policy, and exposes `with_all_ranked_words()` for exhaustive
  local experiments.
- `QuestionGenerator` implements `Iterator<Item = GeneratedQuestion>`, ordered
  by word count.
- `GeneratedQuestion` includes the text, word list, word count,
  `QuestionGrammarClass`, `LogicalMeaningClass`, and combined
  `GeneratedQuestionClass`.
- `QuestionAcceptance` controls whether callers accept any question-like
  candidate, only grammatical candidates, or only grammatical and logically
  meaningful candidates.
- `generated_question_answers(config)` composes the generator with
  `FormalAiEngine::answer`, returning `GeneratedQuestionAnswer` records with the
  normal symbolic answer and trace evidence.

The first grammar/meaning classifier is intentionally deterministic and small.
It recognizes English question openers and auxiliaries, rejects repeated or
out-of-order tiny-vocabulary tuples in the strict mode, and marks one-word
question openers as fragments/open slots. The public labels are designed so a
future dependency-parser-backed classifier can replace the heuristics without
changing caller code.

## 5. Prior Art And Existing Components

| Component | Relevance | Decision |
| --- | --- | --- |
| Existing `FormalAiEngine` and universal solver | Already answer prompts with trace evidence. | Reused through `generated_question_answers`; no parallel answer generator was added. |
| Wordfreq-style multi-source frequency data | Matches the issue's "average from multiple known corpuses" requirement. | Represented by `QuestionWord::from_corpus_scores` without importing a large corpus. |
| Popular search-question lists from Exploding Topics and Ahrefs | Show real high-demand question shapes. | Used as ranking/roadmap evidence; not vendored as data in this first slice. |
| Universal Dependencies | Provides the long-term grammar-classification direction. | Kept as prior art; the first classifier is local and deterministic. |
| Question-generation and answer-generation surveys | Frame QG/QA as broad research areas with structured, unstructured, hybrid, retrieval, extraction, and generative methods. | Used to keep this PR focused on the symbolic lazy core rather than a neural generator. |

## 6. Verification

Reproducing test added before implementation:

```sh
cargo test --test unit issue_527 -- --nocapture
```

Before the implementation, that test failed to compile because
`QuestionGenerator`, `QuestionGenerationConfig`, `QuestionWord`,
`QuestionAcceptance`, the classification enums, and
`generated_question_answers` were not exported.

After the implementation, the same focused test covers:

- lazy word-count ordering;
- one-word fragment classification;
- grammatical and logically meaningful classification;
- strict filtering through `QuestionAcceptance::GrammaticalAndMeaningful`; and
- answer-stream delegation to `FormalAiEngine`.

The documentation contract is covered by
`tests/unit/docs_requirements_issue_527.rs`.
