// Typed payload compatibility at internal evidence/artifact boundaries.
export const FinalPayloadRole = Object.freeze({
  Finding: 'finding', AuditReport: 'audit-report', SourceModule: 'source-module',
});

/** Mirrors `ResolvedPlan::can_deliver_as` in rust/src/agentic_coding/final_result.rs: its whole-source receipt predicate. */
export function payloadCanDeliver(result, requiredRole) {
  if (requiredRole !== FinalPayloadRole.SourceModule) return true;
  const artifact = result.artifact;
  return result.payloadRole === FinalPayloadRole.SourceModule
    && ['observed-complete', 'synthesized-complete'].includes(artifact?.kind)
    && artifact.content === result.text && typeof artifact.contentId === 'string'
    && artifact.contentId.length > 0 && artifact.syntax?.exitCode === 0
    && artifact.syntax.content === artifact.content
    && artifact.syntax.contentId === artifact.contentId;
}
