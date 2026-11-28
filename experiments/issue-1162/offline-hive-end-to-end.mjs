// Issue #1162 acceptance items 2 and 7, offline: one Hive Mind Hello World
// work item driven end to end by the JS planner with no LLM, against a real
// temporary git repository, a bare origin and a fake `gh` that serves the
// issue and records every pull request call. No network, no credentials.
//
// Usage:
//   node experiments/issue-1162/offline-hive-end-to-end.mjs [Kotlin|Rust] [prompt-kind] [rerun]
// prompt-kind: `hive` (Hive Mind's formal-ai prompt, "Update the pull request
// at ..."), `exact` (that prompt verbatim from the 2026-09-27 Kotlin run),
// `regular` (Hive's regular prompt), `restart` (its auto-restart after `?? Main.jar`) or `prepared` ("Your prepared Pull Request: ...").
// `rerun` drives a second session on the finished checkout (no-op continuation).
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import {offlineSourceCache} from './offline-source-cache.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';
import { drive } from '../js_dogfood/drive.mjs';

export const ISSUE = 'https://github.com/example/test-hello-world/issues/1';
export const PR = 'https://github.com/example/test-hello-world/pull/2';
export const BRANCH = 'issue-1-0123456789ab';

export function hivePrompt(kind = 'hive') {
  if (PROMPTS[kind]) {
    return fs.readFileSync(PROMPTS[kind], 'utf8').replace(RECORDED, match => match.endsWith('issues/1') ? ISSUE : match.endsWith('pull/2') ? PR : BRANCH);
  }
  const pull = kind === 'prepared' ? `Your prepared Pull Request: ${PR}` : `Update the pull request at ${PR}.`;
  return `Resolve the GitHub issue at ${ISSUE} in this repository.\nKeep the solution on branch ${BRANCH}.\n${pull}\n\nImplement and verify the solution before reporting completion.\nProceed.\n`;
}

// The fake `gh` is shared with the Rust offline delivery tests
// (rust/tests/fixtures/hive-mind/offline_delivery.rs).
const FAKE_GH = 'rust/tests/fixtures/hive-mind/fake-gh.cjs';
// Hive Mind's exact formal-ai prompt from the 2026-09-27 Kotlin run, and its
// regular prompt; their work item URLs are replaced with the offline ones.
const PROMPTS = {
  exact: 'rust/tests/fixtures/hive-mind/formal-ai-prompt.txt',
  regular: 'rust/tests/fixtures/hive-mind/regular-prompt.txt',
  restart: 'rust/tests/fixtures/hive-mind/restart-uncommitted-main-jar.txt',
};
const RECORDED = /https:\/\/github\.com\/konard\/test-hello-world-[0-9a-f-]+\/(issues\/1|pull\/2)|issue-1-604f2202fd18/gu;

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
  fs.copyFileSync(FAKE_GH, path.join(bin, 'gh'));
  fs.chmodSync(path.join(bin, 'gh'), 0o755);
  return { root, dir, origin, bin, state };
}

export async function session(env, prompt, steps = 30) {
  const restoreSourceCache = offlineSourceCache();
  const saved = { PATH: process.env.PATH, FAKE_GH_STATE: process.env.FAKE_GH_STATE };
  process.env.PATH = `${env.bin}:${process.env.PATH}`;
  process.env.FAKE_GH_STATE = env.state;
  try {
    return await drive(planChatStep, env.dir, prompt, { tools: ['bash', 'read', 'write', 'edit'], steps });
  } finally {
    restoreSourceCache();
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
