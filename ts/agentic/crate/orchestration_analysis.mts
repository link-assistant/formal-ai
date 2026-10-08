// `crate::orchestration::analysis` (rust/src/orchestration/analysis.rs): merge
// the results of several external agents as sourced statements, rank them by
// evidence, run the repository's conservative cross-source preflight, and
// project recorded sessions into the proposal-only client-contract learner.
//
// `extract_agent_result` and `apply_verified_translation` are self-contained.
// `synthesize_sessions` drives the summarization pipeline (summarization.mjs,
// summarization_dedup.mjs, summarization_importance.mjs,
// summarization_recheck.mjs) and the translation formalizer
// (translation_formalization.mjs), the modules the Rust file imports from
// `crate::summarization` and `crate::translation`.

import { clientContractObservation } from './client_contract_learning.mjs';
import { detect, languageFromSlug } from './language.mjs';
import { isJsonObject, jsonValueStream, firstJsonValue } from './orchestration_json_stream.mjs';
import { sessionSha256 } from './orchestration_runner.mjs';
import { localizedResponse } from './seed.mjs';
import { SourceTier } from './relative_meta_logic.mjs';
import {
  SummarizationMode, defaultConfig, deformalize, formalize, summarize, withLanguage, withMode,
} from './summarization.mjs';
import { deduplicate, mergedSources, sourcedStatement } from './summarization_dedup.mjs';
import { rank } from './summarization_importance.mjs';
import {
  checkedText, recheck, survivors, verdictIsPresentable, verdictSlug,
} from './summarization_recheck.mjs';
import { candidateToLinksNotation, formalizePrompt } from './translation_formalization.mjs';

const SOURCES_PLACEHOLDER = '{sources}';
const PROBABILITY_PLACEHOLDER = '{probability}';

/** Mirrors `enum AgentSynthesisError` in rust/src/orchestration/analysis.rs; `message` is its `Display`. */
export class AgentSynthesisError extends Error {
  /** @param {string} variant @param {*} [detail] */
  constructor(variant, detail = null) {
    super(AgentSynthesisError.display(variant, detail));
    this.variant = variant;
    this.detail = detail;
    this.debug = AgentSynthesisError.debugOf(variant, detail);
  }

  /** The derived `Debug` rendering `main` prints for a returned error. */
  static debugOf(variant, detail) {
    switch (variant) {
      case 'unsupported_language': return `UnsupportedLanguage(${JSON.stringify(detail)})`;
      case 'session_digest': return `SessionDigest(${JSON.stringify(detail)})`;
      case 'translation_language_mismatch':
        return `TranslationLanguageMismatch { expected: ${JSON.stringify(detail.expected)}, actual: ${JSON.stringify(detail.actual)} }`;
      default: return 'MissingSources';
    }
  }

  static display(variant, detail) {
    switch (variant) {
      case 'unsupported_language': return `unsupported_response_language:${detail}`;
      case 'session_digest': return `session_digest:${detail}`;
      case 'translation_language_mismatch': return `translation_language_mismatch:${detail.expected}:${detail.actual}`;
      default: return 'missing_agent_sources';
    }
  }
}

/** Mirrors `serde_json::Value::pointer` in rust/src/orchestration/analysis.rs. `Value::pointer` over objects: the value at `/a/b`, or undefined. */
function pointer(value, location) {
  let current = value;
  for (const part of location.split('/').slice(1)) {
    if (!isJsonObject(current) || !(part in current)) return undefined;
    current = current[part];
  }
  return current;
}

/** `Value::get(key)` on an object, or undefined. */
const getKey = (value, key) => (isJsonObject(value) && key in value ? value[key] : undefined);

const asString = (value) => (typeof value === 'string' ? value : undefined);

/** Mirrors `fn text_at` in rust/src/orchestration/analysis.rs. */
function textAt(value, location) {
  const text = asString(pointer(value, location))?.trim();
  return text ? text : null;
}

