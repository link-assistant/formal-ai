// The literal-file backstop (PR #1188 T96; the guard behind G13, G17 and
// T29): the general change plan wrote its target without reading it, so every
// new misreading of an edit request as a whole-file write destroyed the file.
// Unless the request's first write verb is a seeded whole-file write
// (`file_whole_write_action`: create, write, save, new file, ... in every
// registered language), the target is read first, and an existing non-empty
// file is replaced only when the request consents to it
// (`file_overwrite_consent`); otherwise the plan declines with a seeded
// refusal naming the file. A request that declares the file it writes (a
// seeded `file_declared_noun` right before the path, no destination ahead of
// it, the content stated after it: `add file note.txt containing hello`)
// creates that file, whatever its verb. The per-shape guards
// (`namesAnAddition`, the routed-write removal check) stay; this is the
// backstop.
// rust/src/agentic_coding/literal_write_guard.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { detect } from './crate/language.mjs';
import { renderResponse } from './crate/seed.mjs';
import { mentionsRole } from './crate/seed_meanings.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { readSource } from './module_function.mjs';
import { finalAnswer, planOne } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { readArguments } from './workspace_change.mjs';
import {
  bareSurfaces, cleanCueToken, cleanPathToken, firstActionCueEnd, firstActionCueStart, firstContentLeadEnd,
  looksLikeFilePath, tokens,
} from './write_request.mjs';

const ROLE_WHOLE_WRITE = 'file_whole_write_action';
const ROLE_OVERWRITE_CONSENT = 'file_overwrite_consent';
const ROLE_DECLARED_NOUN = 'file_declared_noun';
const LITERAL_FILE = 'literal_file';

/**
 * Mirrors `fn writes_whole_file` in rust/src/agentic_coding/literal_write_guard.rs:
 * whether the request's first write verb names a file's whole new content,
 * or the request consents to replacing it.
 * @param {string} request
 */
export function writesWholeFile(request) {
  if (mentionsRole(ROLE_OVERWRITE_CONSENT, normalizePrompt(request))) return true;
  const toks = tokens(request);
  if (declaresFile(request, toks)) return true;
  const start = firstActionCueStart(toks);
  const end = firstActionCueEnd(toks);
  return start !== null && end !== null && mentionsRole(ROLE_WHOLE_WRITE, normalizePrompt(request.slice(start, end)));
}

/**
 * Mirrors `fn declares_file`: whether the request declares the file it
 * writes -- a path right after a seeded `file_declared_noun`, no destination
 * or location cue ahead of that noun (`add 'x' to the file a.txt` edits it),
 * and a content lead after the path (`new file: notes.txt, contents: hello`).
 * @param {string} request
 * @param {Array<{text: string, start: number, end: number}>} toks
 */
function declaresFile(request, toks) {
  const nouns = bareSurfaces(ROLE_DECLARED_NOUN);
  const cues = [...bareSurfaces('file_write_destination_cue'), ...bareSurfaces('file_write_target_cue')];
  return toks.some((token, index) => index > 0
    && looksLikeFilePath(cleanPathToken(token.text))
    && nouns.includes(cleanCueToken(toks[index - 1].text))
    && !toks.slice(0, index - 1).some((before) => cues.includes(cleanCueToken(before.text)))
    && firstContentLeadEnd(request.slice(token.end).toLowerCase()) !== null);
}

/**
 * Mirrors `fn guarded_step`: the read of the target, or the refusal that
 * keeps an existing file, ahead of a literal-file plan's writes; null when
 * the plan may proceed.
 */
export function guardedStep(plan, messages, toolNames) {
  if (plan.mode !== LITERAL_FILE) return null;
  // A write never targets a path the request names only inside quoted text
  // (PR #1188 G86).
  if (!namesAsDestination(plan.goal, plan.target)) {
    const values = [['path', plan.target]];
    return finalAnswer(renderResponse('general_change_target_unnamed', detect(plan.goal), values)
      ?? renderResponse('general_change_target_unnamed', 'en', values));
  }
  if (writesWholeFile(plan.goal)) return null;
  const read = toolFor(toolNames, Capability.Read);
  if (read === null) return null;
  const source = readSource(messages.slice(evidenceWindowStart(messages)), plan.target);
  if (source === null) return planOne(read, readArguments(plan.target));
  if (source.trim() === '' || source === plan.content) return null;
  const values = [['path', plan.target]];
  return finalAnswer(renderResponse('general_change_existing_file_kept', detect(plan.goal), values)
    ?? renderResponse('general_change_existing_file_kept', 'en', values));
}

/**
 * Mirrors `fn names_as_destination`: whether `request` names `target` outside
 * its quoted text, or -- naming no path outside it -- quotes `target` whole.
 * @param {string} request
 * @param {string} target
 */
export function namesAsDestination(request, target) {
  const segments = quotedSegmentSpans(request);
  const unquoted = tokens(request)
    .filter((token) => !segments.some((segment) => token.start < segment.end && token.end > segment.start))
    .map((token) => cleanPathToken(token.text))
    .filter((path) => looksLikeFilePath(path));
  return unquoted.includes(target) || (unquoted.length === 0 && segments.some((segment) => segment.text.trim() === target));
}

/**
 * Mirrors `fn is_guard_read`: the failure is the guard's read of a target
 * that does not exist yet, which the plan goes on to create.
 * @param {object} plan
 * @param {string|null} failedPath the path of the failed call
 * @param {string} capability the failed call's capability
 */
export function isGuardRead(plan, failedPath, capability) {
  return plan.mode === LITERAL_FILE && capability === Capability.Read && failedPath === plan.target
    && !writesWholeFile(plan.goal);
}
