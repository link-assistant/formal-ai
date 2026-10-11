#!/usr/bin/env node
// Drive the JavaScript Formal AI planner (`planChatStep`, the function the JS
// server runs for /v1/chat/completions in agent mode) through a whole agentic
// session in-process, executing every planned tool call against a real
// directory the way the link-assistant Agent CLI would.
//
// This is the fast inner loop of the PR #1188 dogfooding ledger
// (docs/case-studies/pull-request-1188/formal-ai-dogfood.md): no server, no
// CLI, no Rust build. A task that passes here is then confirmed through the
// real Agent CLI against `node js/server/main.mjs --agent-mode`.
//
// Usage:
//   node experiments/js_dogfood/drive.mjs --dir <sandbox> [--steps 12] <prompt>
//
// The legacy execute projection keeps Agent CLI numbered Read blocks.
// drive attaches actual raw Read bytes and provider status in separate fields;
// Write/Edit text receipts and actual Bash process observations keep their APIs.

import {createHash} from 'node:crypto';
import { appendDefinition, projectAppendContracts } from '../../js/agentic/append_contract.mjs';
import { atomicRecordAppend } from './atomic-record-append.mjs';
import { exclusiveCreate } from './exclusive-create.mjs';
import { ownsCompleteLiteralRequest, ownedAdditiveLiteralFrame, ownedDeclaredCreateFrame, goalLedger } from '../../js/agentic/planner/owned_goals.mjs';
import { parseWriteContract, composeGeneralChangePlan } from '../../js/agentic/general_planner.mjs';
import { planGeneralChangeStep } from '../../js/agentic/general_execution.mjs';
import { planChatStep as maintainedPlanChatStep, planChatStepResolved as maintainedResolvedStep } from '../../js/agentic/planner.mjs';
import { writesWholeFile } from '../../js/agentic/literal_write_guard.mjs';
import { quotedSegmentSpans } from '../../js/agentic/crate/normal_markov.mjs';
import { mentionsRole } from '../../js/agentic/crate/seed_meanings.mjs';
import { normalizePrompt } from '../../js/agentic/crate/engine.mjs';
import { collectionSummaryOwns } from '../../js/agentic/planner/collection_summary.mjs';
import { bareSurfaces, deliveredWriteTarget, firstActionCueStart, rankedBindings, tokens } from '../../js/agentic/write_request.mjs';
import { ownedReadPaths, readPolicyBlocksPlan, modePathsForClause } from '../../js/agentic/file_read/ownership.mjs';
import { sentences } from '../../js/agentic/shell_command_policy.mjs';
import { namesCallableArtifact } from '../../js/agentic/evidence_record/artifact_header.mjs';
import { quoteFault } from '../../js/agentic/crate/normal_markov.mjs';
import { shellCapture } from './shell-capture.mjs';
import { grepCapture } from './grep-capture.mjs';
import { lstatSync, realpathSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import { template } from '../../js/agentic/work_item_steps.mjs';

import { WorkerHost } from '../../js/server/worker-host.mjs';
import { hasHost, host } from '../../js/agentic/host.mjs';
import { installDefaultNodeSourceHost } from '../../js/server/default-node-source-bootstrap.mjs';

/** How long one bash call may run before the driver kills it. */
const BASH_TIMEOUT_MS = Number(process.env.FORMAL_AI_BASH_TIMEOUT_MS ?? 60000);

// Only advertise adapters this in-process driver actually executes.
function ownedReadReportFrame(request) {
  if (quoteFault(request) !== null) return null;
  // The shared seeded grammar consumes the entire request, including its topic data domain.
  let sources = modePathsForClause(request);
  if (sources === null) {
    const clauses = sentences(request);
    if (clauses.length < 3) return null;
    sources = [];
    for (const clause of clauses.slice(0, -2)) {
      const paths = modePathsForClause(clause.text);
      if (paths === null) return null;
      sources.push(...paths);
    }
    const finalSources = modePathsForClause(request.slice(clauses.at(-2).span.start));
    if (finalSources === null) return null;
    sources.push(...finalSources);
  }
  const all = tokens(request);
  const action = firstActionCueStart(all);
  const target = deliveredWriteTarget(request);
  const binding = rankedBindings(all).find(item => item.path === target);
  if (action === null || target === null || !binding
      || binding.index !== all.length - 1 || namesCallableArtifact(request, target)) return null;
  let header = request.slice(action, all[binding.index].start);
  for (const span of quotedSegmentSpans(header)) {
    header = header.slice(0, span.start) + ' '.repeat(span.end - span.start) + header.slice(span.end);
  }
  if (!mentionsRole('evidence-report-artifact-kind', header)) return null;
  return Object.freeze({ sources, target });
}

export const AGENT_CLI_TOOLS = ['bash', 'edit', 'grep', 'list', 'read', 'write'];

function argsOf(call) {
  try {
    return JSON.parse(call.arguments);
  } catch {
    return {};
  }
}

/** The repository root this driver sits in. */
export const REPOSITORY_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
/** Formal AI's plan-event folder, committed in this repository. */
const PLAN_EVENTS = join(REPOSITORY_ROOT, '.formal-ai');
/** Where a run from the repository root writes its plan events instead (git-ignored). */
export const PLAN_EVENTS_SANDBOX = join(REPOSITORY_ROOT, 'experiments/formal_ai_subagent/sandboxes/plan-events');

function within(dir, path) {
  const full = isAbsolute(path) ? path : join(dir, path);
  const resolved = resolve(full);
  if (resolved !== resolve(dir) && !resolved.startsWith(resolve(dir) + sep)) throw new Error(`path escapes the sandbox: ${path}`);
  // Run from the repository root, a plan event goes to a git-ignored sandbox,
  // never over the committed `.formal-ai/` plan log (PR #1188 G15).
  if (resolve(dir) === REPOSITORY_ROOT && (resolved === PLAN_EVENTS || resolved.startsWith(`${PLAN_EVENTS}${sep}`))) {
    return join(PLAN_EVENTS_SANDBOX, relative(PLAN_EVENTS, resolved));
  }
  return resolved;
}

/**
 * The Agent CLI's `grep` (ripgrep underneath): "Found N matches" then each
 * file's absolute path and its `  Line N: text` hits, or "No files found".
 */
function grep(dir, args) {
  const argv = ['--json', '--sort', 'path', '--regexp', args.pattern ?? args.query ?? ''];
  if (args.include) argv.push('--glob', args.include);
  argv.push('--', within(dir, args.path ?? '.'));
  return grepCapture(argv);
}

/** Adapter failures are transport metadata, distinct from authored file/stdout bytes. */
function toolFailure(message) {
  return JSON.stringify({ is_error: true, error: String(message) });
}

/** Read provider metadata is attached outside the actual file bytes. */
export function executeResult(dir, call) {
  const appendArgs = argsOf(call);
  if (call.tool === 'write' && appendArgs.append_mode === 'atomic_record_append') {
    try { return atomicRecordAppend(dir, appendArgs); }
    catch (error) { return { content: toolFailure(error.message), is_error: true }; }
  }
  if (call.tool !== 'read') return { content: execute(dir, call) };
  const args = argsOf(call);
  const path = args.filePath ?? args.file_path ?? args.path;
  try {
    const content = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true })
      .decode(readFileSync(within(dir, path)));
    return { content, source_read: { path, success: true, complete: true, format: 'raw' } };
  } catch (error) {
    return {
      content: toolFailure(error.message), is_error: true,
      source_read: { path, success: false, complete: false, format: 'raw', error_code: error.code },
    };
  }
}

