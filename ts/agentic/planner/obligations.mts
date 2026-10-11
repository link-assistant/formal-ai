// Shared transaction for every declared artifact, before literal payload keywords route.
import * as taskObligations from '../task_obligations.mjs';
import { composeGeneralChangePlan } from '../general_planner.mjs';
import { planGeneralChangeStep } from '../general_execution.mjs';
import { FinalDisposition, resolvedFinalAnswer } from '../plan.mjs';

/** Mirrors `fn plan_obligations_step`: reuse the selected obligation's ordinary execution plan. */
export async function planObligationsStep(task, messages, toolNames, obligations) {
  const next = taskObligations.nextStep(task, messages);
  if (next && (next.kind === 'observe' || next.kind === 'decompose')) {
    const general = composeGeneralChangePlan(next.node.clause);
    return general === null ? null : await planGeneralChangeStep(messages, toolNames, general);
  }
  if (next && next.kind === 'report_gap') {
    return resolvedFinalAnswer(taskObligations.gapAnswer(next.node_id, next.clause, next.span, next.reason),
      FinalDisposition.Gap, 'task_obligation_gap');
  }
  if (taskObligations.successfullyDischarged(task, messages)) {
    for (const obligation of [...obligations].reverse()) {
      const general = composeGeneralChangePlan(obligation.clause);
      if (general !== null) return await planGeneralChangeStep(messages, toolNames, general);
    }
    return null;
  }
  return null;
}
