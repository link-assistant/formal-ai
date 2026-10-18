// Hosted adapter for pinned Hive. External evidence is retained, never counted as same-tree attribution.
import fs from 'node:fs';
import path from 'node:path';
import { createHash, randomUUID } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
export const HIVE_EVIDENCE_PIN = '8f22b87c515adece0427cb59e429531f35d7760a';
const HOSTED_SOURCE_IDENTITY = null;
const SOURCE_BLOBS = {
  'src/formal-ai-attribution.lib.mjs': '333cbf96af3ca5b77cb2b357b948be8b022f41c8',
  'src/development-log.lib.mjs': '5f4cc9fca7987f8f5998c6538dcac516e9b6ef30'
};
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const replaceOnce = (source, original, replacement) => {
  assert.equal(source.split(original).length, 2, 'unrecognized pinned Hive use site');
  return source.replace(original, replacement);
};

/** Caller-owned storage, outside the task checkout; do not derive authority from model output. */
export function externalEvidenceDirectory(repositoryPath, storageRoot, category) {
  assert.ok(path.isAbsolute(storageRoot), 'absolute operational storage required');
  assert.match(category, /^[a-z][a-z-]*$/);
  const repository = fs.realpathSync(repositoryPath);
  let ancestor = path.resolve(storageRoot);
  const absent = [];
  while (!fs.existsSync(ancestor)) {
    absent.unshift(path.basename(ancestor));
    ancestor = path.dirname(ancestor);
  }
  assert.equal(fs.lstatSync(ancestor).isSymbolicLink(), false, 'operational storage symlink');
  const canonical = path.join(fs.realpathSync(ancestor), ...absent);
  const relative = path.relative(repository, canonical);
  assert.ok(relative && (relative === '..' || relative.startsWith('..' + path.sep) || path.isAbsolute(relative)), 'operational storage must be outside task checkout');
  // Check all existing path components, including an alias below an otherwise safe ancestor.
  let component = path.parse(path.resolve(storageRoot)).root;
  for (const segment of path.resolve(storageRoot).slice(component.length).split(path.sep).filter(Boolean)) {
    component = path.join(component, segment);
    if (fs.existsSync(component)) assert.equal(fs.lstatSync(component).isSymbolicLink(), false, 'operational storage symlink');
  }
  const directory = path.join(canonical, category, randomUUID());
  fs.mkdirSync(directory, {
    recursive: true,
    mode: 0o700
  });
  return directory;
}
function inventory(directory) {
  const files = [];
  const pending = [''];
  while (pending.length) {
    const relative = pending.pop();
    for (const entry of fs.readdirSync(path.join(directory, relative), {
      withFileTypes: true
    })) {
      const name = path.join(relative, entry.name);
      const absolute = path.join(directory, name);
      assert.equal(entry.isSymbolicLink(), false, 'evidence symlink');
      if (entry.isDirectory()) {
        pending.push(name);
        continue;
      }
      assert.ok(entry.isFile(), 'regular evidence files required');
      const handle = fs.openSync(absolute, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
      const before = fs.fstatSync(handle);
      const hash = createHash('sha256');
      const chunk = Buffer.alloc(65536);
      try {
        let count;
        while ((count = fs.readSync(handle, chunk, 0, chunk.length, null)) !== 0) hash.update(chunk.subarray(0, count));
        const after = fs.fstatSync(handle);
        assert.equal(after.size, before.size);
        assert.equal(after.mtimeMs, before.mtimeMs, 'evidence changed while hashing');
      } finally {
        fs.closeSync(handle);
      }
      files.push({
        path: name.split(path.sep).join('/'),
        bytes: before.size,
        sha256: hash.digest('hex')
      });
    }
  }
  return files.sort((left, right) => left.path.localeCompare(right.path, 'en'));
}

/** Read-only Git observation. This records custody; it does not prove authorship or causal edit coverage. */
export function retainExternalEvidence({
  repositoryPath,
  directory,
  task,
  session,
  baseRef = null,
  branchName = 'HEAD'
}) {
  const git = arguments_ => execFileSync('git', arguments_, {
    cwd: repositoryPath,
    encoding: 'utf8',
    timeout: 30000,
    maxBuffer: 4 * 1024 * 1024
  }).trim();
  for (const reference of [baseRef, branchName].filter(Boolean)) {
    assert.ok(typeof reference === 'string' && reference.length > 0 && !reference.startsWith('-') && !/[\r\n\0]/.test(reference), 'invalid observed Git reference');
  }
  const head = git(['rev-parse', '--verify', branchName + '^{commit}']);
  const range = baseRef ? baseRef + '..' + head : head;
  const commits = git(['rev-list', '--reverse', range]).split('\n').filter(Boolean).map(commit => ({
    commit,
    tree: git(['rev-parse', commit + '^{tree}']),
    parents: git(['show', '-s', '--format=%P', commit]).split(' ').filter(Boolean)
  }));
  const report = {
    schema: 'formal-ai-hosted-external-evidence/v1',
    upstreamCommit: HIVE_EVIDENCE_PIN,
    upstreamSourceBlobs: SOURCE_BLOBS,
    formalAiSource: HOSTED_SOURCE_IDENTITY,
    task,
    session,
    head,
    tree: git(['rev-parse', head + '^{tree}']),
    commits,
    files: inventory(directory),
    storage: 'external-operational-artifact',
    legacySameCommitTreeAttribution: 'Unknown',
    creditedCommits: [],
    causalEditCoverage: 'not-established',
    authorshipCredit: 0
  };
  const manifest = path.join(directory, 'external-evidence-' + randomUUID() + '.json');
  fs.writeFileSync(manifest, JSON.stringify(report, null, 2) + '\n', {
    flag: 'wx',
    mode: 0o600
  });
  return {
    manifest,
    ...report
  };
}

/** Exact pinned source only. Original sanitizers, identity rejection, and default upstream files remain inspectable. */
export function adaptHiveOperationalSource(name, source, {
  storageRoot = '/home/box/logs/formal-ai-operational'
} = {}) {
  const bytes = Buffer.from(source);
  assert.equal(createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex'), SOURCE_BLOBS[name], 'unreviewed Hive operational source');
  assert.ok(path.isAbsolute(storageRoot));
  const storage = JSON.stringify(storageRoot);
  const imported = "import {externalEvidenceDirectory, retainExternalEvidence} from '../scripts/formal-ai-hive-operational-evidence.mjs';\n";
  let result = source.replace(/^(#![^\n]*\n)?/, '$&' + imported);
  if (name === 'src/development-log.lib.mjs') {
    result = replaceOnce(result, 'const copyKnownSessionFiles = async ({ repositoryPath, sessionRelativeDirectory,', 'const copyKnownSessionFiles = async ({ repositoryPath, storageRoot = repositoryPath, sessionRelativeDirectory,');
    // Keep the original repository for Claude session lookup; redirect only destination writes.
    assert.equal(result.split('const sessionDirectory = path.join(repositoryPath, sessionRelativeDirectory);').length, 3);
    result = result.replace('const sessionDirectory = path.join(repositoryPath, sessionRelativeDirectory);', 'const sessionDirectory = path.join(storageRoot, sessionRelativeDirectory);');
    const start = result.indexOf('export const writeDevelopmentLogArtifacts = async');
    const end = result.indexOf('const verifyDevelopmentLogDirectory', start);
    const writer = result.slice(start, end);
    let redirected = replaceOnce(writer, '({ repositoryPath, logFile,', '({ repositoryPath, storageRoot = repositoryPath, logFile,');
    redirected = redirected.replaceAll('path.join(repositoryPath,', 'path.join(storageRoot,');
    redirected = replaceOnce(redirected, 'const sessionFiles = await copyKnownSessionFiles({\n    repositoryPath,', 'const sessionFiles = await copyKnownSessionFiles({\n    repositoryPath,\n    storageRoot,');
    result = result.slice(0, start) + redirected + result.slice(end);
    const hook = "  // Issue #2048: verbose trace so the commit timing (relative to PR readiness signals) is diagnosable from logs.";
    const collectorStart = result.indexOf(hook);
    assert.ok(collectorStart > 0 && result.trimEnd().endsWith('};'));
    result = result.slice(0, collectorStart) + ('  const storageRoot = externalEvidenceDirectory(repositoryPath, ' + storage + ", 'development-logs');\n" + '  const artifacts = await writeDevelopmentLogArtifacts({repositoryPath, storageRoot, logFile, issueNumber, prNumber, tool, sessionId, branchName, rawCommand, logStartByte});\n' + '  await verifyDevelopmentLogDirectory(path.join(storageRoot, artifacts.relativeDirectory));\n' + '  const evidence = retainExternalEvidence({repositoryPath, directory: storageRoot, task: {issueNumber, prNumber, branchName}, session: {tool, sessionId}});\n' + "  await log?.('Development log retained in external operational storage; same-tree attribution Unknown');\n" + '  return {...artifacts, committed: false, pushed: false, externalEvidence: evidence};\n};\n');
  } else {
    result = replaceOnce(result, 'state.stagingDir = path.join(gitDir, ATTRIBUTION_STAGING_DIRNAME);', 'state.stagingDir = externalEvidenceDirectory(repositoryPath, ' + storage + ", 'attribution');");
    const first = result.indexOf('      const installed = await installAttributionHooks({', result.indexOf('export const createFormalAiAttributionSession'));
    const last = result.indexOf('      state.enabled = true;', first);
    assert.ok(first > 0 && last > first);
    // Do not install staging/trailer hooks. Existing user hooks continue through the unchanged process environment.
    result = result.slice(0, first) + '      state.gitEnv = {};\n' + result.slice(last);
    const marker = "    async finalize({ branchName = null, baseRef = null, remote = 'origin' } = {}) {";
    const external = marker + '\n' + '      await flush();\n' + '      if (!state.stagingDir) return {enabled: false, reason: state.rejection, attributed: [], malformed: []};\n' + '      const externalEvidence = retainExternalEvidence({repositoryPath, directory: state.stagingDir, task: {issueNumber, prNumber, pullRequest: state.prUrl}, session: {sessionId: state.sessionId, version, model, tool, rejection: state.rejection}, baseRef, branchName: branchName || "HEAD"});\n' + '      await log("Formal AI evidence retained externally; legacy same-tree attribution Unknown/uncredited");\n' + '      return {enabled: state.enabled, reason: state.rejection || "external-evidence-not-same-tree", attributed: [], malformed: [], sessionId: state.sessionId, records: state.written, externalEvidence};\n';
    const finalizeStart = result.indexOf(marker);
    const finalizeEnd = result.indexOf('\n    },\n  };\n};', finalizeStart);
    assert.ok(finalizeStart > 0 && finalizeEnd > finalizeStart);
    result = result.slice(0, finalizeStart) + external + result.slice(finalizeEnd);
    result = replaceOnce(result, 'Formal AI attribution enabled (${buildModelTrailerValue(version)})', 'Formal AI external evidence enabled (${buildModelTrailerValue(version)}); same-tree attribution Unknown');
    result = replaceOnce(result, 'Commit hooks:    ${state.hooksDir}', 'External evidence: ${state.stagingDir}');
  }
  return result;
}
export function prepareHiveOperationalEvidence({
  hive,
  root
}) {
  const helperSource = fs.readFileSync(path.join(root, 'scripts/lib/hive-operational-evidence.mjs'), 'utf8');
  const sourceIdentity = {
    commit: execFileSync('git', ['rev-parse', 'HEAD'], {
      cwd: root,
      encoding: 'utf8',
      timeout: 30000
    }).trim(),
    tree: execFileSync('git', ['rev-parse', 'HEAD^{tree}'], {
      cwd: root,
      encoding: 'utf8',
      timeout: 30000
    }).trim()
  };
  assert.equal(helperSource.match(/^const HOSTED_SOURCE_IDENTITY = null;$/gm)?.length, 1);
  const helper = Buffer.from(helperSource.replace(/^const HOSTED_SOURCE_IDENTITY = null;$/m, 'const HOSTED_SOURCE_IDENTITY = ' + JSON.stringify(sourceIdentity) + ';'));
  const reports = [];
  // Qualify every source before any adapter write.
  const prepared = Object.keys(SOURCE_BLOBS).map(name => {
    const original = fs.readFileSync(path.join(hive, name), 'utf8');
    const adapted = adaptHiveOperationalSource(name, original);
    reports.push({
      path: name,
      originalSha256: digest(original),
      adaptedSha256: digest(adapted),
      upstreamBlob: SOURCE_BLOBS[name]
    });
    return {
      name,
      adapted
    };
  });
  fs.writeFileSync(path.join(hive, 'scripts/formal-ai-hive-operational-evidence.mjs'), helper, {
    flag: 'wx'
  });
  for (const {
    name,
    adapted
  } of prepared) fs.writeFileSync(path.join(hive, name), adapted);
  fs.writeFileSync(path.join(hive, '.formal-ai-operational-adapter.json'), JSON.stringify({
    schema: 'formal-ai-reviewed-hive-operational-adapter/v1',
    upstreamCommit: HIVE_EVIDENCE_PIN,
    helperSha256: digest(helper),
    originalHelperSha256: digest(helperSource),
    formalAiSource: sourceIdentity,
    sources: reports,
    storageRoot: '/home/box/logs/formal-ai-operational',
    originalOracleFilesChanged: false,
    taskChanged: false,
    legacySameCommitTreeAttribution: 'Unknown',
    authorshipCredit: 0
  }, null, 2) + '\n', {
    flag: 'wx'
  });
}

/** Hosted owner transfer only: preserve private modes and refuse every symlink before changing any owner. */
export function transferOperationalEvidenceOwner({
  logDirectory,
  userIdentity,
  groupIdentity
}) {
  assert.ok(path.isAbsolute(logDirectory), 'absolute hosted log directory required');
  for (const identity of [userIdentity, groupIdentity]) assert.ok(Number.isSafeInteger(identity) && identity >= 0 && identity <= 4294967294, 'invalid artifact owner');
  const directory = path.join(logDirectory, 'formal-ai-operational');
  try {
    fs.lstatSync(directory);
  } catch (error) {
    if (error.code === 'ENOENT') return {
      transferred: 0,
      absent: true
    };
    throw error;
  }
  assert.equal(fs.realpathSync(logDirectory), path.resolve(logDirectory), 'aliased hosted log directory');
  const pending = [directory];
  const entries = [];
  while (pending.length) {
    const absolute = pending.pop();
    const observed = fs.lstatSync(absolute);
    assert.equal(observed.isSymbolicLink(), false, 'operational ownership symlink');
    assert.ok(observed.isFile() || observed.isDirectory(), 'regular operational evidence required');
    if (observed.isFile()) assert.equal(observed.nlink, 1, 'operational evidence hard link');
    entries.push({
      absolute,
      device: observed.dev,
      inode: observed.ino,
      mode: observed.mode & 0o777
    });
    if (observed.isDirectory()) for (const entry of fs.readdirSync(absolute)) pending.push(path.join(absolute, entry));
  }
  // The hosted container has exited. Preflight the complete exact subtree before any transfer.
  for (const entry of entries) {
    const observed = fs.lstatSync(entry.absolute);
    assert.equal(observed.dev, entry.device);
    assert.equal(observed.ino, entry.inode);
    assert.equal(observed.isSymbolicLink(), false);
    fs.lchownSync(entry.absolute, userIdentity, groupIdentity);
    assert.equal(fs.lstatSync(entry.absolute).mode & 0o777, entry.mode, 'private evidence mode changed');
  }
  return {
    transferred: entries.length,
    absent: false,
    privateModesPreserved: true,
    owner: {
      userIdentity,
      groupIdentity
    }
  };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'hosted ownership boundary only');
  assert.equal(process.argv.length, 6);
  assert.equal(process.argv[2], '--transfer-ownership');
  for (const identity of process.argv.slice(4)) assert.match(identity, /^(0|[1-9][0-9]*)$/);
  console.log(JSON.stringify(transferOperationalEvidenceOwner({
    logDirectory: process.argv[3],
    userIdentity: Number(process.argv[4]),
    groupIdentity: Number(process.argv[5])
  })));
}
