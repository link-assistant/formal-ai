import assert from 'node:assert/strict';
import {test} from 'node:test';
import {mkdtempSync, writeFileSync, readFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

// Three tiny synthetic files exercise the real Node runner without loading
// project workers, native tools, or the repository's functional test suites.
test('ordered JavaScript runner preserves file order and inherited context and fails on a failed suite', () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-ordered-tests-'));
  const output = join(directory, 'observed.jsonl');
  const runner = resolve('scripts/run-ordered-javascript-tests.mjs');
  const first = join(directory, 'z-longest.test.mjs');
  const second = join(directory, 'a-shortest.test.mjs');
  const failure = join(directory, 'failure.test.mjs');
  const context = 'ordered-runner-inherited-context';
  function fixture(label) {
    return `import {test} from 'node:test';\nimport {appendFileSync} from 'node:fs';\ntest(${JSON.stringify(label)}, () => appendFileSync(process.env.ORDERED_TEST_OUTPUT, JSON.stringify({label:${JSON.stringify(label)},context:process.env.ORDERED_TEST_CONTEXT,args:process.execArgv}) + '\\n'));\n`;
  }
  const inheritedArguments = process.execArgv.filter(argument => argument !== '--test');
  const environment = {...process.env, ORDERED_TEST_OUTPUT: output, ORDERED_TEST_CONTEXT: context};
  // This subprocess models a top-level CI invocation, not a recursive test run.
  delete environment.NODE_TEST_CONTEXT;
  const options = {encoding: 'utf8', timeout: 8000, maxBuffer: 512 * 1024, env: environment};
  function assertSummaries(result, expectedFiles, expectedState) {
    const queued = result.stderr.split('\n').filter(line => line.startsWith('[test-file] queued '))
      .map(line => resolve(line.slice('[test-file] queued '.length)));
    assert.deepEqual(queued, expectedFiles, 'record the explicit queue once before Node can emit synchronous events');
    // Local Node20 checks order/environment/TODO/status; hosted Node22+ also qualifies summary events.
    if (Number(process.versions.node.split('.')[0]) < 22) return;
    const lines = result.stderr.split('\n').filter(line => /^\[test-file\] (completed|failed) /u.test(line));
    const summaries = lines.map(line => {
      const match = /^\[test-file\] (completed|failed) (.+) (\{.+\})$/u.exec(line);
      assert.ok(match, line);
      return {state: match[1], file: resolve(match[2]), duration: JSON.parse(match[3]).duration_ms};
    });
    assert.deepEqual(summaries.map(summary => summary.file), expectedFiles);
    assert.ok(summaries.every(summary => summary.state === expectedState));
    assert.ok(summaries.every(summary => Number.isFinite(summary.duration) && summary.duration >= 0));
  }
  try {
    writeFileSync(first, fixture('longest-first'));
    writeFileSync(second, fixture('shortest-second') + "\ntest('controlled TODO', {todo: true}, () => { throw Error('expected TODO'); });\n");
    writeFileSync(failure, "import {test} from 'node:test';\nimport assert from 'node:assert/strict';\ntest('controlled synthetic failure', () => assert.fail('expected synthetic failure'));\n");
    const passed = spawnSync(process.execPath, [...inheritedArguments, '--max-old-space-size=384', runner,
      '--concurrency', '1', first, second], options);
    assert.equal(passed.error, undefined);
    assert.equal(passed.status, 0, passed.stderr + passed.stdout);
    const observed = readFileSync(output, 'utf8').trim().split('\n').map(line => JSON.parse(line));
    assert.deepEqual(observed.map(row => row.label), ['longest-first', 'shortest-second']);
    assert.ok(observed.every(row => row.context === context));
    assert.ok(observed.every(row => row.args.includes('--max-old-space-size=384')));
    assert.match(passed.stdout, /fail 0/);
    assert.match(passed.stdout, /todo 1/);
    assertSummaries(passed, [first, second], 'completed');
    const failed = spawnSync(process.execPath, [...inheritedArguments, '--max-old-space-size=384', runner,
      '--concurrency', '1', failure], options);
    assert.equal(failed.error, undefined);
    assert.equal(failed.status, 1, failed.stderr + failed.stdout);
    assert.match(failed.stdout, /✖/);
    assert.match(failed.stdout, /expected synthetic failure/);
    assertSummaries(failed, [failure], 'failed');
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});

test('ordered runner records its queue when enqueue events precede listener attachment', {timeout: 12000}, () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-synchronous-enqueue-'));
  try {
    const runner = resolve('scripts/run-ordered-javascript-tests.mjs');
    const fixture = join(directory, 'finite.test.mjs');
    const loader = join(directory, 'enqueue-loader.mjs');
    writeFileSync(fixture, "import {test} from 'node:test';test('finite fixture', () => {});\n");
    // Keep the real runner and real test execution; model only the absent
    // later enqueue callback seen in the retained hosted job logs.
    const wrapper = "import {createRequire} from 'node:module';" +
      "const original=createRequire(process.cwd()+'/package.json')('node:test').run;" +
      "export function run(options){const stream=original(options),on=stream.on;" +
      "stream.on=function(event,listener){return event==='test:enqueue'?this:on.call(this,event,listener)};return stream;}";
    const url = 'data:text/javascript;base64,' + Buffer.from(wrapper).toString('base64');
    writeFileSync(loader, 'export function resolve(specifier,context,next){if(specifier==="node:test"&&context.parentURL===' +
      JSON.stringify(new URL('file://' + runner).href) + ')return {url:' + JSON.stringify(url) +
      ',shortCircuit:true};return next(specifier,context); }\n');
    const env = {...process.env}; delete env.NODE_TEST_CONTEXT;
    const result = spawnSync(process.execPath, ['--max-old-space-size=128', '--experimental-loader', loader,
      runner, '--concurrency', '1', fixture], {env, encoding: 'utf8', timeout: 8000, maxBuffer: 512 * 1024});
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stderr + result.stdout);
    const queued = result.stderr.split('\n').filter(line => line.startsWith('[test-file] queued '))
      .map(line => resolve(line.slice('[test-file] queued '.length)));
    assert.deepEqual(queued, [fixture]);
    assert.match(result.stdout, /finite fixture/u);
  } finally { rmSync(directory, {recursive: true, force: true}); }
});


