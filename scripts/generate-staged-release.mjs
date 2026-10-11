import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, existsSync, mkdirSync, realpathSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import YAML from 'yaml';
import { workflowJobs } from './lib/ci-speed-workflows.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export function runStagedReleaseGenerator(arguments_) {
  const evidence = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
  const directoryPosition = arguments_.indexOf('--directory');
  const outputDirectory = directoryPosition < 0 ? null : resolve(arguments_[directoryPosition + 1]);
  assert.ok(arguments_.every((argument, index) => argument === '--check' || argument === '--check-deployed' || argument === '--deployed' || argument === '--write' || argument === '--directory' || directoryPosition >= 0 && index === directoryPosition + 1));
  assert.equal(arguments_.filter(value => ['--check','--check-deployed','--write'].includes(value)).length, 1, 'choose exactly one check or write mode');
  if (arguments_.includes('--write')) assert.ok(outputDirectory, 'write requires an explicit scratch/repository destination');
  const packet = JSON.parse(readFileSync(join(root, evidence, 'stage-source-coverage.json'), 'utf8'));
  const deployed = arguments_.includes('--check-deployed') || arguments_.includes('--deployed');
  const source = readFileSync(join(root, deployed ? evidence + 'original-release-workflow.yml' : '.github/workflows/release.yml'), 'utf8');
  const sha = value => createHash('sha256').update(value).digest('hex');
  assert.equal(sha(source), packet.workflowSha256, 'canonical source drift requires explicit original operation refresh');
  assert.equal(readFileSync(join(root, evidence, 'original-release-workflow.yml'), 'utf8'), source);
  const {operations,binding,outputs}=buildStagedReleaseProjection(source,packet);
  if (deployed) assert.equal(readFileSync(join(root,'.github/workflows/release.yml'),'utf8'),outputs.get(evidence+'candidate-release-caller.yml'),'deployed caller differs from checked original projection');
  for (const [path, bytes] of outputs) {
    const destination = outputDirectory ? join(outputDirectory, path.split('/').at(-1)) : join(root, path);
    if (arguments_.includes('--write')) {
      mkdirSync(dirname(destination), {
        recursive: true
      });
      writeFileSync(destination, bytes);
    } else {
      assert.ok(existsSync(destination), path + ' missing');
      assert.equal(readFileSync(destination, 'utf8'), bytes, path + ' differs from checked generation');
    }
  }
  console.log(JSON.stringify({
    operations,
    bindings: binding.length,
    outputs: outputs.size,
    write: arguments_.includes('--write'),
    callerLeasePreserved: true,
    coldProof: false
  }));
  console.log('STAGED_RELEASE_GENERATOR_PASS');
}
export function buildStagedReleaseProjection(source, packet) {
  const evidence = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
  const sha = value => createHash('sha256').update(value).digest('hex');
  assert.equal(sha(source), packet.workflowSha256, 'canonical source drift requires explicit original operation refresh');
  let operations = 0;
  for (const caller of packet.callers) {
    const job = YAML.parse(source).jobs[caller.caller];
    assert.equal(job.steps.length, caller.steps.length);
    for (const operation of caller.steps) {
      assert.equal(sha(operation.raw), operation.sha256);
      assert.deepEqual(YAML.parse('steps:\n' + operation.raw).steps[0], job.steps[operation.ordinal]);
      operations++;
    }
  }
  assert.equal(operations, 52);
  const expression = body => '${{ ' + body + ' }}';
  const stages = ['prepare-source', 'compile-release', 'verify-package', 'publish-crate', 'registry-availability', 'published-crate-smoke', 'publish-verify-images', 'create-release'];
  const stageNeeds = {
    'prepare-source': [],
    'compile-release': ['prepare-source'],
    'verify-package': ['prepare-source', 'compile-release'],
    'publish-crate': ['prepare-source', 'compile-release', 'verify-package'],
    'registry-availability': ['prepare-source', 'verify-package', 'publish-crate'],
    'published-crate-smoke': ['prepare-source', 'publish-crate', 'registry-availability'],
    'publish-verify-images': ['prepare-source', 'compile-release', 'verify-package', 'publish-crate', 'registry-availability', 'published-crate-smoke'],
    'create-release': ['prepare-source', 'compile-release', 'verify-package', 'publish-crate', 'registry-availability', 'published-crate-smoke', 'publish-verify-images']
  };
  const envSource = source.slice(source.indexOf('\nenv:\n') + 1, source.indexOf('\njobs:\n')).replace(/\n$/, '');
  let yaml = 'name: Staged release candidate\non:\n  workflow_call:\n    inputs:\n      mode:\n        type: string\n        required: true\n    outputs:\n';
  yaml = yaml.replace('    outputs:\n', '    secrets:\n' + ['CARGO_REGISTRY_TOKEN', 'CARGO_TOKEN', 'DOCKERHUB_USERNAME', 'DOCKERHUB_TOKEN'].map(name => '      ' + name + ': {required: false}\n').join('') + '    outputs:\n');
  for (const name of ['pages_sha', 'pages_version', 'container-tag']) yaml += '      ' + name + ':\n        value: ' + expression('jobs.auto_create-release.outputs.' + name + ' || jobs.manual_create-release.outputs.' + name) + '\n';
  yaml += 'permissions:\n  contents: read\n' + envSource + '\n  DOCKERHUB_USERNAME: ' + expression("vars.DOCKERHUB_USERNAME || secrets.DOCKERHUB_USERNAME || 'konard'") + '\n  DOCKERHUB_TOKEN: ' + expression('secrets.DOCKERHUB_TOKEN') + '\n  RELEASE_MODE: ' + expression("github.event.inputs.release_mode || ''") + '\n  STAGED_RELEASE_MODE: ' + expression('inputs.mode') + '\njobs:\n';
  const binding = [];
  for (const caller of packet.callers) {
    const mode = caller.caller.replace('-release', '');
    const id = stage => mode + '_' + stage;
    const prep = id('prepare-source');
    const producerByStep = new Map(caller.steps.filter(s => s.stepId).map(s => [s.stepId, s.stage]));
    const crossNames = new Map();
    for (const transfer of caller.crossStageOutputTransfers) {
      const key = transfer.producerStep + '__' + transfer.output;
      const producer = transfer.producerStage;
      if (!crossNames.has(producer)) crossNames.set(producer, new Map());
      crossNames.get(producer).set(key, {
        step: transfer.producerStep,
        output: transfer.output
      });
    }
    const active = mode === 'auto' ? "steps.check.outputs.should_release == 'true'" : "steps.version.outputs.version_committed == 'true' || steps.version.outputs.already_released == 'true'";
    const version = mode === 'auto' ? "steps.version.outputs.new_version || steps.current_version.outputs.version" : "steps.version.outputs.new_version";
    const needsExpr = (stage, key) => 'needs.' + id(stage) + '.outputs.' + key;
    for (const stage of stages) {
      const needs = stageNeeds[stage];
      yaml += '  ' + id(stage) + ':\n    name: ' + mode + ' ' + stage + '\n';
      if (stage === 'prepare-source') yaml += '    permissions:\n      contents: write\n      actions: read\n';else if (stage === 'create-release') yaml += '    permissions:\n      contents: write\n';else if (stage === 'publish-verify-images') yaml += '    permissions:\n      contents: read\n      packages: write\n';
      if (needs.length) yaml += '    needs: [' + needs.map(id).join(', ') + ']\n';
      if (stage === 'prepare-source') yaml += '    if: ' + expression("inputs.mode == '" + mode + "' && github.ref == 'refs/heads/main' && github.workflow_ref == format('{0}/.github/workflows/release.yml@refs/heads/main', github.repository) && " + (mode === 'auto' ? "github.event_name == 'push'" : "github.event_name == 'workflow_dispatch' && github.event.inputs.release_mode == 'instant'")) + '\n';else if (stage === 'create-release') yaml += '    if: ' + expression("always() && needs." + prep + ".result == 'success' && (needs." + prep + ".outputs.active != 'true' || (" + needs.filter(s => s !== 'prepare-source').map(s => 'needs.' + id(s) + ".result == 'success'").join(' && ') + '))') + '\n';else yaml += '    if: ' + expression('needs.' + prep + ".outputs.active == 'true'") + '\n';
      yaml += '    runs-on: ubuntu-24.04\n    timeout-minutes: 30\n    env:\n';
      if (stage === 'prepare-source') yaml += '      EXPECT_BINARY_PRODUCER: ' + id('compile-release') + '\n';else {
        for (const [name, value] of Object.entries({
          RELEASE_VERSION: 'version',
          EXPECT_SOURCE: 'source_sha',
          EXPECT_TREE: 'source_tree',
          EXPECT_COMPILER: 'compiler_sha',
          EXPECT_SOURCE_DESCRIPTOR: 'source_descriptor_sha',
          EXPECT_SOURCE_ARCHIVE: 'source_archive_sha'
        })) yaml += '      ' + name + ': ' + expression('needs.' + prep + '.outputs.' + value) + '\n';
        yaml += '      EXPECT_SOURCE_PRODUCER: ' + prep + '\n      EXPECT_BINARY_PRODUCER: ' + id('compile-release') + '\n';
      }
      if (stage === 'create-release') yaml += '      EXPECT_FULL_DIGEST: ' + expression(needsExpr('publish-verify-images', 'ghcr-publish__digest')) + '\n      MIRROR_ENABLED: ' + expression(needsExpr('publish-verify-images', 'dockerhub__enabled')) + '\n';
      yaml += '    outputs:\n';
      for (const [key, {
        step,
        output
      }] of crossNames.get(stage) ?? []) yaml += '      ' + key + ': ' + expression('steps.' + step + '.outputs.' + output) + '\n';
      if (stage === 'prepare-source') {
        yaml += '      active: ' + expression(active) + '\n';
        for (const name of ['source_sha', 'source_tree', 'version', 'compiler_sha', 'rust_version']) yaml += '      ' + name + ': ' + expression('steps.capture-source.outputs.' + name + ' || steps.baseline-source.outputs.' + name) + '\n';
        for (const name of ['source_descriptor_sha', 'source_archive_sha']) yaml += '      ' + name + ': ' + expression('steps.capture-source.outputs.' + name) + '\n';
      }
      if (['prepare-source', 'compile-release', 'verify-package', 'publish-verify-images'].includes(stage)) {
        yaml += '      artifact_id: ' + expression('steps.upload.outputs.artifact-id') + '\n      artifact_digest: ' + expression('steps.upload.outputs.artifact-digest') + '\n';
        if (stage !== 'prepare-source') yaml += '      bundle_sha: ' + expression('steps.seal.outputs.bundle_sha') + '\n      descriptor_sha: ' + expression('steps.seal.outputs.descriptor_sha') + '\n';
      }
      if (stage === 'publish-verify-images') yaml += '      ghcr-publish__digest: ' + expression('steps.ghcr-publish.outputs.digest') + '\n';
      if (stage === 'create-release') {
        yaml += '      pages_sha: ' + expression('steps.release_pages_ref.outputs.sha') + '\n      pages_version: ' + expression('steps.release_pages_ref.outputs.version') + '\n      container-tag: ' + expression("steps.create-release.outcome == 'success' && format('v{0}', needs." + prep + ".outputs.version) || ''") + '\n';
      }
      yaml += '    steps:\n';
      const action = (name, body) => {
        yaml += '      - name: ' + name + '\n' + body;
      };
      if (stage !== 'prepare-source') {
        action('Check out exact independently selected source', '        uses: actions/checkout@v7\n        with:\n          ref: ' + expression('needs.' + prep + '.outputs.source_sha') + '\n          fetch-depth: 0\n          persist-credentials: false\n          token: ' + expression('secrets.GITHUB_TOKEN') + '\n');
        action('Install exact selected Rust compiler', '        uses: dtolnay/rust-toolchain@stable\n        with:\n          toolchain: ' + expression('needs.' + prep + '.outputs.rust_version') + '\n');
        action('Install original rust-script runtime', '        run: bash scripts/install-rust-script.sh\n');
      }
      if (stage !== 'prepare-source') action('Check out immutable trusted workflow protocol', '        uses: actions/checkout@v7\n        with: { ref: ' + JSON.stringify(expression('github.sha')) + ', path: _protocol, persist-credentials: false }\n');
      const activeIf = stage === 'create-release' ? '        if: ' + expression('needs.' + prep + ".outputs.active == 'true'") + '\n' : '';
      if (stage !== 'prepare-source') {
        action('Download immutable selected source artifact', activeIf + '        uses: actions/download-artifact@v8\n        env:\n          NODE_OPTIONS: --disable-warning=DEP0005\n        with:\n          artifact-ids: ' + expression('needs.' + prep + '.outputs.artifact_id') + '\n          path: .release-transfer/source\n          digest-mismatch: error\n');
        action('Verify independent source compiler protocol and main tag authority', activeIf + '        run: node _protocol/scripts/release-stage-transfer.mjs verify-source\n');
      }
      const importArtifact = (producer, kind) => {
        action('Download immutable ' + kind + ' artifact', activeIf + '        uses: actions/download-artifact@v8\n        env:\n          NODE_OPTIONS: --disable-warning=DEP0005\n        with:\n          artifact-ids: ' + expression(needsExpr(producer, 'artifact_id')) + '\n          path: .release-transfer/' + kind + '\n          digest-mismatch: error\n');
        action('Validate physical ' + kind + ' producer transfer', activeIf + '        env:\n          EXPECT_BUNDLE: ' + expression(needsExpr(producer, 'bundle_sha')) + '\n          EXPECT_DESCRIPTOR: ' + expression(needsExpr(producer, 'descriptor_sha')) + '\n          EXPECT_PRODUCER: ' + id(producer) + '\n        run: node _protocol/scripts/release-stage-transfer.mjs import ' + kind + '\n');
      };
      if (['verify-package', 'publish-crate', 'publish-verify-images'].includes(stage)) importArtifact('compile-release', 'binary');
      if (['publish-crate', 'registry-availability'].includes(stage)) importArtifact('verify-package', 'package');
      if (stage === 'create-release') importArtifact('publish-verify-images', 'images');
      if (stage === 'create-release') action('Fail closed before any release publication', activeIf + '        run: node _protocol/scripts/release-stage-transfer.mjs guard-create\n');
      for (const operation of caller.steps.filter(s => s.stage === stage)) {
        let raw = operation.raw;
        raw = raw.replace(/steps\.([\w-]+)\.outputs\.([\w-]+)/g, (match, step, output) => {
          const producer = producerByStep.get(step);
          assert.ok(producer, 'unknown original reference ' + match);
          if (producer === stage) return match;
          assert.ok(needs.includes(producer), 'missing direct producer ' + stage + ' ' + producer);
          const replacement = needsExpr(producer, step + '__' + output);
          binding.push({
            caller: caller.caller,
            consumerStage: stage,
            consumerOperation: operation.name,
            original: match,
            replacement,
            producerJob: id(producer),
            producerStep: step,
            output
          });
          return replacement;
        });
        // The source-owned manual version operation needs explicit CLI authentication for guarded GETs.
        if (mode === 'manual' && stage === 'prepare-source' && operation.stepId === 'version') {
          const originalOperation=YAML.parse('steps:\n'+operation.raw).steps[0];
          assert.equal(originalOperation.id,'version');assert.equal(originalOperation.env.GH_TOKEN,undefined);
          assert.equal(raw.split('        env:\n').length,2);
          raw=raw.replace('        env:\n',()=> '        env:\n          GH_TOKEN: '+expression('github.token')+'\n');
        }
        // Runtime caps change at the real execution site; original commands/assertions remain.
        raw = raw.replace(/(^        timeout-minutes:) (\d+)/gm, (_, prefix, value) => prefix + ' ' + Math.min(Number(value), 21));
        // These workflow-owned immutable protocol helpers operate on the checked-out selected source cwd.
        raw = raw.replace(/node scripts\/(prepared-release-binary|release-image-factory)\.mjs/g, 'node _protocol/scripts/$1.mjs');
        if (operation.name === 'Publish Docker image to GHCR') {
          const prefix = "env TEST_BUDGET_ENFORCE=true TEST_BUDGET_GRACE_SECONDS=5 " + "TEST_BUDGET_POLL_SECONDS=1 TEST_WARN_RATIO_PERCENT=70 RUSTC_WRAPPER='' " + 'bash _protocol/scripts/run-with-budget-warning.sh 1250 "Publish prepared release image" ';
          raw = raw.replace('run: node _protocol/', 'run: ' + prefix + 'node _protocol/');
        }
        if (stage === 'prepare-source' && operation.ordinal === 0) yaml += '      # #1079: persist-credentials is retained for the root version/tag git push.\n';
        yaml += raw.endsWith('\n') ? raw : raw + '\n';
        if (stage === 'prepare-source' && operation.ordinal === 0) {
        action('Check out immutable trusted workflow protocol', '        uses: actions/checkout@v7\n        with: { ref: ' + JSON.stringify(expression('github.sha')) + ', path: _protocol, persist-credentials: false }\n');
        action('Set up immutable protocol dependencies', '        uses: oven-sh/setup-bun@v2\n        with:\n          bun-version-file: _protocol/.bun-version\n');
        action('Install immutable locked protocol dependencies', '        timeout-minutes: 5\n        working-directory: _protocol\n        run: bun install --frozen-lockfile --ignore-scripts\n');
        }
      }
      if (stage === 'prepare-source') {
        action('Record actual selected source and compiler baseline', '        id: baseline-source\n        run: node _protocol/scripts/release-stage-transfer.mjs baseline\n');
        action('Seal actual tagged main source artifact', '        id: capture-source\n        if: ' + expression(active) + '\n        env:\n          RELEASE_VERSION: ' + expression(version) + '\n        run: node _protocol/scripts/release-stage-transfer.mjs capture-source\n');
      }
      if (stage === 'registry-availability') action('Audit real registry crate against verified package contents', '        run: node _protocol/scripts/release-stage-transfer.mjs verify-registry\n');
      if (['prepare-source', 'compile-release', 'verify-package', 'publish-verify-images'].includes(stage)) {
        const kind = {
          'prepare-source': 'source',
          'compile-release': 'binary',
          'verify-package': 'package',
          'publish-verify-images': 'images'
        }[stage];
        if (stage !== 'prepare-source') action('Seal successful physical ' + kind + ' transfer', '        id: seal\n' + (stage === 'publish-verify-images' ? '        env:\n          EXPECT_FULL_DIGEST: ' + expression('steps.ghcr-publish.outputs.digest') + '\n          MIRROR_ENABLED: ' + expression('steps.dockerhub.outputs.enabled') + '\n' : '') + '        run: node _protocol/scripts/release-stage-transfer.mjs capture ' + kind + '\n');
        action('Upload immutable ' + kind + ' artifact', '        id: upload\n' + (stage === 'prepare-source' ? '        if: ' + expression(active) + '\n' : '') + '        uses: actions/upload-artifact@v7\n        with:\n          name: staged-' + mode + '-' + kind + '-' + expression('github.run_id') + '-' + expression('github.run_attempt') + '\n          path: .release-transfer/' + kind + '/*\n          include-hidden-files: true\n          if-no-files-found: error\n          retention-days: 7\n');
      }
    }
  }
  assert.equal(binding.length, 104);
  let callerSource = source;
  for (const caller of packet.callers) {
    const original = workflowJobs(source).find(j => j.id === caller.caller);
    const prefix = original.body.slice(0, original.body.indexOf('    runs-on:'));
    const replacement = prefix + '    permissions:\n      contents: write\n      packages: write\n      actions: read\n    uses: ./.github/workflows/release-staged.yml\n    with:\n      mode: ' + caller.caller.replace('-release', '') + '\n    secrets:\n' + ['CARGO_REGISTRY_TOKEN', 'CARGO_TOKEN', 'DOCKERHUB_USERNAME', 'DOCKERHUB_TOKEN'].map(name => '      ' + name + ': ' + expression('secrets.' + name) + '\n').join('');
    assert.equal(callerSource.split(original.body).length, 2);
    callerSource = callerSource.replace(original.body, replacement);
  }
  // Pack short JSON fields to a readable width; preserve long data strings whole.
  function format(value, depth = 0) {
    const width = 160;
    const indent = '  '.repeat(depth), child = indent + '  ';
    const compact = JSON.stringify(value);
    if (value === null || typeof value !== 'object' || compact.length + indent.length <= width) return compact;
    const array = Array.isArray(value);
    const entries = array ? value : Object.entries(value);
    const parts = entries.map(item => array ? format(item, depth + 1)
      : JSON.stringify(item[0]) + ': ' + format(item[1], depth + 1));
    const lines = [];
    let row = '';
    for (const part of parts) {
      if (part.includes('\n') || row && child.length + row.length + 2 + part.length > width) {
        if (row) {
          lines.push(child + row);
          row = '';
        }
        if (part.includes('\n')) {
          lines.push(child + part);
          continue;
        }
      }
      row += (row ? ', ' : '') + part;
    }
    if (row) lines.push(child + row);
    return (array ? '[' : '{') + '\n' + lines.join(',\n') + '\n' + indent + (array ? ']' : '}');
  }
  yaml = yaml.replace(/    outputs:\n(?=    steps:)/g, '');
  const workflow = YAML.parseDocument(yaml);
  assert.equal(workflow.errors.length, 0, 'Generated workflow YAML is invalid');
  const generatedWorkflow = yaml;
  const outputs = new Map([['.github/workflows/release-staged.yml', generatedWorkflow], [evidence + 'candidate-release-caller.yml', callerSource], [evidence + 'candidate-output-bindings.json', JSON.stringify({
    originalSourceSha256: packet.workflowSha256,
    bindings: binding,
    originalOperations: 52,
    scope: 'Executable candidate declaration; no scheduler/native/publication/cold execution proof'
  }, null, 2) + '\n'], [evidence + 'original-release-workflow.yml', source], [evidence + 'stage-source-coverage.json', format(packet) + '\n']]);
  return { operations, binding, outputs };
}
if (process.argv[1] && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) runStagedReleaseGenerator(process.argv.slice(2));
