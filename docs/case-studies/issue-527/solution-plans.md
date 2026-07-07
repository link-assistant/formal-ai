# Issue 527 Solution Plans

## Lazy Question Stream

Represent "all possible questions" as an iterator, not a vector. The stream
keeps only the current word count and index tuple in memory:

1. Start at one-word question-like candidates.
2. Exhaust the current word count in ranked vocabulary order.
3. Increment the word count and reset the tuple.
4. Repeat indefinitely until the caller stops consuming.

This satisfies the issue's smallest-to-largest ordering without pretending the
infinite search space can be materialized.

## Frequency-Tier Vocabulary

Use `QuestionWord` records with multiple corpus scores so ranking can be built
from averaged or merged evidence. The default policy follows the issue's tier
shape:

- one and two words: top 10% of the ranked vocabulary;
- three words: top 5%;
- four words: top 2.5%;
- later lengths: keep halving the basis-point budget, with a small practical
  floor so a local seed remains productive.

Tests can opt into `with_all_ranked_words()` to make the exact stream easy to
verify with a tiny vocabulary.

## Classification Gates

Every emitted candidate carries both syntactic and semantic labels:

- grammar: fragment, grammatical, or ungrammatical;
- logical meaning: meaningful, open slot, or not meaningful;
- combined class: grammatical-and-meaningful, grammatical-open-slot, fragment,
  or ungrammatical.

`QuestionAcceptance` is the configuration surface for "what counts as a
question." The first implementation uses deterministic English heuristics; the
labels intentionally match the future slot where a Universal-Dependencies-style
parser can be inserted.

## Answer Stream

`generated_question_answers(config)` composes the lazy generator with the
existing engine:

1. Pull the next accepted `GeneratedQuestion`.
2. Pass `question.text` to `FormalAiEngine::answer`.
3. Return the generated question, symbolic answer, and standard trace evidence.

This keeps answering aligned with the universal solver instead of creating a
parallel QA path.

## Agentic Recipe (Drive The Agent CLI, Not A Human Editor)

The issue is executed by driving Formal AI through its own Agent CLI, exactly
like the other agentic recipes. `src/agentic_coding/question_catalog.rs` is the
eleventh recipe:

1. `is_question_catalog_task` routes a differently worded request to the recipe
   from the words alone, without colliding with the sibling routers.
2. the deterministic planner walks `write_file → run_command (cat verify) →
   final`; there is no web step because the catalog is a pure function of the
   seed lexicon and the deterministic engine.
3. `render_document` emits the smallest-first classification and the answered
   questions as Links Notation, committed byte-for-byte to
   `data/meta/question-catalog.lino`.
4. `QuestionCatalog::answer_for` is the auto-learning link — a
   case/whitespace-insensitive recall table over the answered questions that
   never mutates the human-gated learning ledger, so recall can never change
   solver behaviour on its own.

The committed `agent-cli-session-question-catalog.json` is byte-for-byte what a
fresh driven run produces, so the whole loop is reproducible and cannot silently
regress into hand-editing without turning a test red.

## Extending, Not Deferring

Larger corpora and stronger classifiers extend the *data and the classifier*,
never the general routing logic: import high-frequency word lists per language
into the seed lexicon, add popular question-pattern seeds from search-query
datasets, ground logical-meaning checks against Formal AI's concept and
source-cache records, and swap the deterministic grammar heuristics for a
dependency parser behind the existing `QuestionGrammarClass` /
`LogicalMeaningClass` labels. Every one of these is a data or classifier change
that the iterator, the answer stream, the recipe, and the recall table already
accommodate without modification.
