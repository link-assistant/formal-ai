// Typed payload compatibility at internal evidence/artifact boundaries.
export const FinalPayloadRole = Object.freeze({
  Finding: 'finding', AuditReport: 'audit-report', SourceModule: 'source-module',
});

/** A source operand needs a receipt for the whole payload, beyond a report finding. */
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