/** Mirrors `fn content_text` in rust/src/orchestration/analysis.rs. */
function contentText(value) {
  if (value === undefined) return null;
  if (typeof value === 'string') return value.trim() === '' ? null : value.trim();
  if (!Array.isArray(value)) return null;
  const parts = value
    .filter((part) => {
      const kind = asString(getKey(part, 'type'));
      return kind === undefined || kind === 'text';
    })
    .map((part) => asString(getKey(part, 'text')))
    .filter((text) => text !== undefined)
    .map((text) => text.trim())
    .filter((text) => text !== '');
  return parts.length > 0 ? parts.join('\n') : null;
}

/** Mirrors `fn result_candidate` in rust/src/orchestration/analysis.rs: `[priority, text]` or null. */
function resultCandidate(value) {
  const eventType = asString(getKey(value, 'type')) ?? '';
  const role = asString(getKey(value, 'role')) ?? asString(pointer(value, '/message/role'));
  if (asString(pointer(value, '/item/type')) === 'agent_message') {
    const text = textAt(value, '/item/text');
    return text === null ? null : [4, text];
  }
  if (role === 'assistant') {
    const direct = getKey(value, 'content');
    const content = contentText(direct !== undefined ? direct : pointer(value, '/message/content'));
    return content === null ? null : [4, content];
  }
  if (eventType === 'result') {
    const found = ['result', 'output', 'text'].map((key) => asString(getKey(value, key))).find((text) => text !== undefined);
    return found === undefined || found.trim() === '' ? null : [3, found.trim()];
  }
  if (eventType === 'text' || eventType === 'assistant') {
    const text = textAt(value, '/part/text') ?? textAt(value, '/text');
    return text === null ? null : [2, text];
  }
  return null;
}

/** Mirrors `fn update_best_candidate` in rust/src/orchestration/analysis.rs; `best` is `{value: [priority, text]|null}`. */
function updateBestCandidate(best, value) {
  const candidate = resultCandidate(value);
  if (candidate !== null && (best.value === null || candidate[0] >= best.value[0])) best.value = candidate;
}

/**
 * Mirrors `fn update_embedded_json_candidates` in rust/src/orchestration/analysis.rs:
 * recover complete JSON values from a line even when a process supervisor
 * writes its own status text on it.
 */
function updateEmbeddedJsonCandidates(best, line) {
  for (let offset = 0; offset < line.length; offset += 1) {
    if (line[offset] !== '{' && line[offset] !== '[') continue;
    const value = firstJsonValue(line.slice(offset));
    if (value !== undefined) updateBestCandidate(best, value);
  }
}

/**
 * Mirrors `fn extract_agent_result` in rust/src/orchestration/analysis.rs:
 * the last assistant answer from common JSON/JSONL agent streams. Plain-text
 * clients pass through unchanged.
 * @param {string} stdout
 */
export function extractAgentResult(stdout) {
  const best = { value: null };
  let complete = true;
  for (const entry of jsonValueStream(stdout)) {
    if (entry.error) {
      complete = false;
      break;
    }
    updateBestCandidate(best, entry.value);
  }
  if (complete && best.value !== null) return best.value[1];
  for (const line of stdout.split(/\r?\n/u)) updateEmbeddedJsonCandidates(best, line);
  return best.value === null ? stdout.trim() : best.value[1];
}

/** Mirrors `fn correction_requests` in rust/src/orchestration/analysis.rs. */
function correctionRequests(sessions, claims) {
  const requests = [];
  for (const claim of claims.filter((entry) => !entry.presented && entry.denied_by.length > 0)) {
    for (const source of claim.sources) {
      const split = source.lastIndexOf(':');
      if (split < 0) continue;
      const cli = source.slice(0, split);
      const index = source.slice(split + 1);
      if (!/^\+?[0-9]+$/.test(index)) continue;
      const session = sessions[Number(index)];
      if (session === undefined) continue;
      const evidence = (localizedResponse('orchestration_cross_agent_denial', detect(claim.text)) ?? 'orchestration_cross_agent_denial')
        .split(SOURCES_PLACEHOLDER).join(claim.denied_by.join(','))
        .split(PROBABILITY_PLACEHOLDER).join(claim.probability.toFixed(6));
      requests.push({ cli, claim: claim.text, evidence, source_session_sha256: sessionSha256(session) });
    }
  }
  return requests;
}

