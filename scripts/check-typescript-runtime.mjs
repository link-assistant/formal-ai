#!/usr/bin/env node
// Execute maintained TS source twins, including the actual VM worker inputs.
// Identity erasure applies only after plain-JS parsing and genuine regeneration.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';
const REPOSITORY = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const HELPERS = path.join(path.dirname(fileURLToPath(import.meta.url)), 'lib');
const digest = bytes => createHash('sha256').update(bytes).digest('hex');

export async function checkTypeScriptRuntime({root = REPOSITORY, translator} = {}) {
  const {parse} = await import(pathToFileURL(path.join(root, 'node_modules/@babel/parser/lib/index.js')));
  const projection = translator || await import(pathToFileURL(path.join(root, 'scripts/translate-es.mjs')));
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-typescript-runtime-'));
  try {
    const listed = execFileSync('git', ['ls-files', '-z', '--', 'js'],
      {cwd: root, encoding: 'utf8', maxBuffer: 2 * 1024 * 1024}).split('\0').filter(Boolean);
    const sources = {};
    for (const file of listed.filter(projection.isTranslatedSource)) {
      const logicalPath = 'ts/' + projection.tsTwinPath(file.slice(3));
      const target = path.join(root, logicalPath);
      const actual = fs.readFileSync(target, 'utf8');
      assert.equal(actual, projection.translateJsToTs(fs.readFileSync(path.join(root, file), 'utf8')),
        'genuine TypeScript projection: ' + logicalPath);
      parse(actual, {sourceType: 'unambiguous'});
      sources[path.join(root, file)] = {path: target, logicalPath, sha256: digest(actual)};
    }
    const parser = 'js/vendor/tree-sitter/web-tree-sitter.mjs';
    const sharedDependencies = {[path.join(root, parser)]: {
      path: path.join(root, parser), logicalPath: parser,
      sha256: digest(fs.readFileSync(path.join(root, parser)))}};
    const configuration = path.join(directory, 'manifest.json');
    const manifest = {root, sources, sharedDependencies, journal: path.join(directory, 'input-')};
    fs.writeFileSync(configuration, JSON.stringify(manifest), {mode: 0o600});
    const runtime = path.join(directory, 'runtime.mjs');
    fs.writeFileSync(runtime, runtimeFixture(root));
    const results = {};
    const execute = (mode, argumentsList) => {
      const home = path.join(directory, mode + '-home');
      const workspace = path.join(directory, mode + '-workspace');
      fs.mkdirSync(home, {recursive: true});
      fs.mkdirSync(workspace, {recursive: true});
      return spawnSync(process.execPath, argumentsList, {
        cwd: root, encoding: 'utf8', timeout: 60000, maxBuffer: 4 * 1024 * 1024,
        env: {...process.env, HOME: home, FORMAL_AI_MEMORY_PATH: path.join(home, 'memory.lino'),
          FORMAL_AI_LIVE_FETCH: '0', NODE_OPTIONS: '', FORMAL_AI_TYPESCRIPT_PROJECTION: configuration,
          QUALIFIED_WORKSPACE: workspace}});
    };
    const hooks = ['--require', path.join(HELPERS, 'typescript-projection-preload.cjs'),
      '--experimental-loader', path.join(HELPERS, 'typescript-projection-loader.mjs')];
    for (const mode of ['javascript', 'typescript']) {
      const observed = execute(mode, [...(mode === 'typescript' ? hooks : []), runtime]);
      assert.equal(observed.error, undefined, mode + ' process observation');
      assert.equal(observed.signal, null, mode + ' process signal');
      assert.equal(observed.status, 0, mode + ' runtime failed:\n' + observed.stderr);
      results[mode] = JSON.parse(observed.stdout.trim().split('\n').at(-1));
      for (const answer of results[mode].answers) assert.match(answer.content, /= 47$/u);
      assert.equal(results[mode].code, 'export const isolated = 61;');
      assert.equal(results[mode].stop, 'final');
    }
    assert.deepEqual(results.typescript, results.javascript);
    const rows = fs.readdirSync(directory).filter(file => /^input-.*\.jsonl$/u.test(file))
      .flatMap(file => fs.readFileSync(path.join(directory, file), 'utf8').trimEnd().split('\n')
        .filter(Boolean).map(line => JSON.parse(line)));
    for (const file of ['ts/server/worker-host.mts', 'ts/worker/formal_ai_worker.ts', 'ts/agentic/planner.mts']) {
      assert(rows.some(row => row.typescript === file), 'actual TypeScript input: ' + file);
    }
    const target = path.join(root, 'js/server/worker-host.mjs');
    const original = manifest.sources[target];
    assert(original);
    delete manifest.sources[target];
    fs.writeFileSync(configuration, JSON.stringify(manifest));
    const probe = ['--input-type=module', '-e', 'await import(' + JSON.stringify(pathToFileURL(target).href) + ')'];
    const missing = execute('missing', [...hooks, ...probe]);
    assert.notEqual(missing.status, 0);
    assert.match(missing.stderr, /TypeScriptRuntimeProjectionMissing/u);
    manifest.sources[target] = {...original, sha256: '0'.repeat(64)};
    fs.writeFileSync(configuration, JSON.stringify(manifest));
    const altered = execute('altered', [...hooks, ...probe]);
    assert.notEqual(altered.status, 0);
    assert.match(altered.stderr, /TypeScriptRuntimeProjectionChanged/u);
    return {schema: 'formal-ai-typescript-runtime-qualification-v1',
      compiler: 'checked identity erasure of annotation-free TypeScript',
      comparedResultsSha256: digest(JSON.stringify(results.javascript)),
      typeScriptInputOperations: rows.filter(row => row.typescript).length,
      uniqueTypeScriptInputs: new Set(rows.filter(row => row.typescript).map(row => row.typescript)).size,
      sharedDependencies: Object.keys(sharedDependencies).map(file => path.relative(root, file)),
      cases: ['three multilingual calculations with independent numeric oracle', 'real filesystem source write',
        'missing owned projection refuses despite existing JS source', 'altered TS identity refuses'],
      originalJavaScriptFallback: false, nativeExecution: false, productWasmExecution: false,
      authorshipCredit: 0, fullRuntimeFeatureParity: false};
  } finally {
    fs.rmSync(directory, {recursive: true, force: true});
  }
}

