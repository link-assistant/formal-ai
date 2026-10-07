// Error-driven repair loop for the agentic executor
// (rust/src/agentic_coding/repair_loop.rs, issue #1185).
//
// A `Diagnostic` is `{file, line, code, message, raw}`; a `FailedStep` is
// `{language, reported, exit_code, failed_command, artifact_path}` (see
// `failedStep`). `RepairOutcome` is `{kind: 'search'|'fetch'|'record_fix'|
// 'retry', plan}` or `{kind: 'stop', reason: 'no_diagnostic'|'no_tools'|
// 'exhausted'|'no_match'}`.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { rustLines } from './content.mjs';
import { fetchArguments, jsonText, planOne, writeArguments } from './plan.mjs';
import { Progress } from './progress.mjs';
import { urlsIn } from './web_research.mjs';
import { pushLinoField } from './crate/links_format.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { findChildValue, parseLinoRoot } from './write_lino.mjs';
import { readText } from './host.mjs';
import { trim, trimEnd, trimStart } from './write_str.mjs';

/** Mirrors `const MAX_REPAIR_RUNGS`. */
export const MAX_REPAIR_RUNGS = 3;
const MAX_REPAIR_SOURCES = 3;
const QUERY_MESSAGE_TOKENS = 12;
const SHAPES_PATH = 'data/seed/diagnostic-code-shapes.lino';

function shapeLanguage(language) {
  const lowered = trim(language).replace(/[A-Z]/g, (letter) => letter.toLowerCase());
  return { 'c++': 'cpp', cxx: 'cpp', 'c#': 'csharp', js: 'javascript', ts: 'typescript' }[lowered] ?? lowered;
}

/**
 * Mirrors `fn shapes_for`. Like Rust, it reads `language` records among the
 * document's top-level nodes (`tree.children`).
 */
function shapesFor(language) {
  const tree = parseLinoRoot(readText(SHAPES_PATH));
  const slug = shapeLanguage(language);
  const shapes = { patterns: [], locations: [], heads: [] };
  // The `language` rows sit under the `diagnostic_code_shapes` wrapper.
  const records = tree.children.flatMap((root) => root.children || []);
  for (const record of records.filter((child) => child.name === 'language' && child.id === slug)) {
    for (const field of record.children) {
      if (!field.id) continue;
      if (field.name === 'pattern') shapes.patterns.push(field.id);
      else if (field.name === 'location') shapes.locations.push(field.id);
      else if (field.name === 'head') shapes.heads.push(field.id);
    }
  }
  return shapes;
}

function matchTemplate(template, line, heads) {
  const literals = [];
  const slots = [];
  let rest = template;
  for (let open = rest.indexOf('{'); open >= 0; open = rest.indexOf('{')) {
    literals.push(rest.slice(0, open));
    const after = rest.slice(open);
    const close = after.indexOf('}');
    if (close < 0) return null;
    slots.push(after.slice(1, close));
    rest = after.slice(close + 1);
  }
  literals.push(rest);
  const captures = [];
  let cursor = line;
  for (let index = 0; index < literals.length; index += 1) {
    const literal = literals[index];
    if (literal) {
      if (!cursor.startsWith(literal)) return null;
      cursor = cursor.slice(literal.length);
    }
    if (index >= slots.length) continue;
    const slot = slots[index];
    const nextLiteral = literals[index + 1] ?? '';
    const split = nextLiteral === '' ? cursor.length : cursor.indexOf(nextLiteral);
    if (split < 0) return null;
    const slotText = cursor.slice(0, split);
    cursor = cursor.slice(split);
    if (!validSlot(slot, slotText, heads)) return null;
    captures.push({ slot, value: slotText });
  }
  return captures;
}

function parseU32(value) {
  return /^\+?[0-9]+$/.test(value) && Number(value.replace('+', '')) <= 0xffffffff ? Number(value.replace('+', '')) : null;
}

