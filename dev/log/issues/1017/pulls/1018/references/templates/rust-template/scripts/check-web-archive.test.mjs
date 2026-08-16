import assert from 'node:assert/strict';
import test from 'node:test';

import { extractBrokenUrls } from './check-web-archive.mjs';

test('extractBrokenUrls ignores successful redirects after the errors section', () => {
  const report = `## Errors per input

### Errors in docs/reference.md

* [502] <https://broken.example/reference> (at 1:1) | Rejected status code: 502

## Redirects per input

### Redirects in README.md

* https://working.example/old --[301]--> https://working.example/current
`;

  assert.deepEqual(extractBrokenUrls(report), [
    'https://broken.example/reference',
  ]);
});

test('extractBrokenUrls retains full-report parsing for legacy output', () => {
  const report = `Broken links

* [404] https://broken.example/legacy | Rejected status code: 404
`;

  assert.deepEqual(extractBrokenUrls(report), [
    'https://broken.example/legacy',
  ]);
});
