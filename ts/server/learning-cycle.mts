// The proposal-only auto-learning cycle (issue #701) the idle dreaming run
// records: rust/src/learning_cycle.rs (`parse_frontier_record`,
// `run_learning_cycle`, `google_trends_learning_cycle`,
// `LearningCycleRun::links_notation`, `derive_candidate`, `frame_of`,
// `classify_slot`, `recovers_query`, `build_proposals`, `meaning_block`)
// and the slice of rust/src/promotion.rs it renders through
// (`render_promotion_proposals`, `manifest_field`, `quote`) with the
// canonical gate identities of rust/src/promotion/gates.rs `required_gates`.
//
// A frontier item is a prompt the engine could not route, kept with the topic
// it was generated from. Deleting the topic leaves the request frame; two
// items of one class must agree on it, and every further item is a held-out
// test the frame must match. Validated frames become promotion proposals in
// the issue-#656 shape; nothing is applied. The proposal summary is data
// (data/meta/server-messages.lino).

import { cached, hasHost, installHost } from '../agentic/host.mjs';
import { normalizePrompt } from '../agentic/crate/engine.mjs';
import { splitPipeList } from '../agentic/crate/seed.mjs';
import { agentInfoValue } from '../agentic/crate/seed_agent_info.mjs';
import { parseLino, readRepoFile } from './lino.mjs';
import { serverMessage } from './messages.mjs';

export const GOOGLE_TRENDS_FRONTIER = 'google-trends';
export const GOOGLE_TRENDS_FRONTIER_RECORD_FILE = 'data/meta/learning-frontier-google-trends.lino';
export const LEARNED_REQUEST_OPENERS_SEED_FILE = 'data/seed/learned-request-openers.lino';
export const TERM_INFORMATION_ROLE = 'term_information_request_opener';
export const MINIMUM_SUPPORT = 2;
const UNIT_SPECS_SUITE = 'formal_ai_unit_specifications';
const BENCHMARK_MANIFESTS = [
  ['data/benchmarks/coding-modification-suite.lino', 'issue_362_multilingual_coding_modification'],
  ['data/benchmarks/industry-suite.lino', 'issue_304_industry_permissive_slice'],
];

/** The repository-data host the crate seed readers need (the agentic server installs the full one). */
export function ensureDataHost() {
  if (!hasHost()) installHost({ readText: readRepoFile, parseLino });
}

/** The top-level nodes of a document, as `parse_lino`'s unnamed root holds them. */
function documentRoots(text) {
  const tree = parseLino(text);
  if (tree.indent === -1 || (tree.name === '' && tree.value === '' && tree.indent === undefined)) {
    return tree.children || [];
  }
  return [tree];
}

const childValue = (node, name) => (node.children || []).find((child) => child.name === name)?.value ?? '';

/** Mirrors `parse_frontier_record` in rust/src/learning_cycle.rs. */
export function parseFrontierRecord(document) {
  const items = [];
  for (const root of documentRoots(document)) {
    for (const node of root.children || []) {
      if (node.name !== 'frontier_prompt') continue;
      const rank = childValue(node, 'rank');
      items.push({
        // `str::parse::<usize>().unwrap_or_default()`.
        rank: /^\+?\d+$/u.test(rank) ? Number(rank) : 0,
        query: childValue(node, 'query'),
        language: childValue(node, 'language'),
        variation: childValue(node, 'variation'),
        prompt: childValue(node, 'prompt'),
        engine_intent: childValue(node, 'engine_intent'),
      });
    }
  }
  return items;
}

/** The frame around `query` inside `prompt` (`frame_of`), or null. */
function frameOf(prompt, query) {
  const normalized = normalizePrompt(prompt);
  const needle = normalizePrompt(query);
  if (!needle) return null;
  const index = normalized.indexOf(needle);
  if (index < 0) return null;
  return [normalized.slice(0, index), normalized.slice(index + needle.length)];
}

/** Mirrors `classify_slot`. */
function classifySlot(before, after) {
  const hasBefore = before.trim() !== '';
  const hasAfter = after.trim() !== '';
  if (hasBefore && hasAfter) return 'circumfix';
  if (hasBefore) return 'prefix';
  if (hasAfter) return 'suffix';
  return 'bare';
}

/** Mirrors `recovers_query`. */
function recoversQuery(before, after, slot, prompt, query) {
  const normalized = normalizePrompt(prompt);
  let recovered = null;
  if (slot === 'prefix') {
    if (normalized.startsWith(before)) recovered = normalized.slice(before.length);
  } else if (slot === 'suffix') {
    if (normalized.endsWith(after)) recovered = normalized.slice(0, normalized.length - after.length);
  } else if (slot === 'circumfix') {
    if (normalized.startsWith(before)) {
      const rest = normalized.slice(before.length);
      if (rest.endsWith(after)) recovered = rest.slice(0, rest.length - after.length);
    }
  }
  return recovered !== null && recovered.trim() === normalizePrompt(query);
}

