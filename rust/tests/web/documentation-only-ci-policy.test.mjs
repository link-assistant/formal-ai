import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync, mkdtempSync, writeFileSync, unlinkSync, mkdirSync, rmSync} from 'node:fs';
import YAML from 'yaml';
import {checkDocumentationPolicy, filteredWorkflow, lightweightWorkflows, contextReportingWorkflows} from '../../../scripts/documentation-only-ci-policy.mjs';
import {applicableWorkflows} from '../../../scripts/lib/hive-delivery-admission.mjs';
import {documentationOnlyPullRequest} from '../../../scripts/lib/documentation-paths.mjs';
import {Script} from 'node:vm';
import {posix, join} from 'node:path';
import {tmpdir} from 'node:os';
import {documentationProblems, requirements as requiredDocumentation} from '../../../scripts/check-required-documentation.mjs';
import {spawnSync} from 'node:child_process';
const workflows = readdirSync('.github/workflows').filter(name => name.endsWith('.yml')).map(name => ({path: '.github/workflows/' + name, document: YAML.parse(readFileSync('.github/workflows/' + name, 'utf8'))}));
test('heavy triggers are filtered while required PR context reporters remain admitted', () => {
  assert.ok(checkDocumentationPolicy() > 10);
});
for (const event of ['push', 'pull_request']) {
  for (const files of [['docs/associative-tech-stack.md'], ['VISION.md','CONTRIBUTING.md'], ['docs/guide.md','changelog.d/pending.md'], ['rust/README.md']]) {
    test(event + ': documentation paths select lightweight checks and required context reporters: ' + files.join(','), () => {
      const selected = applicableWorkflows(workflows, {event, baseBranch: 'main', files});
      assert.ok(selected.includes('.github/workflows/documentation-policy.yml'));
      for (const path of selected) assert.ok(lightweightWorkflows.has(path.split('/').at(-1))
          || (event === 'pull_request' && contextReportingWorkflows.has(path.split('/').at(-1))), path);

      assert.equal(selected.includes('.github/workflows/release.yml'), event === 'pull_request',
          'the required PR contexts report without selecting heavy build jobs');
    });
  }
  for (const path of ['rust/src/lib.rs', 'js/agentic/planner.mjs', 'data/seed/runtime.md', 'data/seed/RUNTIME.MD', '.github/workflows/release.yml', 'Cargo.toml', 'unknown-input']) {
    test(event + ': mixed changes retain main CI and release admission: ' + path, () => {
      assert.ok(applicableWorkflows(workflows, {event, baseBranch: 'main', files: ['docs/guide.md', path]}).includes('.github/workflows/release.yml'));
    });
  }
}
test('transformation preserves every job, manual dispatch and schedule, and is idempotent', () => {
  const source = 'name: sample\non: # retained security annotation\n  push:\n    branches: [main]\n  pull_request:\n    # retained selection rationale\n    paths: ["rust/**"] # retained path comment\n  workflow_dispatch:\n  schedule:\n    - cron: "0 0 * * *"\njobs:\n  retained:\n    runs-on: ubuntu-latest\n    steps: []\n';
  const updated = filteredWorkflow(source);
  assert.equal(updated.split('jobs:')[1], source.split('jobs:')[1]);
  assert.deepEqual(YAML.parse(updated).on.workflow_dispatch, YAML.parse(source).on.workflow_dispatch);
  assert.deepEqual(YAML.parse(updated).on.schedule, YAML.parse(source).on.schedule);
  assert.equal(filteredWorkflow(updated), updated);
  for (const comment of ['# retained security annotation','# retained selection rationale','# retained path comment']) assert.ok(updated.includes(comment));
});
test('automatic bot checks require complete file observations and both sides of renames', () => {
  assert.equal(documentationOnlyPullRequest([[{filename:'docs/guide.md'}]],1),true);
  assert.equal(documentationOnlyPullRequest([[{filename:'docs/guide.md',previous_filename:'rust/src/lib.rs'}]],1),false);
  for (const [pages,count] of [[[],0],[[[]],2],[[[{filename:'docs/guide.md'}]],3001],[[[{filename:'data/seed/runtime.md'}]],1]])
    assert.equal(documentationOnlyPullRequest(pages,count),false);
});
test('lightweight read-only checks cancel superseded PRs and protect main', () => {
  const {concurrency} = workflows.find(workflow => workflow.path.endsWith('/documentation-policy.yml')).document;
  const expression = concurrency['cancel-in-progress'].replace(/^\$\{\{\s*|\s*\}\}$/gu,'');
  const evaluate = (event,action,ref='refs/heads/main') => new Script(expression)
    .runInNewContext({github:{event_name:event,event:{action},ref}}, {timeout:100});
  assert.equal(evaluate('pull_request','synchronize','refs/pull/1/merge'),true);
  for(const action of ['opened','reopened','ready_for_review'])
    assert.equal(evaluate('pull_request',action,'refs/pull/1/merge'),false);
  for(const event of ['workflow_dispatch','push'])
    for(const ref of ['refs/heads/main','refs/heads/fix/example'])assert.equal(evaluate(event,undefined,ref),false);
  assert.ok(concurrency.group.includes('github.workflow') && concurrency.group.includes('github.ref'));
});
test('native admission unit tests run in CI before their outputs select the tiers', () => {
  const {steps} = workflows.find(workflow => workflow.path.endsWith('/layered-ci.yml')).document.jobs.changes;
  const testIndex = steps.findIndex(step => step.run === 'rust-script --test scripts/detect-code-changes.rs');
  const classifyIndex = steps.findIndex(step => step.id === 'changes');
  assert.ok(testIndex >= 0 && testIndex < classifyIndex);
});
test('read-only documentation checkout retains no push credentials', () => {
  const {steps} = workflows.find(workflow => workflow.path.endsWith('/documentation-policy.yml')).document.jobs['documentation-policy'];
  const checkout = steps.find(step => step.uses?.startsWith('actions/checkout@'));
  assert.equal(checkout.with['persist-credentials'],false);
});

