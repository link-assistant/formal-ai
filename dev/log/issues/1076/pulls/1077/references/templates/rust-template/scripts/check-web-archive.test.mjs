// Regression tests for the lychee report parser used by the Broken Link
// Checker workflow.
//
// Issue #136: lychee errors that are not http(s) URLs (missing local files,
// unresolvable root-relative links) were dropped by the parser, so the script
// printed "No broken URLs found" and set `all_archived=true` for a run in
// which lychee had reported real errors.
import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  extractBrokenLinks,
  extractBrokenUrls,
  extractErrorsSection,
} from './check-web-archive.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const report = readFileSync(join(here, 'fixtures', 'lychee-report.md'), 'utf-8');

test('the errors section stops at the next top-level heading', () => {
  const section = extractErrorsSection(report);
  assert.ok(section.includes('Errors in README.md'));
  assert.ok(
    !section.includes('Redirects per input'),
    'the redirects section must not leak into the errors section'
  );
});

test('redirected links are not reported as broken', () => {
  const { urls, others } = extractBrokenLinks(report);
  for (const redirected of [
    'https://docs.rs/link-cli',
    'https://github.com/linksplatform/Protocols.Lino',
    'https://habr.com/ru/articles/804617',
  ]) {
    assert.ok(
      ![...urls, ...others].some((link) => link.startsWith(redirected)),
      `${redirected} redirects successfully and must not be treated as broken`
    );
  }
});

test('every http error is extracted exactly once', () => {
  assert.deepEqual(extractBrokenLinks(report).urls, [
    'https://link-foundation.github.io/missing/csharp/',
    'https://link-foundation.github.io/missing/rust/',
  ]);
});

test('errors that the Wayback Machine cannot answer are still reported', () => {
  const { others } = extractBrokenLinks(report);
  assert.deepEqual(others, [
    'file:///home/runner/work/repo/repo/docs/api/Some.Type.yml',
    'error:',
  ]);
});

test('the parsed error count matches the count lychee reports', () => {
  const { urls, others } = extractBrokenLinks(report);
  const reported = Number(/🚫 Errors\s*\|\s*(\d+)/.exec(report)[1]);
  assert.equal(urls.length + others.length, reported);
});

test('a report whose only errors are non-HTTP is not "all archived"', () => {
  const onlyLocalErrors = `## Errors per input

### Errors in docs/index.md

* [ERROR] <file:///repo/docs/does-not-exist.yml> (at 1:1) | File not found. Check if file exists and path is correct
`;

  const { urls, others } = extractBrokenLinks(onlyLocalErrors);
  assert.deepEqual(urls, []);
  assert.deepEqual(others, ['file:///repo/docs/does-not-exist.yml']);
});

test('a report with only redirects yields no broken links', () => {
  const redirectsOnly = `# Link Checker Report

## Redirects per input

### Redirects in README.md

* https://working.example/old --[301]--> https://working.example/current
`;

  assert.deepEqual(extractBrokenLinks(redirectsOnly), { urls: [], others: [] });
});

test('extractBrokenUrls retains full-report parsing for legacy output', () => {
  const legacy = `Broken links

* [404] https://broken.example/legacy | Rejected status code: 404
`;

  assert.deepEqual(extractBrokenUrls(legacy), ['https://broken.example/legacy']);
});
