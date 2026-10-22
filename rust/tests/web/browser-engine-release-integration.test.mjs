import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync, mkdtempSync, writeFileSync, rmSync, existsSync } from 'node:fs';
import { tmpdir as temporaryDirectory } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { remainingSeconds, deadlineArguments, runBoundedOperation } from '../../../scripts/browser-engine-deadline.mjs';
import { publicationStatus } from '../../../scripts/publish-browser-engine.mjs';
import { RELEASE_PROTOCOL_FILES } from '../../../scripts/release-protocol.mjs';
const YAML = createRequire(import.meta.url)('yaml');
const original = JSON.parse(readFileSync(new URL('./browser-engine-original-operations.json', import.meta.url), 'utf8'));
const workflow = YAML.parse(readFileSync(new URL('../../../.github/workflows/publish-engine.yml', import.meta.url), 'utf8'));
test('browser dependency isolation is immutable and remains inside the original setup deadline',()=>{
  const step=workflow.jobs.package.steps.find(value=>value.name==='Prepare locked browser smoke dependencies');
  assert.equal(step['timeout-minutes'],5);
  const playwright='node "$FORMAL_AI_RELEASE_PROTOCOL_DIR/scripts/browser-engine-deadline.mjs" setup bash "$FORMAL_AI_RELEASE_PROTOCOL_DIR/scripts/with-ubuntu-apt-sources.sh" node vscode/node_modules/playwright/cli.js install chromium --with-deps';
  assert(step.run.includes(playwright));
  assert(RELEASE_PROTOCOL_FILES.engine.includes('scripts/prepare-ubuntu-apt-mirror.sh'));
  assert(RELEASE_PROTOCOL_FILES.engine.includes('scripts/with-ubuntu-apt-sources.sh'));
  assert.doesNotMatch(step.run,/allow-unauthenticated|trusted=yes|no-check-certificate/u);
});
const environment = {
  CI: 'true',
  GITHUB_JOB: 'package',
  GITHUB_RUN_ID: '42',
  GITHUB_RUN_ATTEMPT: '1',
  GITHUB_WORKFLOW_SHA: 'a'.repeat(40)
};
const clock = {
  schema: 'browser-engine-job-clock/v1',
  started: 100000,
  ...environment
};
function checkStructure(value) {
  assert.equal(value.jobs.package['timeout-minutes'], 20);
  const {
    actions,
    ...originalPermissions
  } = value.jobs.package.permissions;
  assert.equal(actions, 'read');
  assert.deepEqual(originalPermissions, original.packagePermissions);
  const boundedResolve = structuredClone(original.resolve);
  boundedResolve.env = {
    GIT_HTTP_LOW_SPEED_LIMIT: '1024',
    GIT_HTTP_LOW_SPEED_TIME: '30'
  };
  Object.assign(boundedResolve.steps[0].with, {
    'sparse-checkout': 'scripts/resolve-package-release.mjs',
    'sparse-checkout-cone-mode': false
  });
  assert.deepEqual(value.jobs.resolve, boundedResolve);
  assert.deepEqual(value.concurrency, original.concurrency);
  assert.deepEqual(value.permissions, original.permissions);
  const steps = value.jobs.package.steps;
  const find = name => {
    const index = steps.findIndex(step => step.name === name);
    assert(index >= 0, name);
    return index;
  };
  assert.equal(find('Start the absolute package job clock'), 0);
  assert.equal(steps[0].id, 'clock');
  assert.match(steps[0].run, /GITHUB_OUTPUT/u);
  for (const step of steps.filter(step => step.run?.includes('browser-engine-deadline.mjs'))) {
    assert.equal(step.env?.ENGINE_CLOCK_SHA256, '${{ steps.clock.outputs.sha256 }}');
  }
  const compile = find('Compile the selected source to WASM and record its complete inputs');
  const prepare = find('Prepare web and engine artifacts');
  assert(compile < prepare);
  assert.equal(steps[compile].env.RUSTUP_TOOLCHAIN, '1.99.0');
  assert.match(steps[compile].run, /build-selected-wasm\.mjs" build/u);
  assert.match(steps[find('Install the pinned selected-source WASM compiler')].run, /toolchain install 1\.99\.0 --profile minimal --target wasm32-unknown-unknown/u);
  const verify = find('Verify the fresh WASM and packaged bytes with a private receipt');
  assert(prepare < verify);
  assert.doesNotMatch(steps[verify].run, /cp .*receipt/u);
  assert.match(steps[verify].run, /assets" "\$WASM_RECEIPT_SHA256" "\$GITHUB_WORKSPACE\/js\/formal_ai_worker\.receipt\.json"/u);
  assert.equal(steps[verify].run.match(/build-selected-wasm\.mjs" verify/gu).length, 2);
  assert(verify < find('Package the engine'));
  const local = find('Cold install the packed engine before public publication');
  assert(find('Package the engine') < local);
  assert(local < find('Attach installable engine package to the release'));
  assert.equal(steps[local]['timeout-minutes'], 8);
  assert.equal(steps[local].env.WASM_RECEIPT_SHA256, '${{ steps.wasm.outputs.receipt-sha256 }}');
  assert.match(steps[local].run, /deadline\.mjs" cold node/u);
  assert.match(steps[local].run, /"\u0024WASM_RECEIPT_SHA256" checks-only/u);
  const publication = steps[find('Publish with provenance')];
  assert.equal(publication.id, 'publication');
  assert.equal(publication.if, "needs.resolve.outputs.publish == 'true'");
  const {
    ENGINE_CLOCK_SHA256,
    ...originalEnvironment
  } = publication.env;
  assert.equal(ENGINE_CLOCK_SHA256, '${{ steps.clock.outputs.sha256 }}');
  assert.deepEqual(originalEnvironment, original.publicationEnvironment);
  assert.match(publication.run, /status=skipped-unconfigured/u);
  assert.match(publication.run, /release-protocol\.mjs" verify/u);
  const checks = steps[find('Cold install and verify the genuine browser worker in checks-only mode')];
  assert.equal(checks.if, "needs.resolve.outputs.publish != 'true'");
  assert.equal(checks['timeout-minutes'], 8);
  assert.equal(checks.env.ENGINE_CLOCK_SHA256, '${{ steps.clock.outputs.sha256 }}');
  assert.equal(checks.env.WASM_RECEIPT_SHA256, '${{ steps.wasm.outputs.receipt-sha256 }}');
  assert.equal(checks.env.PUBLICATION_STATUS, '${{ steps.publication.outputs.status }}');
  assert.equal(checks.env.PUBLICATION_REQUESTED, '${{ needs.resolve.outputs.publish }}');
  assert.match(checks.run, /PUBLICATION_STATUS=checks-only/u);
  assert.match(checks.run, /browser-engine-deadline\.mjs" cold node/u);
  assert.match(checks.run, /browser-engine-cold-install\.mjs/u);
  assert.match(checks.run, /"\u0024WASM_RECEIPT_SHA256" "\u0024PUBLICATION_STATUS"/u);
  assert.doesNotMatch(checks.run, /browser-engine-public-download/u);
  const cold = steps[find('Cold install and verify the genuine browser worker')];
  assert.equal(cold['timeout-minutes'], 8);
  assert.equal(cold.env.PUBLICATION_STATUS, '${{ steps.publication.outputs.status }}');
  assert.equal(cold.env.WASM_RECEIPT_SHA256, '${{ steps.wasm.outputs.receipt-sha256 }}');
  assert.match(cold.run, /deadline\.mjs" cold node/u);
  assert.equal(cold.if, "needs.resolve.outputs.publish == 'true'");
  assert.match(cold.run, /browser-engine-public-download\.mjs/u);
  assert.doesNotMatch(cold.run, /archives\[|packages\/formal-ai-engine\/\*\.tgz/u);
  const durable = steps[find('Publish original engine worker and protocol receipts as durable data')];
  assert.match(durable.run, /worker-receipt\.json/);
  assert.match(durable.run, /protocol-selection\.json/);
  assert(find(durable.name) < find(cold.name));
  assert(find('Publish with provenance') < find(cold.name));
  let previous = -1;
  for (const operation of original.steps) {
    const current = operation.name ? steps.find(step => step.name === operation.name) : steps.find(step => step.uses === operation.uses && JSON.stringify(step.with) === JSON.stringify(operation.with));
    assert(current, 'missing original operation ' + JSON.stringify(operation));
    const currentPosition = steps.indexOf(current);
    assert(currentPosition > previous, 'original operation order');
    previous = currentPosition;
    for (const [key, value] of Object.entries(operation)) if (key !== 'run') {
      if (key === 'env') {
        const {
          ENGINE_CLOCK_SHA256,
          ...preserved
        } = current.env;
        assert.deepEqual(preserved, value);
      } else assert.deepEqual(current[key], value);
    }
    if (operation.run) {
      let normalized = current.run.replaceAll('node "$FORMAL_AI_RELEASE_PROTOCOL_DIR/scripts/browser-engine-deadline.mjs" prepare ',
   '').replaceAll('node "$FORMAL_AI_RELEASE_PROTOCOL_DIR/scripts/browser-engine-deadline.mjs" pack ',
   '').replaceAll('node "$FORMAL_AI_RELEASE_PROTOCOL_DIR/scripts/browser-engine-deadline.mjs" publish ',
   '').replace('            echo "status=skipped-unconfigured" >> "$GITHUB_OUTPUT"\n',
   '');
      // YAML removes scalar indentation; retain the original logical shell line exactly.
      normalized = normalized.replace('  echo "status=skipped-unconfigured" >> "$GITHUB_OUTPUT"\n', '');
      assert.equal(normalized, operation.run);
    }
  }
}
test('original operations and authority remain with selected-source compilation and bounded cold install', () => checkStructure(workflow));
test('missing source build, changed compiler, missing receipt, credential drift and unbounded smoke refuse', () => {
  const mutations = [value => value.jobs.package.steps.splice(0,
   1),
   value => value.jobs.package.steps.find(step => step.id === 'wasm').env.RUSTUP_TOOLCHAIN = 'stable',
   value => value.jobs.package.steps.find(step => step.name === 'Verify the fresh WASM and packaged bytes with a private receipt').run = 'true',
   value => value.jobs.package.steps.find(step => step.id === 'publication').env.NPM_TRUSTED_PUBLISHING = 'true',
   value => value.jobs.package.steps.at(-1)['timeout-minutes'] = 9,
    value => value.jobs.package.steps.at(-1).env.PUBLICATION_STATUS = 'published',
    value => value.jobs.package.steps.find(step => step.name === 'Cold install and verify the genuine browser worker in checks-only mode').if = "needs.resolve.outputs.publish == 'true'",
    value => value.jobs.package.steps.find(step => step.name === 'Cold install and verify the genuine browser worker in checks-only mode').run = 'true'];
  for (const mutate of mutations) {
    const value = structuredClone(workflow);
    mutate(value);
    assert.throws(() => checkStructure(value));
  }
});
test('sparse resolver retains every original authority field and exact checkout stall bounds', () => {
  const mutations = [
    value => delete value.jobs.resolve.env,
    value => value.jobs.resolve.env.GIT_HTTP_LOW_SPEED_LIMIT = '0',
    value => value.jobs.resolve.env.GIT_HTTP_LOW_SPEED_TIME = '300',
    value => delete value.jobs.resolve.steps[0].with['sparse-checkout'],
    value => value.jobs.resolve.steps[0].with['sparse-checkout'] = '**',
    value => value.jobs.resolve.steps[0].with['sparse-checkout-cone-mode'] = true,
    value => value.jobs.resolve.steps[0].with['persist-credentials'] = true,
    value => value.jobs.resolve.steps[0].with.ref = 'untrusted',
    value => value.jobs.resolve['timeout-minutes'] = 10,
    value => value.jobs.resolve.if = 'true',
    value => value.jobs.resolve.outputs.publish = 'true',
    value => value.jobs.resolve.steps.at(-1).env.GH_TOKEN = 'other',
    value => value.jobs.resolve.steps.at(-1).run = 'true'
  ];
  for (const mutate of mutations) {
    const value = structuredClone(workflow);
    mutate(value);
    assert.throws(() => checkStructure(value));
  }
});
test('immutable engine closure includes every authority-bearing new executable', () => {
  for (const path of ['scripts/build-selected-wasm.mjs', 'scripts/browser-engine-deadline.mjs', 'scripts/browser-engine-cold-install.mjs']) assert(RELEASE_PROTOCOL_FILES.engine.includes(path));
});
test('fixed job clock refuses expired, future, spoofed run, attempt, caller and workflow identity', () => {
  assert.equal(remainingSeconds(clock, environment, 101000), 1194);
  for (const alter of [{
    CI: 'false'
  }, {
    GITHUB_JOB: 'resolve'
  }, {
    GITHUB_RUN_ID: '43'
  }, {
    GITHUB_RUN_ATTEMPT: '2'
  }, {
    GITHUB_WORKFLOW_SHA: 'b'.repeat(40)
  }]) {
    assert.throws(() => remainingSeconds(clock, {
      ...environment,
      ...alter
    }, 101000));
  }
  assert.throws(() => remainingSeconds(clock, environment, 1300000));
  assert.throws(() => remainingSeconds(clock, environment, 99999));
  assert.throws(() => deadlineArguments(1200, ['node']));
  assert.deepEqual(deadlineArguments(475, ['node', 'fixture']), ['--signal=TERM', '--kill-after=5s', '475s', 'node', 'fixture']);
});
test('publication status is derived only from exact source-owned success result', () => {
  assert.equal(publicationStatus({
    publish: true
  }), 'published');
  assert.equal(publicationStatus({
    publish: false
  }), 'already-matching');
  for (const value of [undefined, {}, 'published', {
    publish: 'true'
  }]) assert.throws(() => publicationStatus(value));
});
test('real harmless Node processes preserve success, failure, output and kill a timed-out descendant', t => {
  const directory = mkdtempSync(join(temporaryDirectory(), 'engine-deadline-controls-'));
  t.after(() => rmSync(directory, {
    recursive: true,
    force: true
  }));
  const actualEnvironment = {
    ...process.env,
    ...environment,
    RUNNER_TEMP: directory
  };
  const writeClock = () => {
    const bytes = JSON.stringify({
      ...clock,
      started: Date.now() - 1193000
    });
    writeFileSync(join(directory, 'browser-engine-job-clock.json'), bytes);
    actualEnvironment.ENGINE_CLOCK_SHA256 = createHash('sha256').update(bytes).digest('hex');
  };
  writeClock();
  assert.equal(runBoundedOperation('cold', [process.execPath, '-e', 'console.log("actual node success")'], {
    environment: actualEnvironment
  }), 0);
  writeClock();
  assert.equal(runBoundedOperation('cold', [process.execPath, '-e', 'process.exit(17)'], {
    environment: actualEnvironment
  }), 17);
  const childFile = join(directory, 'child.json');
  const child = 'require("node:fs").writeFileSync(' + JSON.stringify(childFile) + ',JSON.stringify(process.pid));setInterval(()=>{},100);';
  const parent = 'require("node:child_process").spawn(process.execPath,["-e",' + JSON.stringify(child) + '],{stdio:"inherit"});setInterval(()=>{},100);';
  writeClock();
  assert.equal(runBoundedOperation('cold', [process.execPath, '-e', parent], {
    environment: actualEnvironment
  }), 124);
  assert(existsSync(childFile));
  const identifier = JSON.parse(readFileSync(childFile, 'utf8'));
  const observationStarted = Date.now();
  let gone = false;
  while (Date.now() - observationStarted < 3000) {
    try {
      process.kill(identifier, 0);
    } catch (error) {
      assert.equal(error.code, 'ESRCH');
      gone = true;
      break;
    }
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10);
  }
  assert(gone, 'timed-out exact descendant remains alive; missing process permission never proves teardown');
});
import { mkdirSync, symlinkSync, unlinkSync, realpathSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { specification, inspectInstalled, validateLock, safeAsset } from '../../../scripts/browser-engine-cold-install.mjs';
test('inert package field fixture preserves exact receipt and refuses source, bytes, lock and path drift', t => {
  const directory = mkdtempSync(join(temporaryDirectory(), 'engine-package-fields-'));
  t.after(() => rmSync(directory, {
    recursive: true,
    force: true
  }));
  const version = '0.352.1',
    wasm = Buffer.from([0, 97, 115, 109, 1, 0, 0, 0]);
  const record = {
    schema: 'selected-source-wasm/v1',
    sourceCommit: 'a'.repeat(40),
    packageVersion: version,
    nativeCompilationEvidence: true,
    output: {
      bytes: wasm.length,
      sha256: createHash('sha256').update(wasm).digest('hex')
    }
  };
  function put(path, bytes) {
    const target = join(directory, path);
    mkdirSync(join(target, '..'), {
      recursive: true
    });
    writeFileSync(target, bytes);
  }
  put('package.json', JSON.stringify({
    name: '@link-assistant/formal-ai-engine',
    version,
    type: 'module',
    exports: {
      '.': {
        import: './src/index.js'
      }
    }
  }));
  for (const name of ['src/index.js', 'assets/memory.js', 'assets/worker/formal_ai_worker.js', 'assets/worker-modules.js', 'assets/seed-files.js']) {
    put(name, '// Inert schema fixture. This is never executed as a worker or native build proof.');
  }
  put('assets/seed-files.js', 'self.FORMAL_AI_SEED_FILES = Object.freeze(["seed/test.lino"]);\n');
  put('assets/formal_ai_worker.wasm', wasm);
  put('assets/seed/test.lino', 'meanings\n');
  assert.equal(inspectInstalled(directory, version, record).length, 1);
  assert.equal(specification(version), '@link-assistant/formal-ai-engine@' + version);
  for (const bad of ['latest', '0.352.1-beta', '0.352.1;true', 'file:///tmp', '1']) assert.throws(() => specification(bad));
  for (const change of [{
    nativeCompilationEvidence: false
  }, {
    sourceCommit: 'spoof'
  }, {
    packageVersion: '0.0.0'
  }, {
    output: {
      ...record.output,
      sha256: 'b'.repeat(64)
    }
  }, {
    output: {
      ...record.output,
      bytes: 9
    }
  }]) {
    assert.throws(() => inspectInstalled(directory, version, {
      ...record,
      ...change
    }));
  }
  const integrity = 'sha512-explicit-fixture',
    entry = {
      version,
      integrity,
      resolved: 'https://registry.npmjs.org/@link-assistant/formal-ai-engine/-/formal-ai-engine-' + version + '.tgz'
    };
  const lock = change => ({
    lockfileVersion: 3,
    packages: {
      'node_modules/@link-assistant/formal-ai-engine': {
        ...entry,
        ...change
      }
    }
  });
  assert.deepEqual(validateLock(lock({}), version, integrity), entry);
  for (const change of [{
    version: '0.0.0'
  }, {
    integrity: 'spoof'
  }, {
    resolved: 'https://evil.test/archive'
  }, {
    resolved: entry.resolved + '?token=secret'
  }, {
    resolved: entry.resolved + '#other'
  }, {
    resolved: entry.resolved.replace('https://', 'https://user:password@')
  }, {
    resolved: entry.resolved.replace(version, '0.0.0')
  }]) {
    assert.throws(() => validateLock(lock(change), version, integrity));
  }
  assert.equal(safeAsset(directory, '/src/index.js'), realpathSync(join(directory, 'src/index.js')));
  for (const value of ['/package.json', '/assets/../../package.json', '/assets/%2e%2e/%2e%2e/etc/passwd', '/assets/%00bad', '/assets/%zz', '/assets/\\etc', '/assets/missing']) {
    assert.equal(safeAsset(directory, value), null);
  }
  symlinkSync(join(directory, 'src/index.js'), join(directory, 'assets/alias.js'));
  assert.equal(safeAsset(directory, '/assets/alias.js'), null);
  assert.throws(() => inspectInstalled(directory, version, record));
  unlinkSync(join(directory, 'assets/alias.js'));
  unlinkSync(join(directory, 'assets/memory.js'));
  assert.throws(() => inspectInstalled(directory, version, record));
});
test('caller environment cannot extend fixed process-group or absolute job bounds', () => {
  const directory = mkdtempSync(join(temporaryDirectory(), 'engine-clock-fields-'));
  try {
    writeFileSync(join(directory, 'browser-engine-job-clock.json'), JSON.stringify(clock));
    let observed;
    const run = (command, argumentsList) => {
      observed = {
        command,
        argumentsList
      };
      return {
        status: 0,
        signal: null
      };
    };
    const value = {
      ...environment,
      RUNNER_TEMP: directory,
      ENGINE_TIMEOUT: '999999',
      ENGINE_COLD_SECONDS: '999999',
      ENGINE_CLOCK_SHA256: createHash('sha256').update(JSON.stringify(clock)).digest('hex')
    };
    assert.equal(runBoundedOperation('cold', ['node', 'declared-fixture'], {
      environment: value,
      run,
      now: 100000
    }), 0);
    assert.deepEqual(observed.argumentsList, deadlineArguments(475, ['node', 'declared-fixture']));
    assert.equal(runBoundedOperation('setup', ['node', 'declared-fixture'], {
      environment: value,
      run,
      now: 100000
    }), 0);
    assert.deepEqual(observed.argumentsList, deadlineArguments(295, ['node', 'declared-fixture']));
    assert.throws(() => runBoundedOperation('unknown', ['node'], {
      environment: value,
      run,
      now: 100000
    }));
    for (const digest of ['', '0'.repeat(64)]) assert.throws(() => runBoundedOperation('cold', ['node'], {
      environment: {
        ...value,
        ENGINE_CLOCK_SHA256: digest
      },
      run,
      now: 100000
    }));
    writeFileSync(join(directory, 'browser-engine-job-clock.json'), JSON.stringify({
      ...clock,
      started: 200000
    }));
    assert.throws(() => runBoundedOperation('cold', ['node'], {
      environment: value,
      run,
      now: 200000
    }));
  } finally {
    rmSync(directory, {
      recursive: true,
      force: true
    });
  }
});

test('locked npm request recovery fits inside the unchanged setup deadline and timeout remains failure', () => {
  const directory = mkdtempSync(join(temporaryDirectory(), 'engine-npm-deadline-'));
  try {
    const bytes = JSON.stringify(clock);
    writeFileSync(join(directory, 'browser-engine-job-clock.json'), bytes);
    const events = [];
    let calls = 0;
    const status = runBoundedOperation('setup', ['npm', 'ci', 'fixture-hidden-argument'], {
      environment: { ...environment, RUNNER_TEMP: directory,
        ENGINE_CLOCK_SHA256: createHash('sha256').update(bytes).digest('hex'),
        PRIVATE_FIXTURE_HINT: 'fixture-hidden-environment' },
      now: clock.started,
      log: record => events.push(record),
      run: () => { calls++; return { status: 124, signal: null }; }
    });
    assert.equal(status, 124);
    assert.equal(calls, 1, 'a failed install is not retried as a whole process');
    assert.deepEqual(events.map(event => event.event), ['started', 'finished']);
    assert(events.every(event => event.mode === 'setup' && event.program === 'npm'));
    assert.equal(events[0].deadlineSeconds, 295);
    assert.equal(events[1].status, 124);
    assert.equal(events[1].spawnError, false);
    assert(Number.isInteger(events[1].elapsedMilliseconds) && events[1].elapsedMilliseconds >= 0);
    assert.doesNotMatch(JSON.stringify(events), /fixture-hidden/u);

    const setup = workflow.jobs.package.steps.find(step => step.name === 'Prepare locked browser smoke dependencies');
    assert.equal(setup['timeout-minutes'], 5);
    assert.match(setup.run, /npm ci --prefix vscode --ignore-scripts/u);
    const number = name => {
      const value = new RegExp('--' + name + '=(\\d+)').exec(setup.run);
      assert(value, 'an explicit request bound is required: ' + name);
      return Number(value[1]);
    };
    const timeout = number('fetch-timeout'), retries = number('fetch-retries');
    const minimum = number('fetch-retry-mintimeout'), maximum = number('fetch-retry-maxtimeout');
    assert(timeout > 0 && retries <= 1 && maximum >= minimum && minimum > 0);
    assert(timeout * (retries + 1) + maximum * retries < events[0].deadlineSeconds * 1000);
    // npm's documented default single request already exceeds this phase.
    assert(300000 > events[0].deadlineSeconds * 1000);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
