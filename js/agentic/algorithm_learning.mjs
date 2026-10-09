// Agent-CLI path for discovering reusable algorithms from supplied traces
// (rust/src/agentic_coding/algorithm_learning.rs).
//
// Routing is based on the portable `demo_memory` data itself: if the supplied
// task contains enough observations to yield a held-out-validated episode, the
// planner runs the public CLI, reads its artifact back, and performs the same
// side-effect-free conformance check a human can replay.

import { Capability } from './capability.mjs';
import { classifyTool, toolFor } from './capability_router.mjs';
import {
  candidateFromLinksNotation, candidatesEqual, conformanceLinksNotation, discoverAlgorithms,
  tracesFromMemoryEvents, validatedCandidates,
} from './crate/algorithm_discovery.mjs';
import { pushLinoNode } from './crate/links_format.mjs';
import { MemoryStore } from './crate/memory.mjs';
import { byteOrder } from './crate/rust_str.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { Progress } from './progress.mjs';
import { plainText } from './content.mjs';
import { commandArgument, observedBytesMatch, observedPayload } from './tool_result.mjs';

/** Mirrors `OBSERVATIONS_PATH` in rust/src/agentic_coding/algorithm_learning.rs. */
export const OBSERVATIONS_PATH = 'algorithm-observations.lino';
/** Mirrors `DISCOVERY_PATH` in rust/src/agentic_coding/algorithm_learning.rs. */
export const DISCOVERY_PATH = 'discovered-algorithms.lino';
/** Mirrors `CONFORMANCE_TRIGGER` in rust/src/agentic_coding/algorithm_learning.rs. */
export const CONFORMANCE_TRIGGER = 'agent-cli-conformance';

/**
 * Mirrors `fn compile_task` in rust/src/agentic_coding/algorithm_learning.rs:
 * an `AlgorithmLearningTask` `{observations, discovery, candidate}` when the
 * task text is a `demo_memory` document yielding a validated episode, else null.
 * @param {string} task
 * @returns {object|null}
 */
export function compileTask(task) {
  const store = new MemoryStore();
  store.replaceFromLinksNotation(task);
  if (store.isEmpty()) return null;
  const discovery = discoverAlgorithms(tracesFromMemoryEvents(store.events()));
  const candidate = validatedCandidates(discovery)[0];
  if (candidate === undefined) return null;
  return { observations: store.exportLinksNotation(), discovery, candidate };
}

/**
 * Mirrors `fn plan_step` in rust/src/agentic_coding/algorithm_learning.rs.
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 * @param {object} task an `AlgorithmLearningTask`
 * @returns {object} an `AgenticPlan`
 */
export function planStep(messages, toolNames, task) {
  const progress = Progress.scan(messages);
  const writeTool = toolFor(toolNames, Capability.Write);
  if (writeTool !== null && !progress.done(Capability.Write)) {
    return planOne(writeTool, writeArguments(OBSERVATIONS_PATH, task.observations));
  }
  if (writeTool === null) return finalAnswer(resultDocument(task, 'observation_write_unavailable', ''));
  const runTool = toolFor(toolNames, Capability.Run);
  if (runTool === null) return finalAnswer(resultDocument(task, 'shell_unavailable', ''));
  const discovery = discoveryCommand(), readback = readbackCommand();
  if (!progress.hasRun(discovery)) return planOne(runTool, jsonText({ command: discovery }));
  if (commandPayload(messages, discovery) === null) return finalAnswer(resultDocument(task, 'artifact_verification_failed', ''));
  if (!progress.hasRun(readback)) return planOne(runTool, jsonText({ command: readback }));
  const output = commandPayload(messages, readback);
  const verified = output === null ? null : candidateFromLinksNotation(output);
  if (!candidatesEqual(verified, task.candidate)) return finalAnswer(resultDocument(task, 'artifact_verification_failed', ''));
  const command = conformanceCommand(task.candidate);
  if (!progress.hasRun(command)) return planOne(runTool, jsonText({ command }));
  const expected = expectedConformance(task.candidate);
  if (commandPayload(messages, command) !== expected) return finalAnswer(resultDocument(task, 'conformance_failed', ''));
  return finalAnswer(resultDocument(task, 'conformance_passed', expected));
}