/** Execute one tool call the way the Agent CLI does; returns its result text. */
export function execute(dir, call) {
  const args = argsOf(call);
  const path = args.filePath ?? args.file_path ?? args.path;
  try {
    switch (call.tool) {
      case 'read': {
        const receipt = executeResult(dir, call);
        if (receipt.is_error) return receipt.content;
        const text = receipt.content;
        const lines = text.split('\n');
        const body = lines.map((line, index) => `${String(index + 1).padStart(5, '0')}| ${line}`).join('\n');
        return `<file>\n${body}\n\n(End of file - total ${lines.length} lines)\n</file>`;
      }
      case 'write': {
        if (args.append_mode !== undefined) return toolFailure('append requires the explicit receipt adapter');
        const target = within(dir, path);
        mkdirSync(dirname(target), { recursive: true });
        writeFileSync(target, args.content ?? '');
        return '';
      }
      case 'edit': {
        const target = within(dir, path);
        const text = existsSync(target) ? readFileSync(target, 'utf8') : '';
        const oldString = args.oldString ?? args.old_string ?? '';
        const newString = args.newString ?? args.new_string ?? '';
        if (oldString === '') {
          writeFileSync(target, newString);
          return '';
        }
        if (!text.includes(oldString)) return toolFailure('oldString not found in content');
        // The Agent CLI refuses an ambiguous oldString rather than editing its first match.
        if (!args.replaceAll && text.indexOf(oldString) !== text.lastIndexOf(oldString)) {
          return toolFailure('Found multiple matches for oldString. Provide more surrounding lines in oldString to identify the correct match.');
        }
        const next = args.replaceAll ? text.split(oldString).join(newString) : text.replace(oldString, () => newString);
        writeFileSync(target, next);
        return '';
      }
      case 'bash': {
        let eventDirectory = null;
        let commandDirectory = dir;
        const eventPrefix = template('plan-event-append-command')?.split('{lock_path}')[0];
        if (resolve(dir) === REPOSITORY_ROOT && eventPrefix && args.command.startsWith(eventPrefix)) {
          mkdirSync(PLAN_EVENTS_SANDBOX, { recursive: true });
          eventDirectory = mkdtempSync(join(tmpdir(), 'formal-ai-plan-events-'));
          symlinkSync(PLAN_EVENTS_SANDBOX, join(eventDirectory, '.formal-ai'), process.platform === 'win32' ? 'junction' : 'dir');
          commandDirectory = eventDirectory;
        }
        try {
          return shellCapture(args.command, {cwd:commandDirectory, timeoutMs:BASH_TIMEOUT_MS});
        } finally {
          if (eventDirectory !== null) rmSync(eventDirectory, { recursive: true, force: true });
        }
      }
      case 'list':
        return readdirSync(within(dir, path ?? '.')).join('\n');
      case 'grep':
        return grep(dir, args);
      default:
        return toolFailure(`the dogfood driver does not execute ${call.tool}`);
    }
  } catch (error) {
    return toolFailure(error.message);
  }
}