test('documentation acquisition is bounded without narrowing policy inputs or changing its checks', () => {
  const job=workflows.find(workflow=>workflow.path.endsWith('/documentation-policy.yml')).document.jobs['documentation-policy'];
  const expectedInputs="/.github/workflows/\n/scripts/documentation-only-ci-policy.mjs\n/scripts/lib/documentation-paths.mjs\n/scripts/lib/hive-delivery-admission.mjs\n/scripts/documentation-policy-dependencies/\n/rust/tests/web/documentation-only-ci-policy.test.mjs\n/package.json\n/scripts/check-required-documentation.mjs\n/data/meta/required-documentation.json\n/README.md\n/CONTRIBUTING.md\n/CHANGELOG.md\n";
  function check(value){
    assert.equal(value['timeout-minutes'],3);
    const checkouts=value.steps.filter(step=>step.uses?.startsWith('actions/checkout@'));
    assert.equal(checkouts.length,1);
    const checkout=checkouts[0];
    assert.equal(checkout.uses,'actions/checkout@v7');
    assert.equal(checkout['timeout-minutes'],2);
    assert.deepEqual(checkout.env,{GIT_HTTP_LOW_SPEED_LIMIT:'1024',GIT_HTTP_LOW_SPEED_TIME:'20'});
    assert.deepEqual(checkout.with,{'persist-credentials':false,'fetch-depth':1,
      filter:'blob:none','sparse-checkout':expectedInputs,'sparse-checkout-cone-mode':false});
    assert.deepEqual(value.steps.slice(1),[
      {uses:'actions/setup-node@v7',with:{'node-version':'22'}},
      {run:'npm ci --ignore-scripts --no-audit --no-fund --prefix scripts/documentation-policy-dependencies'},
      {run:'mkdir -p node_modules\nln -s ../scripts/documentation-policy-dependencies/node_modules/yaml node_modules/yaml\nnode --test rust/tests/web/documentation-only-ci-policy.test.mjs\n'},
      {run:'node scripts/check-required-documentation.mjs'}
    ]);
  }
  check(job);
  const mutations=[
    value=>value['timeout-minutes']=4,
    value=>value.steps[0]['timeout-minutes']=3,
    value=>delete value.steps[0].env,
    value=>value.steps[0].env.GIT_HTTP_LOW_SPEED_LIMIT='0',
    value=>value.steps[0].env.GIT_HTTP_LOW_SPEED_TIME='300',
    value=>value.steps[0].with['persist-credentials']=true,
    value=>value.steps[0].with['fetch-depth']=0,
    value=>value.steps[0].with.ref='refs/heads/foreign',
    value=>delete value.steps[0].with.filter,
    value=>value.steps[0].with['sparse-checkout']='/*\n!/docs/\n',
    value=>value.steps[0].with['sparse-checkout']=expectedInputs+'!/scripts/\n',
    value=>value.steps[0].with['sparse-checkout-cone-mode']=true,
    value=>value.steps.pop()
  ];
  for(const mutate of mutations){
    const changed=structuredClone(job);mutate(changed);assert.throws(()=>check(changed));
  }
});

