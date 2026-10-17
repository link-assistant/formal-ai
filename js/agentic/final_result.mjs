// Internal final-result provenance and public projection.
import { FinalPayloadRole, payloadCanDeliver } from './final_payload.mjs';
export { FinalPayloadRole };
const observedResults = new WeakMap();
// Diagnostics can observe origin without granting delivery authority to a projection.
export function observedFinalResult(plan) {
 if(plan?.result)return finalResult(plan);
 const observed=observedResults.get(plan);
 return observed?.text===plan?.answer?observed:finalResult(plan);
}

export const FinalDisposition = Object.freeze({
  Finding: 'finding', Artifact: 'artifact', Clarification: 'clarification',
  Gap: 'gap', Failure: 'failure', Unknown: 'unknown',
});
/** Mirrors `fn record_with_role` in rust/src/agentic_coding/final_result.rs; JS can also attach an artifact receipt. */
export function resolvedFinalAnswer(answer, disposition, origin, payloadRole = FinalPayloadRole.Finding, artifact = null) {
  return { kind: 'final', answer, result: { text: answer, disposition, origin, payloadRole, artifact } };
}
/** Mirrors `ResolvedPlan::new` in rust/src/agentic_coding/final_result.rs: unmatched final metadata remains unknown. */
export function finalResult(plan) {
  if (plan?.kind !== 'final') return null;
  return plan.result?.text === plan.answer ? plan.result
    : { text: plan.answer, disposition: FinalDisposition.Unknown, origin: null };
}

/** Mirrors `ResolvedPlan::can_deliver_as` in rust/src/agentic_coding/final_result.rs. */
export function canDeliverFinal(plan, requiredRole = FinalPayloadRole.Finding) {
  const result = finalResult(plan);
  return result?.disposition === FinalDisposition.Finding
    && typeof result.origin === 'string' && result.origin.length > 0
    && payloadCanDeliver(result, requiredRole);
}

/** Mirrors `ResolvedPlan::into_plan` in rust/src/agentic_coding/final_result.rs; native also records the result sink. */
export function projectPlan(plan) {
  if (plan?.kind !== 'final') return plan;
  const projected={kind:'final',answer:plan.answer};
  observedResults.set(projected,finalResult(plan));
  return projected;
}
