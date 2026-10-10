import { sourceContractDiagnostic } from "./source-contract-diagnostics.mjs";
import { observedCallableRequest } from "../module_function.mjs";
import { observedCallableGoalLedger } from "./discovery.mjs";
import { Progress } from "../progress.mjs";
import { readPolicyBlocksPlan } from "../file_read/ownership.mjs";
import { planOne } from "../plan.mjs";
import { readArguments } from "../workspace_change.mjs";
import { userRequestText } from "../content.mjs";
import { sha256Hex } from "../crate/source_fetch.mjs";
import { sourceGoalScope } from "./seeded-composition-hook.mjs";

/** Resolve only existing request-derived Read Needs; unknown semantics stay unsolved. */
export function sourceNeedPreflight(source, messages) {
  if (sourceGoalScope(source) === null) return null;
  const user = [...messages].reverse().find(message => message.role === 'user');
  if (!user || userRequestText(user.content) !== source) return null;
  const request = observedCallableRequest(source);
  if (request === null) return null;
  const ledger = observedCallableGoalLedger(request, messages);
  const progress = Progress.scan(messages);
  const operands = [{
    path: request.destination,
    role: 'destination'
  }, ...request.inputs.map(path => ({
    path,
    role: 'source'
  })), ...request.acceptance.map(path => ({
    path,
    role: 'acceptance'
  }))];
  const prerequisites = operands.map(operand => {
    const policy = readPolicyBlocksPlan(source, planOne('read', readArguments(operand.path)));
    const read = progress.sourceReadFor(operand.path);
    let status = 'unattempted';
    if (policy) status = 'policy-refused';else if (read !== null) {
      if (read.error !== null) {
        status = operand.role === 'destination' && /\bENOENT\b/u.test(read.error) ? 'conditional-satisfied-absence' : 'failed';
      } else status = read.complete && typeof read.source === 'string' ? 'conditional-satisfied-read' : 'incomplete';
    }
    return {
      ...operand,
      status,
      sourceIdentity: read?.error === null && read.complete && typeof read.source === 'string' ? sha256Hex(read.source) : null,
      providerStatus: read?.status ?? null,
      providerError: read?.error ?? null,
      requiredBy: 'maintained-observed-callable-request',
      requestIdentity: sha256Hex(source)
    };
  });
  return {
    request,
    ledger,
    prerequisites,
    unresolvedNeeds: ledger.needs.filter(need => !need.kind.startsWith('read-')),
    independentClauses: ledger.clauses,
    semanticCoverage: 'unbound',
    declarationAuthority: 'conditional-request-classifier',
    receiptTrust: sourceContractDiagnostic("unbound-host-message-boundary"),
    moduleEffects: 'unknown',
    writeAuthority: false,
    authored: false,
    verified: false
  };
}
