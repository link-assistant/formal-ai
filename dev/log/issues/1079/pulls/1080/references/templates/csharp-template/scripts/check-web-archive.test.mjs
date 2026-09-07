// Regression tests for the lychee report parser used by the Broken Link
// Checker workflow. Both cases below were live CI defects reported in issue
// #47: a run escalated 9 "broken" links while lychee itself reported 4 errors
// (false positives from the redirects section), and two of the real errors
// were links the script could never verify, yet it reported them as archived
// (false negative).
import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  extractErrorsSection,
  extractBrokenLinks,
} from './check-web-archive.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const report = readFileSync(join(here, 'fixtures', 'lychee-report.md'), 'utf-8');

describe('lychee report parsing', () => {
  test('the errors section stops at the next top-level heading', () => {
    const section = extractErrorsSection(report);

    expect(section).toContain('Errors in README.md');
    expect(section).not.toContain('Redirects per input');
  });

  test('a report without an errors section yields nothing', () => {
    expect(
      extractErrorsSection(
        '# Report\n\n## Redirects per input\n\n* https://example.com --[301]--> https://example.org\n'
      )
    ).toBe('');
  });

  test('redirected links are not reported as broken', () => {
    const { urls } = extractBrokenLinks(report);

    for (const redirected of [
      'https://docs.rs/link-cli',
      'https://github.com/linksplatform/Protocols.Lino',
      'https://habr.com/ru/articles/804617',
      'https://www.nuget.org/packages/MyPackage',
    ]) {
      expect(urls.some((url) => url.startsWith(redirected))).toBe(false);
    }
  });

  test('every http error is extracted exactly once', () => {
    const { urls } = extractBrokenLinks(report);

    expect(urls).toEqual([
      'https://link-foundation.github.io/csharp-ai-driven-development-pipeline-template/csharp/',
      'https://link-foundation.github.io/csharp-ai-driven-development-pipeline-template/api/',
    ]);
  });

  test('errors that the Wayback Machine cannot answer are still reported', () => {
    const { others } = extractBrokenLinks(report);

    expect(others).toHaveLength(2);
    expect(others.some((link) => link.endsWith('MyPackage.yml'))).toBe(true);
    expect(others).toContain('error:');
  });

  test('the parsed error count matches the count lychee reports', () => {
    const { urls, others } = extractBrokenLinks(report);
    const reported = Number(/🚫 Errors\s*\|\s*(\d+)/.exec(report)[1]);

    expect(urls.length + others.length).toBe(reported);
  });
});
