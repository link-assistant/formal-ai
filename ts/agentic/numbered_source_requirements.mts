// Numbered source requirements used as the work for a verified delivery.
import { renderSeededChange } from './code_task.mjs';
import { latestUserRequest } from './content.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { hasHost, host } from './host.mjs';
import { FinalDisposition, resolvedFinalAnswer } from './plan.mjs';
import { resolveRequirementTargets } from './requirement_resolution.mjs';
import { planBoundRequestSteps } from './request_sequence.mjs';
import { planDeclaredMemberChange, planSeedLiteralChange } from './structured_edit.mjs';
import { bareSurfaces, safeRelativePath } from './write_request.mjs';

const MAX_REQUIREMENTS = 64;
const MAX_PARAGRAPH_LENGTH = 8192;
const SOURCE_CONTRACT_MISSING = 'coding-source-authoring-contract-missing';

// Use only the opening instruction paragraph. A numbered list in a quoted
// payload, a declaration example or a later evidence field grants no edits.
function numberedClauses(task) {
  const head = task.trimStart().split(/\r?\n[^\S\r\n]*\r?\n/u, 1)[0];
  if (head.length > MAX_PARAGRAPH_LENGTH) return null;
  const quotes = quotedSegmentSpans(head);
  const markers = [...head.matchAll(/(^|\s)\(([1-9][0-9]?)\)(?=\s)/gu)]
    .filter(match => !quotes.some(quote => match.index + match[1].length >= quote.start
      && match.index + match[1].length < quote.end));
  if (markers.length < 2 || markers.length > MAX_REQUIREMENTS) return null;
  const lead = head.slice(0, markers[0].index).trim();
  if (lead !== '' && !lead.endsWith(':')) return null;
  return markers.map((marker, index) => ({
    number: Number(marker[2]),
    text: head.slice(marker.index + marker[0].length, markers[index + 1]?.index).trim(),
  }));
}

function sourceClause(clause) {
  const words = clause.replace(/\.$/u, '').split(/\s+/u);
  const [action, value, relation] = words.map(word => word.toLowerCase());
  if (bareSurfaces('coding_member_add_action').includes(action)
    && bareSurfaces('file_write_action_cue').includes(action)
    && /^[A-Za-z0-9_-]{1,96}$/u.test(value)
    && [...bareSurfaces('file_edit_new_lead_cue'), ...bareSurfaces('file_write_target_cue')].includes(relation)) {
    return { descriptor: words.slice(3).join(' '), member: words[1] };
  }
  if (!bareSurfaces('file_edit_action_cue').includes(action)) return null;
  const oldLeads = bareSurfaces('file_edit_old_lead_cue');
  const newLeads = bareSurfaces('file_edit_new_lead_cue');
  const from = words.findIndex((word, index) => index >= 2 && oldLeads.includes(word.toLowerCase()));
  const to = words.findLastIndex(word => newLeads.includes(word.toLowerCase()));
  if (from < 0 || to <= from + 1) return null;
  const old = words.slice(from + 1, to).join(' ');
  const next = words.slice(to + 1).join(' ');
  if (old === next || ![old, next].every(text => /^[A-Za-z0-9_ -]{1,96}$/u.test(text))) return null;
  return { descriptor: words.slice(1, from).join(' '), old, next };
}

function gap(task) {
  const answer = renderSeededChange(SOURCE_CONTRACT_MISSING, task, '', []);
  return answer === null ? null : resolvedFinalAnswer(answer, FinalDisposition.Gap, SOURCE_CONTRACT_MISSING);
}

function bindClauses(inputs, targets) {
  const owners = inputs.some(input => input !== null && input.member === undefined) && hasHost()
    ? host().generatedSeedOwnerDeclarations?.() ?? [] : [];
  return inputs.map((input, index) => {
    const target = targets[index];
    if (input === null || target === null || !['const', 'static'].includes(target.kind)) return null;
    if (input.member !== undefined) {
      const path = 'rust/' + target.module_path;
      return path.startsWith('rust/src/') && path.endsWith('.rs') && safeRelativePath(path)
        ? { ...input, path, declaration: target.symbol } : null;
    }
    const seed = owners.filter(owner => owner.module_path === target.module_path && owner.symbol === target.symbol
      && owner.kind === target.kind && input.descriptor.split(/[^A-Za-z0-9_]+/u).includes(owner.symbol));
    return seed.length === 1 ? { ...input, path: seed[0].seed_path } : null;
  });
}

export async function planNumberedSourceRequirements(task, messages, toolNames) {
  const clauses = numberedClauses(task);
  if (clauses === null) return null;
  const inputs = clauses.map(clause => sourceClause(clause.text));
  if (inputs.every(input => input === null)) return null;
  const targets = resolveRequirementTargets(inputs.map(input => input?.descriptor ?? null));
  const bound = bindClauses(inputs, targets);
  if (bound.every(input => input === null)) return null;
  if (bound.some(input => input === null) || clauses.some((clause, index) => clause.number !== index + 1)) return gap(task);
  const texts = clauses.map(clause => clause.text);
  if (new Set(texts).size !== texts.length) return gap(task);
  return await planBoundRequestSteps(texts, messages, toolNames, (own, tools) => {
    const part = latestUserRequest(own);
    const input = bound[texts.indexOf(part)];
    return (input.member !== undefined
      ? planDeclaredMemberChange(part, own, tools, input.path, input.declaration, input.member)
      : planSeedLiteralChange(part, own, tools, input.path, input.old, input.next)) ?? gap(part);
  });
}
