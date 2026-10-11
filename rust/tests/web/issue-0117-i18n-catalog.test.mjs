// Issue #117 (R137-R140): the browser UI translations load through
// lino-i18n from a nested Links Notation catalog, every supported locale
// carries the same keys, and CI fails when a key goes missing. The full
// checker (rust/tests/e2e/scripts/check-i18n-catalog.mjs) needs the e2e
// packages and runs as a registered CI gate; this suite pins the same facts
// with no installed dependency.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const CATALOGS = ['js/i18n-catalog.lino', 'js/i18n-catalog-permissions.lino', 'js/i18n-catalog-messages.lino'];
const LOCALES = ['en', 'ru', 'hi', 'zh'];
const read = (relative) => readFileSync(join(REPO_ROOT, relative), 'utf8');

/** Every key path per locale, merged across the catalog files (as js/i18n.js merges them). */
function keyPaths() {
  const byLocale = new Map(LOCALES.map((locale) => [locale, new Set()]));
  for (const file of CATALOGS) {
    let locale = null;
    let stack = [];
    let inQuoted = false;
    for (const line of read(file).split('\n')) {
      const quotes = (line.match(/"""/g) || []).length;
      if (inQuoted) {
        if (quotes % 2 === 1) inQuoted = false;
        continue;
      }
      if (quotes % 2 === 1) inQuoted = true;
      if (/^\s*(#|$)/.test(line)) continue;
      const top = /^(\S+)\s*$/.exec(line);
      if (top) {
        locale = byLocale.has(top[1]) ? top[1] : null;
        stack = [];
        continue;
      }
      const nested = /^( +)([A-Za-z0-9_]+)/.exec(line);
      if (!locale || !nested) continue;
      const depth = nested[1].length / 2;
      stack = stack.slice(0, depth - 1);
      stack.push(nested[2]);
      byLocale.get(locale).add(stack.join('.'));
    }
  }
  return byLocale;
}

describe('R137/R140: translations load through lino-i18n and CI checks the catalog', () => {
  test('the browser loader takes its runtime from the published lino-i18n package', () => {
    assert.match(read('js/i18n.js'), /PUBLISHED_RUNTIME_SOURCE = "lino-i18n@/);
  });

  test('the catalog coverage checker is a registered CI gate', () => {
    const gate = read('data/meta/ci-gates/check-i18n-catalog-coverage.lino');
    assert.match(gate, /check-i18n-catalog|check:i18n/);
    assert.match(gate, /stage web/);
  });
});

describe('R138/R139: one nested catalog with the same keys in every locale', () => {
  const paths = keyPaths();

  test('messages nest in blocks and long strings use multiline quotes', () => {
    const text = CATALOGS.map(read).join('\n');
    assert.ok(text.includes('"""'), 'no multiline quoted string');
    assert.match(text, /\n {2}buttons\n {4}reportIssue /);
  });

  test('every locale declares the same key paths', () => {
    const english = paths.get('en');
    assert.ok(english.size > 100, `only ${english.size} English keys`);
    for (const locale of LOCALES.slice(1)) {
      const other = paths.get(locale);
      const missing = [...english].filter((key) => !other.has(key));
      const extra = [...other].filter((key) => !english.has(key));
      assert.deepEqual({ missing, extra }, { missing: [], extra: [] }, locale);
    }
  });
});
