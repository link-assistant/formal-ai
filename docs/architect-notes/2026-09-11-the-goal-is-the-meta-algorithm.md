# The goal is the meta algorithm, not a kernel

**Date:** 2026-09-11 · **Source:** direct statement to the assistant

> I did never asked to make Rust a shrinking kernel. That is completely wrong.
> What I asked links network meta language representation of Rust code, that is
> translatable by our own Formal AI system into any programming language, for
> example to JavaScript. We already have some logic about it in relative meta
> logic, we also have meta language itself, that should simplify it. For
> auto-learning system it is important to be able to make modification in meta
> language version of the code and recompile it back to Rust, JavaScript or any
> other language. We must also use meta language as a basis for coding, as it is
> much easier to parse the code into meta language, to transformations and write
> them back. We also may need to report issues to all our dependencies as we go.

> The goal of the project is to produce the meta algorithm - it is not the
> kernel or non-kernel. The entire src must be dedicated to that meta algorithm.
> It should be capable of producting algorithms. For example doublet links are
> capable of representing each and every data structure or sequence (as nested
> pairs), that means doublet-links structure is effectively natural
> deduplicator. Each algorithm at the moment of execution or learning is
> essentially flat sequence. That sequence can be recorded as doublet links, and
> if some operations are repeated they contain loops or recursion, if there are
> alternative branches based of input it is equalivalent to if statements or
> match statements. So if we just record sequence of events, actions,
> transformations, we can infer algorithms from them purely algorithmically. So
> if we do reasoning to just decide which next step is correct, or just read
> manual, by testing each and every step and have enough similar sequences that
> solve similar conceptual action or transformation we will invitably arrive to
> algorithm by just deduplicating it. Meta algorithm at the end must arrive to
> situation where it is capable to modify itself when asked or required by task.
> To do so we always need to have meta language respresention of it, which
> should be translatable to Rust by a function call and be an action that can be
> approved by user once or approvable automatically - by default not approved
> for safety without user permission, and only if user approves it second time -
> we can ask him to aprove it forever.

> Everything that contradicts that vision should be fixed. And we should start
> keep track of architect's (me) notes on the system progress. You can restore
> them from previous issues and GitHub comments, that were clearly written in
> the same style. We should not invent the language or terminalogy I'm not
> using.

## What this withdrew

The division of `src` into a privileged part and the rest, and any measure that
required Rust to shrink. It was never requested: it entered through issue #918's
acceptance criterion and became issue #1085 D1.4.
