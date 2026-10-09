// Internal final-result provenance and public projection.
import { FinalPayloadRole, payloadCanDeliver } from './final_payload.mjs';
export { FinalPayloadRole };

export const FinalDisposition = Object.freeze({
  Finding: 'finding', Artifact: 'artifact', Clarification: 'clarification',
  Gap: 'gap', Failure: 'failure', Unknown: 'unknown',
});

export function resolvedFinalAnswer(answer, disposition, origin, payloadRole = FinalPayloadRole.Finding, artifact = null) {
  return { kind: 'final', answer, result: { text: answer, disposition, origin, payloadRole, artifact } };
}

/** Legacy routes have no evidence certificate and remain unknown. */
export function finalResult(plan) {
  if (plan?.kind !== 'final') return null;
  return plan.result?.text === plan.answer ? plan.result
    : { text: plan.answer, disposition: FinalDisposition.Unknown, origin: null };
}

export function canDeliverFinal(plan, requiredRole = FinalPayloadRole.Finding) {
  const result = finalResult(plan);
  return result?.disposition === FinalDisposition.Finding
    && typeof result.origin === 'string' && result.origin.length > 0
    && payloadCanDeliver(result, requiredRole);
}

export function projectPlan(plan) {
  return plan?.kind === 'final' ? { kind: 'final', answer: plan.answer } : plan;
}
