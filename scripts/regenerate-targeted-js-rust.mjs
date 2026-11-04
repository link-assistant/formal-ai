import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {Worker} from 'node:worker_threads';

assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Translation regeneration is hosted-only');
const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const upstream = path.resolve(process.argv[2] ?? '.meta-language');
const pin = 'eb0574d5528babd3b4aadece81590d8e2230b99b';
const target = 'js/agentic/crate/event_log.mjs';
const dependency = 'js/agentic/crate/engine_stable_identifier.mjs';
const sha256 = text => createHash('sha256').update(text).digest('hex');
const read = file => fs.readFileSync(path.join(repo, file), 'utf8');
const gitHead = directory => execFileSync('git', ['rev-parse', 'HEAD'], {
  cwd: directory, encoding: 'utf8', timeout: 10000, maxBuffer: 4096
}).trim();
const sourceCommit = gitHead(repo);
assert.match(sourceCommit, /^[a-f0-9]{40}$/u);
assert.equal(sourceCommit, process.env.GITHUB_SHA, 'Checkout differs from the dispatched source');
assert.match(process.env.GITHUB_RUN_ID ?? '', /^[1-9][0-9]*$/u);
assert.match(process.env.GITHUB_RUN_ATTEMPT ?? '', /^[1-9][0-9]*$/u);
assert.equal(gitHead(upstream), pin);
assert(fs.existsSync(path.join(upstream, 'js/node_modules')),
  'Install pinned runtime dependencies in the hosted workflow first');

const regenerationTarget = process.env.FORMAL_AI_REGENERATION_TARGET ?? 'translations';
assert(['translations', 'specification-witness', 'debug-owner-baseline', 'module-projection'].includes(regenerationTarget), 'Unknown regeneration target');
if (regenerationTarget === 'debug-owner-baseline') {
  const {regenerateDebugOwnerBaseline} = await import(
    pathToFileURL(path.join(repo, 'scripts/lib/regenerate-debug-owner-baseline.mjs')));
  await regenerateDebugOwnerBaseline({repo, sourceCommit, pin});
  process.exit(0);
}
if (regenerationTarget === 'specification-witness') {
  const {regenerateSpecificationWitness} = await import(
    pathToFileURL(path.join(repo, 'scripts/lib/regenerate-governed-specification.mjs')));
  await regenerateSpecificationWitness({repo, sourceCommit, pin});
  process.exit(0);
}

if (regenerationTarget === 'module-projection') {
  const {regenerateModuleProjection} = await import(
    pathToFileURL(path.join(repo, 'scripts/lib/regenerate-module-projection.mjs')));
  await regenerateModuleProjection({repo, sourceCommit, pin, upstream,
    module: process.env.FORMAL_AI_REGENERATION_MODULE});
  process.exit(0);
}

const api = await import(pathToFileURL(path.join(repo, 'scripts/translate-js-rust.mjs')));
const {namedImports, resolveImport} = await import(
  pathToFileURL(path.join(repo, 'scripts/lib/translation-workarounds.mjs')));
const {renderBlockers} = await import(
  pathToFileURL(path.join(repo, 'scripts/lib/translation-blockers.mjs')));
const ledgerBefore = read(api.LEDGER_FILE);
const committed = api.parseLedger(ledgerBefore);
assert.equal(committed.commit, pin);
const scope = api.scopeModules();
assert(scope.includes(target) && scope.includes(dependency));
const inScope = new Set(scope);
const untouched = new Map(scope.filter(module => module !== target)
  .map(module => [api.projectionPath(module), read(api.projectionPath(module))]));
const targetSource = read(target);
const dependencySource = read(dependency);
assert.equal(sha256(targetSource),
  '93f46625e595dda0ca2c7ad603243eb4d077ce2e8f76713bd170dea661c28fbf',
  'Only the reviewed five-kind event-log change is admitted');
assert.equal(namedImports(dependencySource).length, 0, 'Dependency closure changed');
const targetImports = namedImports(targetSource);
assert.deepEqual(targetImports.map(entry => resolveImport(target, entry.specifier, inScope)),
  [dependency], 'Target import closure changed');
const addedKinds = "\n  'quantity_constraint:frame', 'quantity_constraint:verdict', 'quantity_constraint:unknown',"
  + "\n  'quantity_constraint:expression', 'quantity_constraint:difference',";
assert.equal(targetSource.split(addedKinds).length, 2, 'Reviewed insertion must occur once');
const oldSource = targetSource.replace(addedKinds, '');
assert.equal(sha256(oldSource),
  '017a625906d04f554c9fa97649d333efaafd7e95893d519867fd1c7bdadc56cb',
  'Reversing the insertion must recover the exact frozen old source');
const inputFiles = [target, dependency, 'scripts/regenerate-targeted-js-rust.mjs',
  'scripts/translate-js-rust.mjs', 'data/meta/translation-single-pass-adapter.lino'];
const inputs = inputFiles.map(file => ({path: file, sha256: sha256(read(file))}));
const worker = new Worker(path.join(repo, 'scripts/translate-js-rust.mjs'), {
  workerData: {role: 'translate', dir: upstream}
});

