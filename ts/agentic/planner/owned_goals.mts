import { cached, childrenNamed, parseLino, readText } from '../host.mjs';
import { mentionsRole } from '../write_lexicon.mjs';
import { normalizePrompt } from '../crate/engine.mjs';
import { unquotedPathTokens } from '../positional_edit.mjs';
// Source-owned mixed actions share the existing obligation nodes and request replay.
import { composeEditClauses, preferredBinding, tokens, firstActionCueStart, bareSurfaces, cleanPathToken, looksLikeFilePath } from '../write_request.mjs';
import { literalWriteOwnership, instructionView } from '../general_planner.mjs';
import { sentences } from '../shell_command_policy.mjs';
import { quotedSegmentSpans, quoteFault } from '../crate/normal_markov.mjs';
import { nestedQuoteFault } from '../quote_nesting.mjs';
import { leafNode } from '../crate/obligation_ledger.mjs';
import { gapAnswer } from '../task_obligations.mjs';
import { planBoundRequestSteps } from '../request_sequence.mjs';
import { FinalDisposition, canDeliverFinal, resolvedFinalAnswer } from '../plan.mjs';

const encoder = new TextEncoder();
const grammarTail = (text) => /^[\s.!?。！？।;；]*$/u.test(text);

function closedPayload(request, contract) {
  return contract === null ? null : quotedSegmentSpans(request).find((span) =>
    span.start >= contract.payload.start && /^[\s:]*$/u.test(request.slice(contract.payload.start, span.start))) ?? null;
}

/** Mirrors fn literal_tail: the already-owned destination cue may follow the payload. */
function literalTail(request, contract) {
  const end = closedPayload(request, contract)?.end ?? contract.payload.end;
  if (grammarTail(request.slice(end))) return true;
  const words = tokens(request);
  const binding = preferredBinding(words);
  const target = binding === null ? null : words[binding.index];
  const suffix = target === undefined || target === null ? null : preferredBinding(words.slice(binding.index));
  const suffixOwned = suffix !== null && !suffix.cue_precedes && suffix.path === contract.target
    && suffix.cue_start >= target.end
    && grammarTail(request.slice(target.end, suffix.cue_start))
    && bareSurfaces('file_declared_noun').includes(request.slice(suffix.cue_start, suffix.cue_end).toLowerCase().replace(/[\s.!?。！？।;；]+$/u, ''))
    && grammarTail(request.slice(suffix.cue_end));
  return binding !== null && target !== undefined && target !== null
    && binding.path === contract.target && binding.cue_precedes
    && target.start === contract.targetSpan.start && target.end === contract.targetSpan.end
    && binding.cue_start >= end && binding.cue_end <= target.start
    && grammarTail(request.slice(end, binding.cue_start))
    && grammarTail(request.slice(binding.cue_end, target.start))
    && (grammarTail(request.slice(target.end)) || suffixOwned);
}

/** Mirrors fn goal_ledger: preserve raw UTF16 positions and the existing node's UTF8 source span. */
export function goalLedger(request) {
  const contract = literalWriteOwnership(request);
  let view = closedPayload(request, contract) === null ? instructionView(request, contract) : request;
  if (view === null) return null;
  for (const span of quotedSegmentSpans(request)) view = view.slice(0, span.start) + ' '.repeat(span.end - span.start) + view.slice(span.end);
  const goals = sentences(view).map((sentence) => {
    const raw = request.slice(sentence.span.start, sentence.span.end);
    const clause = raw.trim();
    const start = sentence.span.start + raw.indexOf(clause);
    const span = { start, end: start + clause.length };
    const byteSpan = [encoder.encode(request.slice(0, span.start)).length, encoder.encode(request.slice(0, span.end)).length];
    const literal = literalWriteOwnership(clause);
    const edit = literal === null ? composeEditClauses(clause) : null;
    const completeLiteral = literal !== null && literalTail(clause, literal) && !attributedActionPrefix(clause) && contractActionPrologue(clause, literal);
    const kind = completeLiteral ? 'literal_file' : edit !== null && edit.spans !== null ? 'source_edit' : 'unsupported';
    const target = literal?.target ?? edit?.edit[0] ?? null;
    const expectation = kind === 'literal_file' ? { kind: 'file_bytes', path: target, sha256: null }
      : { kind: 'underivable', reason: kind === 'source_edit' ? 'source-edit-preimage-required' : 'no_artifact_in_clause' };
    return { clause, span, byteSpan, sourceUnit: 'utf16', kind, target, expected: literal?.content ?? null,
      node: leafNode(null, clause, byteSpan, 0, expectation) };
  });
  if (goals.length === 0 || goals.length === 1 && goals[0].kind !== 'unsupported') return null;
  return contract !== null && contract.targetSpan.start >= contract.payload.end
    || goals.some((goal) => literalWriteOwnership(goal.clause) !== null) ? goals : null;
}

function goalGap(goal) {
  return resolvedFinalAnswer(gapAnswer(goal.node.node_id, goal.clause, goal.byteSpan, 'no_artifact_in_clause'),
    FinalDisposition.Gap, 'owned-goal-missing-contract');
}

