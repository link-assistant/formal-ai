import { describe, it, expect } from 'test-anywhere';
import { spawnSync } from 'node:child_process';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath, URL } from 'node:url';

const scriptPath = fileURLToPath(
  new URL('../scripts/push-main-with-rebase-retry.sh', import.meta.url)
);
const isDenoRuntime = typeof Deno !== 'undefined';
const canRunGitFixtures =
  !isDenoRuntime &&
  typeof process !== 'undefined' &&
  process.platform !== 'win32';

function runCommand(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: 'utf8',
  });

  if (result.status !== 0) {
    throw new Error(
      `${command} ${args.join(' ')} failed\nstdout:\n${
        result.stdout
      }\nstderr:\n${result.stderr}`
    );
  }

  return result;
}

function runGit(root, args) {
  return runCommand('git', args, root);
}

function configureGit(root) {
  runGit(root, ['config', 'user.email', 'ci@example.com']);
  runGit(root, ['config', 'user.name', 'CI Test']);
}

function commitAll(root, message) {
  runGit(root, ['add', '.']);
  runGit(root, ['commit', '-m', message]);
}

function createFile(root, filePath, contents) {
  const absolutePath = path.join(root, filePath);
  mkdirSync(path.dirname(absolutePath), { recursive: true });
  writeFileSync(absolutePath, contents);
}

function runPushHelper(root) {
  return spawnSync('bash', [scriptPath], {
    cwd: root,
    encoding: 'utf8',
  });
}

describe('push-main-with-rebase-retry.sh', () => {
  if (canRunGitFixtures) {
    it('rebases and retries when a generated-artifact push races another main writer', () => {
      const root = mkdtempSync(path.join(tmpdir(), 'push-main-retry-'));
      const remote = path.join(root, 'remote.git');
      const previewWriter = path.join(root, 'preview-writer');
      const releaseWriter = path.join(root, 'release-writer');
      const verifier = path.join(root, 'verifier');

      try {
        runGit(root, ['init', '--bare', '--initial-branch=main', remote]);
        runGit(root, ['clone', remote, previewWriter]);
        configureGit(previewWriter);
        createFile(previewWriter, 'README.md', '# Fixture\n');
        commitAll(previewWriter, 'Initial commit');
        runGit(previewWriter, ['push', 'origin', 'HEAD:main']);

        runGit(root, ['clone', remote, releaseWriter]);
        configureGit(releaseWriter);

        createFile(
          previewWriter,
          'docs/screenshots/example-app/example-app.png',
          'new preview image\n'
        );
        commitAll(previewWriter, 'Regenerate preview images');

        createFile(releaseWriter, 'VERSION.txt', '1.0.1\n');
        commitAll(releaseWriter, 'Version bump');
        runGit(releaseWriter, ['push', 'origin', 'HEAD:main']);

        const result = runPushHelper(previewWriter);
        const output = `${result.stdout}\n${result.stderr}`;

        expect(result.status).toBe(0);
        expect(output).toContain('rebasing on origin/main before retry');
        expect(output).toContain('Push succeeded after rebase retry.');

        runGit(root, ['clone', remote, verifier]);
        const history = runGit(verifier, ['log', '--format=%s']).stdout;

        expect(history).toContain('Regenerate preview images');
        expect(history).toContain('Version bump');
        expect(
          readFileSync(
            path.join(verifier, 'docs/screenshots/example-app/example-app.png'),
            'utf8'
          )
        ).toBe('new preview image\n');
        expect(readFileSync(path.join(verifier, 'VERSION.txt'), 'utf8')).toBe(
          '1.0.1\n'
        );
      } finally {
        rmSync(root, { force: true, recursive: true });
      }
    });
  }
});