/**
 * Run a whole session; returns `{transcript, answer, stop}`. `answer` is null
 * when the session ends without one, and `stop` then says why: `no-plan` (the
 * planner and its fall-through planned nothing) or `steps` (the step budget
 * ran out mid-task, PR #1188 G84).
 */

const operationObservations = new WeakMap();
const observationSourcePaths = [fileURLToPath(import.meta.url), fileURLToPath(new URL('./shell-capture.mjs', import.meta.url))];
function observedSourceBytes() {
  return Object.freeze(observationSourcePaths.map(path => Object.freeze({path,
    sha256: createHash('sha256').update(readFileSync(path)).digest('hex')})));
}
function qualifiesObservation(call, commands) {
  if (call.tool !== 'bash' || !Array.isArray(commands) || typeof call.arguments !== 'string') return false;
  try { const value = JSON.parse(call.arguments); return value && Object.keys(value).length === 1
    && typeof value.command === 'string' && commands.includes(value.command); } catch { return false; }
}
function issueObservation(prompt, directory, call, receipt, before) {
  const after = observedSourceBytes();
  if (JSON.stringify(before) !== JSON.stringify(after)) return null;
  const token = Object.freeze(Object.create(null));
  const observation = Object.freeze({need: Object.freeze({original: prompt}),
    command: JSON.parse(call.arguments).command, directory: resolve(directory),
    sources: before, receipt: Object.freeze({...receipt}),
    sourceClosure: 'Unknown', effects: 'Unknown', approval: 'Unknown',
    kind: 'observed-operation-only'});
  operationObservations.set(token, observation);
  return token;
}
export function observedOperation(token, originalNeed, directory) {
  const record = operationObservations.get(token);
  if (!record || record.need.original !== originalNeed || typeof directory !== 'string' || record.directory !== resolve(directory)) return null;
  if (JSON.stringify(record.sources) !== JSON.stringify(observedSourceBytes())) return null;
  return record;
}