test('documentation sparse inputs cover the static policy closure and locked parser inputs', () => {
  const job=workflows.find(workflow=>workflow.path.endsWith('/documentation-policy.yml')).document.jobs['documentation-policy'];
  const patterns=job.steps.find(step=>step.uses?.startsWith('actions/checkout@')).with['sparse-checkout'].trim().split('\n');
  const covered=file=>patterns.some(pattern=>pattern.endsWith('/')
    ?file.startsWith(pattern.slice(1)):file===pattern.slice(1));
  const pending=['rust/tests/web/documentation-only-ci-policy.test.mjs'];
  const visited=new Set(),external=new Set();
  while(pending.length){
    const file=pending.pop();if(visited.has(file))continue;
    assert.ok(covered(file),'missing static module '+file);visited.add(file);
    const source=readFileSync(file,'utf8');
    for(const line of source.split('\n')){
      const imported=/^\s*(?:import|export)\s+.*?\bfrom\s+['"]([^'"]+)['"]/u.exec(line)?.[1];
      if(imported===undefined||imported.startsWith('node:'))continue;
      if(imported==='yaml'){external.add(imported);continue;}
      assert.ok(imported.startsWith('.'),'unreviewed dependency '+imported);
      const next=posix.normalize(posix.join(posix.dirname(file),imported));
      assert.ok(!next.startsWith('../'));pending.push(next);
    }
  }
  assert.deepEqual([...visited].sort(),[
    'rust/tests/web/documentation-only-ci-policy.test.mjs',
    'scripts/check-required-documentation.mjs',
    'scripts/documentation-only-ci-policy.mjs',
    'scripts/lib/documentation-paths.mjs',
    'scripts/lib/hive-delivery-admission.mjs'
  ]);
  assert.deepEqual([...external],['yaml']);
  assert.ok(patterns.includes('/.github/workflows/'),'complete workflow census required');
  for(const file of ['package.json','scripts/documentation-policy-dependencies/package.json',
    'scripts/documentation-policy-dependencies/package-lock.json'])assert.ok(covered(file),file);
  for (const file of ['data/meta/required-documentation.json', ...Object.keys(requiredDocumentation)]) assert.ok(covered(file), file);
  const dependency=JSON.parse(readFileSync('scripts/documentation-policy-dependencies/package.json','utf8'));
  const lock=JSON.parse(readFileSync('scripts/documentation-policy-dependencies/package-lock.json','utf8'));
  assert.deepEqual(Object.keys(dependency.dependencies),['yaml']);
  assert.equal(dependency.dependencies.yaml,lock.packages['node_modules/yaml'].version);
  assert.match(lock.packages['node_modules/yaml'].integrity,/^sha512-/u);
});