function runtimeFixture(root) {
  return [
    "import fs from 'node:fs';import {pathToFileURL} from 'node:url';",
    'const root=' + JSON.stringify(root) + ',workspace=process.env.QUALIFIED_WORKSPACE;',
    "const load=relative=>import(pathToFileURL(root+'/'+relative).href);",
    "const {WorkerHost}=await load('js/server/worker-host.mjs');",
    "const {installNodeHost}=await load('js/agentic/node-host.mjs');",
    "const {planChatStep}=await load('js/agentic/planner.mjs');",
    "const {drive}=await load('experiments/js_dogfood/drive.mjs');",
    "const worker=new WorkerHost({WebAssembly:{compile(){throw Error('ProductWasmDisabled');},",
    "instantiate(){throw Error('ProductWasmDisabled');},instantiateStreaming(){throw Error('ProductWasmDisabled');}}});",
    'await installNodeHost(worker);const answers=[];',
    "for(const prompt of ['What is 18 plus 29?','¿Cuánto es 18 más 29?','Сколько будет 18 плюс 29?'])",
    'answers.push(await worker.solve(prompt));',
    "const result=await drive(planChatStep,workspace,'Create file isolated.mjs containing export const isolated = 61;',{steps:12});",
    "console.log(JSON.stringify({answers,stop:result.stop,code:fs.readFileSync(workspace+'/isolated.mjs','utf8')}));",
  ].join('\n') + '\n';
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  console.log(JSON.stringify(await checkTypeScriptRuntime()));
}