/** Mirrors `fn plan_owned_goal_step`: existing literal-only scheduling remains authoritative. */
export async function planGoalLedger(request, messages, toolNames, planFor) {
  const declaredGoals = goalLedger(request);
  if (declaredGoals === null || quoteFault(request) !== null || nestedQuoteFault(request) !== null) return null;
  const hasContext = declaredGoals.some((goal) => sourceContextDeclaration(goal.clause));
  const goals = declaredGoals.filter((goal) => !sourceContextDeclaration(goal.clause));
  if (goals.length === 0) return null;
  const missingIndex = goals.findIndex((goal) => goal.kind === 'unsupported');
  if (missingIndex >= 0) {
    const missing = goals[missingIndex];
    if (hasContext) return goalGap(missing);
    if (literalWriteOwnership(missing.clause) !== null) return goalGap(missing);
    if (missingIndex === 0) {
      if (hasContext) return goalGap(missing);
      const contract = literalWriteOwnership(request);
      return contract !== null && contract.targetSpan.start >= contract.payload.end
        && (firstActionCueStart(tokens(missing.clause)) !== null
          || contract.payload.start >= missing.span.end && !sourceContextDeclaration(missing.clause) && unquotedPathTokens(missing.clause).some((token) => {
            const path = cleanPathToken(token.text);
            return looksLikeFilePath(path);
          })) ? goalGap(missing) : null;
    }
    const plan = await planBoundRequestSteps(goals.slice(0, missingIndex).map((goal) => goal.clause), messages, toolNames, planFor);
    return plan?.kind === 'final' && canDeliverFinal(plan) ? goalGap(missing) : plan;
  }
  if (!goals.some((goal) => goal.kind === 'source_edit')) return null;
  return planBoundRequestSteps(goals.map((goal) => goal.clause), messages, toolNames, planFor);
}

function contextGrammarPatterns() {
  return cached('source-context-grammar-patterns', () => {
    const root = parseLino(readText('data/seed/source-context-grammar.lino') ?? '');
    const escaped = (form) => form.replace(/[.*+?^\x24{}()|[\]\\]/g, '\\$&');
    return childrenNamed(root, 'language').flatMap((language) => {
      const roles = new Map(childrenNamed(language, 'role').map((role) => [role.id,
        childrenNamed(role, 'form').map((form) => form.id)]));
      return childrenNamed(language, 'pattern').flatMap((pattern) => {
        let missing = false;
        const expressions = ['prefix', 'suffix'].map((side) => {
          const template = childrenNamed(pattern, side)[0]?.id;
          if (typeof template !== 'string') { missing = true; return null; }
          const expanded = template.replace(/\{([a-z]+(?:-[a-z]+)*)\}/gu, (_, role) => {
            const forms = roles.get(role);
            if (forms === undefined || forms.length === 0) { missing = true; return '(?!)'; }
            return '(?:' + forms.map(escaped).join('|') + ')';
          });
          try { return new RegExp(expanded, 'iu'); } catch { missing = true; return null; }
        });
        return missing ? [] : [expressions];
      });
    });
  });
}


function sourceContextWhitespaceSupported(text) {
  if (typeof text !== 'string') return false;
  for (const character of text) {
    if (/[\p{White_Space}\uFEFF]/u.test(character) && !/[ \u0009-\u000D]/u.test(character)) return false;
  }
  return true;
}

function sourceContextAtom(clause) {
  if (!sourceContextWhitespaceSupported(clause)) return false;
  const paths = unquotedPathTokens(clause).filter((token) => looksLikeFilePath(cleanPathToken(token.text)));
  if (paths.length !== 1) return false;
  const token = paths[0], path = cleanPathToken(token.text);
  if (!token.text.startsWith(path)) return false;
  const prefix = clause.slice(0, token.start), suffix = clause.slice(token.start + path.length);
  return contextGrammarPatterns().some(([prefixPattern, suffixPattern]) => prefixPattern.test(prefix) && suffixPattern.test(suffix));
}

function sourceContextDeclaration(clause) {
  if (sourceContextAtom(clause)) return true;
  const joiners = bareSurfaces('file_edit_joiner_cue');
  const boundaries = tokens(clause).filter((token) => joiners.includes(token.text.toLowerCase()));
  if (boundaries.length === 0) return false;
  let start = 0;
  for (const boundary of boundaries) {
    if (!sourceContextAtom(clause.slice(start, boundary.start))) return false;
    start = boundary.end;
  }
  return sourceContextAtom(clause.slice(start));
}

function attributedActionPrefix(clause) {
  let view = clause;
  for (const span of quotedSegmentSpans(clause)) view = view.slice(0, span.start) + ' '.repeat(span.end - span.start) + view.slice(span.end);
  const action = firstActionCueStart(tokens(view));
  return action !== null && mentionsRole('source_attribution_marker', normalizePrompt(view.slice(0, action)));
}

function contractActionPrologue(clause, contract) {
  let view = clause;
  for (const span of quotedSegmentSpans(clause)) view = view.slice(0, span.start) + ' '.repeat(span.end - span.start) + view.slice(span.end);
  const action = firstActionCueStart(tokens(view));
  if (action === null) return false;
  let start = Math.min(action, contract.payload.start, contract.targetSpan.start);
  const words = tokens(clause), binding = preferredBinding(words), target = binding === null ? null : words[binding.index];
  if (binding !== null && target !== null && target !== undefined && binding.path === contract.target
    && target.start === contract.targetSpan.start && target.end === contract.targetSpan.end) start = Math.min(start, binding.cue_start);
  const prologue = normalizePrompt(view.slice(0, start));
  return prologue === '' || ['politeness_cue', 'enumeration_cue', 'file-edit-sequence-cue'].some((role) => bareSurfaces(role).some((surface) => normalizePrompt(surface) === prologue));
}