/** `CandidateSurface::validated`. */
export const validated = (candidate) => candidate.held_out.length > 0 && candidate.held_out.every((test) => test.passed);
const passedCount = (candidate) => candidate.held_out.filter((test) => test.passed).length;
const failedCount = (candidate) => candidate.held_out.length - passedCount(candidate);

/** Mirrors `derive_candidate`: `{candidate}` or `{reason}`. */
function deriveCandidate(language, variation, items) {
  const frames = items.map((item) => [item, frameOf(item.prompt, item.query)]).filter(([, frame]) => frame !== null);
  if (frames.length < MINIMUM_SUPPORT) return { reason: 'insufficient_support' };
  const [before, after] = frames[0][1];
  const support = frames.slice(0, MINIMUM_SUPPORT).map(([item]) => item.query);
  if (frames.slice(0, MINIMUM_SUPPORT).some(([, frame]) => frame[0] !== before || frame[1] !== after)) {
    return { reason: 'supporting_prompts_disagree_on_frame' };
  }
  const slot = classifySlot(before, after);
  if (slot === 'bare') return { reason: 'frame_has_no_subject_slot' };
  const heldOut = frames.slice(MINIMUM_SUPPORT).map(([item]) => ({
    prompt: item.prompt,
    expected_query: item.query,
    passed: recoversQuery(before, after, slot, item.prompt, item.query),
  }));
  if (heldOut.length === 0) return { reason: 'no_held_out_prompts_to_validate_against' };
  return { candidate: { language, variation, surface: `${before}…${after}`, slot, support, held_out: heldOut } };
}

/** Rust `str` ordering: by UTF-8 bytes, which code points preserve. */
function compareStr(left, right) {
  const a = Array.from(left);
  const b = Array.from(right);
  for (let index = 0; index < Math.min(a.length, b.length); index += 1) {
    const diff = a[index].codePointAt(0) - b[index].codePointAt(0);
    if (diff !== 0) return diff;
  }
  return a.length - b.length;
}

function languageOrder(language) {
  const position = splitPipeList(agentInfoValue('supported_languages') ?? '').indexOf(language);
  return position < 0 ? Number.MAX_SAFE_INTEGER : position;
}

/** Mirrors `meaning_block`. */
function meaningBlock(variation, candidates) {
  let out = `  ${variation}_request_opener\n    defined-by inquiry\n    defined-by concept\n    role ${TERM_INFORMATION_ROLE}\n`;
  for (const candidate of candidates) out += `    lexeme ${candidate.language}\n      surface\n        text "${candidate.surface}"\n`;
  return out;
}

/** Mirrors `build_proposals`: one `{source, summary, seed_file, seed_lino}` per validated variation. */
function buildProposals(frontier, candidates) {
  const byVariation = new Map();
  for (const candidate of candidates.filter(validated)) {
    if (!byVariation.has(candidate.variation)) byVariation.set(candidate.variation, []);
    byVariation.get(candidate.variation).push(candidate);
  }
  return [...byVariation.keys()].sort(compareStr).map((variation) => {
    const group = byVariation.get(variation).slice().sort((left, right) => languageOrder(left.language) - languageOrder(right.language));
    const summary = serverMessage('learning_cycle_proposal_summary', {
      count: group.length,
      variation,
      languages: group.map((candidate) => candidate.language).join(', '),
      support: MINIMUM_SUPPORT,
      passed: group.reduce((sum, candidate) => sum + passedCount(candidate), 0),
      failed: group.reduce((sum, candidate) => sum + failedCount(candidate), 0),
    });
    return {
      source: `learning_frontier:${frontier}:${variation}`,
      summary,
      seed_file: LEARNED_REQUEST_OPENERS_SEED_FILE,
      seed_lino: meaningBlock(variation, group),
    };
  });
}

/** Mirrors `run_learning_cycle`. */
export function runLearningCycle(frontier, items) {
  ensureDataHost();
  const classes = new Map();
  for (const item of items) {
    const key = JSON.stringify([item.variation, item.language]);
    if (!classes.has(key)) classes.set(key, []);
    classes.get(key).push(item);
  }
  const ordered = [...classes.keys()].sort((left, right) => {
    const [lv, ll] = JSON.parse(left);
    const [rv, rl] = JSON.parse(right);
    return compareStr(lv, rv) || compareStr(ll, rl);
  });
  const candidates = [];
  const blocked = [];
  for (const key of ordered) {
    const [variation, language] = JSON.parse(key);
    const classItems = classes.get(key);
    const derived = deriveCandidate(language, variation, classItems);
    if (derived.candidate) candidates.push(derived.candidate);
    else blocked.push({ language, variation, reason: derived.reason, sample_prompt: classItems[0]?.prompt ?? '' });
  }
  for (const candidate of candidates) {
    if (validated(candidate)) continue;
    blocked.push({
      language: candidate.language,
      variation: candidate.variation,
      reason: 'held_out_validation_failed',
      sample_prompt: candidate.held_out.find((test) => !test.passed)?.prompt ?? '',
    });
  }
  return { frontier, frontier_items: items.length, candidates, blocked, proposals: buildProposals(frontier, candidates) };
}