export async function drive(planChatStep, dir, prompt, {
  tools = AGENT_CLI_TOOLS, steps = 12, fallthrough = null, allowedCommands = undefined,
  atomicRecordAppend = false,
} = {}) {
  if (!Array.isArray(tools) || new Set(tools).size !== tools.length || tools.some((tool) => !AGENT_CLI_TOOLS.includes(tool))) {
    return { transcript: [], answer: null, stop: 'invalid-tools', toolsAdvertised: [] };
  }
  if (typeof atomicRecordAppend !== 'boolean') {
    return { transcript: [], answer: null, stop: 'invalid-append-policy', toolsAdvertised: tools };
  }
  tools = Object.freeze([...tools]);
  if (allowedCommands !== undefined) {
    if (!Array.isArray(allowedCommands)
        || allowedCommands.some(command => typeof command !== 'string')
        || new Set(allowedCommands).size !== allowedCommands.length) {
      return { transcript: [], answer: null, stop: 'invalid-command-policy', toolsAdvertised: tools };
    }
    allowedCommands = Object.freeze([...allowedCommands]);
  }
  if (!hasHost()) await installDefaultNodeSourceHost(new WorkerHost());
  const sourceSession = host().sourceSession;
  if (sourceSession && !sourceSession.active()) {
    return sourceSession.run({ request: prompt, workspace: dir, tools }, () =>
      drive(planChatStep, dir, prompt, {
        tools, steps, fallthrough, allowedCommands, atomicRecordAppend,
      }));
  }
  const messages = [
    { role: 'system', content: `<env>\n  Working directory: ${dir}\n  Is directory a git repo: yes\n</env>` },
    { role: 'user', content: prompt },
  ];
  const transcript = [];
  // Private live request binding: no caller tool argument can grant creation.
  const ownedCreate = ownedDeclaredCreateFrame(prompt);
  const declaredGoals = goalLedger(prompt);
  const collectionGoals = Array.isArray(declaredGoals) && collectionSummaryOwns(declaredGoals)
    ? declaredGoals.slice(1) : declaredGoals;
  const firstUnsupported = Array.isArray(collectionGoals) ? collectionGoals.findIndex(goal => goal.kind === 'unsupported') : -1;
  const missingClause = firstUnsupported < 0 ? null : normalizePrompt(collectionGoals[firstUnsupported].clause);
  const independentMissing = missingClause !== null && bareSurfaces('enumeration_cue').some(cue => {
    const normalized = normalizePrompt(cue);
    return missingClause.startsWith(normalized) && /[\s,，:：]/u.test(missingClause[normalized.length] ?? '');
  });
  const supportedGoals = Array.isArray(collectionGoals) && firstUnsupported > 0 && independentMissing
    ? collectionGoals.slice(0, firstUnsupported) : collectionGoals;
  const mixedGoals = Array.isArray(supportedGoals) && supportedGoals.some(goal => goal.kind === 'source_edit');
  const collection = Array.isArray(supportedGoals) && supportedGoals.length > 0
    && supportedGoals.every(goal => ['literal_file', 'source_edit'].includes(goal.kind)
      && goal.sourceUnit === 'utf16' && prompt.slice(goal.span.start, goal.span.end) === goal.clause)
    ? supportedGoals.filter(goal => goal.kind === 'literal_file').map(goal => {
      const frame = ownedDeclaredCreateFrame(goal.clause);
      return frame !== null && frame.target === goal.target && frame.content === goal.expected
        ? Object.freeze({ ...frame, request: goal.clause }) : null;
    }) : null;
  const createContracts = ownedCreate !== null
    ? [Object.freeze({ ...ownedCreate, request: prompt })]
    : collection !== null && collection.every(frame => frame !== null)
      && new Set(collection.map(frame => frame.target)).size === collection.length ? collection : [];
  const createContract = createContracts[0] ?? null;
  const createdTargets = new Set();
  // Bounded path checks detect known replacement/aliases; they do not prove namespace atomicity.
  function createdTargetIsCurrent(target) {
    if (!createdTargets.has(target) || !sameWorkspace()) return false;
    try {
      const parts = target.split('/');
      let path = createWorkspace.canonicalPath;
      for (let index = 0; index < parts.length; index += 1) {
        path += '/' + parts[index];
        const item = lstatSync(path);
        if (item.isSymbolicLink() || (index === parts.length - 1 ? !item.isFile() : !item.isDirectory())) return false;
      }
      return true;
    } catch { return false; }
  }
  // This identity qualifies the workspace object, not immutable external ancestry.
  function workspaceIdentity() {
    try {
      const value = lstatSync(dir, { bigint: true });
      if (!value.isDirectory() || value.isSymbolicLink()) return null;
      return Object.freeze({ dev: String(value.dev), ino: String(value.ino), canonicalPath: realpathSync(dir) });
    } catch { return null; }
  }
  const createWorkspace = createContract === null ? null : workspaceIdentity();
  const sameWorkspace = () => {
    const current = workspaceIdentity();
    return current !== null && createWorkspace !== null
      && current.dev === createWorkspace.dev && current.ino === createWorkspace.ino
      && current.canonicalPath === createWorkspace.canonicalPath;
  };
  if (createContract !== null && createWorkspace === null) {
    return { transcript, answer: null, stop: 'unsupported-create-workspace', toolsAdvertised: tools };
  }
  const rawCreate = parseWriteContract(prompt);
  const prefix = rawCreate === null ? '' : normalizePrompt(prompt.slice(0, rawCreate.targetSpan.start));
  const explicitOverwrite = ownsCompleteLiteralRequest(prompt)
    && mentionsRole('file_overwrite_consent', prefix);
  const explicitAddition = ownedAdditiveLiteralFrame(prompt);
  // Quoted source and authored payload are data, never create instructions.
  let creationInstruction = prompt;
  for (const span of quotedSegmentSpans(prompt)) {
    creationInstruction = creationInstruction.slice(0, span.start)
      + ' '.repeat(span.end - span.start) + creationInstruction.slice(span.end);
  }
  // An exact adapter operation is the existing low-level callback API, not a semantic creation request.
  const adapterOperation = AGENT_CLI_TOOLS.includes(prompt) ? prompt : null;
  const createIntent = adapterOperation === null && writesWholeFile(creationInstruction) && !explicitOverwrite
    && !(explicitAddition !== null && explicitAddition.atEnd !== null);
  let readOnlyOperation = false;
  if (createIntent && createContract === null && rawCreate === null) {
    const paths = ownedReadPaths(prompt, 'file_read_action_cue');
    const current = await maintainedResolvedStep(messages, tools);
    readOnlyOperation = paths.length > 0 && current?.kind === 'tool_calls'
      && current.calls.length > 0 && current.calls.every(call => call.tool === 'read')
      && !readPolicyBlocksPlan(prompt, current);
  }
  const readReportFrame = readOnlyOperation ? ownedReadReportFrame(prompt) : null;
  const reportDelivery = sentences(prompt).some(clause => {
    const target = deliveredWriteTarget(clause.text);
    return target !== null && !namesCallableArtifact(clause.text, target)
      && mentionsRole('evidence-report-artifact-kind', clause.text);
  });
  if (readOnlyOperation && reportDelivery && readReportFrame === null) readOnlyOperation = false;
  let sourceEditOperation = false;
  const firstGoal = Array.isArray(declaredGoals) ? declaredGoals[0] : null;
  if (createIntent && createContract === null && firstGoal?.kind === 'source_edit'
      && firstGoal.sourceUnit === 'utf16'
      && prompt.slice(firstGoal.span.start, firstGoal.span.end) === firstGoal.clause
      && rawCreate?.target === firstGoal.target) {
    const current = await maintainedResolvedStep(messages, tools);
    sourceEditOperation = current?.kind === 'tool_calls' && current.calls.length > 0
      && current.calls.every(call => {
        if (call.tool !== 'read') return false;
        const args = argsOf(call);
        return args !== null && typeof args === 'object' && !Array.isArray(args)
          && Object.keys(args).length > 0
          && Object.keys(args).every(key => ['filePath', 'file_path', 'path'].includes(key))
          && Object.values(args).every(path => path === firstGoal.target);
      }) && !readPolicyBlocksPlan(prompt, current);
  }
  if (createIntent && createContract === null && !readOnlyOperation && !sourceEditOperation) {
    const refusal = await maintainedPlanChatStep(messages, tools);
    if (refusal?.kind === 'final') {
      return { transcript, answer: refusal.answer, stop: 'final', toolsAdvertised: tools };
    }
    return { transcript, answer: null, stop: 'unowned-create-request', toolsAdvertised: tools };
  }
  function ownedCreation(call) {
    if (createContract === null || call.tool !== 'write') return null;
    const args = argsOf(call);
    const allowed = new Set(['filePath', 'file_path', 'path', 'content']);
    const aliases = ['filePath', 'file_path', 'path'];
    const contract = createContracts.find(frame => aliases.some(key => Object.hasOwn(args ?? {}, key) && args[key] === frame.target));
    if (mixedGoals && (contract === undefined || args?.content !== contract.content)) return null;
    if (contract === undefined || args === null || typeof args !== 'object' || Array.isArray(args)
        || Object.keys(args).some(key => !allowed.has(key))
        || !aliases.some(key => Object.hasOwn(args, key))
        || aliases.some(key => Object.hasOwn(args, key) && args[key] !== contract.target)
        || args.content !== contract.content) {
      return { content: toolFailure('UnboundDeclaredCreation'), is_error: true };
    }
    if (!sameWorkspace()) return { content: toolFailure('ChangedCreationWorkspace'), is_error: true };
    return exclusiveCreate(createWorkspace.canonicalPath, contract.target, contract.content,
      { dev: createWorkspace.dev, ino: createWorkspace.ino });
  }
  for (let step = 0; step < steps; step += 1) {
    // The server's fall-through: no planned step means the solver answers and
    // the symbolic command reroute may still turn that answer into tool calls
    // (js/server/agentic.mjs commandReroutePlan).
    const contracts = atomicRecordAppend && tools.includes('write') ? [appendDefinition('write')] : [];
    const projectHistory = definitions => projectAppendContracts(messages, definitions).map(message => ({
      ...message,
      // Wire records are callback data; opaque provider tokens retain their identity.
      ...(message.tool_calls ? { tool_calls: message.tool_calls.map(call => ({
        ...call, function: { ...call.function },
      })) } : {}),
      ...(message.source_read ? { source_read: { ...message.source_read } } : {}),
      ...(message.source_creation ? { source_creation: { ...message.source_creation } } : {}),
      ...(message.append_receipt ? { append_receipt: { ...message.append_receipt } } : {}),
    }));
    const scoped = projectHistory(contracts);
    const plan = (await planChatStep(scoped, tools)) ?? (fallthrough ? await fallthrough(projectHistory([]), tools) : null);
    if (!plan) return { transcript, answer: null, stop: 'no-plan', toolsAdvertised: tools };
    if (plan.kind === 'final') {
      if (readOnlyOperation || sourceEditOperation) {
        const current = await maintainedResolvedStep(messages, tools);
        if (current?.kind !== 'final') return { transcript, answer: null, stop: 'unbound-read-operation', toolsAdvertised: tools };
        return { transcript, answer: current.answer, stop: 'final', toolsAdvertised: tools };
      }
      if (createContract !== null) {
        const ownedFinal = await maintainedResolvedStep(messages, tools);
        if (ownedFinal?.kind !== 'final') return { transcript, answer: null, stop: 'unbound-create-operation', toolsAdvertised: tools };
        return { transcript, answer: ownedFinal.answer, stop: 'final', toolsAdvertised: tools };
      }
      return { transcript, answer: plan.answer, stop: 'final', toolsAdvertised: tools };
    }
    // Snapshot the complete batch before validation; getters cannot change executed arguments.
    const calls = plan.calls.map(call => Object.freeze({ tool: call.tool, arguments: call.arguments }));
    if (!atomicRecordAppend && calls.some(call => call.tool === 'write'
      && argsOf(call).append_mode !== undefined)) {
      return { transcript, answer: null, stop: 'undeclared-append-contract', toolsAdvertised: tools };
    }
    if (calls.some((call) => !tools.includes(call.tool))) {
      return { transcript, answer: null, stop: 'undeclared-tool', toolsAdvertised: tools };
    }
    if (allowedCommands !== undefined) {
      const deniedCalls = calls.filter(call => {
        if (call.tool !== 'bash') return false;
        try {
          if (typeof call.arguments !== 'string') return true;
          const arguments_ = JSON.parse(call.arguments);
          return !arguments_ || typeof arguments_.command !== 'string'
            || !allowedCommands.includes(arguments_.command);
        } catch {
          return true;
        }
      });
      if (deniedCalls.length) {
        return { transcript, answer: null, stop: 'command-policy-denied',
          toolsAdvertised: tools, deniedCalls };
      }
    }
    if (adapterOperation !== null && calls.some(call => call.tool !== adapterOperation)) {
      return { transcript, answer: null, stop: 'unbound-adapter-operation', toolsAdvertised: tools };
    }
    if (readOnlyOperation || sourceEditOperation) {
      const current = await maintainedResolvedStep(messages, tools);
      if (current?.kind !== 'tool_calls' || readPolicyBlocksPlan(prompt, current)
          || calls.length !== current.calls.length || !calls.every((call, index) =>
            (sourceEditOperation || readReportFrame !== null || call.tool === 'read') && call.tool === current.calls[index].tool
              && call.arguments === current.calls[index].arguments)) {
        return { transcript, answer: null, stop: 'unbound-read-operation', toolsAdvertised: tools };
      }
    }
    const toolCalls = calls.map((call, index) => ({
      id: `c${step}_${index}`, type: 'function', function: { name: call.tool, arguments: call.arguments },
    }));
    if (createContract !== null && !sameWorkspace()) {
      return { transcript, answer: null, stop: 'changed-create-workspace', toolsAdvertised: tools };
    }
    if (createContract !== null) {
      const livePlan = await maintainedResolvedStep(messages, tools);
      if (livePlan === null || livePlan.kind !== 'tool_calls') {
        return { transcript, answer: livePlan?.kind === 'final' ? livePlan.answer : null,
          stop: livePlan?.kind === 'final' ? 'final' : 'unbound-create-operation', toolsAdvertised: tools };
      }
      // Operation identity belongs to the maintained current source plan.
      // Raw request frames still bind exclusive target and authored bytes.
      const expectedCalls = livePlan.calls;
      const bound = calls.every(call => {
        const args = argsOf(call);
        const readPath = args?.filePath ?? args?.file_path ?? args?.path;
        const creationRead = call.tool === 'read' && createContracts.some(frame => frame.target === readPath);
        const verificationFrames = call.tool === 'bash' ? createContracts.filter(frame => composeGeneralChangePlan(frame.request)?.verification_command === args?.command) : [];
        if (creationRead && !createdTargetIsCurrent(readPath)) return false;
        if (verificationFrames.some(frame => !createdTargetIsCurrent(frame.target))) return false;
        if (mixedGoals && expectedCalls.some(owned => owned.tool === call.tool && owned.arguments === call.arguments)) return true;
        if (call.tool === 'write') {
          if (!mixedGoals) return true;
          const args = argsOf(call);
          return createContracts.some(frame => args?.content === frame.content
            && ['filePath', 'file_path', 'path'].some(key => Object.hasOwn(args ?? {}, key) && args[key] === frame.target));
        }
        if (call.tool === 'read') {
          const args = argsOf(call);
          return args !== null && typeof args === 'object' && !Array.isArray(args)
            && Object.keys(args).every(key => ['filePath', 'file_path', 'path'].includes(key))
            && Object.values(args).length > 0 && Object.values(args).every(path => createContracts.some(frame => frame.target === path));
        }
        return call.tool === 'bash' && expectedCalls.some(owned => owned.tool === call.tool && owned.arguments === call.arguments);
      });
      if (!bound) return { transcript, answer: null, stop: 'unbound-create-operation', toolsAdvertised: tools };
    }
    messages.push({ role: 'assistant', content: '', tool_calls: toolCalls });
    calls.forEach((call, index) => {
      const observationSources = qualifiesObservation(call, allowedCommands)
        ? observedSourceBytes() : null;
      const creation = ownedCreation(call);
      if (creation?.source_creation?.success === true) createdTargets.add(creation.source_creation.path);
      const receipt = creation ?? (sourceSession
        ? sourceSession.executeResult(call, messages, () => executeResult(dir, call))
        : executeResult(dir, call));
      const operationObservation = observationSources === null ? null
        : issueObservation(prompt, dir, call, receipt, observationSources);
      const { content: result, ...metadata } = receipt;
      transcript.push({ tool: call.tool, arguments: call.arguments, result, ...metadata, ...(operationObservation === null ? {} : {operationObservation}) });
      messages.push({ role: 'tool', tool_call_id: toolCalls[index].id, name: call.tool, ...receipt });
    });
  }
  return { transcript, answer: null, stop: 'steps', toolsAdvertised: tools };
}

