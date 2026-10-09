// Request-local source-callable observations; synthesis remains uncertified.
import { Capability } from '../capability.mjs';
import { plainText } from '../content.mjs';
import { classifyTool, toolFor } from '../capability_router.mjs';
import { sourceFromAgentReadResult, sourceFromReadResult } from '../code_artifact.mjs';
import { FinalDisposition, resolvedFinalAnswer } from '../final_result.mjs';
import { agenticMessage } from '../messages.mjs';
import { planOne } from '../plan.mjs';
import { evidenceWindowStart } from '../planner/continuation.mjs';
import { failureMessage } from '../tool_result.mjs';
import { readArguments } from '../workspace_change.mjs';
import { compactJson } from '../crate/rust_str.mjs';
import { sha256Hex } from '../crate/source_fetch.mjs';
import { observeSourceCallables, observedCallableGraphs } from './callable_catalog.mjs';

/** Mirrors `fn plan_observed_callable_step`: reads followed by a typed contract gap. */
export function planObservedCallableStep(request, messages, toolNames) {
  const current = messages.slice(evidenceWindowStart(messages)), observations = [], sources = [];
  const finish = (reason, disposition, detail = null) => {
    const discovery = { reason, request, observations, detail,
      missingContracts: request.inputs.map((path) => ({ path, required: ['inputs', 'result', 'effects', 'imports', 'optional-guard'] })),
      authored: false, verified: false };
    const answer = agenticMessage('callable-discovery-outcome', { reason, discovery: compactJson(discovery) });
    const plan = resolvedFinalAnswer(answer, disposition, 'observed-callable-discovery');
    plan.result.discovery = discovery;
    return plan;
  };
  const operands = [{ path: request.destination, role: 'destination' },
    ...request.inputs.map((path) => ({ path, role: 'source' })),
    ...request.acceptance.map((path) => ({ path, role: 'acceptance' }))];
  for (const operand of operands) {
    const read = readReceipt(current, operand.path);
    if (read === null) {
      const tool = toolFor(toolNames, Capability.Read);
      return tool === null ? finish('MissingReadTool', FinalDisposition.Gap, operand)
        : planOne(tool, readArguments(operand.path));
    }
    const receipt = read.raw;
    const agentSource = sourceFromAgentReadResult(receipt);
    const failure = read.failed || agentSource === null ? failureMessage(receipt, read.failed, false) : null;
    if (failure !== null) {
      if (operand.role === 'destination' && /\bENOENT\b/u.test(failure)) {
        observations.push({ ...operand, state: 'absent', contentId: null, bytes: null });
        continue;
      }
      return finish('ReadFailed', FinalDisposition.Failure, { ...operand, error: failure });
    }
    const content = agentSource ?? sourceFromReadResult(receipt);
    if (operand.role === 'source') sources.push({ path: operand.path, content });
    observations.push({ ...operand, state: 'observed', contentId: sha256Hex(content), bytes: new TextEncoder().encode(content).length,
      ...(operand.role === 'source' ? { catalog: observeSourceCallables(content, operand.path) } : {}) });
  }
  return finish('MissingContract', FinalDisposition.Gap, { graphs: observedCallableGraphs(sources), goal: 'unbound' });
}

// Keep client error flags bound to the same read call as the observed bytes.
function readReceipt(messages, path) {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const result = messages[index];
    if (result.role.toLowerCase() !== 'tool' || result.tool_call_id == null) continue;
    let call = null;
    for (let prior = index - 1; prior >= 0 && call === null; prior -= 1) {
      call = (messages[prior].tool_calls ?? []).findLast((candidate) => candidate.id === result.tool_call_id) ?? null;
    }
    if (call === null || classifyTool(call.function.name) !== Capability.Read) continue;
    let args;
    try { args = JSON.parse(call.function.arguments); } catch { continue; }
    if (!['path', 'filePath', 'file_path'].some((key) => args?.[key] === path)) continue;
    return { raw: plainText(result.content), failed: result.is_error === true || result.isError === true };
  }
  return null;
}

/** Mirrors `fn plan_observed_callable_outcome` in rust/src/agentic_coding/module_function/discovery.rs. */
export function planObservedCallableOutcome(request, messages, toolNames) {
  const plan = planObservedCallableStep(request, messages, toolNames);
  return { plan, disposition: plan.result?.disposition ?? null, witness: plan.result?.discovery ?? null };
}
