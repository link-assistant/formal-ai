// Requirement rows whose deliverable is the preserved research of an issue:
// its case-study README, the raw GitHub data it was written from, and the
// further documents the row names. Each case lists the row ids it pins, so a
// case study that loses a file fails the rows that promised it.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const CASES = [
  { rows: ['R128'], issue: 96, files: [] },
  { rows: ['R142'], issue: 117, files: [] },
  { rows: ['R143', 'R147'], issue: 115, files: ['raw-data/hive-mind/issues-recent.json', 'raw-data/hive-mind/manifest.json'] },
  { rows: ['R172'], issue: 129, files: [] },
  { rows: ['R230'], issue: 196, files: [] },
  { rows: ['R252', 'R253'], issue: 244, files: ['proposed-issues.md'] },
  { rows: ['R320'], issue: 438, files: [] },
  {
    rows: ['R364', 'R367'],
    issue: 492,
    files: ['assets/issue-screenshot.png'],
    cites: ['rust-ai-driven-development-pipeline-template/issues/85'],
  },
  {
    rows: ['R387', 'R388', 'R389', 'R390', 'R391'],
    issue: 558,
    files: ['requirements.md', 'solution-plan.md', 'pr-601-gap-analysis.md', 'raw-data/online-research.md'],
    contains: {
      'pr-601-gap-analysis.md': ['#601'],
      'requirements.md': Array.from({ length: 12 }, (_, index) => `R558-${String(index + 1).padStart(2, '0')}`),
      'raw-data/online-research.md': ['SWE-agent', 'OpenHands', 'Reflexion', 'DSPy'],
      'solution-plan.md': ['Phase 0', 'Phase 1', 'Phase 2', 'Phase 3', 'Phase 4', 'Phase 5'],
    },
  },
  { rows: ['R873-9'], issue: 873, files: ['requirements.md', 'online-research.md', 'self-hosting-authorship'] },
  { rows: ['R265'], issue: 365, files: [], noRawData: true },
  { rows: ['R922-6'], issue: 922, files: ['requirements.md', 'solution-plan.md', 'agent-cli-run'] },
];

const nonEmpty = (path) => {
  const stat = statSync(path);
  return stat.isDirectory() ? readdirSync(path).length > 0 : stat.size > 0;
};

describe('issue research is preserved under docs/case-studies', () => {
  for (const { rows, issue, files, cites = [], contains = {}, noRawData = false } of CASES) {
    test(`${rows.join(', ')}: issue #${issue} keeps its README, raw data and named documents`, () => {
      const directory = join(REPO_ROOT, 'docs/case-studies', `issue-${issue}`);
      const readme = readFileSync(join(directory, 'README.md'), 'utf8');
      assert.ok(readme.trim().length > 0, `issue-${issue}/README.md is empty`);
      if (!noRawData) {
        assert.ok(nonEmpty(join(directory, 'raw-data')), `issue-${issue}/raw-data is empty`);
      }
      for (const file of files) {
        assert.ok(existsSync(join(directory, file)) && nonEmpty(join(directory, file)), `issue-${issue}/${file}`);
      }
      for (const citation of cites) assert.ok(readme.includes(citation), `issue-${issue} README cites ${citation}`);
      for (const [file, needles] of Object.entries(contains)) {
        const text = readFileSync(join(directory, file), 'utf8');
        for (const needle of needles) assert.ok(text.includes(needle), `issue-${issue}/${file} lacks ${needle}`);
      }
    });
  }
});
