// `crate::orchestration::dispatch` (rust/src/orchestration/dispatch.rs): fan a
// task out across external agent CLIs. Decompose mode splits the task once and
// sends independent leaves to the CLIs in parallel; compare mode gives every CLI
// the same task and selects one verified winner; incremental mode (see
// orchestration_incremental.mjs) attempts the whole task first. Every candidate
// runs in its own copy of the workspace, and only a passing candidate's effects
// are composed back, validated against drift first.
//
// Rust joins one OS thread per candidate; JavaScript starts every candidate's
// `runAgent` promise and settles all of them before returning an error, so a
// failing worker never leaves another one running behind the caller.

import fs from 'node:fs';
import path from 'node:path';

import { prepareAttribution } from './orchestration_attribution.mjs';
import { DispatchError } from './orchestration_dispatch_error.mjs';
import { dispatchIncrementally } from './orchestration_incremental.mjs';
import { grantFor, noPermission, permits } from './orchestration_permission.mjs';
import { AgentRunError, agentRunConfig, runAgent, sessionDiffSize, sessionPassed } from './orchestration_runner.mjs';
import { canonicalSessionText } from './orchestration_replay.mjs';
import { writeSession } from './orchestration_session_file.mjs';
import {
  applyChanges, copyWorkspace, ioError, ioErrorFrom, validateChanges,
} from './orchestration_workspace.mjs';
import { decomposeTask, leaves } from './task_decomposition_tree.mjs';

/** Mirrors `DispatchConfig::new` in rust/src/orchestration/dispatch.rs: the defaults a dispatch starts from. */
export function dispatchConfig(task, workspace, clis) {
  const run = agentRunConfig('', '', workspace);
  return {
    task,
    workspace: String(workspace),
    clis,
    mode: 'decompose',
    output_dir: path.join(String(workspace), '.formal-ai-orchestration'),
    model: run.model,
    base_url: run.base_url,
    target: run.target,
    timeout_ms: run.timeout_ms,
    permission: noPermission(),
    allowlisted_agent_commands: new Set(),
    allowlisted_commands: new Set(),
    verification: [],
    controller_program: run.controller_program,
    command_overrides: new Map(),
    max_depth: 3,
    pull_request: null,
  };
}

/** Mirrors `ComparisonLedger::select_winner` in rust/src/orchestration/dispatch.rs: the passing entry with the least diff, then time, cli, file. */
export function selectWinner(entries) {
  const text = (left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right));
  const candidates = entries.filter((entry) => entry.passed).sort((left, right) =>
    left.diff_size - right.diff_size
    || left.wall_time_ms - right.wall_time_ms
    || text(left.cli, right.cli)
    || text(left.session_file, right.session_file));
  return candidates.length > 0 ? candidates[0].cli : null;
}

/** Mirrors `fn safe_name` in rust/src/orchestration/dispatch.rs. */
export function safeName(value) {
  return Array.from(value, (character) => (/^[A-Za-z0-9_-]$/.test(character) ? character : '_')).join('');
}

/** `{index:03}`. */
export const pad3 = (index) => String(index).padStart(3, '0');

/** The path components, `.` dropped (`Path::components`). */
const partsOf = (value) => value.split(path.sep).filter((part) => part !== '' && part !== '.');

/**
 * Mirrors `fn prospective_canonical` in rust/src/orchestration/dispatch.rs: the
 * canonical form of a path that may not exist yet, resolved through its
 * deepest existing ancestor.
 */
function prospectiveCanonical(target) {
  if (target.split(path.sep).includes('..') || target.split('/').includes('..')) throw new DispatchError('output_outside_workspace');
  const absolute = path.isAbsolute(target) ? target : path.join(process.cwd(), target);
  let ancestor = absolute;
  const suffix = [];
  while (!fs.existsSync(ancestor)) {
    const name = path.basename(ancestor);
    const parent = path.dirname(ancestor);
    if (name === '' || parent === ancestor) {
      throw new DispatchError('io', ioError('InvalidInput', 'output_path_has_no_existing_ancestor'));
    }
    suffix.push(name);
    ancestor = parent;
  }
  let resolved;
  try {
    resolved = fs.realpathSync(ancestor);
  } catch (error) {
    throw new DispatchError('io', ioErrorFrom(error));
  }
  for (const name of suffix.reverse()) resolved = path.join(resolved, name);
  return resolved;
}

