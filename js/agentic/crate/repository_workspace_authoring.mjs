// Authoring uses the same declared repository stages through caller-owned ports.
import { EDITOR_AGENT_SESSION, loadProtocol, newProtocolTrace, recordStage, renderProtocolTrace,
  stepAppliesTo } from './repository_workspace.mjs';

/**
 * Mirrors `fn run_authoring_with`: iterate declared stages in document order.
 * serve/session are caller-provided stages, so removing serve from the document
 * cannot spawn a server. Each stage returns a status after observing its work.
 * The caller owns artifact contracts, isolated workspaces and process cleanup.
 */
export async function runAuthoringWith(stages, {
  steps = loadProtocol(), commit = false, writeTrace = async () => {},
} = {}) {
  const trace = newProtocolTrace(steps, 'authoring', EDITOR_AGENT_SESSION);
  const outcome = { trace, open: [], stopped_at: null, committed: false };
  for (const step of steps) {
    if (!stepAppliesTo(step, EDITOR_AGENT_SESSION)) continue;
    try {
      let status;
      if (step.id === 'commit' && !commit) status = 'refused';
      else if (step.id === 'commit' && trace.stages.some((stage) => stage.id !== 'commit'
        && stage.status !== 'observed' && stage.status !== 'not_applicable')) status = 'refused';
      else if (typeof stages[step.id] !== 'function') status = 'unobserved';
      else {
        if (step.id === 'commit') {
          recordStage(trace, step.id, 'requested');
          await writeTrace(renderProtocolTrace(trace));
        }
        status = await stages[step.id](outcome, step) ?? 'unobserved';
      }
      recordStage(trace, step.id, status);
      if (step.id === 'commit') outcome.committed = status === 'requested';
    } catch (error) {
      outcome.stopped_at = step;
      outcome.open.push(error.message);
      trace.open.push(error.message);
      recordStage(trace, step.id, 'stopped');
    }
    await writeTrace(renderProtocolTrace(trace));
    if (outcome.stopped_at) break;
  }
  outcome.trace_document = renderProtocolTrace(trace);
  return outcome;
}
