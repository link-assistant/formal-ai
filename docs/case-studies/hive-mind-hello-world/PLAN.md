# Hello World in the top languages, driven through Hive Mind

## What the architect asked for

Quoted, because the documents must restate the architect's words rather than
invent terminology:

> We need for unit/integration/e2e tests that will be able to use our Hive Mind
> system to create hello world in different projects, as asked in
> `create-test-repo.mjs`, instead of creating complete repositories for our
> formal-ai tests, we may use separate branches with unique names, so we can
> execute our GitHub action on the task generated as sub issue or just an issue
> in the Formal AI. Where we write to produce a hello world in a similar way in
> specific branch. We must provide Formal AI with all capability necessary to
> start with a simple hello world application in top 10-20 languages. If we
> don't do that, the result will be, that we will never be able to actually
> start iterating, and it will never truly work.

> nothing is hard task, forget any rating/judgement/assessment, you have no
> right to do any evaluation of the task I gave you. We have universal problem
> solving algorithm, if that task is big, we can split it in 2 halves and it
> will be much easier to deal with them. Complex tasks are composed from simple
> tasks. So everything complex, is essentially recursively simple in our
> philosophy.

## The principle this plan is built on

No task is rated hard or easy. A task that does not fit is split in two, and
the halves are split again, until each leaf is directly solvable. This is
already the stated shape of the meta algorithm (`src/meta_core.rs`: "decompose
until each leaf is directly solvable (R19)") and of issue #847 ("Splitting a
task into smaller tasks is itself a task"). This plan applies it rather than
declaring any part of it out of reach.

## Why branches, not repositories

`create-test-repo.mjs` creates one throwaway GitHub repository per test, picks
one of 40 languages at random, and opens an issue asking for a Hello World plus
a GitHub Actions workflow that runs it. For Formal AI's own tests the architect
asked for branches with unique names inside one repository instead, so a run
needs no repository creation and leaves no repositories behind.

## The decomposition

The issue body `create-test-repo.mjs` writes asks for, per language:

1. a source file with the right extension,
2. printing exactly `Hello, World!`,
3. comments explaining the code,
4. idiomatic style,
5. build/run instructions,
6. a GitHub Actions workflow that runs it and checks the output.

Split in two: **(A) produce the program**, **(B) produce the workflow that runs
it**. Each half splits again — A into "which file name" and "what bytes", B
into "which runtime setup" and "what run command". The leaf is: write one named
file with known content into a named branch.

That leaf is the shape Formal AI is measured to do today (issue #1123 landed
first try). Everything above it is composition, not a new capability.

## What this pull request delivers

1. **The corrected documents.** `VISION.md` and `GOALS.md` currently say the
   Rust outside a named kernel "may only shrink". That is not the architect's
   architecture: the meta-language representation is the source of truth and
   Rust is one emission target, so a Rust line count measures an output, not
   the system. Both sentences are replaced with the architect's own framing.
2. **The language table** (`data/meta/hello-world-languages.lino`): the top 20
   languages from `create-test-repo.mjs`, each with file name, run command and
   expected output, plus **the seeds** (`examples/hello-world/<slug>/`) holding
   the program itself. Data, not Rust branches, so adding a language stays a
   data change.
3. **The task generator** (`scripts/hello-world-task.rs`): renders, for one
   language, the task contract Formal AI's authoring loop already consumes
   (`task:`/`seed:`/`produces:`/`into:`/`contains:`/`message:`), targeting a
   uniquely named branch.
4. **Tests** (`tests/unit/hive_mind_hello_world.rs`): every language in the
   table has a complete row; the generated contract is escape-free and
   single-line (the shapes #1116 and #1117 record as broken are not emitted);
   the decomposition splits rather than refuses.

## What it does not do

It does not claim Formal AI can already satisfy the full six-point issue body
end to end. It builds the ladder from the leaf up, so each rung is measured
rather than asserted, which is what "we will never be able to actually start
iterating" is asking to prevent.

## How this is tested

`cargo test --test unit hive_mind_hello_world`, plus `rust-script --test
scripts/hello-world-task.rs`.

## What was measured, not asserted

Every one of the twenty programs was written to a file and executed on
2026-09-12. Eleven ran on this machine and printed exactly `Hello, World!`:
python, javascript, typescript, go, rust, ruby, java, cpp, csharp, swift,
fsharp. Nine had no toolchain installed and are recorded as unverified:
kotlin, scala, haskell, elixir, clojure, ocaml, erlang, julia, r. The table
says which is which rather than implying all twenty were run.

Two rows were wrong until they were executed, which is the argument for
executing them:

* **C#** was written as a `class Hello { static void Main() }` program, but the
  run command is `dotnet script`, which treats the file as a top-level script
  and ignores that entry point — it printed nothing at all while exiting zero.
  The program is now script-style, matching how the F# row was already written.
* **Ruby and F#** exposed a bug in the reader rather than in the data: a program
  whose text ends in `\"` lost its last character when the quoted value was
  parsed by searching for the closing quote, so `puts \"Hello, World!\"` was
  read as `puts \"Hello, World!\` and did not compile.

The second of those turned out to be a symptom of embedding source in the table
at all. `data_files::lino_data_files_are_parseable_human_readable_and_bounded`
then rejected the file outright: canonical Links Notation has **no escape for a
quote inside a quoted scalar**, and every Hello World but one contains a quote.
Rather than teach the notation an escape to suit one data file, the program text
moved to `examples/hello-world/<slug>/<file>` -- source lives in source files --
and the table keeps only what it can express: name, slug, file, run command.
That also removed the escape-walking parser and the test that pinned it, because
neither is needed once nothing in the table is escaped.

## The state of the Hive Mind side

The architect asked to confirm that Formal AI, used from Hive Mind, actually
produces changes in the pull request given a task from a test repository. As of
2026-09-12 it does not, and not because it failed:

* hive-mind#2233 asked for the reusable action on `issues: opened`, and
  hive-mind#2234 installed it as `.github/workflows/formal-ai-draft.yml` on
  2026-09-09.
* That workflow has **never run**. No issue has been opened in hive-mind since
  it merged, and `issues: opened` is its only automatic trigger.
* No pull request in hive-mind has ever been authored by Formal AI. Every pull
  request on that repository is a human one.

So the integration is installed and untested in production at the same time.
There is also a silent-skip path worth naming: `decideDraft` denies with
`missingToken` when `FORMAL_AI_DRAFT_TOKEN` is not configured, which is correct
(a pull request opened with `GITHUB_TOKEN` gets no checks) but means an absent
secret looks exactly like "nothing to do". The first evidence either way is one
`workflow_dispatch` against an existing issue; until that runs, whether the
chain produces a change is unknown rather than working.

This ladder is the deliberate answer to that: the leaf task is small enough
that a failure is unambiguous, and there are twenty of them, so the first real
measurement does not depend on hive-mind happening to receive a new issue.
