// Unquoted operation objects own ambiguous action cues; original positions stay authoritative.
import { roleWordForms } from '../write_lexicon.mjs';
import { quotedSegmentSpans } from './normal_markov.mjs';
import { containsCjk } from './coding_catalog.mjs';
import { rawLowercaseSpan } from '../write_request/lowercase_spans.mjs';
import { literalInstructionView } from './literal_instruction_view.mjs';
import { proseSentences } from '../shell_command_policy.mjs';
import { isAlphanumeric } from '../write_str.mjs';
/** Mirrors fn operation_role_spans. */
function operationRoleSpans(request, role) {
  const spans = [];
  const lowered = request.toLowerCase();
  for (const form of roleWordForms(role)) {
    const text = form.slot === 'bare' ? form.text : form.slot === 'prefix' ? form.before : null;
    if (!text?.trim()) continue;
    const needle = text.trim().toLowerCase();
    for (let offset = lowered.indexOf(needle); offset >= 0; offset = lowered.indexOf(needle, offset + 1)) {
      const span = rawLowercaseSpan(request, [offset, offset + needle.length]);
      if (span === null) continue;
      const [start, end] = span;
      const before = Array.from(request.slice(0, start)).at(-1);
      const after = Array.from(request.slice(end))[0];
      const word = (character) => character !== undefined && (isAlphanumeric(character) || character === '_' || character === '-');
      if (containsCjk(needle) || !word(before) && !word(after)) spans.push({ start, end, role });
    }
  }
  return spans;
}

/** Mirrors `fn first_operation_owner`: the first unquoted seeded operation object owns the leading action. */
export function operationOwner(request, contract = null) {
  let view = literalInstructionView(request, contract?.payload ?? null);
  if (view === null) return null;
  for (const span of quotedSegmentSpans(request)) view = view.slice(0, span.start) + ' '.repeat(span.end - span.start) + view.slice(span.end);
  const sentence = proseSentences(view)[0];
  if (!sentence) return null;
  const roles = ['coding_request_object', 'software_artifact_kind', 'software_artifact', 'capability_web_scope'];
  return roles.flatMap((role) => operationRoleSpans(view, role))
    .filter((span) => span.end <= sentence.span.end)
    .sort((left, right) => left.start - right.start || right.end - left.end)[0] ?? null;
}