test('ordered runner completes assertion-only files without confusing file-named cases with file results', {timeout:12000}, () => {
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-file-result-'));
  try {
    const first=join(directory,'assertions-only.test.mjs');
    const failure=join(directory,'throw-only.test.mjs');
    const named=join(directory,'file-named-cases.test.mjs');
    writeFileSync(first,"import assert from 'node:assert/strict'; assert.equal(2+2,4);\n");
    writeFileSync(failure,"throw Error('controlled top-level assertion failure');\n");
    writeFileSync(named,"import test from 'node:test'; test("+JSON.stringify(named)+",()=>{});"+
      "test('later case',async()=>{await new Promise(resolve=>setTimeout(resolve,60));});\n");
    const env={...process.env}; delete env.NODE_TEST_CONTEXT;
    const result=spawnSync(process.execPath,['--max-old-space-size=128',resolve('scripts/run-ordered-javascript-tests.mjs'),
      '--concurrency','1',first,named,failure],{env,encoding:'utf8',timeout:8000,maxBuffer:512*1024});
    assert.equal(result.error,undefined);
    assert.equal(result.status,1,result.stdout+result.stderr);
    const records=result.stderr.split('\n').flatMap(line=>{
      const match=/^\[test-file\] (completed|failed) (.+) (\{.+\})$/u.exec(line);
      return match?[{state:match[1],file:resolve(match[2]),duration:JSON.parse(match[3]).duration_ms}]:[];
    });
    assert.equal(records.length,3,result.stderr);
    assert.equal(new Set(records.map(row=>row.file)).size,3);
    assert.equal(records.find(row=>row.file===first)?.state,'completed');
    assert.equal(records.find(row=>row.file===failure)?.state,'failed');
    const actualFile=records.find(row=>row.file===named);
    assert.equal(actualFile?.state,'completed');
    assert(actualFile.duration>=60,'file completion includes the later case, not its file-named predecessor');
    assert(records.every(row=>Number.isFinite(row.duration)&&row.duration>=0));
    assert.match(result.stdout,/controlled top-level assertion failure/u);
  } finally {rmSync(directory,{recursive:true,force:true});}
});
