# The js -> rust leg through meta-language self-translation (R994, R1000, R1012)

R1000 recorded the js -> rust leg as "awaits meta-language#196 (open draft)".
PR #196 was merged on 2026-10-06 as `679a3b3c` (issue #195 closed; the
remaining scope is #199). No release has been cut: crates.io's latest is
0.58.2 (2026-08-20), from before self-translation, and npm has only 0.46.0.
So the leg pins the main commit and fetches it in CI, instead of waiting for a
release.

## What runs

`node scripts/translate-js-rust.mjs` drives the upstream translator,
`selfTranslate(source, 'JavaScript', 'Rust')` from meta-language main at the
commit `data/meta/js-rust-translation.lino` pins. The scope is the agentic
crate twins (`js/agentic/crate/*.mjs`) and the meta reasoner
(`js/worker/formal_ai_worker_meta_*.js`): 123 tracked modules, 814 KB.

| Mode | Upstream needed | What it does |
| --- | --- | --- |
| `--verify` | no | Every module in scope has a ledger row and a projection recorded from its current bytes (SHA-256 and length in the upstream header). The rows match their projections, the totals are the sums of the rows, and the layered-ci `ref:` equals the ledger's commit. Runs in seconds: the `check_js_rust_translation` gate (stage web) and the web suite. |
| `--check --compile` | yes | Re-translates every module. The ledger and every projection must match byte for byte, and no module's translated count may fall. Every projection is then compiled with `rustc --edition 2024`, and each one is run once more as a program whose `main` asserts the translated Rust against the JavaScript: every translated exported constant against the value the module exports, and every row of `calls.lino`. This is the `js-rust` job of `.github/workflows/layered-ci.yml`. |
| `--write [MODULE...]` | yes | Regenerates everything, or only the named modules. Refuses a regression. |
| `--why MODULE` | yes | Prints every carried item of one module with its full upstream diagnostic. |
| `--compile` | no | Compiles the committed projections and runs the assertions with rustc only. |

`--fetch` downloads the pinned commit's tarball into the temporary directory
and runs `npm ci` in its `js/`. CI instead checks the repository out at the
same SHA, the way the js tier pins relative-meta-logic.

### What is committed

- `rust/tests/fixtures/js-rust-translation/<crate|worker>/<module>.rs`: one
  projection of the upstream output per module. It keeps the provenance header
  (source SHA-256 and length) and the prelude, and every translated block byte
  for byte: the marker with the code's hash, the `// |` source lines and the
  emitted Rust. A carried item keeps its upstream marker plus one
  `// formal-ai:refusal <construct>` line. Its source lines are dropped,
  because the source is the JavaScript module the header hashes. A full
  upstream output would repeat every module in comments: more than 1000
  lines for the large modules, and about 1 MB in all.
- `rust/tests/fixtures/js-rust-translation/calls.lino`: calls of exported
  translated functions with their results. Both runtimes run them.
- `data/meta/js-rust-translation.lino`: the pin, the totals, one row per
  module (`translated`, `inferred`, `carried`) and the census of refused
  constructs.

### The ratchet

A module's `translated` count never falls, and its `inferred` count (translated
items with a parameter that has no JSDoc `@param` type, whose Rust type
meta-language guessed) never rises. Both `--write` and `--check` refuse a
regression, naming the module. A module that leaves the scope drops its row,
and a new module sets its own floor. The totals therefore only move the right
way: translated items and modules up, refused modules down.

## Measured at `679a3b3c` (2026-10-08)

- **214 top-level items translated, 1,926 carried.** Each item is counted with
  the doc comments that travel with it.
- **72 of 123 modules translate at least one item. One translates whole**
  (`issue_report.mjs`: `LINO_FENCE_LANGUAGE` and `fencedBlock`). **51 are
  refused** (nothing translated).
- **1 inferred signature** (`isCount` in `orchestration_replay.mjs`, a JSON
  type guard whose parameter really is any value).
- **All 123 projections compile with rustc.** 72 assertions hold in translated
  Rust: 63 exported constants equal their JavaScript values, and 9 calls
  return their recorded results. `rust/tests/web/js-rust-translation.test.mjs`
  runs the JavaScript side.
