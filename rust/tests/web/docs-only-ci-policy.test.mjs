import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import YAML from 'yaml';
import {checkDocumentationPolicy, filteredWorkflow, lightweightWorkflows} from '../../../scripts/docs-only-ci-policy.mjs';
import {applicableWorkflows} from '../../../scripts/lib/hive-delivery-admission.mjs';
import {documentationOnlyPullRequest} from '../../../scripts/lib/documentation-paths.mjs';
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
  const source = 'name: sample\non:\n  push:\n    branches: [main]\n  pull_request:\n    paths: ["rust/**"]\n  workflow_dispatch:\n  schedule:\n    - cron: "0 0 * * *"\njobs:\n  retained:\n    runs-on: ubuntu-latest\n    steps: []\n';
  const updated = filteredWorkflow(source);
  assert.equal(updated.split('jobs:')[1], source.split('jobs:')[1]);
  assert.deepEqual(YAML.parse(updated).on.workflow_dispatch, YAML.parse(source).on.workflow_dispatch);
  assert.deepEqual(YAML.parse(updated).on.schedule, YAML.parse(source).on.schedule);
  assert.equal(filteredWorkflow(updated), updated);
});
test('automatic bot checks require complete file observations and both sides of renames', () => {
  assert.equal(documentationOnlyPullRequest([[{filename:'docs/guide.md'}]],1),true);
  assert.equal(documentationOnlyPullRequest([[{filename:'docs/guide.md',previous_filename:'rust/src/lib.rs'}]],1),false);
  for (const [pages,count] of [[[],0],[[[]],2],[[[{filename:'docs/guide.md'}]],3001],[[[{filename:'data/seed/runtime.md'}]],1]])
    assert.equal(documentationOnlyPullRequest(pages,count),false);
});
