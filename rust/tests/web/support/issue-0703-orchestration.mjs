// Shared fixtures of the issue #703 orchestration tests in JavaScript
// (rust/tests/web/issue-0703-orchestration-*.test.mjs): the temp workspaces, the
// node "external agent" process that stands in for the Rust tests' re-executed
// test binary (rust/tests/integration/issue_703_orchestration.rs
// `external_agent_fixture_process`), executable fake client scripts, and the
// helpers that run the CLI twin (js/agentic/orchestration_cli.mjs) as a process.

import { spawnSync } from 'node:child_process';
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { grantFor } from '../../../../js/agentic/crate/orchestration_permission.mjs';
import { agentRunConfig, withCommand, withPermission } from '../../../../js/agentic/crate/orchestration_runner.mjs';
import { installNodeHost } from '../../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../../js/server/worker-host.mjs';

export const ROOT = fileURLToPath(new URL('../../../../', import.meta.url));
export const CLI = path.join(ROOT, 'js/agentic/orchestration_cli.mjs');
export const FIXTURE_ENV = 'FORMAL_AI_ISSUE_703_FIXTURE';
export const FIXTURE_RELEASE_ENV = 'FORMAL_AI_ISSUE_703_RELEASE';
export const FIXTURE_STARTED_ENV = 'FORMAL_AI_ISSUE_703_STARTED';
export const OUTPUT_ENV = 'FORMAL_AI_ISSUE_703_OUTPUT';
export const REQUIRED_CLIS = ['agent', 'claude', 'codex', 'gemini', 'qwen', 'opencode'];

/** A committed repository file as text. */
export const readCommitted = (relative) => readFileSync(path.join(ROOT, relative), 'utf8');

/** Boot the worker realm and install the planner host (the lexicon the splitter reads). */
export async function bootHost() {
  await installNodeHost(new WorkerHost());
}

const temps = [];

/** A fresh temp directory, removed by `cleanupTemps`. */
export function makeTemp(label) {
  const directory = mkdtempSync(path.join(tmpdir(), `formal-ai-issue-703-js-${label}-`));
  temps.push(directory);
  return directory;
}

/** Remove every directory `makeTemp` handed out. */
export function cleanupTemps() {
  while (temps.length > 0) rmSync(temps.pop(), { recursive: true, force: true });
}

/** The node program standing in for the external agent; `FORMAL_AI_ISSUE_703_FIXTURE` picks its behaviour. */
const FIXTURE_SOURCE = `
import { spawn } from 'node:child_process';
import { existsSync, writeFileSync } from 'node:fs';

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const mode = process.env.${FIXTURE_ENV};
const output = process.env.${OUTPUT_ENV};
const nativeSession = (id) => process.stderr.write(\`formal-ai: orchestration-session-json:{"id":"\${id}","resume_command":"agent --resume \${id}"}\\n\`);
switch (mode) {
  case 'success':
    writeFileSync('README.md', 'fixture change\\n');
    console.log(output ?? 'fixture_stdout');
    console.error('fixture_stderr');
    break;
  case 'native_session':
    console.log(output ?? 'fixture_stdout');
    nativeSession('ses_issue_703');
    break;
  case 'mismatched_native_session':
    nativeSession('ses_fresh_703');
    break;
  case 'delayed_success':
    await sleep(150);
    writeFileSync('README.md', 'fixture change\\n');
    break;
  case 'coordinated_success': {
    writeFileSync(process.env.${FIXTURE_STARTED_ENV}, 'started\\n');
    const deadline = Date.now() + 10000;
    while (!existsSync(process.env.${FIXTURE_RELEASE_ENV})) {
      if (Date.now() > deadline) throw new Error('fixture release was not signalled');
      await sleep(10);
    }
    writeFileSync('README.md', 'fixture change\\n');
    break;
  }
  case 'timeout':
    await sleep(5000);
    break;
  case 'descendant_timeout': {
    const descendant = spawn('sh', ['-c', 'sleep 20; printf escaped > descendant-survived'], { stdio: 'ignore' });
    writeFileSync('descendant-pid', String(descendant.pid));
    await sleep(30000);
    break;
  }
  case 'failed':
    process.exit(7);
    break;
  case 'failed_change':
    writeFileSync('README.md', 'failed agent change\\n');
    process.exit(7);
    break;
  case 'assert_pwd':
    if (process.env.PWD !== process.cwd()) process.exit(1);
    break;
  case 'slow_verification':
    await sleep(2000);
    break;
  default:
    throw new Error('unknown fixture mode');
}
`;

/** Write the fixture program into `directory`; returns its path. */
export function writeFixture(directory) {
  const file = path.join(directory, 'external-agent-fixture.mjs');
  writeFileSync(file, FIXTURE_SOURCE);
  return file;
}

/** The command line of the fixture in `mode` (`fixture_command`): `{program, args, env}`. */
export function fixtureCommand(fixture, mode, env = {}) {
  return { program: process.execPath, args: [fixture], env: { [FIXTURE_ENV]: mode, ...env } };
}

/**
 * `fixture_config` / `fixture_config_with_output`: a run of the fixture in `mode`
 * under the registered label `cli`, granted its workspace and its own program.
 */
export function fixtureConfig(fixture, cli, workspace, mode, output = null) {
  const command = fixtureCommand(fixture, mode, output === null ? {} : { [OUTPUT_ENV]: output });
  const config = withCommand(withPermission(agentRunConfig(cli, 'fixture task', workspace), grantFor(workspace)), command);
  config.allowlisted_agent_commands.add(command.program);
  return config;
}

/** An executable script `name` in `directory` with `body`; returns its path. */
export function executableScript(directory, name, body) {
  const file = path.join(directory, name);
  writeFileSync(file, body);
  chmodSync(file, 0o755);
  return file;
}

/** A `PATH` value that puts `directory` first. */
export const pathWith = (directory) => `${directory}${path.delimiter}${process.env.PATH ?? ''}`;

/** Run `action` with `PATH` prefixed by `directory`, restoring it afterwards. */
export async function withPath(directory, action) {
  const previous = process.env.PATH;
  process.env.PATH = pathWith(directory);
  try {
    return await action();
  } finally {
    process.env.PATH = previous;
  }
}

/** Run the CLI twin as a process: `{status, stdout, stderr}`. */
export function runCli(args, env = {}) {
  const result = spawnSync(process.execPath, [CLI, ...args], {
    env: { ...process.env, ...env },
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
  });
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

/** Whether `pid` is a running process (a zombie `Z` state has already terminated). */
export function processIsRunning(pid) {
  const observation = spawnSync('ps', ['-o', 'state=', '-p', String(pid)], { encoding: 'utf8' });
  if (observation.status !== 0) return false;
  const state = observation.stdout.trim();
  return state !== '' && !state.startsWith('Z');
}
