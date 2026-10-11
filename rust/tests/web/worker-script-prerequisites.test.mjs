import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import vm from 'node:vm';
import { dependencyRecords, renderWorkerPrerequisites } from '../../../scripts/generate-worker-prerequisites.mjs';
const parse = createRequire(new URL('../../../package.json', import.meta.url))('@babel/parser').parse;
const a = text => ({
    owner: 'js/worker/a.js',
    text
  }),
  b = text => ({
    owner: 'js/worker/b.js',
    text
  });
const compile = s => dependencyRecords(s, parse);
assert.equal(compile([a('function lazy(){return unknownDependency()}'), b('')]).length, 0);
assert.equal(compile([a('const text="// @worker-prerequisite absent.js";'), b('')]).length, 0);
const records = compile([a('// @worker-prerequisite js/worker/b.js\nconst a=1;'), b('const b=2;')]);
assert.equal(records.length, 1);
assert.equal(records[0].requires[0].script, 'js/worker/b.js');
assert.throws(() => compile([a('// @worker-prerequisite js/worker/b.js\n// @worker-prerequisite js/worker/b.js'), b('')]), /DuplicateWorkerPrerequisite/);
assert.throws(() => compile([a('// @worker-prerequisite js/worker/missing.js')]), /UnknownWorkerPrerequisite/);
assert.throws(() => compile([a('// @worker-prerequisite ../b.js'), b('')]), /MalformedWorkerPrerequisite/);
assert.throws(() => compile([a('/* @worker-prerequisite js/worker/b.js */'), b('')]), /MalformedWorkerPrerequisite/);
assert.throws(() => compile([a('// @worker-prerequisite js/worker/b.js'), b('// @worker-prerequisite js/worker/a.js')]), /CyclicWorkerPrerequisite/);
assert.throws(() => compile([a(''), a('')]), /DuplicateWorkerSource/);
const rendered = renderWorkerPrerequisites(records);
const context = vm.createContext({});
new vm.Script(rendered.browser).runInContext(context);
assert.equal(vm.runInContext('Object.isFrozen(formalAiWorkerScriptPrerequisites)&&Object.isFrozen(formalAiWorkerScriptPrerequisites[0])&&Object.isFrozen(formalAiWorkerScriptPrerequisites[0].requires)&&Object.isFrozen(formalAiWorkerScriptPrerequisites[0].requires[0])', context), true);
console.log('T10401_GENERATOR_TEN_CONTROLS_PASS');
import { createBrowserContext, loadBrowserScript, loadWorkerMirror, createWorkerContext } from '../../../js/server/worker-host.mjs';
const missing = createBrowserContext();
assert.throws(() => loadBrowserScript(missing, 'js/worker/formal_ai_worker_meta_synthesis.js'), /IncompleteWorkerScriptPrerequisite/);
const contextA = createBrowserContext(),
  contextB = createBrowserContext();
loadBrowserScript(contextA, 'js/seed_loader.js');
assert.throws(() => loadBrowserScript(contextB, 'js/worker/formal_ai_worker_meta_reasoner.js'), /IncompleteWorkerScriptPrerequisite/);
const entry = createWorkerContext();
assert.equal(vm.runInContext('typeof metaPrograms', entry), 'function');
const mirror = loadWorkerMirror();
assert.equal(vm.runInContext('typeof metaPrograms', mirror), 'function');
console.log('T10395_ORDINARY_ENTRY_MIRROR_CONTEXT_PASS');
