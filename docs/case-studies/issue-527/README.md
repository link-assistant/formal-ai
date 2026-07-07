# Issue 527 Case Study

Status: **delivered and driven by the Agent CLI + Formal AI**, not deferred.
Issue [#527](https://github.com/link-assistant/formal-ai/issues/527) asks for a
system that can generate possible questions from smallest to largest, rank the
word space by frequency, distinguish grammatical from meaningful questions, make
the acceptance criteria configurable, and answer the generated questions.

The maintainer's linchpin instruction is to *fully implement the vision through
generalization of the meta algorithm and to execute the task by driving Formal
AI through its own Agent CLI* — not by hand-editing files. That is how this
change is produced: the eleventh agentic recipe
(`src/agentic_coding/question_catalog.rs`) enumerates, classifies and answers the
questions, and the committed catalog is reproduced **byte-for-byte** by that
driver under test.

## Source material

- GitHub issue: <https://github.com/link-assistant/formal-ai/issues/527>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/638>
- Raw GitHub data: [`raw-data/`](raw-data/)
- Requirements decomposition: [`requirements.md`](requirements.md)
- Per-requirement solution plan: [`solution-plans.md`](solution-plans.md)
- Online research notes: [`raw-data/online-research.md`](raw-data/online-research.md)
- Committed Agent CLI session that produced the catalog:
  [`agent-cli-session-question-catalog.json`](agent-cli-session-question-catalog.json)
- Captured end-to-end Agent CLI run:
  [`agent-cli-e2e-run.log`](agent-cli-e2e-run.log)
- Generated, byte-for-byte-pinned catalog artifact:
  [`../../../data/meta/question-catalog.lino`](../../../data/meta/question-catalog.lino)
- Frequency-tier seed vocabulary that grounds the generator:
  [`../../../data/seed/question-generation-lexicon.lino`](../../../data/seed/question-generation-lexicon.lino)

## 1. Collected Data

- Issue snapshot: `raw-data/issue-527.json`.
- Issue comments: `raw-data/issue-527-comments.json` (empty at collection time).
- Prepared PR snapshot: `raw-data/pr-638.json`.
- PR conversation, review-comment, and review snapshots:
  `raw-data/pr-638-comments.json`, `raw-data/pr-638-review-comments.json`, and
  `raw-data/pr-638-reviews.json`.
- Online prior art and source notes: `raw-data/online-research.md`.
- The committed Agent CLI session
  (`agent-cli-session-question-catalog.json`) and its captured transcript
  (`agent-cli-e2e-run.log`) record the actual tool loop that produced the
  catalog, so the evidence lives beside the code.

No issue screenshots were present, so there were no image attachments to
download or verify.

## 2. Requirements

| ID | Requirement | Implementation |
| --- | --- | --- |
| R527-1 | Preserve research, requirements, raw data, and solution planning under this case-study directory. | This README plus `requirements.md`, `solution-plans.md`, and `raw-data/` keep the evidence together. |
| R527-2 | Generate questions from one word to two words and onward as a lazy sequence. | `QuestionGenerator` is an iterator that advances by word count and never materializes the full sequence. |
| R527-3 | Use frequency-ranked words, with the issue's 10%, 5%, 2.5%, ... policy available. | `QuestionGenerationConfig` ranks `QuestionWord` values and applies the issue #527 percentage policy by default; the ranking is grounded in `data/seed/question-generation-lexicon.lino`. |
| R527-4 | Distinguish grammatical, fragmentary, and ungrammatical candidates. | `GeneratedQuestion.grammar` stores `QuestionGrammarClass`. |
| R527-5 | Split grammatical questions into logically meaningful versus open-slot candidates. | `GeneratedQuestion.logical_meaning` and `GeneratedQuestion.class` store the semantic classification. |
| R527-6 | Make the question criteria configurable. | `QuestionAcceptance` selects any question-like, grammatical, or grammatical-and-meaningful candidates. |
| R527-7 | Answer generated questions through the existing Formal AI engine. | `generated_question_answers` delegates every question to `FormalAiEngine::answer`. |
| R527-8 | Research popular question demand, word-frequency data, grammar frameworks, and question/answer generation. | `raw-data/online-research.md` records the online research and design implications. |
| R527-9 | Execute the task by driving Formal AI through its own Agent CLI, not by hand-editing files. | The `question_catalog` recipe walks `write_file → run_command (cat verify) → final`; `data/meta/question-catalog.lino` is byte-for-byte what the driver writes, asserted under test. |
| R527-10 | Enumerate, classify and answer smallest-first in one reviewable artifact. | `QuestionCatalog` records the smallest-first four-way classification and answers the grammatical-and-meaningful questions with the deterministic engine, rendered as Links Notation. |
| R527-11 | Wire answered questions into auto-learning without changing solver behaviour behind a human's back. | `QuestionCatalog::answer_for` is a case/whitespace-insensitive recall table over the answered questions; it never mutates the human-gated learning ledger, so recall never silently changes what the solver does. |
| R527-12 | Keep the whole loop reproducible. | `run_agentic_task(QUESTION_CATALOG_TASK)` regenerates the committed Agent CLI session byte-for-byte, pinned by `tests/unit/issue_527_question_catalog.rs`. |

## 3. Root Cause

Before this PR, Formal AI could answer a prompt that a user supplied, but there
was no first-class representation for the inverse problem: systematically
constructing prompts/questions for the engine to answer, then *driving the Agent
CLI to do it*. The project also had no configured boundary for the key policy
questions raised in the issue:

- which vocabulary slice is in scope for a given word count;
- whether fragments such as `what?` should be kept or filtered;
- whether a grammatical but incomplete question should be treated differently
  from a logically meaningful one;
- how generated questions should be routed back through the normal solver; and
- how the answered questions become durable, reviewable records.

The missing abstraction was a lazy candidate stream with classification
metadata, plus an agentic recipe that turns the stream into a committed,
answered catalog through the Agent CLI.

## 4. Implemented Design

`src/question_generation.rs` provides the issue #527 core surface:

- `QuestionWord` records a word surface plus averaged frequency evidence from
  one or more corpus scores.
- `QuestionGenerationConfig` ranks those words, applies the default issue #527
  frequency-tier policy (top 10% at one/two words, 5% at three, 2.5% at four,
  halving thereafter with a practical floor), and exposes
  `with_all_ranked_words()` for exhaustive local experiments. The ranking and
  grammar roles are grounded in the committed seed lexicon
  `data/seed/question-generation-lexicon.lino`, not hardcoded in production code.
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

`src/agentic_coding/question_catalog.rs` is the eleventh agentic recipe. It is
reachable through the Agent CLI exactly like its siblings (formalize, meaning,
diagram, self-AST, ledger, repair-strategy, …):

- `is_question_catalog_task` routes a *differently worded* request to the recipe
  from the words alone, without colliding with the sibling routers.
- the deterministic planner walks it `write_file → run_command → final`; there is
  no web step because the catalog is a pure function of the seed lexicon and the
  deterministic engine.
- `render_document` renders the smallest-first classification and the answered
  questions as Links Notation, committed byte-for-byte to
  `data/meta/question-catalog.lino`.
- `QuestionCatalog::answer_for` is the auto-learning link: a case/whitespace-
  insensitive recall table over the answered questions. It is deliberately a
  *recall* table — it never mutates the human-gated learning ledger, so it can
  never change solver behaviour on its own. A question the catalog never
  generated is not recalled (no hallucinated answers).

The first grammar/meaning classifier is intentionally deterministic and small.
It recognizes English question openers and auxiliaries, rejects repeated or
out-of-order tiny-vocabulary tuples in the strict mode, and marks one-word
question openers as fragments/open slots. The public labels are designed so a
future dependency-parser-backed classifier can replace the heuristics without
changing caller code — the recipe, the answer stream, and the recall table all
keep their contracts.

## 5. Prior Art And Existing Components

| Component | Relevance | Decision |
| --- | --- | --- |
| Existing `FormalAiEngine` and universal solver | Already answer prompts with trace evidence. | Reused through `generated_question_answers` and the recipe's answered catalog; no parallel answer generator was added. |
| The agentic-recipe pattern (`src/agentic_coding/`) | Every capability is exercised by driving the Agent CLI, with a byte-for-byte committed artifact. | The question catalog is added as the eleventh recipe, mirroring the ledger/repair-strategy/self-AST recipes exactly. |
| The human-gated learning ledger (issue #558) | The only path allowed to change solver behaviour. | Left untouched; `answer_for` is a pure recall table, preserving the invariant that only the human-gated path mutates behaviour. |
| Wordfreq-style multi-source frequency data | Matches the issue's "average from multiple known corpuses" requirement. | Represented by `QuestionWord::from_corpus_scores` and grounded in the committed seed lexicon without importing a large corpus. |
| Popular search-question lists from Exploding Topics and Ahrefs | Show real high-demand question shapes. | Used as ranking/roadmap evidence; the smallest-first tiered vocabulary optimizes for the most-frequent openers first. |
| Universal Dependencies | Provides the long-term grammar-classification direction. | Kept as prior art; the first classifier is local and deterministic, with labels shaped for a future parser. |
| Question-generation and answer-generation surveys | Frame QG/QA as broad research areas with structured, unstructured, hybrid, retrieval, extraction, and generative methods. | Used to keep this PR focused on the symbolic lazy core rather than a neural generator (neural inference is a project NON-GOAL). |

## 6. Verification

Reproducing test added before implementation:

```sh
cargo test --test unit issue_527 -- --nocapture
```

Before the implementation, that test failed to compile because
`QuestionGenerator`, `QuestionGenerationConfig`, `QuestionWord`,
`QuestionAcceptance`, the classification enums, and
`generated_question_answers` were not exported.

After the implementation, the focused tests cover:

- lazy word-count ordering, one-word fragment classification, grammatical and
  logically meaningful classification, strict filtering through
  `QuestionAcceptance::GrammaticalAndMeaningful`, and answer-stream delegation to
  `FormalAiEngine` (`tests/unit/issue_527.rs`);
- the agentic capability (`tests/unit/issue_527_question_catalog.rs`): the
  smallest-first four-way classification, the answered questions forming a recall
  table, the committed `data/meta/question-catalog.lino` being byte-for-byte what
  the recipe renders and valid Links Notation, the recipe routing without
  colliding with the sibling recipes, the planner walking it `write → verify →
  final`, the in-repo Agent CLI driver driving it to the expected write, and the
  committed Agent CLI session
  (`agent-cli-session-question-catalog.json`) matching a fresh driven run
  byte-for-byte.

The documentation contract is covered by
`tests/unit/docs_requirements_issue_527.rs`.

To reproduce the Agent CLI session locally:

```sh
formal-ai agent \
  --task "$QUESTION_CATALOG_TASK" \
  --session-json docs/case-studies/issue-527/agent-cli-session-question-catalog.json
```

where `$QUESTION_CATALOG_TASK` is the recipe's own task wording
(`formal_ai::agentic_coding::QUESTION_CATALOG_TASK`).
