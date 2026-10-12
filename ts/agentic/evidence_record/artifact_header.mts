// Artifact type belongs to the delivery header, before its value/content clause.
import { normalizePrompt } from '../crate/engine.mjs';
import { quotedSegmentSpans } from '../crate/normal_markov.mjs';
import { pathsIn, signature } from '../module_function.mjs';
import { mentionsRole } from '../write_lexicon.mjs';
import { firstActionCueStart, firstContentLeadEnd, rankedBindings, tokens } from '../write_request.mjs';

/** Mirrors `fn delivery_header`: preserve raw offsets while excluding target/value bytes. */
function deliveryHeader(sentence, target) {
  const all = tokens(sentence);
  const action = firstActionCueStart(all) ?? 0;
  const binding = rankedBindings(all).find((item) => item.path === target);
  if (action === null || !binding) return null;
  const raw = sentence.slice(action);
  const token = all[binding.index];
  const start = token.start - action;
  const end = token.end - action;
  if (start < 0) return null;
  let masked = raw.slice(0, start) + ' '.repeat(end - start) + raw.slice(end);
  for (const segment of quotedSegmentSpans(masked)) {
    masked = masked.slice(0, segment.start) + ' '.repeat(segment.end - segment.start) + masked.slice(segment.end);
  }
  let lowered = '';
  const offsets = [];
  let position = 0;
  for (const character of masked) {
    const lower = character.toLowerCase();
    for (let unit = 0; unit < lower.length; unit++) offsets.push(position);
    lowered += lower;
    position += character.length;
  }
  const content = firstContentLeadEnd(lowered);
  const limit = content === null ? masked.length : offsets[content[0]];
  return { raw: raw.slice(0, limit), masked: masked.slice(0, limit) };
}

/** Mirrors `fn names_callable_artifact`: source-type evidence is scoped to its action/target header. */
export function namesCallableArtifact(sentence, target) {
  const header = deliveryHeader(sentence, target);
  if (header === null) return false;
  const normalized = normalizePrompt(header.masked);
  const operand = normalized.split(' ').find((word) =>
    mentionsRole('coding-source-artifact-kind', word) || mentionsRole('evidence-report-artifact-kind', word));
  if (operand && mentionsRole('coding-source-artifact-kind', operand)
    && ['coding_request_verb', 'coding_member_add_action'].some((role) => mentionsRole(role, normalized))) return true;
  const stated = signature(header.masked);
  if (stated === null || !pathsIn(header.raw.slice(stated.at)).includes(target)) return false;
  const lead = normalizePrompt(header.masked.slice(0, stated.at));
  return ['coding_request_verb', 'coding_member_add_action'].some((role) => mentionsRole(role, lead));
}