// Execute the retained workflow conditions for a real documentation-only
// classification; an event-level reporting exception cannot admit a build.
function assertDocumentationContextSafety(release, native) {
  const outputs = new Proxy({}, {get: (_, key) => key === 'docs-changed' ? 'true' : 'false'});
  const results = new Map();
  const needs = new Proxy({}, {get: (_, id) => {
    assert.ok(Object.hasOwn(release.jobs, id), 'unknown job dependency ' + id);
    if (!results.has(id)) results.set(id, evaluate(release.jobs[id].if) ? 'success' : 'skipped');
    return {result: results.get(id), outputs};
  }});
  const evaluate = (expression, inputs = {}) => {
    if (expression === undefined) return true;
    const source = String(expression).trim().replace(/^\$\{\{\s*|\s*\}\}$/gu, '')
      .replace(/\.([a-zA-Z_][a-zA-Z0-9_-]*)/gu, (_, key) => '[' + JSON.stringify(key) + ']');
    return Boolean(new Script(source).runInNewContext({
      github: {event_name: 'pull_request', ref: 'refs/pull/1/merge', event: {inputs: {}, workflow_run: {head_branch: 'main'}}},
      needs, inputs, vars: {}, cancelled: () => false, always: () => true,
      fromJSON: JSON.parse, contains: (container, value) => container.includes(value),
    }, {timeout: 100}));
  };
  assert.equal(release.on.pull_request['paths-ignore'], undefined);
  const selected = Object.entries(release.jobs).filter(([, job]) => evaluate(job.if)).map(([id]) => id).sort();
  assert.deepEqual(selected, ['base', 'detect-changes', 'fast-pr-lint', 'hive-source-provenance',
    'native-type-check', 'pipeline-status', 'release-preflight', 'secrets-scan', 'version-check']);
  const call = release.jobs['native-type-check'];
  assert.equal(call.uses, './.github/workflows/native-type-check.yml');
  const compile = evaluate(call.with['compile-inputs-changed']);
  assert.equal(compile, false, 'a complete docs-only observation does not compile');
  const steps = native.jobs.clippy.steps.filter(step => evaluate(step.if, {'compile-inputs-changed': compile}));
  assert.equal(steps.length, 1);
  assert.equal(steps[0].name, 'Report unchanged compile inputs');
  assert.match(steps[0].run, /^echo '::notice::Rust compile inputs unchanged;/u);
  assert.equal(steps[0].uses, undefined);
}

test('documentation PR context reporting runs no native tool, product build or publication job', () => {
  const release = workflows.find(row => row.path.endsWith('/release.yml')).document;
  const native = workflows.find(row => row.path.endsWith('/native-type-check.yml')).document;
  assertDocumentationContextSafety(release, native);
  for (const mutate of [
    value => delete value.jobs['build-artifacts'].if,
    value => value.jobs['native-type-check'].with['compile-inputs-changed'] = true,
    value => value.on.pull_request['paths-ignore'] = ['docs/**'],
  ]) {
    const changed = structuredClone(release); mutate(changed);
    assert.throws(() => assertDocumentationContextSafety(changed, native));
  }
  for (const name of ['Check out native inputs', 'Retry native input checkout once',
    'Require a successful native checkout', 'Type-check and lint the library and binaries']) {
    const changed = structuredClone(native);
    delete changed.jobs.clippy.steps.find(step => step.name === name).if;
    assert.throws(() => assertDocumentationContextSafety(release, changed), name);
  }
});

test('context reporter exemption preserves push filtering and does not apply to other heavy workflows', () => {
  const source = 'on:\n  push:\n  pull_request:\njobs:\n  retained:\n    steps: []\n';
  const reporter = YAML.parse(filteredWorkflow(source, {file: 'release.yml'}));
  assert.ok(reporter.on.push['paths-ignore'].includes('docs/**'));
  assert.equal(reporter.on.pull_request, null);
  const other = YAML.parse(filteredWorkflow(source, {file: 'another-heavy-workflow.yml'}));
  assert.ok(other.on.pull_request['paths-ignore'].includes('docs/**'));
});

test('required public documentation is present with the declared level-two sections', () => {
  assert.deepEqual(documentationProblems(), []);
});

test('missing files and headings cannot be replaced by prose, code examples or deeper headings', () => {
  const root = mkdtempSync(join(tmpdir(), 'required-documentation-'));
  const contract = {'README.md': ['Install', 'Quick Start'], 'CONTRIBUTING.md': ['Development Setup'], 'CHANGELOG.md': []};
  try {
    writeFileSync(join(root, 'README.md'), '# Guide\n## Install\n## Quick Start\n');
    writeFileSync(join(root, 'CONTRIBUTING.md'), '## Development Setup\n');
    writeFileSync(join(root, 'CHANGELOG.md'), '# Changelog\n');
    assert.deepEqual(documentationProblems(root, contract), []);
    for (const file of Object.keys(contract)) {
      const target = join(root, file), original = readFileSync(target);
      unlinkSync(target);
      assert.deepEqual(documentationProblems(root, contract), [file + ': cannot read required documentation (ENOENT)']);
      writeFileSync(target, original);
    }
    for (const replacement of [
      'Quick Start is described here', '### Quick Start', '## Quick Starting',
      '```markdown\n## Quick Start\n```', '~~~\n## Quick Start\n~~~',
      '````markdown\n```\n## Quick Start\n```\n````',
      '```markdown\n~~~\n## Quick Start\n~~~\n```',
      '    ## Quick Start',
    ]) {
      writeFileSync(join(root, 'README.md'), '## Install\n' + replacement + '\n');
      assert.deepEqual(documentationProblems(root, contract), ['README.md: missing required heading "## Quick Start"'], replacement);
    }
    writeFileSync(join(root, 'README.md'), '## Install\r\n  ## Quick Start ###\r\n');
    assert.deepEqual(documentationProblems(root, contract), []);
    unlinkSync(join(root, 'CHANGELOG.md'));mkdirSync(join(root, 'CHANGELOG.md'));
    assert.match(documentationProblems(root, contract)[0], /^CHANGELOG\.md: cannot read required documentation/u);
  } finally { rmSync(root, {recursive:true, force:true}); }
});

test('required documentation uses one contract for listing and execution', () => {
  const run = spawnSync(process.execPath, ['scripts/check-required-documentation.mjs', '--list'], {encoding:'utf8', timeout:5000});
  assert.equal(run.status, 0, run.stderr);
  assert.deepEqual(JSON.parse(run.stdout), requiredDocumentation);
  const invalid = spawnSync(process.execPath, ['scripts/check-required-documentation.mjs', '--unknown'], {encoding:'utf8', timeout:5000});
  assert.equal(invalid.status, 2);
});
