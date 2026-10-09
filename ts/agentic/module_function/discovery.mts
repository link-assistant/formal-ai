// Request-local source-callable observations; synthesis remains uncertified.
import { Capability } from '../capability.mjs';
import { Progress } from '../progress.mjs';
import { toolFor } from '../capability_router.mjs';
import { FinalDisposition, resolvedFinalAnswer } from '../final_result.mjs';
import { agenticMessage } from '../messages.mjs';
import { planOne } from '../plan.mjs';
import { readArguments } from '../workspace_change.mjs';
import { compactJson } from '../crate/rust_str.mjs';
import { sha256Hex } from '../crate/source_fetch.mjs';
import { observeSourceCallables, observedCallableGraphs } from './callable_catalog.mjs';

/** Mirrors `fn plan_observed_callable_step`: reads followed by a typed contract gap. */
export function planObservedCallableStep(request, messages, toolNames) {
  const progress = Progress.scan(messages), observations = [], sources = [];
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
    const read = progress.sourceReadFor(operand.path);
    if (read === null) {
      const tool = toolFor(toolNames, Capability.Read);
      return tool === null ? finish('MissingReadTool', FinalDisposition.Gap, operand)
        : planOne(tool, readArguments(operand.path));
    }
    const failure = read.error;
    if (failure !== null) {
      if (operand.role === 'destination' && /\bENOENT\b/u.test(failure)) {
        observations.push({ ...operand, state: 'absent', contentId: null, bytes: null });
        continue;
      }
      return finish('ReadFailed', FinalDisposition.Failure, { ...operand, error: failure });
    }
    const content = read.source ?? '';
    if (operand.role === 'source' && read.complete) sources.push({ path: operand.path, content });
    observations.push({ ...operand, state: 'observed', contentId: sha256Hex(content), bytes: new TextEncoder().encode(content).length, complete: read.complete, providerStatus: read.status,
      ...(operand.role === 'source' && read.complete ? { catalog: observeSourceCallables(content, operand.path) } : {}) });
  }
  return finish('MissingContract', FinalDisposition.Gap, { graphs: observedCallableGraphs(sources), goal: 'unbound' });
}

/** Mirrors `fn plan_observed_callable_outcome` in rust/src/agentic_coding/module_function/discovery.rs. */
export function planObservedCallableOutcome(request, messages, toolNames) {
  const plan = planObservedCallableStep(request, messages, toolNames);
  return { plan, disposition: plan.result?.disposition ?? null, witness: plan.result?.discovery ?? null };
}
