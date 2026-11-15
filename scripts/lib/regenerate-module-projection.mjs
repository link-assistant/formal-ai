import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';

// The existing translator owns import signatures, CST rendering and ratchets.
// This hosted wrapper bounds its named-module closure and artifact authority.
export async function regenerateModuleProjection({repo, sourceCommit, pin, upstream, module}) {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Hosted-only projection regeneration');
  assert.equal(process.env.RUNNER_OS, 'Linux');
  assert.match(sourceCommit, /^[a-f0-9]{40}$/u);
  assert.match(pin, /^[a-f0-9]{40}$/u);
  assert.match(process.env.GITHUB_RUN_ID ?? '', /^[1-9][0-9]*$/u);
  assert.match(process.env.GITHUB_RUN_ATTEMPT ?? '', /^[1-9][0-9]*$/u);
  assert.equal(process.env.GITHUB_SHA, sourceCommit);
  assert.equal(fs.realpathSync(repo), repo, 'Canonical checkout root required');
  const sha = bytes => createHash('sha256').update(bytes).digest('hex');
  const readSizes = new Map();
  const read = file => {
    assert.equal(typeof file, 'string');
    assert(!path.isAbsolute(file) && !file.includes('\\') && !file.includes('\0'));
    assert(file.split('/').every(part => part && part !== '.' && part !== '..'));
    const absolute = path.join(repo, file);
    const stat = fs.lstatSync(absolute);
    assert(stat.isFile() && !stat.isSymbolicLink() && stat.size <= 1048576,
      'Bounded regular source required: ' + file);
    assert.equal(fs.realpathSync(absolute), absolute, 'Input symlink traversal refused');
    readSizes.set(file, stat.size);
    assert([...readSizes.values()].reduce((sum, size) => sum + size, 0) <= 67108864,
      'Selected source and projection reads exceed 64MiB');
    return fs.readFileSync(absolute);
  };
  const git = args => execFileSync('git', args, {
    cwd: repo, encoding: 'utf8', timeout: 10000, maxBuffer: 1048576
  }).trim();
  assert.equal(git(['rev-parse', 'HEAD']), sourceCommit);
  assert.equal(git(['diff', '--name-only', 'HEAD', '--']), '', 'Clean committed source required');
  const api = await import(pathToFileURL(path.join(repo, 'scripts/translate-js-rust.mjs')));
  const {importGraph, translationClosure} = await import(
    pathToFileURL(path.join(repo, 'scripts/lib/translation-workarounds.mjs')));
  const scope = api.scopeModules(repo);
  assert(scope.length > 0 && scope.length <= 1024, 'Bounded tracked module inventory');
  assert.equal(typeof module, 'string');
  assert(scope.includes(module), 'Target must be an exact tracked translation module');
  const ledgerBefore = api.parseLedger(read(api.LEDGER_FILE).toString('utf8'));
  assert.equal(ledgerBefore.commit, pin);
  const sources = new Map(scope.map(file => [file, read(file).toString('utf8')]));
  const closure = translationClosure([module], importGraph(sources));
  assert(closure.includes(module) && closure.length <= scope.length);
  const projectionFiles = scope.map(api.projectionPath);
  const outputFiles = [api.projectionPath(module), api.LEDGER_FILE, api.BLOCKERS_FILE];
  const before = new Map([...projectionFiles, api.LEDGER_FILE, api.BLOCKERS_FILE]
    .map(file => [file, read(file)]));
  const tooling = ['scripts/regenerate-targeted-js-rust.mjs',
    'scripts/lib/regenerate-module-projection.mjs', 'scripts/translate-js-rust.mjs',
    'scripts/lib/rust-projection-presentation.mjs', 'scripts/lib/translation-single-pass.mjs',
    'scripts/lib/translation-blockers.mjs', 'scripts/lib/translation-lowering.mjs',
    'scripts/lib/translation-workarounds.mjs', 'data/meta/translation-workarounds.lino',
    'scripts/self-translation/lexer.mjs', 'scripts/check-readable-code.mjs',
    'data/meta/translation-single-pass-adapter.lino',
    api.CALLS_FILE, '.github/workflows/layered-ci.yml'];
  const inputFiles = [...new Set([...scope, ...tooling])].sort();
  assert(inputFiles.length <= 1024, 'Bounded selected input inventory');
  const inputs = inputFiles.map(file => {
    const bytes = read(file);
    const committed = execFileSync('git', ['show', sourceCommit + ':' + file], {
      cwd: repo, timeout: 10000, maxBuffer: 1048576
    });
    assert.equal(sha(bytes), sha(committed), 'Input differs from its immutable Git blob: ' + file);
    return {path: file, bytes: bytes.length, sha256: sha(bytes)};
  });
  assert.deepEqual(api.verify(repo), [module + ' changed since its translation'],
    'Only the requested source projection may initially be stale');
  console.log(JSON.stringify({phase: 'module-translation-plan', source: sourceCommit,
    upstream: pin, module, closure, targetSource: inputs.find(input => input.path === module)}));
  // GNU timeout owns the child process group (do not use --foreground), so
  // its finite TERM/KILL deadline also reaps worker-thread translation work.
  const command = ['timeout', '--signal=TERM', '--kill-after=5s', '420s', process.execPath,
    '--max-old-space-size=384', 'scripts/translate-js-rust.mjs', '--write',
    '--meta-language', upstream, '--jobs', '1', module];
  execFileSync(command[0], command.slice(1), {
    cwd: repo, stdio: 'inherit', timeout: 430000, killSignal: 'SIGKILL'
  });
  assert.deepEqual(api.scopeModules(repo), scope, 'Module inventory changed');
  for (const input of inputs)
    assert.equal(sha(read(input.path)), input.sha256, 'Immutable input changed: ' + input.path);
  for (const file of projectionFiles.filter(file => file !== outputFiles[0]))
    assert.deepEqual(read(file), before.get(file), 'Non-target projection changed: ' + file);
  const changed = git(['diff', '--name-only', 'HEAD', '--']).split('\n').filter(Boolean).sort();
  assert(changed.every(file => outputFiles.includes(file)), 'Unapproved output changed');
  assert(changed.includes(outputFiles[0]), 'Requested projection must actually be regenerated');
  const after = api.parseLedger(read(api.LEDGER_FILE).toString('utf8'));
  assert.equal(after.commit, pin);
  assert.deepEqual(api.regressions(ledgerBefore.modules, after.modules), []);
  assert.deepEqual(after.modules.filter(row => row.path !== module),
    ledgerBefore.modules.filter(row => row.path !== module), 'Non-target ledger rows changed');
  assert.deepEqual(api.verify(repo), [], 'Complete generated projections and census must verify');
  const files = outputFiles.map(file => ({path: file, bytes: read(file).length,
    beforeSha256: sha(before.get(file)), sha256: sha(read(file))}));
  assert(files.every(file => file.bytes <= 262144));
  assert(files.reduce((sum, file) => sum + file.bytes, 0) <= 524288);
  const output = path.join(repo, 'translation-regeneration');
  assert(!fs.existsSync(output), 'Artifact directory must start absent');
  const manifest = {kind: 'module-projection', source: sourceCommit, upstream: pin, module, closure,
    run: process.env.GITHUB_RUN_ID, attempt: process.env.GITHUB_RUN_ATTEMPT,
    selectedInputsScope: 'All tracked translator-scope modules and the listed local implementation/contracts; '
      + 'the source commit binds other repository reads and the upstream pin binds upstream implementation/runtime inputs.',
    rustCompiled: false, runtimeQualified: false, command,
    deadline: {termMs: 420000, killAfterMs: 5000, outerMs: 430000},
    inputs, changed, files, before: ledgerBefore.modules.find(row => row.path === module),
    after: after.modules.find(row => row.path === module), verified: true,
    unchangedProjections: projectionFiles.filter(file => file !== outputFiles[0])
      .map(file => ({path: file, sha256: sha(before.get(file))}))};
  const manifestBytes = Buffer.from(JSON.stringify(manifest, null, 2) + '\n');
  assert(manifestBytes.length <= 262144, 'Bounded source/hash manifest');
  assert(files.reduce((sum, file) => sum + file.bytes, manifestBytes.length) <= 1048576,
    'Complete source artifact including manifest exceeds 1MiB');
  for (const file of outputFiles) {
    const destination = path.join(output, file);
    fs.mkdirSync(path.dirname(destination), {recursive: true});
    fs.copyFileSync(path.join(repo, file), destination);
  }
  fs.writeFileSync(path.join(output, 'manifest.json'), manifestBytes);
  console.log(JSON.stringify({phase: 'module-artifact-ready', manifest}));
}
