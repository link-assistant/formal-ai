// Issue #1162 acceptance items 2 and 7, offline: one Hive Mind Hello World
// work item driven end to end by the JS planner with no LLM, against a real
// temporary git repository, a bare origin and a fake `gh` that serves the
// issue and records every pull request call. No network, no credentials.
//
// Usage:
//   node experiments/issue-1162/offline-hive-e2e.mjs [Kotlin|Rust] [prompt-kind] [rerun]
// prompt-kind: `hive` (Hive Mind's formal-ai prompt, "Update the pull request
// at ...") or `prepared` ("Your prepared Pull Request: ...").
// `rerun` drives a second session on the finished checkout (no-op continuation).
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { planChatStep } from '../../js/agentic/planner.mjs';
import { drive } from '../js_dogfood/drive.mjs';

export const ISSUE = 'https://github.com/example/test-hello-world/issues/1';
export const PR = 'https://github.com/example/test-hello-world/pull/2';
export const BRANCH = 'issue-1-0123456789ab';

export function hivePrompt(kind = 'hive') {
  const pull = kind === 'prepared' ? `Your prepared Pull Request: ${PR}` : `Update the pull request at ${PR}.`;
  return `Resolve the GitHub issue at ${ISSUE} in this repository.\nKeep the solution on branch ${BRANCH}.\n${pull}\n\nImplement and verify the solution before reporting completion.\nProceed.\n`;
}

const FAKE_GH = String.raw`#!/usr/bin/env node
// Fake gh for the offline Hello World e2e: state lives in $FAKE_GH_STATE.
const fs = require('node:fs');
const file = process.env.FAKE_GH_STATE;
const state = JSON.parse(fs.readFileSync(file, 'utf8'));
const args = process.argv.slice(2);
state.calls.push(args);
const flag = name => { const i = args.indexOf(name); return i < 0 ? null : args[i + 1]; };
const save = () => fs.writeFileSync(file, JSON.stringify(state, null, 2));
const [noun, verb] = args;
let out = '';
if (noun === 'issue' && verb === 'view') {
  const field = flag('--json');
  out = field === 'title' ? state.issue.title : field === 'body' ? state.issue.body : JSON.stringify(state.issue);
} else if (noun === 'pr' && verb === 'view') {
  const field = flag('--json');
  if (field === 'title') out = state.pr.title;
  else if (field === 'body') out = state.pr.body;
  else if (field === 'isDraft') out = String(state.pr.isDraft);
  else if (field === 'comments,reviews') {
    // Apply the caller's own jq filter to the recorded feedback, as gh does.
    const { execFileSync } = require('node:child_process');
    out = execFileSync('jq', ['-c', flag('--jq')], { input: JSON.stringify({ comments: state.pr.comments ?? [], reviews: state.pr.reviews ?? [] }), encoding: 'utf8' }).trim();
  } else if (field === 'headRefOid,headRefName,isDraft,state,url') {
    const { execFileSync } = require('node:child_process');
    const head = execFileSync('git', ['ls-remote', 'origin', 'refs/heads/' + state.branch], { encoding: 'utf8' }).split(/\s/u)[0];
    out = JSON.stringify({ headRefOid: head, headRefName: state.branch, isDraft: state.pr.isDraft, state: 'OPEN', url: state.pr.url });
  } else out = '';
} else if (noun === 'pr' && verb === 'checks') {
  // Simulated CI: one passing check per workflow in the pushed head, named as
  // the workflow names itself. Not a real CI run.
  if (flag('--json') !== null) {
    const { execFileSync } = require('node:child_process');
    execFileSync('git', ['fetch', '-q', 'origin', state.branch]);
    const files = execFileSync('git', ['ls-tree', '-r', '--name-only', 'FETCH_HEAD'], { encoding: 'utf8', env: { ...process.env } }).split('\n');
    out = JSON.stringify(files.filter(f => /^(\.github\/workflows|ci)\/[^/]+\.ya?ml$/u.test(f)).map(f => {
      const text = execFileSync('git', ['show', 'FETCH_HEAD:' + f], { encoding: 'utf8' });
      const name = /^name: (.+)$/mu.exec(text)?.[1] ?? f;
      return { bucket: 'pass', name: 'run', workflow: name };
    }));
  }
  state.checks = (state.checks ?? 0) + 1;
} else if (noun === 'pr' && verb === 'edit') {
  if (flag('--title') !== null) state.pr.title = flag('--title');
  if (flag('--body') !== null) state.pr.body = flag('--body');
} else if (noun === 'pr' && verb === 'ready') {
  state.pr.isDraft = false;
} else {
  save();
  process.stderr.write('fake gh: unsupported ' + args.join(' ') + '\n');
  process.exit(2);
}
save();
if (out) process.stdout.write(out + '\n');
`;

