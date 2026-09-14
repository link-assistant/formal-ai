import { describe, expect, test } from 'bun:test';
import { execFileSync, spawnSync } from 'node:child_process';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const SCRIPT_PATH = path.join(HERE, 'version-and-commit.mjs');

function git(args, cwd) {
  execFileSync('git', args, { cwd, stdio: 'pipe', encoding: 'utf-8' });
}

function setupRepo({ startingVersion, changesetBump }) {
  const root = mkdtempSync(path.join(tmpdir(), 'version-and-commit-'));
  const remote = path.join(root, 'remote.git');
  const repo = path.join(root, 'repo');
  mkdirSync(repo);

  // Initialize a bare remote so `git push` succeeds without network access.
  execFileSync('git', ['init', '-q', '--bare', '-b', 'main', remote]);

  git(['init', '-q', '-b', 'main'], repo);
  git(['config', 'user.email', 'test@example.com'], repo);
  git(['config', 'user.name', 'Test User'], repo);
  git(['config', 'commit.gpgsign', 'false'], repo);
  git(['config', 'tag.gpgsign', 'false'], repo);
  git(['remote', 'add', 'origin', remote], repo);
  git(['commit', '--allow-empty', '-q', '-m', 'init'], repo);

  mkdirSync(path.join(repo, 'src', 'MyPackage'), { recursive: true });
  writeFileSync(
    path.join(repo, 'src', 'MyPackage', 'MyPackage.csproj'),
    `<Project Sdk="Microsoft.NET.Sdk">\n  <PropertyGroup>\n    <Version>${startingVersion}</Version>\n  </PropertyGroup>\n</Project>\n`
  );

  mkdirSync(path.join(repo, '.changeset'), { recursive: true });
  if (changesetBump) {
    writeFileSync(
      path.join(repo, '.changeset', 'feature.md'),
      `---\n'MyPackage': ${changesetBump}\n---\n\nAdd a feature\n`
    );
  }

  git(['add', '-A'], repo);
  git(['commit', '-q', '-m', 'snapshot'], repo);
  git(['push', '-q', '-u', 'origin', 'main'], repo);

  return { repo, root };
}

function runScript(repo, extraArgs = ['--mode', 'changeset']) {
  const outputFile = path.join(repo, 'gh-output.txt');
  writeFileSync(outputFile, '');

  const result = spawnSync('bun', ['run', SCRIPT_PATH, ...extraArgs], {
    cwd: repo,
    env: { ...process.env, GITHUB_OUTPUT: outputFile },
    encoding: 'utf-8',
  });

  const outputs = Object.fromEntries(
    readFileSync(outputFile, 'utf-8')
      .split('\n')
      .filter(Boolean)
      .map((line) => {
        const eq = line.indexOf('=');
        return [line.slice(0, eq), line.slice(eq + 1)];
      })
  );

  return {
    exitCode: result.status,
    stdout: result.stdout || '',
    stderr: result.stderr || '',
    outputs,
  };
}

describe('version-and-commit', () => {
  test('does not report already_released when the target tag is missing', () => {
    // Regression test for
    // https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/9
    // Starting at 2.3.0 with a minor changeset should produce 2.4.0,
    // and the script must not claim 2.4.0 was already released.
    const { repo, root } = setupRepo({
      startingVersion: '2.3.0',
      changesetBump: 'minor',
    });
    try {
      const probe = spawnSync(
        'git',
        ['rev-parse', '--verify', '--quiet', 'refs/tags/v2.4.0'],
        { cwd: repo, encoding: 'utf-8' }
      );
      expect(probe.status).not.toBe(0);

      const { exitCode, outputs, stdout, stderr } = runScript(repo);

      expect({ exitCode, stdout, stderr }).toEqual({
        exitCode: 0,
        stdout: expect.any(String),
        stderr: expect.any(String),
      });
      expect(outputs.already_released).not.toBe('true');
      expect(outputs.new_version).toBe('2.4.0');
      expect(outputs.version_committed).toBe('true');

      const csproj = readFileSync(
        path.join(repo, 'src', 'MyPackage', 'MyPackage.csproj'),
        'utf-8'
      );
      expect(csproj).toContain('<Version>2.4.0</Version>');

      const log = execFileSync('git', ['log', '--oneline'], {
        cwd: repo,
        encoding: 'utf-8',
      });
      expect(log).toContain('chore: release v2.4.0');

      const tagListing = execFileSync('git', ['tag', '-l', 'v2.4.0'], {
        cwd: repo,
        encoding: 'utf-8',
      }).trim();
      expect(tagListing).toBe('v2.4.0');
    } finally {
      rmSync(root, { force: true, recursive: true });
    }
  });

  test('reports already_released when the target tag does exist', () => {
    const { repo, root } = setupRepo({
      startingVersion: '2.3.0',
      changesetBump: 'minor',
    });
    try {
      git(['tag', 'v2.4.0'], repo);

      const { exitCode, outputs } = runScript(repo);

      expect(exitCode).toBe(0);
      expect(outputs.already_released).toBe('true');
      expect(outputs.new_version).toBe('2.4.0');
    } finally {
      rmSync(root, { force: true, recursive: true });
    }
  });
});
