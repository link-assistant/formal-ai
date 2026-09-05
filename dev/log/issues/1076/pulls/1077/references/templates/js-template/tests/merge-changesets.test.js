/**
 * Tests for merge-changesets.mjs release-time changeset merging.
 * Reproduces issue #87: malformed changesets must fail instead of being
 * skipped while valid changesets are merged.
 */

import { spawnSync } from 'node:child_process';
import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, URL } from 'node:url';

import { describe, it, expect } from 'test-anywhere';

const scriptPath = fileURLToPath(
  new URL('../scripts/merge-changesets.mjs', import.meta.url)
);
const canRunCliFixtures =
  typeof Deno === 'undefined' &&
  typeof process !== 'undefined' &&
  process.execPath;

function createFixture() {
  const root = mkdtempSync(path.join(tmpdir(), 'merge-changesets-'));
  const changesetDir = path.join(root, '.changeset');

  mkdirSync(changesetDir, { recursive: true });
  writeFileSync(
    path.join(root, 'package.json'),
    JSON.stringify({
      name: 'fixture-package',
      version: '1.0.0',
    })
  );

  return { changesetDir, root };
}

function writeChangeset(changesetDir, fileName, content) {
  writeFileSync(path.join(changesetDir, fileName), content);
}

function runMergeChangesets(root) {
  return spawnSync(process.execPath, [scriptPath], {
    cwd: root,
    encoding: 'utf8',
  });
}

describe('merge-changesets.mjs', () => {
  if (canRunCliFixtures) {
    it('fails without merging when any changeset has an unparseable bump type', () => {
      const { changesetDir, root } = createFixture();

      try {
        writeChangeset(
          changesetDir,
          'valid.md',
          `---
'fixture-package': patch
---

Keep this release note.
`
        );
        writeChangeset(
          changesetDir,
          'malformed.md',
          `---
'fixture-package': patches
---

This note must not be silently dropped.
`
        );

        const result = runMergeChangesets(root);
        const output = `${result.stdout}\n${result.stderr}`;

        expect(result.status).not.toBe(0);
        expect(output).toContain('Could not parse version type');
        expect(output).toContain('malformed.md');
        expect(existsSync(path.join(changesetDir, 'valid.md'))).toBe(true);
        expect(existsSync(path.join(changesetDir, 'malformed.md'))).toBe(true);
        expect(
          readdirSync(changesetDir).filter((file) => file.endsWith('.md'))
            .length
        ).toBe(2);
      } finally {
        rmSync(root, { recursive: true, force: true });
      }
    });
  }
});
