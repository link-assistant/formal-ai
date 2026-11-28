// The issue-#468 text -> Links Notation formalization recipe
// (rust/src/agentic_coding/formalization_recipe.rs).
//
// State machine: discover missing terms when necessary -> bind the requested
// source -> write_file(formalize) -> run_command(verify) -> final.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { offlineRegistryLookup, unknownSurfaces } from './crate/concept_lookup.mjs';
import { formalizeDeeply } from './crate/formalization_concept_links.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { LINO_FENCE_LANGUAGE, fencedBlock } from './crate/issue_report.mjs';
import { detect } from './crate/language.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { replaceAllLiteral, trim } from './crate/rust_str.mjs';
import { localizedResponse, renderResponse } from './crate/seed.mjs';
import { mentionsRole } from './crate/seed_meanings.mjs';
import {
  FISHERMAN_DOC_ID, PRIMITIVE_KINDS, coverageLine, formalizeTextToLinks, totalRecords,
} from './formalize.mjs';
import { Lexicon } from './lexicon.mjs';
import { agenticMessage } from './messages.mjs';
import { fetchArguments, finalAnswer as finalPlan, jsonText, planOne, writeArguments } from './plan.mjs';
import { traceRoute } from './planner/continuation.mjs';
import { Progress } from './progress.mjs';
import { decodeEntities } from './work_item_page.mjs';

/** Mirrors `const KB_PATH`. */
export const KB_PATH = 'knowledge-base.lino';

/** Mirrors `crate::seed::ROLE_AGENT_ACTION_FORMALIZE_VERB`. */
const ROLE_AGENT_ACTION_FORMALIZE_VERB = 'agent_action_formalize_verb';

export function isFormalizationTask(prompt) {
  return mentionsRole(ROLE_AGENT_ACTION_FORMALIZE_VERB, normalizePrompt(prompt));
}

/** Build the publisher request from a registered MediaWiki work. */
export function publicationUrl(work) {
  if (!work?.publisher_api || !work.publisher_title) return null;
  let url;
  try {
    url = new URL(work.publisher_api);
  } catch {
    return null;
  }
  if (url.protocol !== 'https:' || url.search) return null;
  url.search = new URLSearchParams({ action: 'parse', page: work.publisher_title, prop: 'text', formatversion: '2', format: 'json' });
  return url.href;
}

export function publicationText(work, capture) {
  if (typeof capture !== 'string' || new TextEncoder().encode(capture).length > 1048576) return null;
  let page;
  try {
    page = JSON.parse(capture)?.parse;
  } catch {
    return null;
  }
  if (page?.title !== work.publisher_title || typeof page.text !== 'string' || !work.publisher_content_class) return null;
  for (const div of page.text.matchAll(/<div class="([^"]+)"[^>]*>/gu)) {
    if (!div[1].split(/\s+/u).includes(work.publisher_content_class)) continue;
    const start = div.index + div[0].length;
    const end = page.text.indexOf('</div>', start);
    if (end < 0) return null;
    const body = page.text.slice(start, end).replace(/<br ?\/?>\n?/gu, '\n').replace(/<[^>]*>/gu, '');
    const clean = trim(body.split('\n').map((line) => line.trimEnd()).join('\n'));
    return clean ? decodeEntities(clean) : null;
  }
  return null;
}

function callerDocument(task) {
  const at = task.indexOf(':');
  const source = trim(at < 0 ? task : task.slice(at + 1));
  return source || null;
}

/** Mirrors `fn discovery_query`: the first unresolved surface. */
function discoveryQuery(source) {
  return unknownSurfaces(source, detect(source))[0] ?? null;
}

/**
 * Mirrors `fn deep_grounding`: the offline registry lookup over the process
 * source cache. A host without captured sources answers every need
 * `unsatisfiable` (see crate/concept_lookup.mjs `offlineRegistryLookup`).
 */
