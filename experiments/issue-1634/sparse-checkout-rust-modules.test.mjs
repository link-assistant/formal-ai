import assert from 'node:assert/strict';
import test from 'node:test';
import {checkSource} from '../../scripts/check-sparse-checkout-rust-modules.mjs';

const read = file => ({'scripts/a.rs': '#[path = "b.rs"]\nmod b;', 'scripts/b.rs': ''})[file] ?? '';
const workflow = extra => `jobs:\n  x:\n    steps:\n      - uses: actions/checkout@v6\n        with:\n          sparse-checkout: |\n            /scripts/a.rs\n${extra}          sparse-checkout-cone-mode: false\n`;

test('reports a listed Rust script whose #[path] module is missing (issue #1634)', () => {
  assert.equal(checkSource('w.yml', workflow(''), read).length, 1);
});
test('accepts the module when it is listed', () => {
  assert.deepEqual(checkSource('w.yml', workflow('            /scripts/b.rs\n'), read), []);
});
test('accepts a directory pattern covering the module', () => {
  assert.deepEqual(checkSource('w.yml', workflow('            /scripts/\n'), read), []);
});
