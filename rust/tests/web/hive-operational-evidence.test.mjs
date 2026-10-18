import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { externalEvidenceDirectory, retainExternalEvidence, adaptHiveOperationalSource, transferOperationalEvidenceOwner } from '../../../scripts/lib/hive-operational-evidence.mjs';
function fixture() {
  const temporary = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'formal-ai-operational-evidence-'));
  const repository = path.join(temporary, 'task');
  const storage = path.join(temporary, 'logs');
  fs.mkdirSync(repository);
  fs.mkdirSync(storage);
  const git = arguments_ => execFileSync('git', arguments_, {
    cwd: repository,
    encoding: 'utf8'
  }).trim();
  git(['init', '-q']);
  git(['config', 'user.name', 'Fixture']);
  git(['config', 'user.email', 'fixture@example.invalid']);
  fs.writeFileSync(path.join(repository, 'main.rs'), 'fn main() {}\n');
  git(['add', '--', 'main.rs']);
  git(['commit', '-qm', 'Task source']);
  return {
    temporary,
    repository,
    storage,
    git,
    cleanup: () => fs.rmSync(temporary, {
      recursive: true,
      force: true
    })
  };
}
test('operational evidence retains exact sanitized bytes outside task commits and preserves staged work', () => {
  const context = fixture();
  try {
    const {
      repository,
      storage,
      git
    } = context;
    fs.writeFileSync(path.join(repository, 'unrelated.txt'), 'unrelated staged work\n');
    git(['add', '--', 'unrelated.txt']);
    const indexBefore = git(['write-tree']);
    const headBefore = git(['rev-parse', 'HEAD']);
    const directory = externalEvidenceDirectory(repository, storage, 'attribution');
    const payload = '{"session_id":"held-out-session","text":"already sanitized complete stream"}\n';
    fs.writeFileSync(path.join(directory, 'claude-stream.jsonl'), payload);
    fs.writeFileSync(path.join(directory, 'session-id.txt'), 'held-out-session\n');
    const report = retainExternalEvidence({
      repositoryPath: repository,
      directory,
      task: {
        issueNumber: 9001,
        prNumber: 9002
      },
      session: {
        sessionId: 'held-out-session',
        model: 'formal-ai'
      }
    });
    assert.equal(git(['write-tree']), indexBefore);
    assert.equal(git(['rev-parse', 'HEAD']), headBefore);
    assert.equal(git(['ls-tree', '-r', '--name-only', 'HEAD']), 'main.rs');
    assert.equal(git(['status', '--porcelain']), 'A  unrelated.txt');
    assert.equal(fs.readFileSync(path.join(directory, 'claude-stream.jsonl'), 'utf8'), payload);
    assert.equal(report.files.find(file => file.path === 'claude-stream.jsonl').sha256, createHash('sha256').update(payload).digest('hex'));
    assert.equal(report.head, headBefore);
    assert.equal(report.commits[0].tree, git(['rev-parse', 'HEAD^{tree}']));
    assert.equal(report.legacySameCommitTreeAttribution, 'Unknown');
    assert.deepEqual(report.creditedCommits, []);
    assert.equal(report.authorshipCredit, 0);
    assert.deepEqual(JSON.parse(fs.readFileSync(report.manifest)).files, report.files);
  } finally {
    context.cleanup();
  }
});
test('owned external directory refuses task-local storage and aliases before any write', () => {
  const context = fixture();
  try {
    const absent = path.join(context.repository, 'absent', 'logs');
    assert.throws(() => externalEvidenceDirectory(context.repository, absent, 'attribution'), /outside task checkout/);
    assert.equal(fs.existsSync(path.dirname(absent)), false);
    const alias = path.join(context.temporary, 'alias');
    fs.symlinkSync(context.repository, alias);
    assert.throws(() => externalEvidenceDirectory(context.repository, path.join(alias, 'logs'), 'attribution'));
    assert.equal(fs.existsSync(path.join(context.repository, 'logs')), false);
    assert.throws(() => externalEvidenceDirectory(context.repository, context.storage, '../escape'));
  } finally {
    context.cleanup();
  }
});
test('manifest refuses symlink payload and option-shaped Git references without changing index', () => {
  const context = fixture();
  try {
    const directory = externalEvidenceDirectory(context.repository, context.storage, 'development-logs');
    fs.symlinkSync(path.join(context.repository, 'main.rs'), path.join(directory, 'foreign'));
    const before = context.git(['write-tree']);
    assert.throws(() => retainExternalEvidence({
      repositoryPath: context.repository,
      directory,
      task: {},
      session: {}
    }), /symlink/);
    assert.throws(() => retainExternalEvidence({
      repositoryPath: context.repository,
      directory,
      task: {},
      session: {},
      branchName: '--all'
    }), /invalid observed Git reference/);
    assert.equal(context.git(['write-tree']), before);
    assert.equal(fs.readdirSync(directory).length, 1);
  } finally {
    context.cleanup();
  }
});
test('unrecognized upstream source cannot receive a partial or silently weaker adapter', () => {
  assert.throws(() => adaptHiveOperationalSource('src/formal-ai-attribution.lib.mjs', 'export const copied = true;'), /unreviewed Hive/);
  assert.throws(() => adaptHiveOperationalSource('src/development-log.lib.mjs', ''), /unreviewed Hive/);
});
test('artifact owner transfer preserves private modes, contents, and exact subtree', () => {
  const context = fixture();
  try {
    const directory = path.join(context.storage, 'formal-ai-operational');
    fs.mkdirSync(directory, {
      mode: 0o700
    });
    const evidence = path.join(directory, 'session.jsonl');
    fs.writeFileSync(evidence, 'sanitized retained stream\n', {
      mode: 0o600
    });
    const outside = path.join(context.storage, 'outside.txt');
    fs.writeFileSync(outside, 'outside', {
      mode: 0o600
    });
    const result = transferOperationalEvidenceOwner({
      logDirectory: context.storage,
      userIdentity: process.getuid(),
      groupIdentity: process.getgid()
    });
    assert.equal(result.transferred, 2);
    assert.equal(fs.statSync(directory).mode & 0o777, 0o700);
    assert.equal(fs.statSync(evidence).mode & 0o777, 0o600);
    assert.equal(fs.readFileSync(evidence, 'utf8'), 'sanitized retained stream\n');
    assert.equal(fs.readFileSync(outside, 'utf8'), 'outside');
    fs.symlinkSync(outside, path.join(directory, 'alias'));
    assert.throws(() => transferOperationalEvidenceOwner({
      logDirectory: context.storage,
      userIdentity: process.getuid(),
      groupIdentity: process.getgid()
    }), /symlink/);
    assert.equal(fs.statSync(outside).mode & 0o777, 0o600);
    assert.throws(() => transferOperationalEvidenceOwner({
      logDirectory: context.storage,
      userIdentity: -1,
      groupIdentity: process.getgid()
    }), /invalid artifact owner/);
  } finally {
    context.cleanup();
  }
});
