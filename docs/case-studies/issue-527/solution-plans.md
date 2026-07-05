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

## Follow-Up Scaling

The current PR establishes the executable contract. Larger follow-ups can add:

- imported high-frequency word lists per supported language;
- popular question-pattern seeds from search-query datasets;
- template sources from knowledge-base facts and repository-local documents;
- stronger grammaticality checks through dependency parsing;
- logical-meaning checks against Formal AI's concept and source-cache records.
