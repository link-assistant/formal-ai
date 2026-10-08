# The js -> rust leg beyond its committed scope (R992, R994, R1000)

The committed scope of [the js -> rust leg](js-rust-translation.md) covers the
agentic crate twins (`js/agentic/crate/*.mjs`) and the meta reasoner
(`js/worker/formal_ai_worker_meta_*.js`). This report measures the rest of the
JavaScript root with the same translator, without committing it: the agentic
planner (`js/agentic/*.mjs` and its `file_read/`, `learning_report/` and
`planner/` folders), the Node server (`js/server/*.mjs`) and the browser
worker (`js/worker/formal_ai_worker*.js` outside the meta reasoner).

## How it was measured

```sh
node experiments/formal_ai_subagent/translation-scope.mjs --meta-language <checkout>
```

The script runs `selfTranslate(source, 'JavaScript', 'Rust')` from
link-foundation/meta-language main at `679a3b3c`, the commit the ledger pins.
It classifies every item with the functions `scripts/translate-js-rust.mjs`
uses (`measure`, `refusalClass`), so its counts match the ledger's for the
same files. Run over the committed scope, it gives exactly the ledger's
totals. It writes nothing outside its `--json` file. Measured on 2026-10-08
over the working tree of PR #1188. The wider roots took about 25 CPU-minutes,
or 7 minutes on four worker threads.

"With a function" counts the translated items that emit a Rust `fn`. The rest
are constants: string and number literals, and arrays of them.

## Results

| Root | Modules | Items | Translated | With a function | Carried | Translated share | Refused modules |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| **Committed scope** | | | | | | | |
| `js/agentic/crate` | 125 | 2,295 | 248 | 24 | 2,047 | 10.8 % | 47 |
| `js/worker` (meta reasoner) | 3 | 74 | 1 | 0 | 73 | 1.4 % | 2 |
| **Subtotal** | 128 | 2,369 | 249 | 24 | 2,120 | 10.5 % | 49 |
| **Outside the scope** | | | | | | | |
| `js/agentic` (planner) | 87 | 2,205 | 215 | 14 | 1,990 | 9.8 % | 26 |
| `js/agentic/file_read` | 3 | 38 | 1 | 0 | 37 | 2.6 % | 2 |
| `js/agentic/learning_report` | 6 | 60 | 6 | 0 | 54 | 10.0 % | 0 |
| `js/agentic/planner` | 1 | 19 | 0 | 0 | 19 | 0 % | 1 |
| `js/server` | 64 | 1,452 | 177 | 9 | 1,275 | 12.2 % | 20 |
| `js/worker` (the rest) | 72 | 2,790 | 420 | 36 | 2,370 | 15.1 % | 23 |
| **Subtotal** | 233 | 6,564 | 819 | 59 | 5,745 | 12.5 % | 72 |

No module outside the scope translates fully. Across both scopes, 83 of 1,068
translated items are functions, and 59 of those 83 lie outside the committed
scope.

## What keeps the items out

The construct the portable core refuses first, for the 5,745 carried items
outside the scope, matched to the upstream issue filed for it:

