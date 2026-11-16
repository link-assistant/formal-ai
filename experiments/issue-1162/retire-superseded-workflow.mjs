// Issue #1162: a resumed session wrote .github/workflows/Main.yml beside the
// run.yml an earlier session committed for the same program, so the pull
// request ran Hello World twice. This replays that tree in a scratch
// repository and runs the generated recipe commit twice.
//
// Usage: node experiments/issue-1162/retire-superseded-workflow.mjs
import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { recipeCommitCommand } from '../../js/agentic/git_commit.mjs';
import { attach } from '../../js/agentic/ci_workflow.mjs';

await installNodeHost(new WorkerHost());
const root = mkdtempSync(join(tmpdir(), 'issue-1162-retire-'));
const remote = join(root, 'remote.git');
const work = join(root, 'work');
const sh = (cmd, cwd = work) => execFileSync('sh', ['-c', cmd], { cwd, encoding: 'utf8' });
try {
  sh(`git init -q --bare ${remote} && git init -q -b issue-1 ${work}`, root);
  sh('git config user.email t@example.invalid && git config user.name t && git remote add origin ' + remote);

  const recipe = {
    path: 'Main.kt',
    language: 'kotlin',
    source: 'fun main() = println("Hello, World!")\n',
    commands: ['sh tests/verify-output.sh'],
    supporting_files: [],
  };
  attach(recipe, 'Create a GitHub Actions workflow that runs the program on every push and pull request.');
  const workflow = recipe.supporting_files.find((file) => file.path.startsWith('.github/workflows/'));
  if (!workflow) throw new Error('no workflow attached');

  // The earlier session's tree: its run.yml carries the generator header for
  // the same program but older pins and one more step.
  mkdirSync(join(work, '.github/workflows'), { recursive: true });
  const stale = workflow.source.replace(/\n {6}- run: \|/, '\n      - run: |\n          kotlinc Main.kt\n      - run: |');
  writeFileSync(join(work, '.github/workflows/run.yml'), stale.replace(/^# [^R][^\n]*\n/gm, ''));
  writeFileSync(join(work, '.github/workflows/lint.yml'), 'name: Lint\non: push\njobs: {}\n');
  writeFileSync(join(work, '.github/workflows/other.yml'), stale.replaceAll('Main.kt', 'Other.kt'));
  writeFileSync(join(work, 'Main.kt'), recipe.source);
  sh('git add -A && git commit -q -m earlier && git push -q origin issue-1');

  // The resumed session writes the program and its workflow, then commits.
  writeFileSync(join(work, workflow.path), workflow.source);
  const command = recipeCommitCommand(recipe, { branch: null, reference: 'https://github.com/o/r/issues/1' });
  console.log('first run:', sh(command).trim());
  const tracked = sh('git ls-files .github/workflows').trim().split('\n');
  console.log('tracked workflows:', tracked.join(', '));
  console.log('last commit:', sh('git show --stat --format=%s HEAD').trim().replaceAll('\n', ' | '));
  const ok1 = !tracked.includes('.github/workflows/run.yml')
    && tracked.includes(workflow.path)
    && tracked.includes('.github/workflows/lint.yml')
    && tracked.includes('.github/workflows/other.yml')
    && !existsSync(join(work, '.github/workflows/run.yml'))
    && sh('git status --porcelain') === ''
    && sh('git rev-parse HEAD') === sh(`git --git-dir=${remote} rev-parse issue-1`);
  const before = sh('git rev-parse HEAD');
  const second = sh(command).trim();
  console.log('second run:', second);
  const ok2 = second.includes('__formal_ai_commit_unchanged') && sh('git rev-parse HEAD') === before;
  console.log(ok1 && ok2 ? 'PASS' : 'FAIL');
  process.exitCode = ok1 && ok2 ? 0 : 1;
} finally {
  rmSync(root, { recursive: true, force: true });
}
