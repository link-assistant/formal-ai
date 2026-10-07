<!-- repo: link-foundation/meta-language -->
<!-- title: selfTranslate: let an item call the module's other items and the functions it imports -->

### Problem

`selfTranslate` (main at `679a3b3c`, `js/src/self-translation.js`, `translateGroup`) translates each top-level item on its own: `checkProgram(parseJavaScript(text))` sees only that item's text. So an item that calls another function of the same module is carried as `(type)`, with the diagnostic `unknown name crate.<name>`. `translateProgram`, given the whole module, translates the same module:

```js
import { selfTranslate, translateProgram } from 'meta-language';
const source = `/**
 * @param {number} a
 * @returns {number}
 */
export function twice(a) {
  return a * 2;
}

/**
 * @param {number} a
 * @returns {number}
 */
export function quad(a) {
  return twice(twice(a));
}
`;
selfTranslate(source, 'JavaScript', 'Rust').items.map((item) => `${item.term} ${item.status} ${item.reason}`);
// [ 'comment translated null', 'export_statement translated null',
//   'comment carried type',    'export_statement carried type' ]
translateProgram(source, 'JavaScript', 'Rust').diagnostic; // null: the whole program translates
```

A call to an imported function fails earlier. `import { twice } from './twice.mjs'` is carried as `(unsupported)`, and its diagnostic is misleading: `import { twice }: import the assertion module as a whole, e.g. import assert from 'node:assert/strict'`. The named-import check runs before the module check, so every named import of a sibling module is told to import `node:assert`. Every item that calls an imported name is then carried as `unknown name` too.

### Consumer need

link-assistant/formal-ai translates 123 JavaScript modules with `selfTranslate` ([link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188), refusal census in `data/meta/js-rust-translation.lino`). Of its 1,926 carried items:

- 378 are imports (366 named imports from sibling modules, 12 others);
- 105 are carried only because they call a sibling item or an imported name (`unknown name crate.…`).

These modules are hand-ported twins of Rust modules, so a function that calls its neighbours is the normal case. While items are checked in isolation, a module translates only up to its first helper call, however portable its code is.

### Request

1. Check each item against the declarations of the whole module: the signatures of its other top-level items (translated or not) are in scope, and a call to a translated sibling becomes a call to its translation.
2. Read `import { a, b } from './x.mjs'` as a module dependency. Emit `use super::x::{a, b};` (or a configurable path) for Rust, take the imported function's signature from the other module's JSDoc when it is given, and carry the call only when the signature cannot be known.
3. Make the named-import diagnostic name the real gap ("imports of modules other than node:assert are outside the portable core"), not the `node:assert` advice.

Acceptance: the `twice`/`quad` module above self-translates both items, and a two-module corpus case (one module importing the other) translates both and round-trips byte for byte.

---
Found while wiring the js -> rust leg in [link-assistant/formal-ai#1188](https://github.com/link-assistant/formal-ai/pull/1188) (`docs/case-studies/pull-request-1188/js-rust-translation.md`).
