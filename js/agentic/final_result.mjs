// Internal final-result dispositions and the public protocol projection.

/** Internal outcome semantics; public final answers project to their text. */
export const FinalDisposition = Object.freeze({
  Finding: 'finding', Artifact: 'artifact', Clarification: 'clarification',
  Gap: 'gap', Failure: 'failure', Unknown: 'unknown',
});

/** Construct a final result at the route that knows its disposition and origin. */
export function resolvedFinalAnswer(answer, disposition, origin) {
  return { kind: 'final', answer, result: { text: answer, disposition, origin } };
}

/** Legacy routes have no evidence certificate and remain unknown. */
export function finalResult(plan) {
  if (plan?.kind !== 'final') return null;
  return plan.result?.text === plan.answer ? plan.result
    : { text: plan.answer, disposition: FinalDisposition.Unknown, origin: null };
}

/** Only a route-certified finding can become an investigation's file content. */
export function canDeliverFinal(plan) {
  const result = finalResult(plan);
  return result?.disposition === FinalDisposition.Finding
    && typeof result.origin === 'string' && result.origin.length > 0;
}

/** Preserve the existing public tool-call/final-answer protocol. */
export function projectPlan(plan) {
  return plan?.kind === 'final' ? { kind: 'final', answer: plan.answer } : plan;
}
