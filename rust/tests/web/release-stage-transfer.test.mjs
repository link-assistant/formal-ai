import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, copyFileSync, statSync, chmodSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { seal, importBundle, verifyDescriptor, verifySourceDescriptor } from '../../../scripts/release-stage-transfer.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
test('immutable release transfers preserve bytes/mode and refuse missing, spoofed, drifted and extra input', () => {
  const previousDirectory = process.cwd();
  const keys = ["GITHUB_REPOSITORY", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_SHA", "EXPECT_SOURCE", "EXPECT_TREE", "RELEASE_VERSION", "EXPECT_COMPILER", "GITHUB_JOB", "GITHUB_OUTPUT", "EXPECT_PRODUCER", "EXPECT_BUNDLE", "EXPECT_DESCRIPTOR"];
  const previousEnvironment = Object.fromEntries(keys.map(key => [key, process.env[key]]));
  try {
    const root = mkdtempSync(join(tmpdir(), 'pr1188-transfer-'));
    const saved = process.cwd();
    const compiler = 'fixture compiler contract';
    Object.assign(process.env, {
      GITHUB_REPOSITORY: 'example/formal-ai',
      GITHUB_RUN_ID: '123',
      GITHUB_RUN_ATTEMPT: '1',
      GITHUB_SHA: '1'.repeat(40),
      EXPECT_SOURCE: '2'.repeat(40),
      EXPECT_TREE: '3'.repeat(40),
      RELEASE_VERSION: '1.2.3',
      EXPECT_COMPILER: sha(compiler),
      GITHUB_JOB: 'auto_compile-release',
      GITHUB_OUTPUT: root + '/outputs'
    });
    writeFileSync(process.env.GITHUB_OUTPUT, '');
    const producer = root + '/producer';
    mkdirSync(producer);
    process.chdir(producer);
    mkdirSync('prepared-release/binary', {
      recursive: true
    });
    writeFileSync('prepared-release/binary/formal-ai', 'Controlled fixture bytes; never executable native code');
    chmodSync('prepared-release/binary/formal-ai', 0o755);
    writeFileSync('prepared-release/prepared-build.json', 'fixture receipt bytes');
    const sealed = seal('binary', ['prepared-release']);
    function consumer(label) {
      const path = root + '/' + label;
      mkdirSync(path);
      mkdirSync(path + '/.release-transfer/binary', {
        recursive: true
      });
      for (const file of ['bundle.tar', 'descriptor.json']) copyFileSync(producer + '/.release-transfer/binary/' + file, path + '/.release-transfer/binary/' + file);
      process.chdir(path);
      Object.assign(process.env, {
        EXPECT_PRODUCER: 'auto_compile-release',
        EXPECT_BUNDLE: sealed.sha256,
        EXPECT_DESCRIPTOR: sha(readFileSync('.release-transfer/binary/descriptor.json'))
      });
      return path;
    }
    const positive = consumer('positive');
    importBundle('binary');
    assert.equal(readFileSync('prepared-release/binary/formal-ai', 'utf8'), 'Controlled fixture bytes; never executable native code');
    assert.equal(statSync('prepared-release/binary/formal-ai').mode & 0o777, 0o755);
    let refusals = 0;
    consumer('missing');
    writeFileSync('.release-transfer/binary/descriptor.json', '{}');
    assert.throws(() => importBundle('binary'));
    refusals++;
    consumer('truncated');
    writeFileSync('.release-transfer/binary/bundle.tar', 'truncated');
    assert.throws(() => importBundle('binary'));
    refusals++;
    consumer('spoof-producer');
    process.env.EXPECT_PRODUCER = 'manual_compile-release';
    assert.throws(() => importBundle('binary'));
    refusals++;
    consumer('source-drift');
    process.env.EXPECT_SOURCE = '4'.repeat(40);
    assert.throws(() => importBundle('binary'));
    refusals++;
    process.env.EXPECT_SOURCE = '2'.repeat(40);
    consumer('wrong-type');
    assert.throws(() => importBundle('images'));
    refusals++;
    consumer('unmanifested');
    const descriptor = JSON.parse(readFileSync('.release-transfer/binary/descriptor.json'));
    descriptor.inventory.pop();
    writeFileSync('.release-transfer/binary/descriptor.json', JSON.stringify(descriptor));
    process.env.EXPECT_DESCRIPTOR = sha(readFileSync('.release-transfer/binary/descriptor.json'));
    assert.throws(() => importBundle('binary'));
    refusals++;
    const linked = root + '/symlink';
    mkdirSync(linked);
    process.chdir(linked);
    mkdirSync('prepared-release/binary', {
      recursive: true
    });
    writeFileSync('prepared-release/binary/actual', 'fixture');
    symlinkSync('actual', 'prepared-release/binary/formal-ai');
    assert.throws(() => seal('binary', ['prepared-release']));
    refusals++;
    process.chdir(saved);
    assert.equal(refusals, 7);
  } finally {
    process.chdir(previousDirectory);
    for (const key of keys) {
      if (previousEnvironment[key] === undefined) delete process.env[key];else process.env[key] = previousEnvironment[key];
    }
  }
});
test('independent source and producer identities refuse unknown, failed, interrupted, spoofed and drifted transfers', () => {
  const bytes = Buffer.from('Controlled transferred executable bytes; no native binary claim');
  const compiler = 'controlled actual compiler contract fixture';
  const expected = {
    repository: 'example/formal-ai',
    run: '123',
    attempt: '1',
    protocol: '1'.repeat(40),
    source: '2'.repeat(40),
    tree: '3'.repeat(40),
    version: '1.2.3',
    compilerSha256: sha(compiler),
    producerJob: 'auto_compile-release',
    kind: 'binary',
    bundleSha256: sha(bytes)
  };
  const record = {
    schema: 'release-stage-transfer/v1',
    identity: Object.fromEntries(['repository', 'run', 'attempt', 'protocol', 'source', 'tree', 'version', 'compilerSha256'].map(key => [key, expected[key]])),
    producerJob: expected.producerJob,
    kind: 'binary',
    complete: true,
    status: 0,
    signal: null,
    sha256: sha(bytes),
    bytes: bytes.length
  };
  verifyDescriptor(record, expected, bytes);
  let transferRefusals = 0;
  for (const mutate of [r => delete r.identity.source,
     r => r.identity.source = '9'.repeat(40),
     r => r.identity.protocol = '9'.repeat(40),
     r => r.identity.compilerSha256 = '9'.repeat(64),
     r => r.identity.run = '124',
     r => r.identity.attempt = '2',
     r => r.kind = 'images',
     r => r.producerJob = 'auto_verify-package',
     r => r.complete = false,
     r => r.status = 1,
     r => r.signal = 'SIGKILL',
     r => r.bytes++,
     r => r.sha256 = '9'.repeat(64)]) {

    const changed = structuredClone(record);
    mutate(changed);
    assert.throws(() => verifyDescriptor(changed, expected, bytes));
    transferRefusals++;
  }
  assert.throws(() => verifyDescriptor(record, expected, Buffer.from('truncated')));
  transferRefusals++;
  const missing = structuredClone(expected);
  delete missing.source;
  assert.throws(() => verifyDescriptor(record, missing, bytes));
  transferRefusals++;
  const archive = Buffer.from('controlled immutable full source archive bytes');
  const sourceExpected = {
    ...expected,
    producerJob: 'auto_prepare-source',
    archiveSha256: sha(archive)
  };
  const sourceRecord = {
    schema: 'release-stage-source/v1',
    producerJob: 'auto_prepare-source',
    identity: record.identity,
    archiveSha256: sha(archive),
    compiler
  };
  verifySourceDescriptor(sourceRecord, sourceExpected, archive, archive);
  let sourceRefusals = 0;
  for (const bad of [Buffer.from('download modified'), Buffer.from('source drift')]) {
    assert.throws(() => verifySourceDescriptor(sourceRecord, sourceExpected, bad, archive));
    sourceRefusals++;
  }
  assert.throws(() => verifySourceDescriptor(sourceRecord, sourceExpected, archive, Buffer.from('independent checkout differs')));
  sourceRefusals++;
  const spoofProducer = structuredClone(sourceRecord);
  spoofProducer.producerJob = 'manual_prepare-source';
  assert.throws(() => verifySourceDescriptor(spoofProducer, sourceExpected, archive, archive));
  sourceRefusals++;
  const wrong = structuredClone(sourceRecord);
  wrong.identity.source = '4'.repeat(40);
  assert.throws(() => verifySourceDescriptor(wrong, sourceExpected, archive, archive));
  sourceRefusals++;
  assert.equal(transferRefusals, 15);
  assert.equal(sourceRefusals, 5);
});
import YAML from 'yaml';
const repository = new URL('../../../', import.meta.url);
const evidence = new URL('experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/', repository);
const originalSnapshot = readFileSync(new URL('original-release-workflow.yml', evidence), 'utf8');
const coverage = JSON.parse(readFileSync(new URL('stage-source-coverage.json', evidence), 'utf8'));
const readProjection = () => YAML.parse(readFileSync(new URL('.github/workflows/release-staged.yml', repository), 'utf8'));

// This is a checked derivative of the retained canonical source. Any canonical byte change
// requires an explicit refreshed projection and source review; it never silently drifts.
function validateProjection(projection, canonicalBytes = readFileSync(new URL('.github/workflows/release.yml', repository), 'utf8')) {
  assert.equal(canonicalBytes, originalSnapshot, 'canonical release changed: regenerate/review dormant projection and source snapshot');
  assert.equal(sha(canonicalBytes), coverage.workflowSha256, 'canonical source SHA differs');
  const canonical = YAML.parse(canonicalBytes);
  assert.equal(Object.keys(projection.jobs).length, 16);
  assert.equal(projection.concurrency, undefined, 'only original caller owns writer lease');
  for (const [name, value] of Object.entries(canonical.env)) assert.deepEqual(projection.env[name], value, 'explicit callee environment differs: ' + name);
  assert.equal(projection.env.STAGED_RELEASE_MODE, '${{ inputs.mode }}');
  for (const name of ['pages_sha', 'pages_version', 'container-tag']) assert.equal(projection.on.workflow_call.outputs[name].value, '${{ jobs.auto_create-release.outputs.' + name + ' || jobs.manual_create-release.outputs.' + name + ' }}');
  let operations = 0;
  let transferredReferences = 0;
  for (const original of coverage.callers) {
    const mode = original.caller.replace('-release', '');
    const sourceJob = canonical.jobs[original.caller];
    assert.deepEqual(sourceJob.concurrency, {
      group: 'formal-ai-repository-writes',
      queue: 'max'
    });
    assert.equal(sourceJob['timeout-minutes'], 90, 'original timing floor/caller must remain production');
    assert.equal(sourceJob.steps.length, original.steps.length);
    const byStep = new Map(original.steps.filter(step => step.stepId).map(step => [step.stepId, step.stage]));
    const prepare = projection.jobs[mode + '_prepare-source'];
    const expectedGuard = '${{ inputs.mode == \'' + mode + '\' && github.ref == \'refs/heads/main\' && github.workflow_ref == format(\'{0}/.github/workflows/release.yml@refs/heads/main\', github.repository) && ' + (mode === 'auto' ? "github.event_name == 'push'" : "github.event_name == 'workflow_dispatch' && github.event.inputs.release_mode == 'instant'") + ' }}';
    assert.equal(prepare.if, expectedGuard, 'guard must precede any version/source mutation');
    const checkoutIndex = prepare.steps.findIndex(step => step.uses === 'actions/checkout@v7' && !step.with?.path);
    const protocolIndex = prepare.steps.findIndex(step => step.name === 'Check out immutable trusted workflow protocol');
    const captureIndex = prepare.steps.findIndex(step => step.id === 'capture-source');
    assert.ok(checkoutIndex >= 0 && protocolIndex > checkoutIndex && captureIndex > protocolIndex, 'checkout clean must precede protocol and source seal');
    assert.equal(prepare.steps[protocolIndex].with.ref, '${{ github.sha }}');
    assert.equal(prepare.outputs.active, mode === 'auto' ? "${{ steps.check.outputs.should_release == 'true' }}" : "${{ steps.version.outputs.version_committed == 'true' || steps.version.outputs.already_released == 'true' }}");
    for (const name of ['source_sha', 'source_tree', 'version', 'compiler_sha', 'rust_version']) assert.equal(prepare.outputs[name], '${{ steps.capture-source.outputs.' + name + ' || steps.baseline-source.outputs.' + name + ' }}', 'inactive branch needs explicit baseline output');
    const orderByStage = new Map();
    for (let ordinal = 0; ordinal < original.steps.length; ordinal++) {
      const operation = original.steps[ordinal];
      assert.equal(operation.ordinal, ordinal);
      assert.equal(sha(operation.raw), operation.sha256);
      const parsedRaw = YAML.parse('steps:\n' + operation.raw).steps[0];
      assert.deepEqual(parsedRaw, sourceJob.steps[ordinal], 'retained operation snapshot/order differs from canonical');
      const stage = projection.jobs[mode + '_' + operation.stage];
      assert.ok(stage);
      let rewritten = operation.raw.replace(/steps\.([\w-]+)\.outputs\.([\w-]+)/g, (reference, step, output) => {
        const producerStage = byStep.get(step);
        assert.ok(producerStage, 'unknown producer step');
        if (producerStage === operation.stage) return reference;
        const producerJob = mode + '_' + producerStage;
        assert.ok(stage.needs.includes(producerJob), 'consumer requires actual direct producer');
        const outputName = step + '__' + output;
        assert.equal(projection.jobs[producerJob].outputs[outputName], '${{ steps.' + step + '.outputs.' + output + ' }}', 'producer must expose actual original step output');
        transferredReferences++;
        return 'needs.' + producerJob + '.outputs.' + outputName;
      });
      rewritten = rewritten.replace(/(^        timeout-minutes:) (\d+)/gm, (_, prefix, value) => prefix + ' ' + Math.min(Number(value), 21));
      rewritten = rewritten.replace(/node scripts\/(prepared-release-binary|release-image-factory)\.mjs/g, 'node _protocol/scripts/$1.mjs');
      if (operation.name === 'Publish Docker image to GHCR') {
        const prefix = "env TEST_BUDGET_ENFORCE=true TEST_BUDGET_GRACE_SECONDS=5 " +
          "TEST_BUDGET_POLL_SECONDS=1 TEST_WARN_RATIO_PERCENT=70 RUSTC_WRAPPER='' " +
          'bash _protocol/scripts/run-with-budget-warning.sh 1250 "Publish prepared release image" ';
        rewritten = rewritten.replace('run: node _protocol/', 'run: ' + prefix + 'node _protocol/');
        assert.equal(stage['timeout-minutes'], 30);
        assert.ok(1250 + 5 + 1 <= 21 * 60);
      }
      // Only the retained manual version operation receives explicit hosted GET authentication.
      if (mode === 'manual' && operation.stage === 'prepare-source' && operation.stepId === 'version') {
        assert.equal(parsedRaw.id, 'version');
        assert.equal(parsedRaw.env.GH_TOKEN, undefined, 'canonical original token input must remain absent');
        assert.equal(rewritten.split('        env:\n').length, 2);
        rewritten = rewritten.replace('        env:\n', () => '        env:\n          GH_TOKEN: ${{ github.token }}\n');
      }
      const expected = YAML.parse('steps:\n' + rewritten).steps[0];
      const matches = stage.steps.map((step, index) => ({
        step,
        index
      })).filter(({
        step
      }) => (step.name || step.uses) === operation.name);
      assert.equal(matches.length, 1, 'each original operation appears exactly once in its stage');
      assert.deepEqual(matches[0].step, expected, 'original command, condition, argument or assertion changed');
      assert.ok(matches[0].index > (orderByStage.get(operation.stage) ?? -1), 'original within-stage operation order changed');
      orderByStage.set(operation.stage, matches[0].index);
      operations++;
    }
    const jobs = Object.entries(projection.jobs).filter(([id]) => id.startsWith(mode + '_'));
    const completed = new Set();
    const visiting = new Set();
    function visit(id) {
      assert.ok(projection.jobs[id]);
      assert.ok(!visiting.has(id), 'stage cycle');
      if (completed.has(id)) return;
      visiting.add(id);
      for (const dependency of projection.jobs[id].needs ?? []) visit(dependency);
      visiting.delete(id);
      completed.add(id);
    }
    for (const [id, job] of jobs) {
      assert.equal(job.concurrency, undefined);
      assert.equal(job['timeout-minutes'], 30, 'actual dormant stage cap');
      visit(id);
      if (id.endsWith('_prepare-source')) continue;
      for (const [name, output] of Object.entries({
        EXPECT_SOURCE: 'source_sha',
        EXPECT_TREE: 'source_tree',
        EXPECT_COMPILER: 'compiler_sha',
        RELEASE_VERSION: 'version'
      })) assert.equal(job.env[name], '${{ needs.' + mode + '_prepare-source.outputs.' + output + ' }}');
      const sourceDownload = job.steps.find(step => step.name === 'Download immutable selected source artifact');
      assert.equal(sourceDownload.with['artifact-ids'], '${{ needs.' + mode + '_prepare-source.outputs.artifact_id }}');
      assert.equal(sourceDownload.with.path, '.release-transfer/source');
      assert.equal(sourceDownload.with['digest-mismatch'], 'error');
      const sourceVerifier = job.steps.find(step => step.run === 'node _protocol/scripts/release-stage-transfer.mjs verify-source');
      assert.ok(sourceVerifier);
      if (id.endsWith('_create-release')) {
        assert.equal(sourceDownload.if, '${{ needs.' + mode + "_prepare-source.outputs.active == 'true' }}");
        assert.equal(sourceVerifier.if, sourceDownload.if, 'no-release branch must not load missing artifact/source authority');
      }
      for (const step of job.steps.filter(step => step.name?.startsWith('Validate physical '))) {
        assert.ok(step.env.EXPECT_BUNDLE && step.env.EXPECT_DESCRIPTOR && step.env.EXPECT_PRODUCER);
        assert.ok(job.needs.includes(step.env.EXPECT_PRODUCER));
      }
    }
    const final = projection.jobs[mode + '_create-release'];
    for (const prerequisite of ['compile-release', 'verify-package', 'publish-crate', 'registry-availability', 'published-crate-smoke', 'publish-verify-images']) assert.ok(final.needs.includes(mode + '_' + prerequisite));
    assert.ok(final.if.includes('always()') && final.if.includes("outputs.active != 'true'"), 'inactive branch remains Pages-only with skipped compute jobs');
    assert.ok(final.steps.some(step => step.run === 'node _protocol/scripts/release-stage-transfer.mjs guard-create'));
    assert.equal(final.outputs['container-tag'], "${{ steps.create-release.outcome == 'success' && format('v{0}', needs." + mode + "_prepare-source.outputs.version) || '' }}");
  }
  assert.equal(operations, 52);
  assert.equal(transferredReferences, 104);
  return {
    operations,
    transferredReferences
  };
}
test('dormant workflow is a checked exact canonical operation/order/output/authority projection', () => {
  assert.deepEqual(validateProjection(readProjection()), {
    operations: 52,
    transferredReferences: 104
  });
});
test('canonical drift and omitted, reordered, spoofed, ungated or prematurely released projections fail', () => {
  const projection = readProjection();
  assert.throws(() => validateProjection(projection, originalSnapshot + '\n# canonical changed'));
  const mutations = [candidate => candidate.jobs['auto_verify-package'].steps.splice(candidate.jobs['auto_verify-package'].steps.findIndex(step => step.name === 'Verify packaged crate archive'),
     1),
     candidate => candidate.jobs['auto_published-crate-smoke'].steps.splice(candidate.jobs['auto_published-crate-smoke'].steps.findIndex(step => step.name === 'Smoke test the published crate'),
     1),
     candidate => candidate.jobs['auto_create-release'].needs.pop(),
     candidate => candidate.jobs['auto_prepare-source'].if = "${{ inputs.mode == 'auto' }}",
     candidate => candidate.jobs['auto_prepare-source'].outputs.source_sha = 'invented',
     candidate => candidate.jobs['auto_publish-crate'].outputs['publish-crate__publish_result'] = 'success',
     candidate => candidate.jobs['auto_compile-release'].env.EXPECT_SOURCE = 'untrusted',
     candidate => candidate.jobs['auto_compile-release'].concurrency = {

    group: 'formal-ai-repository-writes'
  }
    ,
     candidate => candidate.jobs['auto_compile-release']['timeout-minutes'] = 31,
     candidate => candidate.jobs['auto_create-release'].outputs['container-tag'] = 'v-fake',
     candidate => candidate.on.workflow_call.outputs.pages_sha.value = 'fake',
     candidate => candidate.jobs['auto_create-release'].steps.find(step => step.name === 'Download immutable selected source artifact').if = undefined,
     candidate => candidate.jobs['auto_prepare-source'].steps.reverse(),
     candidate => candidate.jobs['auto_publish-verify-images'].steps.reverse()];

  for (const mutation of mutations) {
    const changed = structuredClone(projection);
    mutation(changed);
    assert.throws(() => validateProjection(changed));
  }
});

test('copied image operation deadline cannot inherit extension or exceed job share', () => {
  for (const mode of ['auto', 'manual']) {
    for (const mutate of [
      s => s['timeout-minutes'] = 22,
      s => s.run = s.run.replace('TEST_BUDGET_ENFORCE=true', 'TEST_BUDGET_ENFORCE=false'),
      s => s.run = s.run.replace('TEST_BUDGET_GRACE_SECONDS=5', 'TEST_BUDGET_GRACE_SECONDS=500'),
      s => s.run = s.run.replace(' 1250 ', ' 1800 '),
      s => s.run = s.run.replace('_protocol/scripts/run-with-budget-warning.sh', 'scripts/run-with-budget-warning.sh'),
      s => s.run = s.run.replace('publish prepared-release', 'mirror prepared-release')
    ]) {
      const p = readProjection();
      mutate(p.jobs[mode + '_publish-verify-images'].steps.find(s => s.name === 'Publish Docker image to GHCR'));
      assert.throws(() => validateProjection(p));
    }
  }
});

test('manual hosted token projection is limited to the actual retained version operation', () => {
  const projection = readProjection();
  const select = candidate => candidate.jobs['manual_prepare-source'].steps.find(step => step.id === 'version');
  assert.equal(select(projection).env.GH_TOKEN, '${{ github.token }}');
  for (const mutation of [
    candidate => { delete select(candidate).env.GH_TOKEN; },
    candidate => { select(candidate).env.GH_TOKEN = 'untrusted'; },
    candidate => { select(candidate).id = 'untrusted'; },
    candidate => {
      const foreign = candidate.jobs['manual_prepare-source'].steps.find(step => step.uses === 'actions/checkout@v7' && !step.with?.path);
      assert.ok(foreign);
      foreign.env = { ...foreign.env, GH_TOKEN: '${{ github.token }}' };
    }
  ]) {
    const candidate = structuredClone(projection);
    mutation(candidate);
    assert.throws(() => validateProjection(candidate));
  }
});
