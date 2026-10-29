import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import YAML from 'yaml';
import {checkDocumentationPolicy, filteredWorkflow, lightweightWorkflows} from '../../../scripts/documentation-only-ci-policy.mjs';
import {applicableWorkflows} from '../../../scripts/lib/hive-delivery-admission.mjs';
import {documentationOnlyPullRequest} from '../../../scripts/lib/documentation-paths.mjs';
import {Script} from 'node:vm';
const workflows = readdirSync('.github/workflows').filter(name => name.endsWith('.yml')).map(name => ({path: '.github/workflows/' + name, document: YAML.parse(readFileSync('.github/workflows/' + name, 'utf8'))}));
test('all heavy push and PR triggers carry the generated documentation policy', () => {
  assert.ok(checkDocumentationPolicy() > 10);
});
for (const event of ['push', 'pull_request']) {
  for (const files of [['docs/associative-tech-stack.md'], ['VISION.md','CONTRIBUTING.md'], ['docs/guide.md','changelog.d/pending.md'], ['rust/README.md']]) {
    test(event + ': documentation paths select lightweight checks only: ' + files.join(','), () => {
      const selected = applicableWorkflows(workflows, {event, baseBranch: 'main', files});
      assert.ok(selected.includes('.github/workflows/documentation-policy.yml'));
      for (const path of selected) assert.ok(lightweightWorkflows.has(path.split('/').at(-1)), path);
      assert.ok(!selected.includes('.github/workflows/release.yml'));
    });
  }
  for (const path of ['rust/src/lib.rs', 'js/agentic/planner.mjs', 'data/seed/runtime.md', '.github/workflows/release.yml', 'Cargo.toml', 'unknown-input']) {
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
  const evaluate = ref => new Script(expression).runInNewContext({github:{ref}}, {timeout:100});
  assert.equal(evaluate('refs/pull/1/merge'),true);
  assert.equal(evaluate('refs/heads/fix/example'),true);
  assert.equal(evaluate('refs/heads/main'),false);
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