- A full re-translation takes about 7.5 CPU-minutes, or 2.5 minutes on four
  worker threads.

### What the first measurement changed

The first run counted 218 translated items, 17 of them with inferred
signatures. 9 of those contradicted their callers: six took a string
parameter typed `f64` (`pluralize`, `trailer`, `bare`, `joinPath`, `span`,
`byteOrder`), and three took a string typed `Vec<String>` (`escape`,
`leadingSpaces`, `stringEnd`). JSDoc types were added to 16 of the 17
functions, in the portable-subset style R1000 asks of new code:

- 5 now translate with `String` parameters.
- 4 are carried, because string indexing, iteration and ordering are outside
  the portable core. That is the honest count.
- The other 7 kept the types they had already been given.

`isCount` keeps its inferred signature, since its parameter is untyped JSON.

### A parity finding

`fencedBlock` (JavaScript) trims with `String.prototype.trimEnd`, which strips
U+FEFF. The translated Rust reproduces this exactly. The hand-written
`fenced_block` in `rust/src/issue_report.rs` uses `str::trim_end`, which
does not strip U+FEFF, because U+FEFF is not Unicode White_Space. So
`fenced_block("lino", "x\u{feff}")` differs between the hand-written Rust and
both the JavaScript twin and its translation. `calls.lino` pins the translated
behaviour, and the hand-written Rust is left as it is for its owner to decide.

### What keeps items out (the refusal census)

| Group | Items |
| --- | ---: |
| Methods of values (`.find`, `.split`, `.some`, `.filter`, `.slice`, `.map`, …) | 423 |
| Imports (366 named imports of sibling modules) | 378 |
| `null`, `undefined`, `??`, `?.` | 196 |
| JSDoc types outside the core (`{object}`, `{string\|null}`, records, tuples) | 189 |
| Objects without a `$` tag, `new`, field access, object spread | 180 |
| Arrow functions and functions as values | 165 |
| Calls of a sibling item or an imported name (`unknown name crate.…`) | 105 |
| Standard library (`Array.from`, `Object.keys`, `JSON.stringify`, …) | 56 |
| Syntax errors that are lexer gaps (regex bodies, escapes, optional `@param`) | 39 |
| Everything else (regular expressions, `try`, `typeof`, case tests, …) | 195 |

## Upstream drafts

| Draft | Gap |
| --- | --- |
| [08](upstream-issue-drafts/08-meta-language.md) | Release #196 to crates.io and npm, so the SHA pin can become a version. |
| [09](upstream-issue-drafts/09-meta-language.md) | `selfTranslate` checks each item alone, so sibling calls (105) and imports (378) are carried, and named imports get a misleading `node:assert` diagnostic. |
| [10](upstream-issue-drafts/10-meta-language.md) | The lexer reads regex bodies as code and refuses valid escapes and optional `@param [name]` (39 syntax errors). |
| [11](upstream-issue-drafts/11-meta-language.md) | An unannotated parameter's type is guessed from one item's body, and the item is still reported as translated. |
| [12](upstream-issue-drafts/12-meta-language.md) | Carried items keep only the error kind, so the consumer re-runs `translateProgram` per item to name the construct. |
| [13](upstream-issue-drafts/13-meta-language.md) | Two lossless parses per module at about 5 KB/s dominate the run time. |

Draft [07](upstream-issue-drafts/07-meta-language.md) (clippy-pedantic output)
still applies. The projections are compiled with plain `rustc`, not under the
crate's clippy profile, because upstream emits `(a + b)` tails and `String`
parameters.

## What is not done

- The translated Rust does not replace any hand-written module yet. The
  hand ports of R1012 (`rust/src/meta_reasoner/`) remain. The three meta
  reasoner modules translate 1 of their 73 items (`composite` 0 of 6,
  `reasoner` 1 of 43, `synthesis` 0 of 24), and each refused construct is
  listed in its projection under `rust/tests/fixtures/js-rust-translation/worker/`.
- The calls compare translated Rust with JavaScript. They are not run against
  the hand-written Rust, because most twins' Rust originals are `pub(crate)`
  and out of an integration test's reach.
