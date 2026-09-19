# Know how to get to know anything when it is needed

Source: the architect's 2026-09-14 instruction for the work continued in
[issue #1138](https://github.com/link-assistant/formal-ai/issues/1138), describing
the intended generalization of pull request #888. The complete verbatim
instruction is quoted in
[`docs/case-studies/issue-710/plans/README.md`](../../case-studies/issue-710/plans/README.md);
the continuation work it spawned on 2026-09-15 is recorded as requirements
(the R710-R rows), not as further architect words.

> We need to have dynamic discovery algorithms, that use primarely trusted
> sources. So the goal is not to know everything in advance, the goal to know
> how to get know anything when it is needed.

> The data seed is not there to memoize everything, it it should have enough
> data to speed up tests and have some general knowledge just enough to be able
> to solve any task on demand. So memory may grow to cache discovered knowledge,
> yet we never expect for our system to know everything in advance, as it is
> impossible.

> Also if our system knows it has algorithm for rediscovery it can decide to
> free memory space if constrained, yet all the data cannot be exactly loaded
> from internet (like chat history and our own experience, that should be kept
> by default in the memory, until user decides to delete it modify).