/** Mirrors `fn prepare_output_dir` in rust/src/orchestration/dispatch.rs. */
function prepareOutputDir(target) {
  try {
    if (fs.existsSync(target) && fs.readdirSync(target).length > 0) {
      throw ioError('AlreadyExists', 'orchestration_output_not_empty');
    }
    fs.mkdirSync(path.join(target, 'candidates'), { recursive: true });
    fs.mkdirSync(path.join(target, 'sessions'), { recursive: true });
  } catch (error) {
    throw new DispatchError('io', ioErrorFrom(error));
  }
}

/**
 * Mirrors `fn candidate_run_config` in rust/src/orchestration/dispatch.rs: the
 * run configuration of one candidate, granted exactly its own workspace.
 */
export function candidateRunConfig(dispatch, cli, task, workspace, orchestrationHome) {
  const run = agentRunConfig(cli, task, workspace);
  run.permission = grantFor(workspace);
  run.model = dispatch.model;
  run.base_url = dispatch.base_url;
  run.target = dispatch.target;
  run.timeout_ms = dispatch.timeout_ms;
  run.allowlisted_agent_commands = new Set(dispatch.allowlisted_agent_commands);
  run.allowlisted_commands = new Set(dispatch.allowlisted_commands);
  run.verification = [...dispatch.verification];
  run.controller_program = dispatch.controller_program;
  run.orchestration_home = orchestrationHome;
  run.command_override = dispatch.command_overrides.get(cli) ?? null;
  return run;
}

/** Two workspace changes are the same effect (`WorkspaceChange: PartialEq`). */
const sameChange = (left, right) => left.path === right.path && left.kind === right.kind
  && left.before_sha256 === right.before_sha256 && left.after_sha256 === right.after_sha256
  && left.bytes_changed === right.bytes_changed;

const byteOrder = (left, right) => Buffer.compare(Buffer.from(left), Buffer.from(right));

/** Mirrors `fn compose_decomposed` in rust/src/orchestration/dispatch.rs. */
function composeDecomposed(workspace, completed) {
  const owners = new Map();
  for (const entry of completed.filter((item) => sessionPassed(item.session))) {
    for (const change of entry.session.changes) {
      const prior = owners.get(change.path);
      if (prior) {
        if (!sameChange(prior.change, change)) throw new DispatchError('composition_conflict', change.path);
      } else {
        owners.set(change.path, { candidate: entry.candidate, change });
      }
    }
  }
  const ordered = [...owners.keys()].sort(byteOrder).map((key) => owners.get(key));
  try {
    for (const owner of ordered) validateChanges(workspace, owner.candidate, [owner.change]);
    for (const owner of ordered) applyChanges(workspace, owner.candidate, [owner.change]);
  } catch (error) {
    throw new DispatchError('io', ioErrorFrom(error));
  }
  return ordered.map((owner) => owner.change);
}

/** Mirrors `fn compose` in rust/src/orchestration/dispatch.rs (decompose and compare modes). */
function compose(mode, workspace, completed, winner) {
  if (mode === 'compare') {
    const selected = completed.find((entry) => entry.cli === winner);
    if (!selected) return [];
    try {
      applyChanges(workspace, selected.candidate, selected.session.changes);
    } catch (error) {
      throw new DispatchError('io', ioErrorFrom(error));
    }
    return selected.session.changes;
  }
  return composeDecomposed(workspace, completed);
}

/** Wrap a rejection from a candidate worker the way `JoinHandle::join` reports it. */
const workerError = (error) => (error instanceof AgentRunError ? new DispatchError('run', error) : new DispatchError('worker_panicked'));

/**
 * Mirrors `fn dispatch_agents` in rust/src/orchestration/dispatch.rs.
 * @param {ReturnType<typeof dispatchConfig>} config
 * @returns {Promise<object>} the `DispatchReport`
 */