/** Mirrors `fn command_payload`: exact current-call binding precedes transport decoding. */
function commandPayload(messages, command) {
  const current = messages.slice(Math.max(0, messages.findLastIndex(message => message.role.toLowerCase() === 'user')));
  for (let index = current.length - 1; index >= 0; index -= 1) {
    const message = current[index];
    if (message.role.toLowerCase() !== 'tool' || message.tool_call_id == null) continue;
    const call = current.slice(0, index).flatMap(prior => prior.tool_calls ?? []).findLast(item => item.id === message.tool_call_id);
    if (!call || classifyTool(call.function.name) !== Capability.Run || commandArgument(call.function.arguments) !== command) continue;
    if (message.is_error === true || message.isError === true || (message.name && message.name !== call.function.name)) return null;
    const raw = plainText(message.content);
    let receipt; try { receipt = JSON.parse(raw); } catch { receipt = null; }
    if (receipt?.command !== undefined && receipt.command !== command) return null;
    const payload = observedPayload(raw);
    return payload !== null && observedBytesMatch(raw, payload) ? payload : null;
  }
  return null;
}

/** Mirrors `fn discovery_command` in rust/src/agentic_coding/algorithm_learning.rs. */
function discoveryCommand() {
  return ['formal-ai', 'learn', 'algorithms', '--from', OBSERVATIONS_PATH, '--output', DISCOVERY_PATH].join(' ');
}

/** Mirrors `fn readback_command` in rust/src/agentic_coding/algorithm_learning.rs. */
function readbackCommand() {
  return ['cat', DISCOVERY_PATH].join(' ');
}

/**
 * Mirrors `fn conformance_bindings` in rust/src/agentic_coding/algorithm_learning.rs:
 * every inferred parameter bound to `agent-cli-value`, in name order.
 * @param {object} candidate
 * @returns {Map<string, string>}
 */
export function conformanceBindings(candidate) {
  const names = new Set();
  for (const step of candidate.steps) {
    for (const [, pattern] of step.arguments) if (pattern.parameter !== undefined) names.add(pattern.parameter);
  }
  return new Map([...names].sort(byteOrder).map((name) => [name, 'agent-cli-value']));
}

/** Mirrors `fn expected_conformance` in rust/src/agentic_coding/algorithm_learning.rs. */
export function expectedConformance(candidate) {
  const record = conformanceLinksNotation(candidate, CONFORMANCE_TRIGGER, conformanceBindings(candidate));
  if (record === null) throw new Error('algorithm_learning_conformance_unbound');
  return record;
}

/** Mirrors `fn conformance_command` in rust/src/agentic_coding/algorithm_learning.rs. */
export function conformanceCommand(candidate) {
  const parts = ['formal-ai', 'algorithm', 'conformance', '--artifact', DISCOVERY_PATH, '--trigger', CONFORMANCE_TRIGGER];
  for (const [name, value] of conformanceBindings(candidate)) parts.push('--binding', `${name}=${value}`);
  return parts.join(' ');
}

/**
 * Mirrors `fn trace_from_driver_outcome` in rust/src/agentic_coding/algorithm_learning.rs:
 * project an executed transcript (`{steps: [{tool, arguments}]}`) onto an execution trace.
 * @param {string} id
 * @param {{steps: Array<{tool: string, arguments: string}>}} outcome
 */
export function traceFromDriverOutcome(id, outcome) {
  return {
    id,
    steps: outcome.steps.map((step) => {
      let parsed;
      try {
        parsed = JSON.parse(step.arguments);
      } catch {
        parsed = null;
      }
      const object = parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
      const args = Object.entries(object)
        .map(([name, value]) => [name, typeof value === 'string' ? value : jsonText(value)])
        .sort(([left], [right]) => byteOrder(left, right));
      return { operation: step.tool, arguments: args };
    }),
  };
}

/** Mirrors `fn result_document` in rust/src/agentic_coding/algorithm_learning.rs. */
function resultDocument(task, status, conformance) {
  let output = '';
  output = pushLinoNode(output, 0, 'agent_algorithm_learning', task.candidate.id);
  output = pushLinoNode(output, 2, 'status', status);
  output = pushLinoNode(output, 2, 'observations', OBSERVATIONS_PATH);
  output = pushLinoNode(output, 2, 'artifact', DISCOVERY_PATH);
  output = pushLinoNode(output, 2, 'human_gated', 'true');
  output = pushLinoNode(output, 2, 'execution_mode', 'proposal_conformance');
  output = pushLinoNode(output, 2, 'held_out_tests', String(task.candidate.held_out.length));
  output = pushLinoNode(output, 2, 'associative_compression_lossless',
    task.discovery.associative_compression_lossless ? 'true' : 'false');
  if (conformance !== '') output = pushLinoNode(output, 2, 'conformance', conformance);
  return output;
}
