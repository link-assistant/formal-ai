import { describe, it, expect } from 'test-anywhere';
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

import { runCli } from '../bin/example-package-name.js';

const packageJson = JSON.parse(readFileSync('package.json', 'utf8'));
const lockJson = JSON.parse(readFileSync('package-lock.json', 'utf8'));

describe('publishable package metadata', () => {
  it('uses the real link-foundation example package name', () => {
    expect(packageJson.name).toBe('@link-foundation/example-package-name');
    expect(packageJson.publishConfig).toEqual({ access: 'public' });
    expect(lockJson.name).toBe('@link-foundation/example-package-name');
    expect(lockJson.packages[''].name).toBe(
      '@link-foundation/example-package-name'
    );
  });

  it('defines a globally installable CLI command', () => {
    expect(packageJson.bin).toEqual({
      'example-package-name': './bin/example-package-name.js',
    });
    expect(existsSync('bin/example-package-name.js')).toBe(true);
  });

  it('runs package functions through the CLI command', () => {
    const stdout = [];
    const stderr = [];

    expect(
      runCli(['add', '2', '3'], {
        stderr: (line) => stderr.push(line),
        stdout: (line) => stdout.push(line),
      })
    ).toBe(0);

    expect(stdout).toEqual(['5']);
    expect(stderr).toEqual([]);
  });

  it('runs when invoked through an npm-style bin symlink', () => {
    if (typeof Deno !== 'undefined') {
      return;
    }

    const tempRoot = mkdtempSync(join(tmpdir(), 'example-package-name-'));
    const linkPath = join(tempRoot, 'example-package-name');

    try {
      symlinkSync(resolve('bin/example-package-name.js'), linkPath);
    } catch (error) {
      rmSync(tempRoot, { force: true, recursive: true });

      if (process.platform === 'win32') {
        expect(error.code).toBe('EPERM');
        return;
      }

      throw error;
    }

    try {
      const result = spawnSync(
        process.execPath,
        [linkPath, 'multiply', '6', '7'],
        { encoding: 'utf8' }
      );

      expect(result.status).toBe(0);
      expect(result.stdout.trim()).toBe('42');
      expect(result.stderr).toBe('');
    } finally {
      rmSync(tempRoot, { force: true, recursive: true });
    }
  });

  it('publishes only the package runtime surface', () => {
    expect(packageJson.files).toEqual([
      'bin/',
      'src/',
      'CHANGELOG.md',
      'LICENSE',
      'README.md',
    ]);
  });
});
