Fixes https://github.com/link-assistant/formal-ai/issues/1021

## What is broken

Formal AI answered seven of the prompts issue #1021 collects wrongly — a bare `ls` was refused, `Execute ls command` ran `ls command`, `List me files here` reached web search, a copy-stdin-to-stdout request ran `cp stdin stdout`, a Rosetta Code URL became a `cp` of its slug, a filesystem move matched nothing at all, and a request naming PHP Laravel fell to the uncatalogued-language fallback — and it produced none of the process artifacts a change needs to land: no changelog fragment, and no pull-request body linking the issue it answers.

## Why

Every wrong answer had its own rule behind it, each written for the phrasing that was in front of it rather than for the structure of the request, so an unseen word order could not match. And nothing in the solver knew the shape of the two artifacts the repository's own gates read, or decided whether a command that publishes a change may run at all.

## The fix

The rules are corrected where they were wrong and their vocabulary moved into seed data, so the fix generalizes past the reported wording; PHP joins the coding catalog with the templates every other catalogued language carries, copying standard input to standard output becomes a catalogued task templated in all thirteen of them, and Laravel joins as an implementation target so a request naming a framework is answered in that framework; `src/contribution_artifacts.rs` composes the changelog fragment and the pull-request body from `data/seed/contribution-artifacts.lino`; and `src/contribution_write_path.rs` puts the publishing commands on a ladder that refuses by default, with `gh issue create` refused in both states. The whole loop is captured as a replayable session.

## Verification

Automated coverage, all of it runnable from a clean checkout:

- `cargo test --test unit -- issue_1021`
- `cargo test --test unit`
- `cargo test --test source`
- `cargo test --test integration`
- `rust-script scripts/run-ci-gates.rs --stage rust`
- `node tests/e2e/scripts/check-multilingual-intent-coverage.mjs`
