import { describe, it, expect } from 'test-anywhere';
import { spawnSync } from 'node:child_process';
import {
  chmodSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, URL } from 'node:url';

const scriptPath = fileURLToPath(
  new URL('../scripts/check-file-line-limits.sh', import.meta.url)
);
const isDenoRuntime = typeof Deno !== 'undefined';
const canRunBashFixtures =
  !isDenoRuntime &&
  typeof process !== 'undefined' &&
  process.platform !== 'win32';

function lines(count) {
  return `${Array.from({ length: count }, (_, index) => `line-${index + 1}`).join('\n')}\n`;
}

function createFixture(files) {
  const root = mkdtempSync(path.join(tmpdir(), 'line-limit-'));

  for (const [filePath, lineCount] of Object.entries(files)) {
    const absolutePath = path.join(root, filePath);
    mkdirSync(path.dirname(absolutePath), { recursive: true });
    writeFileSync(absolutePath, lines(lineCount));
  }

  return root;
}

function runLineLimitCheck(root, env) {
  return spawnSync('bash', [scriptPath], {
    cwd: root,
    encoding: 'utf8',
    env,
  });
}

describe('check-file-line-limits.sh', () => {
  it('defines a warning band below the hard limit', () => {
    const script = readFileSync(scriptPath, 'utf8');

    expect(script).toContain('LIMIT=1500');
    expect(script).toContain('WARN_THRESHOLD=1350');
    expect(script).toContain('WARNINGS=()');
    expect(script).toContain('::warning file=');
  });

  if (canRunBashFixtures) {
    it('warns without failing for files above the warning threshold', () => {
      const root = createFixture({
        'src/near-limit.mjs': 1351,
        '.github/workflows/release.yml': 1351,
      });

      try {
        const result = runLineLimitCheck(root);

        expect(result.status).toBe(0);
        expect(result.stdout).toContain(
          'WARNING: ./src/near-limit.mjs has 1351 lines'
        );
        expect(result.stdout).toContain(
          'WARNING: .github/workflows/release.yml has 1351 lines'
        );
        expect(result.stdout).toContain(
          'The following files are approaching the 1500 line limit (>1350 lines):'
        );
        expect(result.stdout).toContain('  ./src/near-limit.mjs');
        expect(result.stdout).toContain('  .github/workflows/release.yml');
        expect(result.stdout).not.toContain(
          'The following files exceed the 1500 line limit:'
        );
      } finally {
        rmSync(root, { recursive: true, force: true });
      }
    });

    it('still fails files over the hard limit', () => {
      const root = createFixture({
        'src/too-large.mjs': 1501,
      });

      try {
        const result = runLineLimitCheck(root);

        expect(result.status).toBe(1);
        expect(result.stdout).toContain(
          'ERROR: ./src/too-large.mjs has 1501 lines (limit: 1500)'
        );
        expect(result.stdout).toContain(
          'The following files exceed the 1500 line limit:'
        );
        expect(result.stdout).toContain('  ./src/too-large.mjs');
      } finally {
        rmSync(root, { recursive: true, force: true });
      }
    });

    it('fails .js and .cjs files over the hard limit', () => {
      const root = createFixture({
        'src/too-large.js': 1501,
        'src/legacy.cjs': 1600,
      });

      try {
        const result = runLineLimitCheck(root);

        expect(result.status).toBe(1);
        expect(result.stdout).toContain(
          'ERROR: ./src/too-large.js has 1501 lines (limit: 1500)'
        );
        expect(result.stdout).toContain(
          'ERROR: ./src/legacy.cjs has 1600 lines (limit: 1500)'
        );
        expect(result.stdout).toContain(
          'The following files exceed the 1500 line limit:'
        );
      } finally {
        rmSync(root, { recursive: true, force: true });
      }
    });

    it('fails Markdown files over the hard limit', () => {
      const root = createFixture({
        'docs/HUGE.md': 1501,
      });

      try {
        const result = runLineLimitCheck(root);

        expect(result.status).toBe(1);
        expect(result.stdout).toContain(
          'ERROR: ./docs/HUGE.md has 1501 lines (limit: 1500)'
        );
        expect(result.stdout).toContain(
          'The following files exceed the 1500 line limit:'
        );
      } finally {
        rmSync(root, { recursive: true, force: true });
      }
    });

    it('exempts case-study generated-data files from the limit', () => {
      const root = createFixture({
        'docs/case-studies/issue-99/data/raw.md': 5000,
        'docs/case-studies/issue-99/data/sample.cjs': 5000,
      });

      try {
        const result = runLineLimitCheck(root);

        expect(result.status).toBe(0);
        expect(result.stdout).not.toContain(
          'The following files exceed the 1500 line limit:'
        );
        expect(result.stdout).not.toContain('data/raw.md');
        expect(result.stdout).not.toContain('data/sample.cjs');
      } finally {
        rmSync(root, { recursive: true, force: true });
      }
    });

    it('normalizes padded wc output from BSD-like environments', () => {
      const root = createFixture({
        'src/near-limit.mjs': 1351,
      });
      const binPath = path.join(root, 'bin');
      const fakeWcPath = path.join(binPath, 'wc');

      try {
        mkdirSync(binPath, { recursive: true });
        writeFileSync(
          fakeWcPath,
          `#!/bin/sh
awk 'END { printf "    %d\\n", NR }'
`
        );
        chmodSync(fakeWcPath, 0o755);

        const result = runLineLimitCheck(root, {
          ...process.env,
          PATH: `${binPath}${path.delimiter}${process.env.PATH ?? ''}`,
        });

        expect(result.status).toBe(0);
        expect(result.stdout).toContain(
          'WARNING: ./src/near-limit.mjs has 1351 lines'
        );
        expect(result.stdout).not.toContain('has     1351 lines');
      } finally {
        rmSync(root, { recursive: true, force: true });
      }
    });
  }
});