function translate(module, text, imports = {}, bound = {}) {
  return new Promise((resolve, reject) => {
    const finish = (error, message) => {
      clearTimeout(timer);
      worker.off('message', onMessage);
      worker.off('error', onError);
      worker.off('exit', onExit);
      if (error) reject(error);
      else resolve(message);
    };
    const onMessage = message => finish(message?.path === module
      ? null : new Error('TargetedTranslationResponseMismatch'), message);
    const onError = error => finish(error);
    const onExit = () => finish(new Error('TargetedTranslationWorkerExited'));
    const timer = setTimeout(() => finish(new Error('TargetedTranslationWorkerTimeout')), 60000);
    worker.once('message', onMessage);
    worker.once('error', onError);
    worker.once('exit', onExit);
    try { worker.postMessage({path: module, text, imports, bound, signatures: true}); }
    catch (error) { finish(error); }
  });
}

let measured;
try {
  const dependencyResult = await translate(dependency, dependencySource);
  const measuredDependency = api.measure(dependency, {...dependencyResult, text: dependencySource});
  assert.equal(measuredDependency.projection, untouched.get(api.projectionPath(dependency)),
    'Actual dependency translation must exactly reproduce its unchanged committed projection');
  assert(Array.isArray(dependencyResult.signatures), 'Actual exported signatures required');
  const imports = {}, bound = {};
  for (const entry of targetImports) {
    imports[entry.specifier] = dependencyResult.signatures;
    bound[entry.specifier] = dependencyResult.signatures.map(item => item.name);
  }
  const oldResult = await translate(target, oldSource, imports, bound);
  const measuredOld = api.measure(target, {...oldResult, text: oldSource});
  assert.equal(measuredOld.projection, read(api.projectionPath(target)),
    'Actual old-source translation must reproduce its committed projection');
  const currentResult = await translate(target, targetSource, imports, bound);
  assert(Array.isArray(oldResult.signatures) && Array.isArray(currentResult.signatures),
    'Both actual target translations must return exported-signature arrays');
  assert.deepEqual(currentResult.signatures, oldResult.signatures,
    'Unchanged importer projections require actual unchanged exported signatures');
  measured = api.measure(target, {...currentResult, text: targetSource});
  console.log(JSON.stringify({phase: 'three-real-translations', dependency,
    dependencyMs: dependencyResult.ms, oldTargetMs: oldResult.ms,
    newTargetMs: currentResult.ms, exportedSignaturesUnchanged: true}));
} finally {
  await worker.terminate();
}

const modules = scope.map(module => module === target ? measured
  : {...api.readProjection(read(api.projectionPath(module))), path: module,
    projection: read(api.projectionPath(module))});
assert.deepEqual(api.regressions(committed.modules, modules), []);
for (const module of modules.filter(module => module.path !== target)) {
  const old = committed.modules.find(row => row.path === module.path);
  assert(old, 'Every untouched module has a committed row');
  for (const key of ['translated', 'workaround', 'inferred', 'carried'])
    assert.equal(module[key], old[key], 'Untouched ledger measurement');
}
const ledger = api.renderLedger(committed.commit, modules);
const blockers = renderBlockers(modules.flatMap(module => module.blockers.map((reasons, index) => ({
  module: module.path, reason: module.refusals[index], blockers: reasons
}))));
const outputs = new Map([[api.projectionPath(target), measured.projection],
  [api.LEDGER_FILE, ledger], [api.BLOCKERS_FILE, blockers]]);
assert.equal(outputs.size, 3, 'Only one projection plus the generated ledger and blockers');
const changed = [];
for (const [file, content] of outputs) {
  if (!fs.existsSync(path.join(repo, file)) || read(file) !== content) {
    fs.writeFileSync(path.join(repo, file), content);
    changed.push(file);
  }
}
assert.deepEqual(api.verify(repo), [], 'Actual generated source projections must verify');
assert.deepEqual(api.scopeModules(), scope, 'Translation source inventory changed');
for (const [file, content] of untouched)
  assert.equal(read(file), content, 'Untouched projection bytes');
for (const input of inputs)
  assert.equal(sha256(read(input.path)), input.sha256, 'Source input changed during regeneration');

const output = path.join(repo, 'translation-regeneration');
assert(!fs.existsSync(output), 'Regeneration output must start absent');
const files = [...outputs.keys()].map(file => ({path: file,
  bytes: Buffer.byteLength(read(file)), sha256: sha256(read(file))}));
assert(files.every(file => file.bytes <= 262144), 'Generated source file size bound');
assert(files.reduce((total, file) => total + file.bytes, 0) <= 524288,
  'Generated source artifact size bound');
const manifest = {source: sourceCommit, upstream: committed.commit,
  run: process.env.GITHUB_RUN_ID, attempt: process.env.GITHUB_RUN_ATTEMPT,
  inputs, oldSourceSha256: sha256(oldSource), exportedSignaturesUnchanged: true, files};
for (const file of outputs.keys()) {
  const destination = path.join(output, file);
  fs.mkdirSync(path.dirname(destination), {recursive: true});
  fs.copyFileSync(path.join(repo, file), destination);
}
fs.writeFileSync(path.join(output, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
console.log(JSON.stringify({phase: 'artifact-ready', changed, manifest}, null, 2));