function git(dir, ...args) {
  return execFileSync('git', args, { cwd: dir, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}

export function workspace(language) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-1162-e2e-'));
  const dir = path.join(root, 'work');
  const origin = path.join(root, 'origin.git');
  const bin = path.join(root, 'bin');
  fs.mkdirSync(dir);
  fs.mkdirSync(bin);
  git(root, 'init', '-q', '--bare', origin);
  git(dir, 'init', '-q', '-b', BRANCH);
  git(dir, 'config', 'user.name', 'Offline fixture');
  git(dir, 'config', 'user.email', 'fixture@example.invalid');
  fs.writeFileSync(path.join(dir, 'README.md'), '# test-hello-world\n');
  git(dir, 'add', '--', 'README.md');
  git(dir, 'commit', '-qm', 'Initial commit');
  git(dir, 'remote', 'add', 'origin', origin);
  git(dir, 'push', '-q', '-u', 'origin', BRANCH);
  const body = fs.readFileSync('rust/tests/fixtures/issue-1162/hello-world-kotlin-issue.md', 'utf8').replaceAll('Kotlin', language);
  const state = path.join(root, 'gh-state.json');
  fs.writeFileSync(state, JSON.stringify({
    calls: [],
    issue: { title: `Implement Hello World in ${language}`, body },
    branch: BRANCH,
    pr: { title: '[WIP] Implement Hello World', body: 'Work in progress', isDraft: true, url: PR },
  }, null, 2));
  fs.writeFileSync(path.join(bin, 'gh'), FAKE_GH, { mode: 0o755 });
  return { root, dir, origin, bin, state };
}

export async function session(env, prompt, steps = 30) {
  const saved = { PATH: process.env.PATH, FAKE_GH_STATE: process.env.FAKE_GH_STATE };
  process.env.PATH = `${env.bin}:${process.env.PATH}`;
  process.env.FAKE_GH_STATE = env.state;
  try {
    return await drive(planChatStep, env.dir, prompt, { tools: ['bash', 'read', 'write', 'edit'], steps });
  } finally {
    process.env.PATH = saved.PATH;
    if (saved.FAKE_GH_STATE === undefined) delete process.env.FAKE_GH_STATE; else process.env.FAKE_GH_STATE = saved.FAKE_GH_STATE;
  }
}

export function outcome(env) {
  const state = JSON.parse(fs.readFileSync(env.state, 'utf8'));
  return {
    pr: state.pr,
    ghCalls: state.calls.map(call => call.slice(0, 2).join(' ')),
    status: git(env.dir, 'status', '--porcelain'),
    tracked: git(env.dir, 'ls-files').split('\n'),
    pushed: git(env.origin, 'ls-tree', '-r', '--name-only', BRANCH).split('\n'),
    commits: git(env.dir, 'rev-list', '--count', 'HEAD'),
  };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const [language = 'Kotlin', kind = 'hive', rerun] = process.argv.slice(2);
  const env = workspace(language);
  const first = await session(env, hivePrompt(kind));
  for (const entry of first.transcript) {
    const args = JSON.parse(entry.arguments);
    console.log('step', entry.tool, String(args.command ?? args.path ?? args.filePath ?? '').slice(-260).replace(/\n/gu, ' '));
    if (process.env.VERBOSE) console.log('  =>', String(typeof entry.result === 'string' ? entry.result : JSON.stringify(entry.result)).slice(0, 400).replace(/\n/gu, ' '));
  }
  console.log('stop', first.stop);
  console.log('answer', String(first.answer ?? '').slice(-900));
  console.log(JSON.stringify(outcome(env), null, 1));
  if (rerun) {
    const second = await session(env, hivePrompt(kind));
    console.log('rerun stop', second.stop);
    console.log('rerun answer', String(second.answer ?? '').slice(-700));
    console.log(JSON.stringify(outcome(env), null, 1));
  }
  if (!process.env.KEEP) fs.rmSync(env.root, { recursive: true, force: true });
}
