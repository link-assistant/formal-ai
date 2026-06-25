# Manual summaries for the sampled repository files

Sample source: `raw-data/random-files-sampled.txt`.

## `docs/case-studies/issue-140/raw-data/issue-140.json`

One-line GitHub issue snapshot for issue #140. It stores the issue author,
title, state, body, labels, creation time, and one maintainer comment. The body
captures a Russian-language demo issue report where large chats made the
prefilled GitHub issue URL too long; the comment specifies the desired
shortening and case-study workflow.

Expected generalized summary behavior: detect JSON, report file size/line
count, extract top-level keys such as `author`, `body`, `comments`, `createdAt`,
`labels`, `number`, `state`, and `title`, and preserve the highest-signal prose
when it is later passed through statement summarization.

## `data/cache/wordnet/en/identity.json`

WordNet cache entry for the English lemma `identity`. It records source and
license metadata plus four noun senses covering personal identity, recognition
characteristics, the mathematical identity operator, and exact sameness.

Expected generalized summary behavior: detect JSON, report file size/line
count, extract top-level keys such as `lemma`, `language`, `source`, `license`,
and `senses`, and avoid requiring a bespoke WordNet-specific parser before the
generic structured-file summary is useful.