function validSlot(slot, value, heads) {
  if (slot === 'file') return value.includes('.') || value.includes('/');
  if (slot === 'line' || slot === 'column') return parseU32(value) !== null;
  if (slot === 'head') return heads.includes(value);
  if (slot === 'code') return /^\p{Alphabetic}/u.test(value) && /\p{N}/u.test(value);
  return true;
}

function capture(captures, slot) {
  const found = captures.find((candidate) => candidate.slot === slot);
  return found && found.value !== '' ? found.value : null;
}

/** Mirrors `fn formalize_diagnostic`. @returns {Array<object>} */
export function formalizeDiagnostic(language, rawOutput) {
  const shapes = shapesFor(language);
  if (!shapes.patterns.length) return [];
  const diagnostics = [];
  let pending = null;
  for (const line of rustLines(rawOutput)) {
    const trimmed = trimStart(line);
    if (!trimmed) continue;
    let found = null;
    for (const template of shapes.locations) {
      found = matchTemplate(template, trimmed, shapes.heads);
      if (found) break;
    }
    if (found) {
      const file = capture(found, 'file');
      const lineValue = capture(found, 'line');
      const lineNumber = lineValue === null ? null : parseU32(lineValue);
      if (file !== null && lineNumber !== null) {
        const previous = diagnostics[diagnostics.length - 1];
        if (previous && previous.file === null) {
          previous.file = file;
          previous.line = lineNumber;
        } else {
          pending = [file, lineNumber];
        }
      }
      continue;
    }
    for (const template of shapes.patterns) {
      found = matchTemplate(template, trimmed, shapes.heads);
      if (found) break;
    }
    if (!found) continue;
    const message = capture(found, 'message') ?? '';
    if (!message) continue;
    // A pattern that names the location itself ("{file}:{line}: error")
    // binds it; otherwise a location line read before it does.
    const ownFile = capture(found, 'file');
    const ownLine = capture(found, 'line');
    const own = ownFile !== null && ownLine !== null && parseU32(ownLine) !== null ? [ownFile, parseU32(ownLine)] : null;
    const [file, lineNumber] = own ?? pending ?? [null, null];
    pending = null;
    diagnostics.push({ file, line: lineNumber, code: capture(found, 'code'), message, raw: trimEnd(line) });
  }
  return diagnostics;
}

function messageTokens(message) {
  return message.split(/[^\p{Alphabetic}\p{N}]/u).filter((token) => Array.from(token).length >= 2);
}

/** Mirrors `fn search_query`. */
export function searchQuery(language, diagnostic) {
  const tokens = [shapeLanguage(language)];
  if (diagnostic.code !== null) tokens.push(diagnostic.code);
  tokens.push(...messageTokens(diagnostic.message).slice(0, QUERY_MESSAGE_TOKENS));
  return tokens.join(' ');
}

function messageCoverage(text, diagnostic) {
  const page = text.toLowerCase();
  const tokens = messageTokens(diagnostic.message).map((token) => token.toLowerCase());
  if (!tokens.length) return 0;
  return Math.fround(tokens.filter((token) => page.includes(token)).length / tokens.length);
}

function pageAddresses(text, diagnostic) {
  if (diagnostic.code !== null && text.includes(diagnostic.code)) return true;
  return messageCoverage(text, diagnostic) > 0.5;
}

