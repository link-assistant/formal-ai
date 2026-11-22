// Plan a human-gated learning report from the captured publisher feed.

import { host } from './host.mjs';
import { linoField, loadCatalog } from './google_trends_catalog.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';
import { learningSources, matchDirective } from './crate/seed_learning_sources.mjs';
import { findChildValue, parseRoot } from './crate/seed_parser.mjs';

/** Mirrors `GOOGLE_TRENDS_LEARNING_PATH` in rust/src/agentic_coding/google_trends_learning.rs. */
export const GOOGLE_TRENDS_LEARNING_PATH = 'google-trends-learning.lino';

/** Mirrors `GOOGLE_TRENDS_LEARNING_TASK` in rust/src/agentic_coding/google_trends_learning.rs. */
export function googleTrendsLearningTask() {
  return agenticMessage('google_trends_learning_task');
}

/** Mirrors `LEARNING_CAPABILITY` in rust/src/agentic_coding/google_trends_learning.rs. */
const LEARNING_CAPABILITY = 'google_trends_learning';

const GOOGLE_TRENDS_KEYWORDS = ['google trends', 'trending search'];

/**
 * Mirrors `fn is_google_trends_learning_task` in rust/src/agentic_coding/google_trends_learning.rs.
 * @param {string} prompt
 */
export function isGoogleTrendsLearningTask(prompt) {
  const lower = prompt.toLowerCase();
  if (isLearnFromSourceDirective(lower)) return true;
  const cannotResolve = lower.includes('resolve')
    && (lower.includes('cannot') || lower.includes("can't") || lower.includes('not yet'));
  return GOOGLE_TRENDS_KEYWORDS.some((keyword) => lower.includes(keyword))
    && (agenticMessage('google_trends_learning_frontier_cues').split('|').some((cue) => lower.includes(cue)) || cannotResolve);
}

/** Mirrors `fn is_learn_from_source_directive` in rust/src/agentic_coding/google_trends_learning.rs. */
function isLearnFromSourceDirective(lowercased) {
  return matchDirective(learningSources(), lowercased)?.capability === LEARNING_CAPABILITY;
}

/** Mirrors `trending_learning_report_from_feed`. */
export function renderLearningReport(catalog) {
  const frontier = [];
  const unresolved = [];
  let total = 0;
  for (const topic of catalog.topics) {
    for (const { language, variation, prompt, answer } of topic.answered) {
      total += 1;
      if (answer.intent !== 'unknown') continue;
      frontier.push({ rank: topic.rank, query: topic.query, language, variation, prompt, engine_intent: answer.intent, routed_to: 'human_triage' });
      unresolved.push({ prompt, intent: answer.intent, events: answer.solver_events ?? [] });
    }
  }
  const run = host().reviewUnknownAnswers(unresolved);
  const reason = run.rejections[0]?.reason;
  const summary = agenticMessage(frontier.length ? 'google_trends_frontier_triage' : 'google_trends_frontier_empty');
  const fields = {
    record_type: 'google_trends_learning_report',
    issue: '498',
    auto_learning_loop: 'issue_558_self_improvement',
    human_gated: 'true',
    summary,
    source_url: catalog.url,
    source_sha256: catalog.sha256,
    total_prompts: total,
    handled_by_engine: total - frontier.length,
    learning_frontier: frontier.length,
    learning_run_id: run.id,
    learning_run_trace_count: run.trace_count,
    learning_run_proposals: run.proposals.length,
    learning_run_adopted: 0,
    learning_run_rejection_reason: reason && run.rejections.every((item) => item.reason === reason) ? reason : 'none',
    benchmark_suite: run.gate.suite_id,
    benchmark_status: run.gate.statusSlug(),
  };
  let out = 'google_trends_learning\n';
  for (const [key, value] of Object.entries(fields)) out += linoField(2, key, value);
  for (const entry of frontier) {
    out += '  frontier_prompt\n';
    for (const [key, value] of Object.entries(entry)) out += linoField(4, key, value);
  }
  return out;
}

/** Mirrors `document_for_task`. */
export async function renderDocument(task) {
  const catalog = await loadCatalog(task);
  return catalog && renderLearningReport(catalog);
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/google_trends_learning.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  const report = parseRoot(document).children[0];
  return agenticMessage('google_trends_learning_final_answer', {
    total: findChildValue(report, 'total_prompts'),
    handled: findChildValue(report, 'handled_by_engine'),
    frontier: findChildValue(report, 'learning_frontier'),
    proposals: findChildValue(report, 'learning_run_proposals'),
    adopted: findChildValue(report, 'learning_run_adopted'),
    source_url: findChildValue(report, 'source_url'),
    path: GOOGLE_TRENDS_LEARNING_PATH,
    document: trimEnd(document),
  });
}

/** Mirrors `fn verification_command` in rust/src/agentic_coding/google_trends_learning.rs. */
export function verificationCommand() {
  const program = `p='${GOOGLE_TRENDS_LEARNING_PATH}';s=open(p).read().splitlines();print(len(s));print('\\n'.join(s[:14]))`;
  return "python3 -c '" + program.replaceAll("'", String.fromCharCode(39, 92, 39, 39)) + "'";
}
