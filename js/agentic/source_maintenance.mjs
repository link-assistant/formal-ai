import { deriveSourceMaintenance } from './source_maintenance_transform.mjs';
import { readText, parseLino } from './host.mjs';
import { namedTargetAndPayloads } from './workspace_line_operation.mjs';
import { sentences } from './shell_command_policy.mjs';
import { Progress } from './progress.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { planOne, resolvedFinalAnswer, FinalDisposition, jsonText } from './plan.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { sourceMaintenanceNormalizeCueKernel, sourceMaintenanceLanguageKernel, sourceMaintenanceOwnershipKernel, sourceMaintenanceStageKernel, sourceMaintenanceEffectKernel } from './portable/source_maintenance_state.mjs';
const normalized = text => sourceMaintenanceNormalizeCueKernel(Array.from(text));
function grammar() {
  try {
    const text = readText('data/meta/source-maintenance-grammar.lino');
    if (typeof text !== 'string') return null;
    const parsed = parseLino(text);
    const root = parsed.name === 'source-maintenance-grammar' ? parsed : parsed.children?.find(node => node.name === 'source-maintenance-grammar');
    return root?.children ?? null;
  } catch {
    return null;
  }
}
/** Native request adapter consumes the same canonical ownership decisions. */
export function ownedSourceMaintenanceRequest(request) {
  if (typeof request !== 'string') return null;
  const clauses = sentences(request).flatMap(clause => clause.text.split(';')).map(text => text.trim()).filter(Boolean);
  const first = clauses[0];
  const named = namedTargetAndPayloads(request);
  if (!first || !named) return null;
  const target = named.target;
  const position = typeof first === 'string' ? first.indexOf(target) : -1;
  const prefix = position < 0 ? '' : normalized(first.slice(0, position).replace(/['"`]$/u, ''));
  const suffix = position < 0 ? '' : normalized(first.slice(position + target.length).replace(/^['"`]/u, ''));
  const rules = grammar();
  if (rules === null) return null;
  const values = name => rules.filter(node => node.name === name).map(node => node.value ?? node.id);
  const patterns = rules.filter(node => node.name === 'constraint').map(node => node.children?.find(child => child.name === 'pattern')?.value);
  const grammarValid = patterns.every(pattern => typeof pattern === 'string');
  const constraints = grammarValid ? clauses.slice(1).map(clause => patterns.some(pattern => new RegExp('^(?:' + pattern + ')$', 'u').test(normalized(clause)))) : [false];
  const language = sourceMaintenanceLanguageKernel(Array.from(target));
  const owned = sourceMaintenanceOwnershipKernel(named !== null && named !== undefined && position >= 0, named?.payloads.length ?? 0, language, prefix, suffix, grammarValid, values('action'), values('property'), values('preposition'), values('scope'), constraints);
  return owned ? Object.freeze({
    target,
    property: 'trailing-horizontal-whitespace',
    language
  }) : null;
}
function refused(decision) {
  return resolvedFinalAnswer(decision.reason, decision.failure ? FinalDisposition.Failure : FinalDisposition.Gap, 'source-maintenance-observed');
}
function matchingArguments(attempt, target, capability) {
  if (attempt.capability !== capability) return null;
  try {
    const argumentsValue = JSON.parse(attempt.arguments);
    return argumentsValue.filePath === target ? argumentsValue : null;
  } catch {
    return null;
  }
}
export async function planSourceMaintenanceStep(request, messages, tools) {
  const contract = ownedSourceMaintenanceRequest(request);
  if (contract === null) return null;
  const progress = Progress.scan(messages);
  const read = toolFor(tools, Capability.Read);
  const edit = toolFor(tools, Capability.Edit);
  const observed = progress.sourceReadFor(contract.target);
  const edits = progress.attempts.map((attempt, index) => ({
    attempt,
    index,
    argumentsValue: matchingArguments(attempt, contract.target, Capability.Edit)
  })).filter(value => value.argumentsValue !== null);
  const latest = edits.at(-1) ?? null;
  const complete = observed?.status === 'reported-success' && observed.complete === true && !observed.absent && typeof observed.source === 'string';
  const readbackSeen = latest !== null && progress.attempts.slice(latest.index + 1).some(attempt => matchingArguments(attempt, contract.target, Capability.Read) !== null);
  const stage = sourceMaintenanceStageKernel(Boolean(read), observed !== null, complete, edits.some(value => !value.attempt.succeeded), latest !== null, readbackSeen, latest !== null && latest.argumentsValue.newString === observed?.source);
  if (stage.$ === 'refused') return refused(stage);
  if (stage.$ === 'read') return planOne(read, jsonText({
    filePath: contract.target
  }));
  const derived = deriveSourceMaintenance(contract.property, contract.language, observed.source);
  const decision = sourceMaintenanceEffectKernel(derived?.gap ?? (typeof derived?.source === 'string' ? '' : 'SourceMaintenanceValidationUnavailable'), derived?.source === observed.source, latest !== null, Boolean(edit));
  if (decision.$ === 'refused') return refused(decision);
  if (decision.$ === 'finding') return resolvedFinalAnswer(jsonText({
    sourceMaintenance: {
      path: contract.target,
      property: contract.property,
      verified: true,
      written: latest !== null,
      sha256: sha256Hex(observed.source)
    }
  }), FinalDisposition.Finding, 'source-maintenance-observed');
  return planOne(edit, jsonText({
    filePath: contract.target,
    oldString: observed.source,
    newString: derived.source
  }));
}
