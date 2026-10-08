// Issue #550 pins the bundler's JSX transform to the classic runtime with `h`
// as the factory (tsconfig.json), so every file under js/app/ that writes JSX
// must bind `h` (and `Fragment` for `<>…</>`) from React itself. Bun compiles a
// file that forgets the binding without complaint. The page then fails only
// when that component first renders. PR #1188: js/app/debugger-view.jsx lost the
// binding when it was rewritten, so diagnostics mode crashed the web app and the
// VS Code debugger with "h is not defined". No suite rendered the view, so
// nothing failed; rust/tests/e2e/tests/issue-667-debugger-view.spec.js now
// renders it. This guard catches the same slip in any file without a browser.
// The JSX transform exists only in the browser app, so there is no Rust twin.

import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, test } from 'node:test';

const APP = new URL('../../../js/app/', import.meta.url);

// An element starts where an expression can start: after `return`, `=>`, `?`,
// `:`, `(`, `{` or `,`, followed by `<` and a tag name or `>` for a fragment.
const ELEMENT = /(?:\breturn|=>|[?:({,])\s*<(?:[A-Za-z][\w.]*|>)/;
const FRAGMENT = /(?:\breturn|=>|[?:({,])\s*<>/;
// `const { createElement: h, Fragment } = React;`, the binding every file uses.
const BINDS_H = /const\s*\{[^}]*\bcreateElement:\s*h\b[^}]*\}\s*=\s*React\b/;
const BINDS_FRAGMENT = /const\s*\{[^}]*\bFragment\b[^}]*\}\s*=\s*React\b/;

const sources = readdirSync(APP)
  .filter((file) => file.endsWith('.jsx'))
  .map((file) => [file, readFileSync(join(APP.pathname, file), 'utf8')]);

describe('every JSX file binds the factory the bundler calls', () => {
  test('the heuristic sees the files known to write JSX', () => {
    const writers = sources.filter(([, text]) => ELEMENT.test(text)).map(([file]) => file);
    for (const file of ['app.jsx', 'debugger-view.jsx']) assert.ok(writers.includes(file), file);
  });

  for (const [file, text] of sources) {
    if (!ELEMENT.test(text)) continue;
    test(`${file} binds h`, () => {
      assert.match(text, BINDS_H, `${file} writes JSX without binding h`);
    });
    if (FRAGMENT.test(text)) {
      test(`${file} binds Fragment`, () => {
        assert.match(text, BINDS_FRAGMENT, `${file} writes <>…</> without binding Fragment`);
      });
    }
  }
});
