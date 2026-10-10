import { parseWriteContract } from '../general_planner.mjs';
import { bareSurfaces, cleanCueToken, cleanPathToken, firstActionCueEnd, firstActionCueStart, firstContentLeadEnd, looksLikeFilePath, tokens } from '../write_request.mjs';
import { quotedSegmentSpans, quoteFault } from '../crate/normal_markov.mjs';
import { sectionScope } from '../markdown_section.mjs';
import { normalizePrompt } from '../crate/engine.mjs';

/** Mirrors owns_additive_scope: accepted composers still own payload interpretation. */
export function ownsAdditiveScope(request) {
  if (quoteFault(request) !== null) return false;
  const contract = parseWriteContract(request);
  const spans = quotedSegmentSpans(request).map(span => [span.start, span.end]);

  for (const token of tokens(request)) {
    if (looksLikeFilePath(cleanPathToken(token.text))) spans.push([token.start, token.end]);
  }
  if (contract !== null && contract.targetSpan.end <= contract.payload.start
    && firstContentLeadEnd(request.slice(contract.targetSpan.end).toLowerCase()) !== null) {
    spans.push([contract.targetSpan.end, contract.payload.end]);
  }
  if (quotedSegmentSpans(request).length === 0) {
    const paths = tokens(request).filter(token => looksLikeFilePath(cleanPathToken(token.text)));
    const words = tokens(request);
    const leads = bareSurfaces('file_edit_line_lead').map(surface => surface.toLowerCase().split(/\s+/u))
      .filter(surface => surface.length > 0).sort((left, right) => right.length - left.length);
    let lead = null;
    for (let index = 0; index < words.length && lead === null; index++) {
      const matched = leads.find(surface => surface.every((word, offset) =>
        words[index + offset] !== undefined && cleanCueToken(words[index + offset].text) === word));
      if (matched !== undefined) lead = [words[index].start, words[index + matched.length - 1].end];
    }
    if (paths.length === 1 && lead !== null && lead[1] < paths[0].start) {
      const destinations = bareSurfaces('file_write_destination_cue');
      const cues = tokens(request.slice(lead[1], paths[0].start)).filter(token =>
        destinations.includes(token.text.toLowerCase().replace(/[.!?,:;]+$/u, '')));
      const destination = cues.at(-1);
      if (destination !== undefined) spans.push([lead[1], lead[1] + destination.start]);
    }
  }
  const ownedEnd = spans.reduce((end, span) => Math.max(end, span[1]), 0);
  if (ownedEnd > 0 && !/^[\s.!?。！？।,，:：;；]*$/u.test(request.slice(ownedEnd))) return false;
  const characters = request.split('');
  for (const [start, end] of spans) for (let index = start; index < end; index++) characters[index] = ' ';
  const view = characters.join('');
  if ([...view].some(character => /[\p{White_Space}\uFEFF]/u.test(character)
    && !' \t\n\r\v\f'.includes(character))) return false;
  let remaining = normalizePrompt(view);
  if (remaining.split(/[.!?。！？।;；]/u).filter(part => part.trim() !== '').length > 1) return false;
  const actionStart = firstActionCueStart(tokens(remaining));
  const actionEnd = firstActionCueEnd(tokens(remaining));
  if (actionStart !== null && actionEnd !== null) remaining = remaining.slice(0, actionStart)
    + ' '.repeat(actionEnd - actionStart) + remaining.slice(actionEnd);
  const roles = ['request_function_word', 'enumeration_cue', 'file_edit_position_end',
    'file_edit_position_start', 'file_edit_line_lead', 'file_edit_blank_line',
    'file_write_destination_cue', 'file_edit_target_cue', 'file_edit_joiner_cue',
    'file_declared_noun', 'file_contents_source_cue'];
  if (sectionScope(request, view.toLowerCase()) !== null) {
    roles.push('file_section_noun');
    const action = bareSurfaces('coding_member_add_action').map(normalizePrompt)
      .sort((left, right) => right.length - left.length)
      .find(surface => remaining.startsWith(surface)
        && !/[\p{L}\p{N}_]/u.test(remaining[surface.length] ?? ''));
    if (action !== undefined) remaining = ' '.repeat(action.length) + remaining.slice(action.length);
  }
  const surfaces = [...new Set(roles.flatMap(role => bareSurfaces(role).map(normalizePrompt)))]
    .filter(Boolean).sort((left, right) => right.length - left.length);
  const word = character => character !== undefined && /[\p{L}\p{N}_]/u.test(character);
  for (const surface of surfaces) {
    let offset = 0;
    while ((offset = remaining.indexOf(surface, offset)) >= 0) {
      const end = offset + surface.length;
      const before = [...remaining.slice(0, offset)].at(-1);
      const after = [...remaining.slice(end)][0];
      if (!word(before) && !word(after)) remaining = remaining.slice(0, offset)
        + ' '.repeat(surface.length) + remaining.slice(end);
      offset = end;
    }
  }
  return /^[\s.!?。！？।,，:：;；()\[\]{}\-—–]*$/u.test(remaining);
}
