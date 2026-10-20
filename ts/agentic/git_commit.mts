import { lower as canonicalUnaryLower } from './crate/rust_str.mjs';
import {quoteShellWord as shellQuote} from './portable/string_contracts.mjs';
// Committing and pushing what a task produced
// (rust/src/agentic_coding/git_commit.rs, issue #1133).
//
// A `CommitTarget` is `{branch: string|null, reference: string}`.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { rustLines } from './content.mjs';
import { mentionsSoftwareAuthoring, repositoryWorkReference } from './general_planner.mjs';
import { finalAnswer, jsonText, planOne } from './plan.mjs';
import { Progress } from './progress.mjs';
import { render } from './tool_result.mjs';
import { fill } from './work_item_steps.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { mentionsRole, wordsForRole } from './write_lexicon.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { textOutsideQuotedSegments } from './crate/coding_program_contract.mjs';
import { charIn, isAscii, isAsciiAlphanumeric, isWhitespace, trim, trimEnd, trimMatches, trimStart } from './write_str.mjs';

/** Mirrors `CommitTarget::push_ref`. */
export function pushRef(target) {
  return target.branch ?? 'HEAD';
}

/** Mirrors `fn target_of`: a `CommitTarget` or null. */
export function targetOf(request) {
  const reference = repositoryWorkReference(request);
  if (reference === null) return null;
  const binding = requestTargetBinding(request, 'branch-binding-patterns', validBranchTarget);
  return { branch: binding.kind === 'bound' ? binding.value : null, reference, bindingRefused: binding.kind === 'refused' };
}

/** Mirrors `fn named_branch`. @returns {string|null} */
export function namedBranch(request) {
  const binding = requestTargetBinding(request, 'branch-binding-patterns', validBranchTarget);
  return binding.kind === 'bound' ? binding.value : null;
}

