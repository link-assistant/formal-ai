import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { workflowJobs } from '../../../scripts/lib/ci-speed-workflows.mjs';
import { browserInventory, browserPlan, browserDurationModel, browserShardCount, collectBrowserShards, completedTestSummary, completeCoverageRecords, contentDigest } from '../../../scripts/lib/browser-coverage-shards.mjs';
const root = fileURLToPath(new URL('../../../', import.meta.url));
const identity = {
  source: 'source-one',
  run: 'run-one',
  attempt: '1',
  sourceDigest: 'sources-one'
};
const output = 'ℹ tests 1\nℹ pass 1\nℹ fail 0\nℹ cancelled 0\nℹ skipped 0\nℹ todo 0\nℹ duration_ms 1\n';
const report = 'TN:\nSF:js/a.mjs\nFN:1,cold\nFNDA:0,cold\nDA:1,0\nend_of_record\n';
const inventory = Array.from({
  length: 12
}, (_, index) => ({
  path: `rust/tests/web/fixture-${index}.test.mjs`,
  sha256: `source-${index}`,
  bytes: index + 1
}));
const plan = browserPlan(inventory, 6);
function packet(index, coverage = report, stdout = output, stderr = '') {
  return {
    coverage: Buffer.from(coverage),
    stdout: Buffer.from(stdout),
    stderr: Buffer.from(stderr),
    receipt: {
      ...identity,
      shard: index + 1,
      total: plan.shards.length,
      planDigest: plan.digest,
      selected: plan.shards[index],
      exitCode: 0,
      signal: null,
      sourceUnchanged: true,
      summary: completedTestSummary(stdout),
      coverageDigest: contentDigest(coverage),
      stdoutDigest: contentDigest(stdout),
      stderrDigest: contentDigest(stderr)
    }
  };
}
const packets = () => plan.shards.map((_, index) => packet(index));
test('the complete browser inventory partitions exactly once without test exclusions', () => {
  const observed = browserInventory(root);
  const expected = fs.readdirSync(path.join(root, 'rust/tests/web')).filter(name => name.endsWith('.test.mjs')).sort();
  assert.deepEqual(observed.map(item => path.basename(item.path)), expected);
  const selected = browserPlan(observed).shards.flat().sort();
  assert.deepEqual(selected, observed.map(item => item.path).sort());
  assert.equal(new Set(selected).size, observed.length);
});
test('measured browser work is balanced and begins longest first while retaining every file', () => {
  const measured = new Map(inventory.map((item, index) => [item.path, index + 1]));
  const weighted = browserPlan(inventory, 6, {
    seconds: measured,
    digest: 'measured-one'
  });
  assert.deepEqual(weighted.shards.flat().sort(), inventory.map(item => item.path).sort());
  assert(weighted.shards.every(shard => shard.every((item, index) => index === 0 || measured.get(shard[index - 1]) >= measured.get(item))));
  assert.deepEqual(weighted.shards.map(shard => shard.reduce((total, item) => total + measured.get(item), 0)), [13, 13, 13, 13, 13, 13]);
  const sameAssignmentNewSource = browserPlan(inventory, 6, {
    seconds: measured,
    digest: 'measured-two'
  });
  assert.deepEqual(weighted.shards, sameAssignmentNewSource.shards);
  assert.notEqual(weighted.digest, sameAssignmentNewSource.digest);
  const selected = packets();
  selected[0].receipt.planDigest = weighted.digest;
  assert.throws(() => collectBrowserShards(plan, identity, selected));
});
test('empty or duplicate test inventories cannot produce a coverage plan', () => {
  assert.throws(() => browserPlan([]));
  assert.throws(() => browserPlan([inventory[0], inventory[0]]));
  assert.throws(() => browserPlan(inventory, 13));
});
test('all original LCOV bytes and cold source declarations survive collection', () => {
  const selected = packets();
  const merged = collectBrowserShards(plan, identity, [...selected].reverse());
  assert.deepEqual(merged, Buffer.concat(selected.map(item => item.coverage)));
  assert.equal(merged.toString().split('FN:1,cold').length - 1, 6);
});
for (let index = 0; index < plan.shards.length; index += 1) {
  test(`missing or duplicate coverage shard ${index + 1} is refused`, () => {
    const missing = packets();
    missing.splice(index, 1);
    assert.throws(() => collectBrowserShards(plan, identity, missing));
    const duplicate = packets();
    duplicate[index] = duplicate[(index + 1) % duplicate.length];
    assert.throws(() => collectBrowserShards(plan, identity, duplicate));
  });
}
for (const key of ['source', 'run', 'attempt', 'sourceDigest']) {
  test(`foreign or stale ${key} cannot certify browser coverage`, () => {
    const selected = packets();
    selected[0].receipt[key] = 'foreign';
    assert.throws(() => collectBrowserShards(plan, identity, selected));
  });
}
for (const change of [receipt => {
  receipt.planDigest = 'stale';
}, receipt => {
  receipt.selected = [...receipt.selected, 'extra.test.mjs'];
}, receipt => {
  receipt.total += 1;
}, receipt => {
  receipt.exitCode = 7;
}, receipt => {
  receipt.signal = 'SIGTERM';
}, receipt => {
  receipt.sourceUnchanged = false;
}]) {
  test('changed plan, failed process and source mutation are refused', () => {
    const selected = packets();
    change(selected[0].receipt);
    assert.throws(() => collectBrowserShards(plan, identity, selected));
  });
}
for (const key of ['coverage', 'stdout', 'stderr']) {
  test(`counterfeit ${key} bytes cannot match an old receipt`, () => {
    const selected = packets();
    selected[0][key] = Buffer.from('counterfeit');
    assert.throws(() => collectBrowserShards(plan, identity, selected));
  });
}
test('missing completion, zero tests, cancellation and failing assertions remain failures', () => {
  for (const altered of ['', output.replace('tests 1', 'tests 0'), output.replace('fail 0', 'fail 1'), output.replace('cancelled 0', 'cancelled 1')]) {
    assert.throws(() => completedTestSummary(altered));
  }
  assert.throws(() => completeCoverageRecords(report.replace('end_of_record\n', '')));
  assert.throws(() => completeCoverageRecords('SF:one\nSF:two\nend_of_record\n'));
});
function coverageMetrics(text, directory) {
  const sources = new Map();
  let source;
  for (const line of text.split('\n')) {
    if (line.startsWith('SF:')) {
      source = path.resolve(directory, line.slice(3));
      if (!sources.has(source)) sources.set(source, {
        lines: new Map(),
        functions: new Map()
      });
    } else if (line === 'end_of_record') source = undefined;else if (source && line.startsWith('DA:')) {
      const [number, count] = line.slice(3).split(',');
      const lines = sources.get(source).lines;
      lines.set(number, Math.max(lines.get(number) ?? 0, Number(count)));
    } else if (source && line.startsWith('FN:')) {
      const name = line.slice(line.indexOf(',') + 1);
      const functions = sources.get(source).functions;
      if (!functions.has(name)) functions.set(name, 0);
    } else if (source && line.startsWith('FNDA:')) {
      const comma = line.indexOf(',');
      const name = line.slice(comma + 1);
      const functions = sources.get(source).functions;
      functions.set(name, Math.max(functions.get(name) ?? 0, Number(line.slice(5, comma))));
    }
  }
  return sources;
}
test('real Node coverage shards preserve the original whole-run source hit union', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'browser-coverage-union-'));
  try {
    const source = path.join(directory, 'source.mjs');
    fs.writeFileSync(source, 'export function first() { return 1; }\nexport function second() { return 2; }\nexport function cold() { return 3; }\n');
    for (const name of ['first', 'second']) {
      fs.writeFileSync(path.join(directory, `${name}.test.mjs`), `import test from 'node:test';\nimport assert from 'node:assert/strict';\nimport { ${name} } from './source.mjs';\ntest('${name}', () => assert.equal(${name}(), ${name === 'first' ? 1 : 2}));\n`);
    }
    const measure = (name, files) => {
      const destination = path.join(directory, `${name}.info`);
      const childEnvironment = {
        ...process.env
      };
      delete childEnvironment.NODE_TEST_CONTEXT;
      const result = spawnSync(process.execPath, ['--test', '--experimental-test-coverage', '--test-reporter=lcov', `--test-reporter-destination=${destination}`, '--test-reporter=spec', '--test-reporter-destination=stdout', ...files.map(file => path.join(directory, `${file}.test.mjs`))], {
        cwd: directory,
        encoding: 'utf8',
        env: childEnvironment
      });
      assert.equal(result.status, 0, result.stderr);
      assert.equal(result.signal, null);
      completedTestSummary(result.stdout);
      const bytes = fs.readFileSync(destination, 'utf8');
      completeCoverageRecords(bytes);
      return bytes;
    };
    const whole = coverageMetrics(measure('whole', ['first', 'second']), directory).get(source);
    const separate = coverageMetrics(measure('one', ['first']) + measure('two', ['second']), directory).get(source);
    assert(whole && separate, 'both reports must bind the actual physical source');
    const metrics = observed => ({
      lines: [...observed.lines].map(([number, count]) => [number, count > 0]),
      functions: [...observed.functions].map(([name, count]) => [name, count > 0])
    });
    // Node sums initialization counts across files; the unchanged ratchet
    // measures the exact source/line/function hit union, not execution counts.
    assert.notDeepEqual(separate.lines, whole.lines);
    assert.deepEqual(metrics(separate), metrics(whole));
    assert([...whole.functions.values()].includes(0), 'cold functions remain in the denominator');
  } finally {
    fs.rmSync(directory, {
      recursive: true,
      force: true
    });
  }
});
test('the hosted sixteen-way matrix and canonical producer/collector share the complete inventory and unchanged clock', () => {
  assert.equal(browserShardCount, 16);
  const workflow = fs.readFileSync('.github/workflows/coverage.yml', 'utf8');
  const job = workflowJobs(workflow).find(item => item.id === 'browser-coverage-shard');
  assert.ok(job);
  assert.equal(job.timeout, '15');
  const matrix = job.body.match(/shard: \[([^\]]+)\]/u);
  assert.ok(matrix);
  assert.deepEqual(matrix[1].split(',').map(value => Number(value.trim())), Array.from({
    length: browserShardCount
  }, (_, index) => index + 1));
  const observed = browserInventory(root);
  const model = browserDurationModel(root);
  const selected = browserPlan(observed, browserShardCount, model);
  assert.equal(selected.shards.length, 16);
  assert.ok(selected.shards.every(shard => shard.length > 0));
  assert.deepEqual(selected.shards.flat().sort(), observed.map(item => item.path).sort());
  assert.equal(new Set(selected.shards.flat()).size, observed.length);
  assert.deepEqual(browserPlan(observed, browserShardCount, model), selected);
  assert.deepEqual(browserPlan([...observed].reverse(), browserShardCount, model).shards, selected.shards);
});
test('sixteen complete synthetic receipts join every original LCOV byte and refuse every missing or duplicate partition', () => {
  const declared = Array.from({
    length: 64
  }, (_, index) => ({
    path: 'rust/tests/web/held-out-' + index + '.test.mjs',
    sha256: 'held-out-source-' + index,
    bytes: index + 1
  }));
  const selected = browserPlan(declared, 16);
  const supplied = selected.shards.map((paths, index) => {
    const coverage = Buffer.from('TN:\nSF:js/held-out-' + index + '.mjs\nFN:1,cold\nFNDA:0,cold\nDA:1,0\nend_of_record\n');
    const stdout = Buffer.from(output),
      stderr = Buffer.from('');
    return {
      coverage,
      stdout,
      stderr,
      receipt: {
        ...identity,
        shard: index + 1,
        total: 16,
        planDigest: selected.digest,
        selected: paths,
        exitCode: 0,
        signal: null,
        sourceUnchanged: true,
        summary: completedTestSummary(output),
        coverageDigest: contentDigest(coverage),
        stdoutDigest: contentDigest(stdout),
        stderrDigest: contentDigest(stderr)
      }
    };
  });
  assert.deepEqual(collectBrowserShards(selected, identity, [...supplied].reverse()), Buffer.concat(supplied.map(packet => packet.coverage)));
  for (let index = 0; index < 16; index += 1) {
    const missing = supplied.filter((_, position) => position !== index);
    assert.throws(() => collectBrowserShards(selected, identity, missing));
    const duplicate = [...supplied];
    duplicate[index] = supplied[(index + 1) % 16];
    assert.throws(() => collectBrowserShards(selected, identity, duplicate));
  }
  for (const [field, value] of [['shard', 0], ['shard', 17], ['shard', 1.5], ['total', 6], ['planDigest', 'stale'], ['source', 'foreign'], ['exitCode', 9], ['signal', 'SIGTERM'], ['sourceUnchanged', false]]) {
    const altered = supplied.map(packet => ({
      ...packet,
      receipt: {
        ...packet.receipt
      }
    }));
    altered[0].receipt[field] = value;
    assert.throws(() => collectBrowserShards(selected, identity, altered));
  }
  const extra = [...supplied, supplied[0]];
  assert.throws(() => collectBrowserShards(selected, identity, extra));
  for (const count of [0, -1, 1.5, NaN, Infinity, '16', 65]) {
    assert.throws(() => browserPlan(declared, count), /invalid browser shard count/);
  }
});
