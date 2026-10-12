# JavaScript first, full parity, then translate

Source: the architect's 2026-10-06 instructions while continuing
[PR #1188](https://github.com/link-assistant/formal-ai/pull/1188) (the bulk
QA, reasoning and coding batch). The requirements they became are the rows of
the 2026-10-06 standing doctrine in `REQUIREMENTS.md` (R997-R1000); this note
keeps the words and the decisions taken to honor them.

> we need to give priority for JavaScript version of the system, make sure it
> have full features parity, as development speed is faster with JavaScript,
> we should make sure we first implement all the requirements from all issues
> related to coding, qa and so on in JavaScript, and only after that by using
> automated translation for that use
> https://github.com/link-foundation/meta-language/pull/196 and
> github.com/link-foundation/relative-meta-logic you can take a copy of best
> practices we can apply here, so we have easy way to convert between
> JavaScript, Meta Language and Rust, so we can do more with less effort.

> double check to make bulk code changes for all requirements, and use local
> resources carefully as possible minimize rust building or delegate it fully
> to CI/CD (meaning to trigger Rust builds we just do push).

## What changed

The 2026-09-24 note made JavaScript a full implementation root; this one makes
it the **first** root. A requirement is implemented and tested in JavaScript
first; Rust follows by translation (or, until the translator covers the
construct, by a port that names its JavaScript original).

## Decisions

1. **The shrink-only worker budget no longer blocks parity.** R995 kept
   `scripts/check-worker-line-budget.rs` as downward pressure on the JS worker.
   A module's ceiling may now rise when the growth is a JavaScript twin of a
   native handler; the shard's rationale names the handler keys it adds.
2. **The parity direction is ratcheted instead.** The number of handler rows
   the browser registry marks `null` ("native surface only") may only fall:
   `scripts/check-js-parity.mjs` against `data/meta/js-parity-ratchet.lino`,
   run in the js tier of `layered-ci.yml`.
3. **Translation runs in JavaScript too.** `scripts/translate-es.mjs` is the
   byte-for-byte twin of the native js → ts leg, so `ts/` regenerates without
   compiling the crate; the rust tier proves both translators agree.
4. **Rust compiles in CI, not locally.** Local work runs the JavaScript suites
   (`npm run test:web`, seconds) and formatters; a push is the Rust build.

## Practices adopted from the sibling repositories

From [link-foundation/relative-meta-logic](https://github.com/link-foundation/relative-meta-logic):
mirrored file names between roots, one shared golden corpus in links notation
both runtimes walk, a differential parity job, path-filtered workflows.

From [link-foundation/meta-language#196](https://github.com/link-foundation/meta-language/pull/196)
(open draft as of 2026-10-06; no release ships `translate` yet): the portable
JavaScript subset its JavaScript → Rust translator accepts — top-level
functions with JSDoc types, `const`/`let`, tagged-union objects dispatched by
`switch (x.$)`, `throw new Error('literal')`, no classes, generators,
destructuring, default or rest parameters — is the style new handler modules
are written in, so they translate the day the translator is released; and its
item-level round trip (unedited items restored byte for byte by hash) is the
model for the js ↔ rust leg once it is adopted as a dependency.
