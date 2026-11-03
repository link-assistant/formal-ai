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
    const failed = spawnSync(process.execPath, [...inheritedArguments, '--max-old-space-size=384', runner,
      '--concurrency', '1', failure], options);
    assert.equal(failed.error, undefined);
    assert.equal(failed.status, 1, failed.stderr + failed.stdout);
    assert.match(failed.stdout, /✖/);
    assert.match(failed.stdout, /expected synthetic failure/);
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});
