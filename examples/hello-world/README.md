# Hello World seeds

One directory per language in `data/meta/hello-world-languages.lino`, each
holding the reference program for that language.

These are seeds, not fixtures. A generated task contract points `seed:` at the
directory so the authoring run starts in a workspace that already has the file
name and the run command settled, and the run's job is to write the program.
The committed program is what that run is compared against.

The program lives here and not in `data/meta/hello-world-languages.lino` because
canonical Links Notation has no escape for a quote inside a quoted scalar, and
every Hello World but one contains a quote. The table carries what it can
express -- name, slug, file name, run command -- and the source stays in source
files. `tests/unit/hive_mind_hello_world.rs` asserts every language in the table
has a seed here that prints the expected output, so the two cannot drift.

Generate a contract for one language with:

```sh
rust-script scripts/hello-world-task.rs --list
rust-script scripts/hello-world-task.rs --language ruby
```

Eleven of the twenty were executed on 2026-09-12 and printed exactly
`Hello, World!`; the other nine had no toolchain on that machine. Which is which
is recorded in the table's `verified` field, and the reasoning is in
`docs/case-studies/hive-mind-hello-world/PLAN.md`.