function deepGrounding(source, docId) {
  const graph = formalizeDeeply(source, docId, offlineRegistryLookup, null, 1);
  return graph.needs.length ? graph : null;
}

/** Mirrors `fn formalize_with_grounding`: `[base, grounding]`. */
function formalizeWithGrounding(source) {
  const base = formalizeTextToLinks(source, '');
  if (base.summary.needs_raised === 0) return [base, null];
  return [base, deepGrounding(source, base.summary.doc_id)];
}

/**
 * Mirrors `fn plan_formalization_step` in rust/src/agentic_coding/formalization_recipe.rs.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planFormalizationStep(task, messages, toolNames) {
  const writeTool = toolFor(toolNames, Capability.Write);
  const runTool = toolFor(toolNames, Capability.Run);
  const progress = Progress.scan(messages);

  const inline = quotedSegmentSpans(task).find((segment) => trim(segment.text));
  const lexicon = Lexicon.standard();
  const work = inline ? lexicon.workForTitle(trim(inline.text)) : lexicon.bestWorkFor(task);
  const publication = publicationUrl(work);
  const requested = publication ? null : inline ? trim(inline.text) : callerDocument(task);
  traceRoute('formalization_source', requested ?? work?.doc_id ?? '');

  const searchTool = toolFor(toolNames, Capability.Search);
  if ((!inline || publication) && searchTool !== null && !progress.done(Capability.Search)) {
    const query = publication ? work.publisher_title : requested ? discoveryQuery(requested) : null;
    if (query) return planOne(searchTool, jsonText({ query }));
  }
  let source = requested;
  if (publication) {
    const fetchTool = toolFor(toolNames, Capability.Fetch);
    if (fetchTool !== null && !progress.done(Capability.Fetch)) return planOne(fetchTool, fetchArguments(publication));
    const evidence = progress.fetched_pages.findLast(([url]) => url === publication);
    source = publicationText(work, evidence?.[1]);
    if (source === null) return finalPlan(agenticMessage('formalization_recipe_source_unavailable', { source: publication }));
  }
  if (source === null) return finalPlan(agenticMessage('formalization_recipe_source_required'));
  const [formalized, grounding] = formalizeWithGrounding(source);
  let knowledgeBase = formalized.links_notation;
  if (grounding !== null) knowledgeBase += `\n${grounding.toLinksNotation()}\n`;

  if (writeTool !== null && !progress.done(Capability.Write)) {
    return planOne(writeTool, writeArguments(KB_PATH, knowledgeBase));
  }
  if (runTool !== null && !progress.done(Capability.Run)) {
    return planOne(runTool, jsonText({ command: `cat ${KB_PATH}` }));
  }
  return finalPlan(finalAnswer(formalized, knowledgeBase, grounding, detect(task)));
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/formalization_recipe.rs. */
function finalAnswer(formalized, knowledgeBase, grounding, language) {
  const summary = formalized.summary;
  const subject = summary.doc_id === FISHERMAN_DOC_ID
    ? agenticMessage('formalization_recipe_fisherman_subject')
    : agenticMessage('formalization_recipe_source_subject', { doc_id: summary.doc_id });
  let needReport = '';
  if (grounding !== null) {
    const [grounded, total] = grounding.groundedRatio();
    const line = renderResponse('agentic_formalization_need_report', language, [['total', String(total)], ['grounded', String(grounded)]]);
    needReport = line === null ? '' : `${line}\n\n`;
  }
  return [
    ['{subject}', subject],
    ['{records}', String(totalRecords(summary))],
    ['{covered_count}', String(summary.covered.length)],
    ['{primitive_count}', String(PRIMITIVE_KINDS.length)],
    ['{coverage}', coverageLine(summary)],
    ['{path}', KB_PATH],
    ['{kb}', `${needReport}${fencedBlock(LINO_FENCE_LANGUAGE, knowledgeBase)}`],
  ].reduce((text, [slot, value]) => replaceAllLiteral(text, slot, value), localizedResponse('agentic_formalization_report', language) ?? '');
}