/**
 * Mirrors `fn synthesize_sessions` in rust/src/orchestration/analysis.rs: merge
 * agent output as sourced statements, rank it by evidence, and run the
 * conservative cross-source preflight before presentation. The scope is named
 * `cross_agent_evidence_preflight`: model agreement is not external proof.
 * @param {object[]} sessions the recorded `AgentSession`s
 * @param {string} targetLanguage a language slug
 * @returns {Promise<object>} the `AgentSynthesisReport`
 */
export async function synthesizeSessions(sessions, targetLanguage) {
  if (sessions.length === 0) throw new AgentSynthesisError('missing_sources');
  if (languageFromSlug(targetLanguage) === null) throw new AgentSynthesisError('unsupported_language', targetLanguage);
  const sources = [];
  const observations = [];
  for (const [index, session] of sessions.entries()) {
    const source = `${session.cli}:${index}`;
    const result = extractAgentResult(session.stdout);
    const detected = detect(result);
    sources.push({
      cli: session.cli,
      session_sha256: sessionSha256(session),
      detected_language: detected,
      meta_language: candidateToLinksNotation(formalizePrompt(result, detected)),
    });
    for (const statement of formalize(result)) observations.push(sourcedStatement(statement, source, SourceTier.OriginalFirstParty));
  }
  const dedup = deduplicate(observations);
  const ranked = rank(dedup);
  const checked = recheck(ranked);
  const claims = checked.checked.map((item) => ({
    id: item.ranked.statement.id,
    text: checkedText(item),
    sources: mergedSources(item.ranked.statement),
    denied_by: item.ranked.denied_by,
    probability: item.ranked.probability,
    importance: item.ranked.score.weight,
    verdict: verdictSlug(item.verdict),
    presented: verdictIsPresentable(item.verdict),
  }));
  const surviving = survivors(checked).map((item) => item.ranked.statement.representative);
  const summary = deformalize(summarize(surviving, withLanguage(withMode(defaultConfig(), SummarizationMode.Standard), targetLanguage)));
  const finalLanguage = detect(summary);
  return {
    schema: 'formal-ai-agent-synthesis-v1',
    target_language: targetLanguage,
    final_language: finalLanguage,
    fact_check_scope: 'cross_agent_evidence_preflight',
    sources,
    claims,
    contradictions: dedup.contradictions.map((entry) => ({ asserted: entry.asserted, denied: entry.denied, terms: entry.terms })),
    corrections: correctionRequests(sessions, claims),
    summary,
    final_answer: summary,
    translation_required: summary !== '' && finalLanguage !== targetLanguage,
    translation: null,
  };
}

/**
 * Mirrors `fn apply_verified_translation` in rust/src/orchestration/analysis.rs:
 * accept translated bytes only when their detected language matches the
 * requested language, and retain the translating session digest.
 */
export function applyVerifiedTranslation(report, text, translatorSessionSha256) {
  const actual = detect(text);
  if (actual !== report.target_language) {
    throw new AgentSynthesisError('translation_language_mismatch', { expected: report.target_language, actual });
  }
  report.final_answer = text;
  report.final_language = actual;
  report.translation_required = false;
  report.translation = { text, language: actual, session_sha256: translatorSessionSha256 };
}

/**
 * Mirrors `fn observe_orchestration_session` in rust/src/orchestration/analysis.rs:
 * project one real orchestration session into the proposal-only client-contract learner.
 */
export function observeOrchestrationSession(session, evidence) {
  const observation = clientContractObservation(session.cli, 'agent_orchestration', session.task, 'in_band', [], evidence);
  observation.observed_contract.orchestration_target = [session.target];
  observation.observed_contract.orchestration_program = [session.program];
  if (session.native_session) observation.observed_contract.native_resume = ['true'];
  return observation;
}
