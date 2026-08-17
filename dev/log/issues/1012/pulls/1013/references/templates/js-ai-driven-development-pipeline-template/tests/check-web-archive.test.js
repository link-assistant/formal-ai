import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { extractBrokenUrls } from '../scripts/check-web-archive.mjs';

describe('extractBrokenUrls', () => {
  it('ignores successful redirects outside the errors section', () => {
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

  it('parses the full document when the errors section is absent', () => {
    const legacyReport = `# Link report

* [404] https://broken.example/legacy
`;

    assert.deepEqual(extractBrokenUrls(legacyReport), [
      'https://broken.example/legacy',
    ]);
  });
});
