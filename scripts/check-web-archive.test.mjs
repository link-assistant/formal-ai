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


// The three tests above all describe a report that *has* an errors section,
// which is the one shape the old single-heading lookup got right. Every failure
// below is a shape it got wrong.

test('extractBrokenUrls reports a timeout when the report has no errors section', () => {
  // Lychee writes only the sections it has links for. A run whose sole failure
  // is a timeout has no `## Errors per input` at all, and the old parser fell
  // back to reading the whole document -- collecting every healthy redirect in
  // it. This is run 32454084765 in miniature; the full report is the fixture in
  // experiments/issue-1021-link-checker-false-positive/.
  const report = `# Summary

| Status         | Count |
|----------------|-------|
| ⏳ Timeouts    | 1     |
| 🔀 Redirected  | 2     |
| 🚫 Errors      | 0     |

## Timeouts per input

### Timeouts in docs/configuration/orchestration.md

* [TIMEOUT] <https://slow.example/page> (at 235:33) | Request timed out

## Redirects per input

### Redirects in README.md

* https://working.example/old --[301]--> https://working.example/current

### Redirects in CONTRIBUTING.md

* https://working.example/other --[302]--> https://working.example/elsewhere
`;

  assert.deepEqual(extractBrokenUrls(report), ['https://slow.example/page']);
});

test('extractBrokenUrls collects failures from every failing section, not just the first', () => {
  const report = `## Errors per input

### Errors in docs/a.md

* [404] <https://broken.example/a> (at 1:1) | Rejected status code: 404

## Redirects per input

### Redirects in README.md

* https://working.example/old --[301]--> https://working.example/current

## Timeouts per input

### Timeouts in docs/b.md

* [TIMEOUT] <https://slow.example/b> (at 2:1) | Request timed out
`;

  assert.deepEqual(extractBrokenUrls(report), [
    'https://broken.example/a',
    'https://slow.example/b',
  ]);
});

test('extractBrokenUrls treats an unrecognised section as failing', () => {
  // The selection is by exclusion, so a category this parser has never heard of
  // is reported rather than dropped. Reporting a healthy link is loud and gets
  // corrected; dropping a broken one is silent and ships.
  const report = `## Errors per input

### Errors in docs/a.md

* [404] <https://broken.example/a> (at 1:1) | Rejected status code: 404

## Quarantined per input

### Quarantined in docs/c.md

* [ERROR] <https://unheard-of.example/c> (at 3:1) | Something new
`;

  assert.deepEqual(extractBrokenUrls(report), [
    'https://broken.example/a',
    'https://unheard-of.example/c',
  ]);
});

test('extractBrokenUrls ignores the summary table and the healthy sections around it', () => {
  const report = `# Summary

| Status         | Count |
|----------------|-------|
| ✅ Successful  | 1     |
| 👻 Excluded    | 1     |

## Successes per input

### Successes in README.md

* [200] <https://working.example/fine> (at 1:1)

## Excluded per input

### Excluded in README.md

* https://excluded.example/skipped

## Suggestions per input

* https://broken.example/a | https://web.archive.org/web/2026/https://broken.example/a
`;

  assert.deepEqual(extractBrokenUrls(report), []);
});