/** What the driver prints when a session ends with no answer: never a bare `null`. */
export function unanswered(stop, steps, calls) {
  if (stop === 'unowned-create-request') return '(no execution: the request does not establish one complete declared-create contract)';
  if (stop === 'unsupported-create-workspace') return '(no execution: the workspace does not establish the required canonical directory object)';
  if (stop === 'changed-create-workspace') return '(no execution: the bound workspace directory object changed before the operation)';
  if (stop === 'steps') return `(no answer: the ${steps}-step budget ran out after ${calls} tool calls, before the task finished; run again with a larger --steps)`;
  return `(no answer: the planner planned no step after ${calls} tool calls)`;
}

async function main(argv) {
  let dir = null;
  let steps = 12;
  const rest = [];
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] === '--dir') dir = argv[++index];
    else if (argv[index] === '--steps') steps = Number(argv[++index]);
    else rest.push(argv[index]);
  }
  if (!dir || rest.length === 0) throw new Error('usage: drive.mjs --dir <sandbox> [--steps N] <prompt>');
  await installDefaultNodeSourceHost(new WorkerHost());
  const { planChatStep } = await import('../../js/agentic/planner.mjs');
  const { solve } = await import('../../js/agentic/host.mjs');
  const { planSymbolicCommandReroute } = await import('../../js/agentic/command_reroute.mjs');
  const { latestUserRequest } = await import('../../js/agentic/content.mjs');
  const fallthrough = async (messages, tools) => {
    const symbolic = await solve(latestUserRequest(messages) ?? '', []);
    return planSymbolicCommandReroute(messages, tools, symbolic) ?? { kind: 'final', answer: symbolic.answer };
  };
  const { transcript, answer, stop } = await drive(planChatStep, resolve(dir), rest.join(' '), { steps, fallthrough });
  const out = [];
  for (const entry of transcript) {
    out.push(`>> ${entry.tool} ${entry.arguments}`);
    if (entry.result) out.push(entry.result.split('\n').map((line) => `   ${line}`).join('\n'));
  }
  out.push(`== answer ==\n${answer ?? unanswered(stop, steps, transcript.length)}`);
  // Await the complete stdout write before terminating the worker-owned session.
  await new Promise((resolve, reject) => {
    process.stdout.write(`${out.join('\n')}\n`, error => error ? reject(error) : resolve());
  });
  process.exit(0);
}

if (import.meta.url === `file://${process.argv[1]}`) await main(process.argv.slice(2));
