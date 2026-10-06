## Standing Doctrine: Work Is Delegated to Formal AI Itself (2026-10-07)

Stated by the project owner while continuing PR #1188: "keep delegating to
Formal AI itself some work, so at least it will be able to do smallest
possible tasks just fine, by generalization and discovery with caching, not by
hardcoding. As human programmer would traditionally do if learning something
new." It extends the task ladder of the recursive meta algorithm
([R1009](doctrine-standing-doctrine-recursive-meta-algorithm-2026-10-06.md)).

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1017 | Development work keeps being put to Formal AI itself: the small real tasks met while building the system (scripts over the repository's own files, text edits, list and word programs) are asked of `scripts/formal-ai-js.mjs` first. A task it cannot derive becomes a ladder rung, and it is made to work only by a general mechanism — a seeded cue, affix or documented operation, a gloss-grounding rule, a cached dictionary capture — never by a handler or rule written for that request. | In progress 2026-10-07: five tasks from this pull request's own work were put to it (a bare directory operand, a symbol named by a word, symbol replacement, duplicate removal plus sorting claimed by the `algorithm` handler, a superlative) and are being made to derive as ladder rungs in `rust/tests/fixtures/meta-reasoner/ladder.lino`. A sixth, "Reverse the order of words", failed while "reverse the words" worked; the English `reverse_words` row of `data/seed/operation-vocabulary.lino` gained the `reverse+words` token combo the other languages already carry, pinned by `rust/tests/web/r1017-delegated-tasks.test.mjs`. |
