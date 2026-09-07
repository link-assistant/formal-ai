import { describe, expect, test } from 'bun:test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import {
  DEFAULT_MAX_ATTEMPTS,
  DEFAULT_SLEEP_SECONDS,
  buildSmokeTestProgram,
  parseArgs,
  smokeTestNugetPackage,
} from './smoke-test-nuget-package.mjs';

function createTempDir(prefix = 'smoke-test-nuget-package-') {
  return mkdtempSync(path.join(tmpdir(), prefix));
}

function createDotnetRunner({ failedAddsBeforeSuccess = 0 } = {}) {
  const calls = [];
  let addAttempts = 0;

  return {
    calls,
    run: (command, args, options) => {
      calls.push({ args, command, cwd: options.cwd });

      if (command !== 'dotnet') {
        return {
          status: 1,
          stdout: '',
          stderr: `Unexpected command: ${command}`,
        };
      }

      if (args[0] === 'add') {
        addAttempts += 1;
        if (addAttempts <= failedAddsBeforeSuccess) {
          return {
            status: 1,
            stdout: 'Package source has not propagated yet',
            stderr: 'error: Unable to find package MyPackage',
          };
        }
      }

      return {
        status: 0,
        stdout: `ok: dotnet ${args.join(' ')}\nline 2\nline 3\nline 4\nline 5\nline 6`,
        stderr: '',
      };
    },
  };
}

describe('smoke-test-nuget-package parseArgs()', () => {
  test('defaults to a NuGet consumer smoke test for MyPackage', () => {
    const config = parseArgs(
      ['--package-id', 'MyPackage', '--release-version', '1.2.3'],
      {}
    );

    expect(config.packageId).toBe('MyPackage');
    expect(config.releaseVersion).toBe('1.2.3');
    expect(config.source).toBe('https://api.nuget.org/v3/index.json');
    expect(config.framework).toBe('net8.0');
    expect(config.libraryNamespace).toBe('MyPackage');
    expect(config.maxAttempts).toBe(DEFAULT_MAX_ATTEMPTS);
    expect(config.sleepSeconds).toBe(DEFAULT_SLEEP_SECONDS);
  });

  test('accepts CLI and environment overrides', () => {
    const config = parseArgs(
      [
        '--package-id=Template.Package',
        '--version=2.0.0',
        '--source',
        '/tmp/packages',
        '--framework',
        'net9.0',
        '--library-namespace',
        'TemplatePackage',
        '--max-attempts',
        '2',
        '--sleep-seconds',
        '3',
      ],
      {
        NUGET_SMOKE_SOURCE: 'https://example.invalid/v3/index.json',
      }
    );

    expect(config).toEqual({
      framework: 'net9.0',
      libraryNamespace: 'TemplatePackage',
      maxAttempts: 2,
      packageId: 'Template.Package',
      releaseVersion: '2.0.0',
      sleepSeconds: 3,
      source: '/tmp/packages',
    });
  });
});

describe('smoke-test-nuget-package buildSmokeTestProgram()', () => {
  test('exercises the package API advertised by the template example', () => {
    const program = buildSmokeTestProgram({
      expectedVersion: '1.2.3',
      libraryNamespace: 'MyPackage',
    });

    expect(program).toContain('using MyPackage;');
    expect(program).toContain('Calculator.Add(2, 3)');
    expect(program).toContain('Calculator.Multiply(2, 3)');
    expect(program).toContain('PackageInfo.Version');
    expect(program).toContain('"1.2.3"');
  });
});

describe('smoke-test-nuget-package smokeTestNugetPackage()', () => {
  test('retries package installation, then builds and runs the throwaway consumer', async () => {
    const runner = createDotnetRunner({ failedAddsBeforeSuccess: 1 });
    const sleeps = [];
    const tempDir = createTempDir();

    try {
      await smokeTestNugetPackage({
        config: {
          framework: 'net8.0',
          libraryNamespace: 'MyPackage',
          maxAttempts: 3,
          packageId: 'MyPackage',
          releaseVersion: '1.2.3',
          sleepSeconds: 7,
          source: 'https://api.nuget.org/v3/index.json',
        },
        makeTempDir: () => tempDir,
        runCommand: runner.run,
        sleepFn: async (seconds) => {
          sleeps.push(seconds);
        },
        stdout: () => {},
      });

      expect(runner.calls.map((call) => call.args[0])).toEqual([
        'new',
        'add',
        'add',
        'build',
        'run',
      ]);
      expect(sleeps).toEqual([7]);
    } finally {
      rmSync(tempDir, { force: true, recursive: true });
    }
  });

  test('fails before build when the package cannot be installed after all attempts', async () => {
    const runner = createDotnetRunner({ failedAddsBeforeSuccess: 3 });
    const tempDir = createTempDir();

    try {
      await expect(
        smokeTestNugetPackage({
          config: {
            framework: 'net8.0',
            libraryNamespace: 'MyPackage',
            maxAttempts: 2,
            packageId: 'MyPackage',
            releaseVersion: '1.2.3',
            sleepSeconds: 1,
            source: 'https://api.nuget.org/v3/index.json',
          },
          makeTempDir: () => tempDir,
          runCommand: runner.run,
          sleepFn: async () => {},
          stdout: () => {},
        })
      ).rejects.toThrow(/Unable to install MyPackage@1.2.3/);

      expect(runner.calls.map((call) => call.args[0])).toEqual([
        'new',
        'add',
        'add',
      ]);
    } finally {
      rmSync(tempDir, { force: true, recursive: true });
    }
  });
});