| Refusal | Outside | In scope | Upstream |
| --- | ---: | ---: | --- |
| `import { … }`, `import from '…'`, `namespace import` | 1,083 | 417 | [#203](https://github.com/link-foundation/meta-language/issues/203) |
| sibling items: `function value …`, `type: unknown name` | 775 | 209 | [#202](https://github.com/link-foundation/meta-language/issues/202) |
| `JSDoc type {…}` | 679 | 198 | [#205](https://github.com/link-foundation/meta-language/issues/205) |
| `null`, `undefined`, `??`, `?.` | 683 | 218 | [#204](https://github.com/link-foundation/meta-language/issues/204) |
| string methods (`.split`, `.slice`, `.replace`, `.indexOf`, …) | 450+ | 183 | [#209](https://github.com/link-foundation/meta-language/issues/209) |
| array methods (`.map`, `.some`, `.push`, `.filter`, `Array.from`, `Array.isArray`, …) | 526+ | 324 | [#208](https://github.com/link-foundation/meta-language/issues/208) |
| `object without a $ tag`, `field access`, namespace properties | 304 | 120 | [#206](https://github.com/link-foundation/meta-language/issues/206) |
| `new expression`, classes | 171 | 110 | [#210](https://github.com/link-foundation/meta-language/issues/210) |
| regular expressions and the lexer errors they cause | 251 | 84 | [#211](https://github.com/link-foundation/meta-language/issues/211) |
| `arrow function` | 136 | 99 | [#207](https://github.com/link-foundation/meta-language/issues/207) |
| `typeof`, `try`, `case test`, `for (const [a] …)`, `let x;` | 178 | 62 | [#212](https://github.com/link-foundation/meta-language/issues/212) |

The outside-scope rows marked "+" add up only the methods the census lists by
name. The full refusal lists per root are in the script's `--json` output.

Outside the scope, imports and sibling references alone stop 1,858 of the
5,745 carried items (32 %), against 626 of 2,120 in scope (30 %). The browser
worker leans less on imports, because it is a classic script that shares
globals, and more on JSDoc record types (507) and sibling globals (347).

## Decision: the committed scope stays as it is

Widening the scope would add 233 projections and ledger rows. 5,745 of their
6,564 items (88 %) would be carried, and only 59 of the 819 translated items
would be functions. The ratchet would then mostly freeze constants. The CI
`js-rust` job would also take about three times as long. No plan could move
those items from formal-ai's side today: the top four refusal classes are
upstream gaps, and the rewrites below found no gain from the JavaScript side.

The scope widens when the measurement changes, not by date. Re-run
`translation-scope.mjs` after a meta-language commit that fixes #202 or #203,
the sibling and import binding. Add a root to `SCOPES` in
`scripts/translate-js-rust.mjs` once a majority of its translated items are
functions.

## Translation-friendly JavaScript: measured, no gain

Each rewrite below was applied to sandbox copies of the crate modules and
measured with `selfTranslate`. None was committed, because none raised a
module's translated count:

| Rewrite | Modules | Translated items gained | Why not |
| --- | ---: | ---: | --- |
| `for (const x of …)` → `for (let x of …)`, the form the core asks for | 94 | 0 | Each of the 11 items it unblocks then stops at the next construct: a destructured `Map` entry, a sibling call or an imported helper. |
| `.find(…) ?? null` → an index loop and a `''` sentinel (`concept_lookup.mjs` `outsideQuotes`) | 1 | 0, and one more carried item | The loop reads the module's `PAIRS` constant (`function value PAIRS.length`, #202). Iterating a string by code point fails type inference (`one value is used as a string and as an array`, #209). |

The 120 carried crate items whose JSDoc types are all portable (`string`,
`number`, `boolean` and arrays of them) were also sorted by the construct they
stop at. Reading the ones a loop could replace found none that JavaScript can
rewrite without another refused construct taking its place:

- 37 stop at a string or array method (`.split`, `.replace`, `.find`,
  `.codePointAt`, `.lastIndexOf`, `.toFixed`, …), and the string methods have
  no portable spelling at all;
- 31 stop at a sibling constant or function;
- 17 stop at `null`, `undefined`, `??` or `?.`;
- 7 stop at a string `switch`;
- the other 28 stop at plain objects, classes, `Array.from`, arrow
  functions, string iteration, `const` loop destructuring or regex bodies.

## Upstream issues filed on 2026-10-08

One issue per refusal class, each with formal-ai's count, a minimal JavaScript
input checked at `679a3b3c` and the Rust it should produce:
[#202](https://github.com/link-foundation/meta-language/issues/202) (sibling items),
[#203](https://github.com/link-foundation/meta-language/issues/203) (imports),
[#204](https://github.com/link-foundation/meta-language/issues/204) (`null` as `Option`),
[#205](https://github.com/link-foundation/meta-language/issues/205) (JSDoc types, optional `[param]`),
[#206](https://github.com/link-foundation/meta-language/issues/206) (plain object records),
[#207](https://github.com/link-foundation/meta-language/issues/207) (closures),
[#208](https://github.com/link-foundation/meta-language/issues/208) (array methods),
[#209](https://github.com/link-foundation/meta-language/issues/209) (string methods),
[#210](https://github.com/link-foundation/meta-language/issues/210) (`new`, `Set`, `Map`, classes),
[#211](https://github.com/link-foundation/meta-language/issues/211) (regex literals and the lexer bug),
[#212](https://github.com/link-foundation/meta-language/issues/212) (small statement gaps).

Drafts 11, 12 and 13 were filed as
[#213](https://github.com/link-foundation/meta-language/issues/213) (inferred signatures),
[#214](https://github.com/link-foundation/meta-language/issues/214) (keep the diagnostic in the item record) and
[#215](https://github.com/link-foundation/meta-language/issues/215) (double parse).
The release request for PR #196 is a
[comment on #199](https://github.com/link-foundation/meta-language/issues/199#issuecomment-6055834620),
the issue that tracks the release.
