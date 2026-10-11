// Issue #1021 R1021-34: a gate that reports a failure names the thing that
// failed, not the healthy things beside it.
//
// The Broken Link Checker went red three times naming links that answer 200:
// `extractBrokenUrls` (scripts/check-web-archive.mjs) found the failing part
// of lychee's report by one hard-coded `## Errors per input` heading, so a
// report whose only failure was a timeout was parsed whole, healthy redirects
// included. The real report of run 32454084765 is committed beside its
// reproduction (experiments/issue-1021-link-checker-false-positive/) and is
// replayed here, together with the shapes the old lookup got wrong.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import { extractBrokenUrls } from '../../../scripts/check-web-archive.mjs';
import { REPO_ROOT } from './support/browser-runtime.mjs';

const REPORT = readFileSync(
  `${REPO_ROOT}/experiments/issue-1021-link-checker-false-positive/lychee-out-32454084765.md`, 'utf8');

test('the real report of run 32454084765 names only the link that timed out', () => {
  assert.deepEqual(extractBrokenUrls(REPORT), ['https://docs.anthropic.com/en/docs/claude-code/cli-usage']);
});

test('no link lychee called a healthy redirect is reported broken', () => {
  const redirects = REPORT.slice(REPORT.indexOf('## Redirects per input'));
  const healthy = [...redirects.matchAll(/^\*\s+(\S+)\s+--\[/gm)].map((match) => match[1])
    .filter((url) => url !== 'https://docs.anthropic.com/en/docs/claude-code/cli-usage');
  assert.ok(healthy.length >= 10, 'the fixture carries the healthy redirects that were misreported');
  const broken = new Set(extractBrokenUrls(REPORT));
  assert.deepEqual(healthy.filter((url) => broken.has(url)), []);
});

test('a section of an unrecognised category is reported, not dropped', () => {
  const report = [
    '# Summary', '',
    '## Redirects per input', '',
    '### Errors in README.md', '',
    '* https://healthy.example/ --[301]--> https://healthy.example/new', '',
    '## Novel outcome per input', '',
    '### Errors in README.md', '',
    '* [NOVEL] <https://failing.example/page> | something new went wrong', '',
  ].join('\n');
  assert.deepEqual(extractBrokenUrls(report), ['https://failing.example/page']);
});
