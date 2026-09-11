# Architect's vision

The standing guideline for this repository. Read it before analysing, planning,
or concluding anything here. It records the architect's (konard's) own words.
Where a document, gate, requirement or plan contradicts this page, the document
is wrong and must be fixed.

Two rules about the page itself:

- **Do not invent terminology the architect does not use.** Restate the vision
  in his words, quoted, rather than in substituted vocabulary.
- **It is kept up to date from his own writing** — issues, pull-request comments
  and review notes. New statements are added here verbatim, with a reference.

## 1. The goal is the meta algorithm

> The goal of the project is to produce the meta algorithm - it is not the
> kernel or non-kernel. The entire src must be dedicated to that meta algorithm.
> It should be capable of producing algorithms.

There is no kernel/non-kernel division of the source. All of `src/` serves the
meta algorithm.

## 2. Code is held in the meta language, and emitted into languages

> We must have .lino files containing full representation of rust code, and we
> must have /src folder with Rust code all 1 to 1 on each pull request merged.

> What I asked links network meta language representation of Rust code, that is
> translatable by our own Formal AI system into any programming language, for
> example to JavaScript. [...] For auto-learning system it is important to be
> able to make modification in meta language version of the code and recompile
> it back to Rust, JavaScript or any other language. We must also use meta
> language as a basis for coding, as it is much easier to parse the code into
> meta language, to transformations and write them back.

Stated earlier in issue #558 ("Auto learning"):

> we need to make sure we can translate entire source code of our system to
> links/meta language, and it should be present in the seed data [...] and we
> should be able to translate that meta language representation back to the
> source code, recompile and reattach it to the UI.

**Consequence.** Rust is one emission target of that representation, not the
system. A count of Rust lines measures an output. A more general meta algorithm
may emit *more* Rust. No release condition may be a Rust line count.

> We also may need to report issues to all our dependencies as we go.

## 3. Self-modification is an action the user approves

> To do so we always need to have meta language representation of it, which
> should be translatable to Rust by a function call and be an action that can be
> approved by user once or approvable automatically - by default not approved
> for safety without user permission, and only if user approves it second time -
> we can ask him to approve it forever.

Three states, in order: not approved by default; approved once; and only after a
second approval may approve-forever be offered.

## 4. Algorithms are arrived at by deduplication

> doublet links are capable of representing each and every data structure or
> sequence (as nested pairs), that means doublet-links structure is effectively
> natural deduplicator. Each algorithm at the moment of execution or learning is
> essentially flat sequence. That sequence can be recorded as doublet links, and
> if some operations are repeated they contain loops or recursion, if there are
> alternative branches based of input it is equivalent to if statements or match
> statements. So if we just record sequence of events, actions, transformations,
> we can infer algorithms from them purely algorithmically.

> if we do reasoning to just decide which next step is correct, or just read
> manual, by testing each and every step and have enough similar sequences that
> solve similar conceptual action or transformation we will inevitably arrive to
> algorithm by just deduplicating it.

From issue #531:

> Инвариант это вид повторения, как и любая закономерность, нужно повторы
> искать. Без дедупликации тут ничего и никогда не сделать. Потому что говорят
> что интеллект это и есть алгоритм сжатия.

> Сам по себе повтор это ситуация, строго определённая в контексте связей, это
> когда множество связей используют одну и ту же связь.

And the end state, from the same architect statement as §3:

> Meta algorithm at the end must arrive to situation where it is capable to
> modify itself when asked or required by task.

## 5. Nothing is a hard task

> nothing is hard task, forget any rating/judgement/assessment, you have no
> right to do any evaluation of the task I gave you. We have universal problem
> solving algorithm, if that task is big, we can split it in 2 halves and it
> will be much easier to deal with them. Complex tasks are composed from simple
> tasks. So everything complex, is essentially recursively simple in our
> philosophy.

Do not call a task hard, complex, ambitious or difficult, and do not decline one
on those grounds. Split it in two, split the halves, and continue until each
leaf is directly solvable. Splitting is itself a task (issue #847), and the
descent is already the stated shape of the meta algorithm (`src/meta_core.rs`,
R19: decompose until each leaf is directly solvable).

## 6. Requirements must not obstruct progression to the vision

> Everything that is in the way especially by made up reasons must be
> eliminated, we must keep my vision, but we should not place unreasonable
> requirements for progression to the vision. We need to find a balance - we
> should force each pull request to use Formal AI to code part of it (as big as
> it can be, but as small as it actually can, if that will take more than 1
> actual time of coding). So if pull request exists for for more than a day we
> can relax our requirements, and still be able to produce the release.

The practical aim this serves:

> it is critical to be able to produce formal-ai releases for testing in Hive
> Mind on real GitHub issues.

## 7. Start from a working Hello World

> We must provide Formal AI with all capability necessary to start with a simple
> hello world application in top 10-20 languages. If we don't do that, the
> result will be, that we will never be able to actually start iterating, and it
> will never truly work.

> instead of creating complete repositories for our formal-ai tests, we may use
> separate branches with unique names, so we can execute our GitHub action on
> the task generated as sub issue or just an issue in the Formal AI.

> we must ensure that when Formal AI will be used in Hive Mind it will actually
> produce changes in the pull request given the task from test repositories.

## What this page withdrew

`non_kernel_rust_lines` and the kernel/non-kernel split (issue #1085 D1.4,
inherited from #918's acceptance criterion) contradict §1 and §2 and are
withdrawn. They were never a release condition in fact — `release.yml` cuts on
CI correctness alone since #1085 D3.5, and `self-development-status.yml` only
reports — so no release depended on them.
