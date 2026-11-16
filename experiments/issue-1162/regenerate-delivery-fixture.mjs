// Issue #1162: regenerate rust/tests/fixtures/issue-1162/delivery-commands.json.
// The JS runtime writes the expected values; the Rust in-crate test
// (tests/unit/agentic-coding/issue_1162_delivery_commands.rs) asserts the same
// values, so the fixture is a Rust/JS parity lane for the delivery commands.
//
// Usage: node experiments/issue-1162/regenerate-delivery-fixture.mjs [--write]
import { writeFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { readText } from '../../js/agentic/host.mjs';
import { programContractAnswer } from '../../js/agentic/crate/coding_program_contract.mjs';
import { attach, requestedIn } from '../../js/agentic/ci_workflow.mjs';
import { supersededWorkflowStep } from '../../js/agentic/git_commit.mjs';
import { prCompletionCommands } from '../../js/agentic/command_reroute.mjs';
import { pullRequestFeedbackFilter } from '../../js/agentic/restart_feedback.mjs';
import { exactSuccessfulStdout } from '../../js/agentic/tool_result.mjs';

await installNodeHost(new WorkerHost());
const path = 'rust/tests/fixtures/issue-1162/delivery-commands.json';
const kotlinIssue = readText('rust/tests/fixtures/issue-1162/hello-world-kotlin-issue.md');
const fixture = {
  issue: 1162,
  purpose: 'Delivery commands both runtimes must emit for one Hello World work item: the superseded-workflow retirement step, the pull-request completion commands, the bounded feedback filter and the exact-stdout read of a client envelope.',
  generated_by: 'experiments/issue-1162/regenerate-delivery-fixture.mjs',
  workflows: [
    { id: 'kotlin-issue-body', prompt_file: 'hello-world-kotlin-issue.md' },
    { id: 'scala-bound-ci-path', prompt: 'Write a Scala program that prints "Hello, World!" to stdout. Create a GitHub Actions workflow that runs it on every push and pull request.\nSave the workflow as ci/test-hello-world.yml' },
    { id: 'rust-without-workflow', prompt: 'Write a Rust program that prints "Hello, World!" to stdout.' },
  ],
  pull_request: { target: 'https://github.com/o/r/pull/2', issue: 'https://github.com/o/r/issues/1' },
  stdout: [
    { id: 'codex-porcelain-leading-space', raw: 'Chunk ID: 1\nWall time: 0.1 seconds\nProcess exited with code 0\nOriginal token count: 3\nOutput:\n M main.rs\n' },
    { id: 'json-envelope-success', raw: '{"output":" M Main.kt\\n?? Main.jar\\n","exit_code":0}' },
    { id: 'codex-failed-command', raw: 'Chunk ID: 2\nWall time: 0.1 seconds\nProcess exited with code 1\nOriginal token count: 3\nOutput:\n M main.rs\n' },
  ],
};
for (const entry of fixture.workflows) {
  const prompt = entry.prompt ?? kotlinIssue;
  const recipe = programContractAnswer(prompt)?.execution_recipe;
  if (!recipe) throw new Error(`no recipe for ${entry.id}`);
  if (requestedIn(prompt)) attach(recipe, prompt);
  entry.program = recipe.path;
  entry.workflow = recipe.supporting_files.find(file => /\.ya?ml$/u.test(file.path))?.path ?? null;
  entry.retire_step = supersededWorkflowStep(recipe);
  if (entry.id === 'kotlin-issue-body') {
    const target = { branch: null, reference: fixture.pull_request.target, bindingRefused: false };
    fixture.pull_request.with_issue = prCompletionCommands(recipe, target, fixture.pull_request.issue);
    fixture.pull_request.without_issue = prCompletionCommands(recipe, target, fixture.pull_request.target);
  }
  console.log(`== ${entry.id}: ${entry.program} ${entry.workflow}\n${entry.retire_step}`);
}
fixture.feedback_filter = pullRequestFeedbackFilter();
for (const entry of fixture.stdout) entry.exact = exactSuccessfulStdout(entry.raw);
console.log(JSON.stringify(fixture.stdout, null, 2));
if (process.argv.includes('--write')) writeFileSync(path, `${JSON.stringify(fixture, null, 2)}\n`);