/** Mirrors `google_trends_learning_cycle`: the cycle over the frozen Google Trends frontier. */
export function googleTrendsLearningCycle() {
  return runLearningCycle(GOOGLE_TRENDS_FRONTIER, parseFrontierRecord(readRepoFile(GOOGLE_TRENDS_FRONTIER_RECORD_FILE)));
}

/** Mirrors `quote` in rust/src/promotion.rs. */
function quote(value) {
  return value.replaceAll('\\', '\\\\').replaceAll('"', '\\"').replaceAll('\n', '\\n').replaceAll('\r', '\\r').replaceAll('\t', '\\t');
}

/** Mirrors `unquote` in rust/src/promotion.rs. */
function unquote(raw) {
  const value = raw.length >= 2 && raw.startsWith('"') && raw.endsWith('"') ? raw.slice(1, -1) : raw;
  let out = '';
  for (let index = 0; index < value.length; index += 1) {
    const character = value[index];
    if (character !== '\\') {
      out += character;
      continue;
    }
    index += 1;
    const next = value[index];
    if (next === 'n') out += '\n';
    else if (next === 't') out += '\t';
    else if (next === 'r') out += '\r';
    else if (next === '"') out += '"';
    else if (next === '\\' || next === undefined) out += '\\';
    else out += `\\${next}`;
  }
  return out;
}

/** Mirrors `manifest_field` in rust/src/promotion.rs. */
function manifestField(manifest, key) {
  for (const line of manifest.split('\n')) {
    const trimmed = line.trim();
    if (trimmed.startsWith(key) && /^\s/u.test(trimmed.slice(key.length))) return unquote(trimmed.slice(key.length).trim());
  }
  return null;
}

/** The canonical suite identities every proposal document declares (`required_gates`). */
export function requiredGateSuites() {
  return cached('required-gate-suites', () => [
    ...BENCHMARK_MANIFESTS.map(([file, fallback]) => manifestField(readRepoFile(file), 'id') ?? fallback),
    UNIT_SPECS_SUITE,
  ]);
}

/** Mirrors `render_promotion_proposals` in rust/src/promotion.rs. */
export function renderPromotionProposals(proposals) {
  ensureDataHost();
  let out = 'promotion_proposals\n';
  const field = (depth, key, value) => `${'  '.repeat(depth)}${key} "${quote(value)}"\n`;
  for (const proposal of proposals) {
    out += '  proposal\n';
    out += field(2, 'source', proposal.source) + field(2, 'summary', proposal.summary)
      + field(2, 'seed_file', proposal.seed_file) + field(2, 'seed_lino', proposal.seed_lino);
    for (const suite of requiredGateSuites()) out += `    gate\n${field(3, 'suite', suite)}`;
  }
  return out.trimEnd();
}

/** Mirrors `LearningCycleRun::links_notation`: the auditable record, trimmed at the end. */
export function learningCycleLinksNotation(run) {
  const validatedCount = run.candidates.filter(validated).length;
  const heldOut = run.candidates.reduce((sum, candidate) => sum + candidate.held_out.length, 0);
  let out = 'learning_cycle\n  record_type "learning_cycle_run"\n  issue "701"\n'
    + `  frontier "${run.frontier}"\n  mode "proposal_only"\n  human_gated "true"\n`
    + `  frontier_items "${run.frontier_items}"\n  candidates "${run.candidates.length}"\n`
    + `  validated_candidates "${validatedCount}"\n  held_out_tests "${heldOut}"\n`
    + `  proposals "${run.proposals.length}"\n  blocked_classes "${run.blocked.length}"\n`;
  for (const candidate of run.candidates) {
    out += `  candidate\n    language "${candidate.language}"\n    variation "${candidate.variation}"\n`
      + `    role "${TERM_INFORMATION_ROLE}"\n    slot "${candidate.slot}"\n    surface "${candidate.surface}"\n`;
    for (const query of candidate.support) out += `    derived_from "${query}"\n`;
    out += `    held_out_tests "${candidate.held_out.length}"\n    held_out_passed "${passedCount(candidate)}"\n`
      + `    validated "${validated(candidate)}"\n`;
  }
  for (const blocked of run.blocked) {
    out += `  blocked_class\n    language "${blocked.language}"\n    variation "${blocked.variation}"\n`
      + `    reason "${blocked.reason}"\n    sample_prompt "${blocked.sample_prompt}"\n    routed_to "human_triage"\n`;
  }
  for (const line of renderPromotionProposals(run.proposals).split('\n').slice(1)) out += `${line}\n`;
  return out.trimEnd();
}
