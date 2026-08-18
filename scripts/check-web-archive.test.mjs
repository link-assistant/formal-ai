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

test('extractBrokenUrls keeps every error inside the errors section', () => {
  const report = `## Errors per input

### Errors in docs/a.md

* [404] <https://broken.example/a> (at 1:1) | Rejected status code: 404

### Errors in docs/b.md

* [ERROR] <https://broken.example/b> (at 2:1) | Failed: connection refused

## Suggestions per input

* https://broken.example/a | https://web.archive.org/web/2026/https://broken.example/a
`;

  assert.deepEqual(extractBrokenUrls(report), [
    'https://broken.example/a',
    'https://broken.example/b',
  ]);
});
