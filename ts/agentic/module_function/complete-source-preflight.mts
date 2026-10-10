import { sourceContractDiagnostic } from "./source-contract-diagnostics.mjs";
import { observedCallableRequest, signature } from "../module_function.mjs";
import { sentences } from "../shell_command_policy.mjs";
import { parseLinoRoot, findChildValue } from "../write_lino.mjs";
import { sourceEvidence } from "./source_contract.mjs";
import { sha256Hex } from "../crate/source_fetch.mjs";
import { sourceGoalScope } from "./seeded-composition-hook.mjs";
const kinds = ['declaration', 'goal', 'source-discovery', 'reuse-imports', 'read-prerequisites', 'repair-plan', 'verification', 'protect-inputs', 'gap-before-write'];
/** Source-bound finite grammar; its goal and effect obligations are still separately checked. */
export function deriveCompleteSourceRequest(text, seed) {
  const scope = sourceGoalScope(text),
    request = observedCallableRequest(text);
  if (scope === null || request === null) return null;
  const root = parseLinoRoot(seed).children.find(node => node.name === 'source-request-preflight');
  if (!root) return null;
  const forms = root.children.filter(node => node.name === 'form');
  if (forms.length !== kinds.length || !kinds.every(kind => forms.filter(form => form.id === kind).length === 1)) return null;
  const clauses = [];
  const encoder = new TextEncoder();
  for (const sentence of sentences(scope)) {
    const matches = forms.flatMap(form => {
      let match;
      try {
        match = new RegExp(findChildValue(form, 'pattern'), 'iu').exec(sentence.text);
      } catch {
        return [];
      }
      return match && match[0] === sentence.text ? [{
        kind: form.id,
        captures: {
          ...match.groups
        }
      }] : [];
    });
    if (matches.length !== 1) return null;
    const item = matches[0],
      capture = item.captures;
    if (item.kind === 'declaration') {
      const observed = signature(capture.signature);
      if (!observed || observed.at !== 0 || !capture.signature.endsWith(')') || observed.name !== request.name || JSON.stringify(observed.parameters) !== JSON.stringify(request.parameters) || capture.destination !== request.destination) return null;
    }
    if (item.kind === 'source-discovery' && capture.sources !== request.inputs.join(findChildValue(root, 'source-joiner'))) return null;
    if (item.kind === 'verification' && (request.acceptance.length !== 1 || capture.acceptance !== request.acceptance[0] || capture.command !== request.command)) return null;
    const start = encoder.encode(scope.slice(0, sentence.span.start)).length;
    const end = encoder.encode(scope.slice(0, sentence.span.end)).length;
    clauses.push({
      ...item,
      sentence: sentence.text,
      evidence: sourceEvidence(text, start, end),
      status: 'requires-contract'
    });
  }
  if (!kinds.every(kind => clauses.filter(clause => clause.kind === kind).length === 1)) return null;
  return {
    request,
    requestIdentity: sha256Hex(text),
    seedIdentity: sha256Hex(seed),
    clauses,
    syntaxCoverage: 'complete-finite-seeded-forms',
    semanticAuthority: false,
    writeAuthority: false
  };
}
/** Discharge only a reversible candidate authoring transaction, not universal effect purity. */
export function bindCompleteSourceNeeds(frame, composition, preflight, canonicalSources) {
  if (!frame || !composition || !preflight || frame.requestIdentity !== composition.requestIdentity || preflight.ledger.sourceIdentity !== frame.requestIdentity) throw Error(sourceContractDiagnostic("unbound-request-contracts"));
  if (!preflight.prerequisites.every(value => ['conditional-satisfied-read', 'conditional-satisfied-absence'].includes(value.status))) throw Error(sourceContractDiagnostic("unsolved-read-prerequisite"));
  const goal = frame.clauses.find(value => value.kind === 'goal');
  if (goal.sentence !== composition.returnForm.sentence || composition.goalClauseCoverage !== 'complete-seeded-return-form') throw Error(sourceContractDiagnostic("unbound-source-goal"));
  if (!composition.imports.every(binding => canonicalSources.some(value => value.moduleURL === binding.moduleURL && value.sha256 === binding.sourceIdentity))) throw Error(sourceContractDiagnostic("unbound-canonical-source-receipt"));
  if (composition.effectPlan.kind !== 'checked-source-import-composition' || composition.effectPlan.localMutation !== false || composition.effectPlan.additionalModuleInitialization !== false) throw Error(sourceContractDiagnostic("unbound-imported-implementation"));
  return {
    requestIdentity: frame.requestIdentity,
    clauses: frame.clauses,
    readPrerequisites: preflight.prerequisites,
    boundedRepairPlan: {
      graph: composition.graph,
      imports: composition.imports,
      effects: composition.effectPlan
    },
    pending: [{
      kind: 'verification',
      command: frame.request.command
    }, {
      kind: 'protected-input-byte-recheck'
    }],
    allowedDestination: frame.request.destination,
    allowedCommand: frame.request.command,
    candidateWriteAuthority: 'conditional-reversible-draft-only',
    unconditionalDeliveryAuthority: false,
    sourceEffects: 'conditional',
    moduleInitialization: 'inherited same provider-approved Node operation; not purity',
    loadedByteRaceProof: false,
    autonomousCredit: 0
  };
}