export async function dispatchAgents(config) {
  if (!permits(config.permission, config.workspace)) throw new DispatchError('permission_denied');
  if (config.clis.length === 0) throw new DispatchError('missing_cli');
  const unique = new Set();
  for (const cli of config.clis) {
    if (unique.has(cli)) throw new DispatchError('duplicate_cli', cli);
    unique.add(cli);
  }
  let workspace;
  try {
    workspace = fs.realpathSync(config.workspace);
  } catch (error) {
    throw new DispatchError('io', ioErrorFrom(error));
  }
  if (config.pull_request !== null) {
    if (config.mode !== 'incremental') throw new DispatchError('attribution', 'incremental_mode_required');
    prepareAttribution(workspace, config.pull_request);
  }
  const prospectiveOutput = prospectiveCanonical(config.output_dir);
  const outputParts = partsOf(prospectiveOutput);
  const workspaceParts = partsOf(workspace);
  if (outputParts.length === workspaceParts.length && outputParts.every((part, index) => part === workspaceParts[index])
    || !workspaceParts.every((part, index) => part === outputParts[index])) {
    throw new DispatchError('output_outside_workspace');
  }
  prepareOutputDir(config.output_dir);
  let outputDir;
  try {
    outputDir = fs.realpathSync(config.output_dir);
  } catch (error) {
    throw new DispatchError('io', ioErrorFrom(error));
  }
  if (config.mode === 'incremental') {
    // The whole task is the plan until a failure says otherwise, so this mode
    // never builds a job list: it discovers one.
    return dispatchIncrementally(config, workspace, outputDir);
  }
  let tasks;
  if (config.mode === 'compare') {
    tasks = config.clis.map(() => config.task);
  } else {
    const found = leaves(decomposeTask(config.task, config.max_depth));
    tasks = found.length === 0 ? [config.task] : found.map((leaf) => leaf.text);
  }
  const jobs = config.mode === 'compare'
    ? config.clis.map((cli, index) => [cli, tasks[index]])
    : tasks.map((task, index) => [config.clis[index % config.clis.length], task]);
  const handles = [];
  let setupError = null;
  for (const [index, [cli, task]] of jobs.entries()) {
    try {
      const candidateId = `${pad3(index)}-${safeName(cli)}`;
      const candidate = path.join(config.output_dir, 'candidates', candidateId);
      try {
        copyWorkspace(workspace, candidate, outputDir);
      } catch (error) {
        throw new DispatchError('io', ioErrorFrom(error));
      }
      const home = path.join(outputDir, 'native-sessions', candidateId);
      const run = candidateRunConfig(config, cli, task, candidate, home);
      handles.push({ index, cli, candidate, promise: runAgent(run) });
    } catch (error) {
      setupError = error;
      break;
    }
  }
  const settled = await Promise.allSettled(handles.map((handle) => handle.promise));
  if (setupError !== null) throw setupError;
  const completed = [];
  let firstError = null;
  for (const [position, handle] of handles.entries()) {
    const result = settled[position];
    if (result.status === 'fulfilled') completed.push({ index: handle.index, cli: handle.cli, candidate: handle.candidate, session: result.value });
    else if (firstError === null) firstError = workerError(result.reason);
  }
  if (firstError !== null) throw firstError;
  completed.sort((left, right) => left.index - right.index);
  const entries = [];
  for (const { index, cli, session } of completed) {
    const relative = `sessions/${pad3(index)}-${safeName(cli)}.json`;
    try {
      writeSession(path.join(config.output_dir, relative), session);
    } catch (error) {
      throw new DispatchError('replay', error);
    }
    entries.push({
      cli,
      task: session.task,
      passed: sessionPassed(session),
      diff_size: sessionDiffSize(session),
      wall_time_ms: session.wall_time_ms,
      session_file: relative,
    });
  }
  const winner = config.mode === 'compare' ? selectWinner(entries) : null;
  const composedChanges = compose(config.mode, workspace, completed, winner);
  const ledger = {
    schema: 'formal-ai-comparison-ledger-v1',
    selection_rule: 'pass,diff_size,wall_time,cli,session_file',
    entries,
    winner,
  };
  try {
    fs.writeFileSync(path.join(config.output_dir, 'comparison-ledger.json'), canonicalSessionText(ledger));
  } catch (error) {
    throw new DispatchError('io', ioErrorFrom(error));
  }
  return {
    schema: 'formal-ai-dispatch-report-v1',
    mode: config.mode,
    tasks,
    sessions: completed.map((entry) => entry.session),
    ledger,
    composed_changes: composedChanges,
  };
}