/** Validate the complete ref, following git-check-ref-format --branch rules. */
export function validBranchTarget(value) {
  return value !== '' && value !== '@' && !value.startsWith('-')
    && !value.endsWith('.') && !JSON.parse(fill('branch-forbidden-substrings', [])).some(part => value.includes(part))
    && !/[\x00-\x20\x7f~^:?*\[\\]/u.test(value)
    && value.split('/').every(part => part !== '' && !part.startsWith('.') && !part.endsWith('.lock'));
}
export function validWorkflowTarget(value) {
  return /\.ya?ml$/iu.test(value) && !value.startsWith('/')
    && !/[\x00-\x20\x7f\\:;|&$<>\x60"']/u.test(value)
    && value.split('/').every(part => part !== '' && part !== '.' && part !== '..');
}
/** Positive owned clauses bind arguments; quoted documents are not directives. */
export function requestTargetBinding(request, key, validate) {
  const patterns = JSON.parse(fill(key, []));
  const excludedPatterns = JSON.parse(fill(key.replace('-binding-patterns', '-excluded-patterns'), []));
  const values = new Set(), excluded = new Set();
  let refused = false, fenced = false, offset = 0;
  const quoted = quotedSegmentSpans(request);
  for (const raw of request.split('\n')) {
    const lineOffset = offset; offset += raw.length + 1;
    let line = raw.trim();
    if (JSON.parse(fill('owned-clause-fence-prefixes', [])).some(prefix => line.startsWith(prefix))) { fenced = !fenced; continue; }
    if (fenced) continue;
    line = line.replace(/^(?:[-*+] |\d+[.)] )/u, '').trim();
    const ownedAt = lineOffset + raw.indexOf(line);
    if (quoted.some(segment => segment.start <= ownedAt && ownedAt < segment.end)) continue;
    for (const [isExcluded, pattern] of [...patterns.map(pattern => [false, pattern]), ...excludedPatterns.map(pattern => [true, pattern])]) {
      const matched = new RegExp(pattern, 'iu').exec(line);
      if (!matched) continue;
      let value = matched[1].trim();
      if (value.endsWith('.') || value.endsWith('。') || value.endsWith('।')) value = value.slice(0, -1);
      if (value.endsWith(',')) value = value.slice(0, -1);
      const literal = quotedSegmentSpans(value);
      if (literal.length === 1 && literal[0].start === 0 && literal[0].end === value.length) value = literal[0].text;
      if (!validate(value)) refused = true; else (isExcluded ? excluded : values).add(value);
    }
  }
  return refused || values.size > 1 || (values.size === 0 && excluded.size > 0) || [...values].some(value => excluded.has(value)) ? {kind:'refused'} : values.size === 1 ? {kind:'bound',value:[...values][0]} : {kind:'absent'};
}

/** Mirrors `fn commit_command`. @param {string|null} body */
export function commitCommand(subject, body, pushRefName) {
  const quoted = shellQuote(subject);
  if (body === null) {
    return fill('commit_command_without_body', [['{subject}', quoted], ['{branch}', pushRefName]]);
  }
  return fill('commit_command', [['{subject}', quoted], ['{body}', shellQuote(body)], ['{branch}', pushRefName]]);
}

/** Mirrors `fn resolves_body`. */
export function resolvesBody(reference) {
  return fill('body_resolves', [['{reference}', reference]]);
}

/** Mirrors `fn shell_quote`. */
export {shellQuote};

/** Mirrors `fn recipe_commit_command`. */
export function recipeCommitCommand(recipe, target) {
  if (target.bindingRefused) return fill('target_binding_refused_command', []);
  const files = [recipe.path, ...recipe.supporting_files.map((file) => file.path)].map(shellQuote).join(' ');
  const command = fill('recipe_commit_command', [
    ['{files}', '"$@"'],
    ['{subject}', shellQuote(recipeSubject(recipe))],
    ['{body}', shellQuote(resolvesBody(target.reference))],
    ['{branch}', shellQuote(pushRef(target))],
  ]);
  return fill('task-scaffold-commit-command', [
    ['{files}', files],
    ['{reference}', shellQuote(target.reference)],
    ['{commit}', command],
  ]);
}

/** Mirrors `fn recipe_subject`. */
export function recipeSubject(recipe) {
  return fill('subject_added', [['{path}', recipe.path]]);
}

/**
 * Mirrors `fn plan_commit_step`.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planCommitStep(task, messages, toolNames) {
  // The commit cue must be the request's own words: `Append "Commit all
  // changes" to notes.txt.` quotes it as text to write (PR #1188 gap 2).
  const normalized = normalizePrompt(textOutsideQuotedSegments(task));
  if (!mentionsRole('git_commit_request', normalized)) return null;
  // A seeded commit phrase may contain an otherwise generic authoring verb.
  // Exclude only complete cue spans; independent authoring still owns its work.
  const authoringText = wordsForRole('git_commit_request').reduce((text, cue) => {
    const escaped = cue.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&');
    return text.replace(new RegExp('(?<![\\p{L}\\p{N}])' + escaped
      + '(?![\\p{L}\\p{N}])', 'gu'), ' ');
  }, normalized);
  if (repositoryWorkReference(task) !== null || mentionsSoftwareAuthoring(authoringText)) return null;
  const run = toolFor(toolNames, Capability.Run);
  if (!run) return null;
  const progress = Progress.scan(messages);
  if (progress.run_outputs.length) {
    return finalAnswer(render('git commit', progress.run_outputs[progress.run_outputs.length - 1], task));
  }
  return planOne(run, jsonText({ command: commitCommand(statedSubject(task) ?? subjectForListing(task), null, 'HEAD') }));
}

/**
 * Mirrors `fn stated_subject`: the first quoted text after a seeded
 * `git_commit_message_lead`, or null.
 */
function statedSubject(task) {
  const lowered = task.toLowerCase();
  const ends = wordsForRole('git_commit_message_lead').map(canonicalUnaryLower)
    .map((word) => { const at = lowered.indexOf(word); return at < 0 ? null : at + word.length; })
    .filter((end) => end !== null);
  if (!ends.length) return null;
  const from = Math.min(...ends);
  const segment = quotedSegmentSpans(task).find((candidate) => candidate.start >= from && trim(candidate.text) !== '');
  return segment ? segment.text : null;
}

function subjectForListing(task) {
  const added = [];
  const changed = [];
  for (const line of rustLines(task)) {
    const entry = porcelainEntry(line);
    if (!entry) continue;
    if (entry[0] === '??' || entry[0] === 'A') added.push(entry[1]);
    else changed.push(entry[1]);
  }
  if (!added.length && !changed.length) return fill('subject_none', []);
  if (added.length === 1 && !changed.length) return fill('subject_added', [['{path}', added[0]]]);
  if (!added.length && changed.length === 1) return fill('subject_updated', [['{path}', changed[0]]]);
  const all = [...added, ...changed].slice(0, 5);
  return fill('subject_many', [['{count}', String(added.length + changed.length)], ['{paths}', all.join(', ')]]);
}

function porcelainEntry(rawLine) {
  const raw = trimStart(trimEnd(rawLine));
  const chars = Array.from(raw);
  const split = chars.findIndex(isWhitespace);
  if (split < 0) return null;
  const rawStatus = chars.slice(0, split).join('');
  const rest = chars.slice(split + 1).join('');
  const status = rawStatus === '??' ? rawStatus : trimMatches(rawStatus, (character) => character === '?');
  const known = ['??', 'A', 'M', 'MM', 'AM', 'D', 'R', 'RM'].includes(status);
  const path = trim(rest);
  return known && path !== '' && !/\p{White_Space}/u.test(path) ? [status, path] : null;
}