function fixFragment(page, diagnostic) {
  let best = null;
  let cursor = 0;
  for (;;) {
    const open = page.indexOf('```', cursor);
    if (open < 0) break;
    const newline = page.indexOf('\n', open + 3);
    const bodyStart = newline < 0 ? open + 3 : newline + 1;
    const close = page.indexOf('```', bodyStart);
    if (close < 0) break;
    const before = Array.from(page.slice(0, open));
    const window = before.slice(Math.max(0, before.length - 400)).join('');
    const code = diagnostic.code ?? '';
    const windowAddresses = (code !== '' && window.includes(code)) || messageCoverage(window, diagnostic) > 0.5;
    if (windowAddresses) {
      const score = messageCoverage(window, diagnostic);
      const candidate = trim(page.slice(bodyStart, close));
      if ((best === null || score > best[0]) && candidate) best = [score, candidate];
    }
    cursor = Math.min(close + 3, page.length);
    if (cursor >= page.length) break;
  }
  if (best) return best[1];
  let bestSentence = null;
  for (const sentence of page.split(/[.\n]/)) {
    const trimmed = trim(sentence);
    if (!trimmed || !pageAddresses(trimmed, diagnostic)) continue;
    const score = messageCoverage(trimmed, diagnostic);
    if (bestSentence === null || score > bestSentence[0]) bestSentence = [score, trimmed];
  }
  return bestSentence ? bestSentence[1] : null;
}

const linoEscape = (value) => value.replaceAll('\\', '\\\\').replaceAll('"', '\\"');

/** Mirrors `fn repair_document_path`. */
export function repairDocumentPath(artifactPath, diagnostic) {
  const dot = artifactPath.lastIndexOf('.');
  let stem = dot < 0 ? artifactPath : artifactPath.slice(0, dot);
  if (stem === '') stem = diagnostic.file ?? 'artifact';
  return `${stem}.repair.lino`;
}

/** Mirrors `fn repair_edit_document`. @param {string|null} fix */
export function repairEditDocument(language, diagnostic, fix, sourceUrl) {
  const quoted = (value) => `"${linoEscape(value)}"`;
  let out = '';
  out = pushLinoField(out, 0, 'repair_edit', null);
  out = pushLinoField(out, 2, 'language', shapeLanguage(language));
  if (diagnostic.file !== null) out = pushLinoField(out, 2, 'file', quoted(diagnostic.file));
  if (diagnostic.line !== null) out = pushLinoField(out, 2, 'line', String(diagnostic.line));
  if (diagnostic.code !== null) out = pushLinoField(out, 2, 'error_code', quoted(diagnostic.code));
  out = pushLinoField(out, 2, 'message', quoted(diagnostic.message));
  out = pushLinoField(out, 2, 'source', quoted(sourceUrl));
  out = pushLinoField(out, 2, 'fix', null);
  out = pushLinoField(out, 4, 'retained', fix === null ? 'none' : quoted(fix));
  out = pushLinoField(out, 2, 'rendering', 'meta_language');
  out = pushLinoField(out, 2, 'applied', 'false');
  return `${trimEnd(out)}\n`;
}

/** Mirrors `RepairAttempt::links_notation`. */
export function repairAttemptLinksNotation(attempt) {
  let out = 'repair_attempt\n';
  out += `  query "${linoEscape(attempt.search_query)}"\n`;
  if (attempt.diagnostic.code !== null) out += `  error_code "${linoEscape(attempt.diagnostic.code)}"\n`;
  out += `  candidate_fix ${attempt.candidate_fix !== null}\n`;
  out += `  applied ${attempt.applied}\n`;
  out += `  resolved ${attempt.resolved}\n`;
  return `${trimEnd(out)}\n`;
}

/** Mirrors `fn attempts_from`. */
export function attemptsFrom(messages, failure) {
  const primary = formalizeDiagnostic(failure.language, failure.reported)[0];
  if (!primary) return [];
  const progress = Progress.scan(messages);
  const addressing = progress.fetched_pages.find(([, text]) => pageAddresses(text, primary));
  const applied = Boolean(addressing)
    && progress.successfulWriteFor(repairDocumentPath(failure.artifact_path, primary));
  const resolved = failure.failed_command !== null && progress.successfulRunCountFor(failure.failed_command) > 0;
  return [{
    diagnostic: primary,
    search_query: searchQuery(failure.language, primary),
    candidate_fix: addressing ? fixFragment(addressing[1], primary) : null,
    applied,
    resolved,
  }];
}

