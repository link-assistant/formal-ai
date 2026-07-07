# Issue 527 Online Research

Research collected on 2026-07-05 for
<https://github.com/link-assistant/formal-ai/issues/527>.

## Popular Question Demand

- Exploding Topics, ["100 Most Asked Questions on Google (February 2026)"](https://explodingtopics.com/blog/top-google-questions), says it analyzed over 4 billion search terms and lists high-volume question patterns such as "what is today", "how to change a tire", "how to find ip address", "when is easter", and "what is AI". Its source note says the search-volume data comes from Semrush.
- Ahrefs, ["100 Most Asked Questions on Google (July 2026)"](https://ahrefs.com/blog/top-google-questions/), says it analyzed a 28.7-billion-keyword database and published current U.S. and global high-volume question lists. The top U.S. entries are dominated by `what`, `how many`, `how to`, and `when` forms.

Design implication: issue #527 should not start with arbitrary random strings.
It should order candidates by short question length and high-frequency question
openers, then let popular real-world question patterns become later seed or
ranking data.

## Word Frequency Sources

- ConceptNet's Wordfreq overview, ["wordfreq 1.5: More data, more languages, more accuracy"](https://blog.conceptnet.io/wordfreq/), describes a multilingual word-frequency dataset/library that combines multiple sources rather than a single corpus. Sources named there include Wikipedia, Google Books, Reddit, Twitter, SUBTLEX, OpenSubtitles, the Leeds Internet Corpus, and Common Crawl. The note also explains why a median-style merge is less fragile than a plain mean when one source has outliers.

Design implication: the issue's "average from multiple known corpuses" should be
represented as a ranked word record with multiple source scores. The first
implementation uses `QuestionWord::from_corpus_scores` to make this explicit
without adding a large external corpus dependency.

## Grammar and Meaning Classification

- [Universal Dependencies](https://universaldependencies.org/) defines a cross-lingual framework for grammar annotation, including parts of speech, morphological features, and dependency relations. The project describes itself as a community effort with hundreds of contributors, more than 200 treebanks, and more than 150 languages; its site also links release 2.18 from 2026-05-15.

Design implication: a production-scale version should use a real dependency
parser or UD-style annotations. The current implementation keeps a lightweight
deterministic classifier for the first English slice and exposes the categories
(`Fragment`, `Grammatical`, `Ungrammatical`, `Meaningful`, `OpenSlot`) so a
future parser can replace the internals without changing the public contract.

## Question Generation

- Guo et al., ["A Survey on Neural Question Generation: Methods, Applications, and Prospects"](https://www.ijcai.org/proceedings/2024/0889.pdf), frames question generation as producing questions from sources such as knowledge bases, natural-language text, and images. The survey groups neural question-generation work into structured, unstructured, and hybrid input settings and lists applications including QA data augmentation, tutoring, conversation, and fact verification.
- Liu et al., ["Capturing Greater Context for Question Generation"](https://cdn.aaai.org/ojs/6440/6440-13-9665-1-10-20200517.pdf), notes that question generation has historically used both rule/template methods and learning-based methods, and evaluates generation over SQuAD, MS MARCO, and NewsQA.

Design implication: Formal AI's first slice should be a deterministic symbolic
question enumerator, not a neural generator. It can later ingest structured
knowledge-base facts or text-derived templates as additional ranked word and
template sources.

## Answer Generation

- Balaji et al., ["A comprehensive survey on answer generation methods using NLP"](https://www.sciencedirect.com/science/article/pii/S2949719124000360), describes question-answering systems in terms of question analysis, answer extraction, passage retrieval, and answer-generation strategy classes such as extraction-based, retrieval-based, opinion-based, and generative methods.

Design implication: the first implementation should reuse Formal AI's existing
answer pipeline instead of creating a parallel answer generator. The new
`generated_question_answers` stream delegates each generated question to
`FormalAiEngine`, preserving the standard symbolic answer and trace evidence.