/** Mirrors `fn evidence_document`. */
export function evidenceDocument(attempts) {
  let out = `repair_attempts\n  attempt_count ${attempts.length}\n`;
  for (const attempt of attempts) {
    for (const line of rustLines(repairAttemptLinksNotation(attempt))) out += `  ${line}\n`;
  }
  return `${trimEnd(out)}\n`;
}

/**
 * Mirrors `FailedStep::new` plus its builders.
 * @param {string} language
 * @param {string} reported
 * @param {{exit_code?: number|null, failed_command?: string|null, artifact_path?: string}} [rest]
 */
export function failedStep(language, reported, rest = {}) {
  return {
    language,
    reported,
    exit_code: rest.exit_code ?? null,
    failed_command: rest.failed_command ?? null,
    artifact_path: rest.artifact_path ?? '',
  };
}

function sourceUrls(text) {
  const seen = new Set();
  return urlsIn(text).filter((url) => !seen.has(url) && seen.add(url)).slice(0, MAX_REPAIR_SOURCES);
}

const stop = (reason) => ({ kind: 'stop', reason });

/** Mirrors `fn repair_step`. */
export function repairStep(messages, toolNames, failure, ladderRung, maxRungs) {
  if (ladderRung >= maxRungs) return stop('exhausted');
  const primary = formalizeDiagnostic(failure.language, failure.reported)[0];
  if (!primary) return stop('no_diagnostic');
  const progress = Progress.scan(messages);
  if (progress.search_output === null || progress.search_output === undefined) {
    const tool = toolFor(toolNames, Capability.Search);
    if (!tool) return stop('no_tools');
    return { kind: 'search', plan: planOne(tool, jsonText({ query: searchQuery(failure.language, primary) })) };
  }
  const fetchTool = toolFor(toolNames, Capability.Fetch);
  if (fetchTool) {
    const already = new Set(progress.attempted_fetches);
    const url = sourceUrls(progress.search_output ?? '').find((candidate) => !already.has(candidate));
    if (url !== undefined) return { kind: 'fetch', plan: planOne(fetchTool, fetchArguments(url)) };
  }
  const found = progress.fetched_pages.find(([, text]) => pageAddresses(text, primary));
  if (!found) return stop('no_match');
  const [url, page] = found;
  const documentPath = repairDocumentPath(failure.artifact_path, primary);
  if (!progress.successfulWriteFor(documentPath)) {
    const tool = toolFor(toolNames, Capability.Write);
    if (!tool) return stop('no_tools');
    const document = repairEditDocument(failure.language, primary, fixFragment(page, primary), url);
    return { kind: 'record_fix', plan: planOne(tool, writeArguments(documentPath, document)) };
  }
  if (failure.failed_command === null) return stop('no_match');
  const tool = toolFor(toolNames, Capability.Run);
  if (!tool) return stop('no_tools');
  return { kind: 'retry', plan: planOne(tool, jsonText({ command: failure.failed_command })) };
}

/** Mirrors `fn plan_repair`: the plan, or null. */
export function planRepair(messages, toolNames, failure, ladderRung, maxRungs) {
  const outcome = repairStep(messages, toolNames, failure, ladderRung, maxRungs);
  return outcome.kind === 'stop' ? null : outcome.plan;
}

function template(intent, language, values) {
  let out = localizedResponse(intent, language) ?? '';
  for (const [key, value] of values) out = out.split(`{${key}}`).join(value);
  return out;
}

/** Mirrors `fn ladder_note`. */
export function ladderNote(language, rung, maxRungs) {
  return template('repair_ladder_exhausted', language, [['rung', String(rung)], ['max_rungs', String(maxRungs)]]);
}

/** Mirrors `fn unresolved_note`. */
export function unresolvedNote(language, query) {
  return template('repair_unresolved_need', language, [['query', query]]);
}

