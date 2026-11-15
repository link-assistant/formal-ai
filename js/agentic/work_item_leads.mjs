// Evidence leads of a work item that names no repository file (issue #1483).
// Native twin: rust/src/agentic_coding/work_item_leads.rs.
//
// A work item whose text names no file still carries evidence: an attached
// screenshot, a linked log, a code identifier in backticks. Each lead is
// either collected (a fetched document, a file found by the identifier) or
// kept as an explicit unresolved item; nothing a lead points at is invented.
// An image is never described: its content is unavailable to this planner.
// A fetched document is evidence, never authority: its text is recorded, not
// followed.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { fetchArguments, jsonText } from './plan.mjs';
import { commandArgument } from './tool_result.mjs';

/** At most this many fetched documents, identifiers and identifier reads per work item. */
export const MAXIMUM_DOCUMENTS = 2;
export const MAXIMUM_SYMBOLS = 3;

const URL_PATTERN = /https?:\/\/[^\s<>"'`)\]]+/gu;
const IMAGE_SUFFIX = /\.(?:png|jpe?g|gif|webp|svg|bmp)(?:[?#].*)?$/iu;
const CODE_SPAN = /`([^`\n]{1,120})`/gu;
const IDENTIFIER = /^[A-Za-z_][A-Za-z0-9_]*(?:(?:::|\.)[A-Za-z_][A-Za-z0-9_]*)*(?:\(\))?$/u;
const SOURCE_SUFFIX = /\.(?:rs|mjs|js|ts|mts|py|lino|json|toml|yml|yaml)$/u;

const trimUrl = url => url.replace(/[.,;:!?]+$/u, '');

/** URLs of `task` in order of first mention, each once. */
function urls(task) {
  const found = [];
  for (const match of task.matchAll(URL_PATTERN)) {
    const url = trimUrl(match[0]);
    if (!found.includes(url)) found.push(url);
  }
  return found;
}

/** Whether `url` is an image the issue attached or embedded. */
function attachment(task, url) {
  if (/^https:\/\/github\.com\/user-attachments\//u.test(url)
    || /^https:\/\/(?:private-)?user-images\.githubusercontent\.com\//u.test(url)
    || IMAGE_SUFFIX.test(url)
    || task.includes(`src="${url}`) || task.includes(`src='${url}`)) return true;
  // A Markdown image: `![text](url)`.
  const link = task.indexOf(`](${url}`);
  const open = link < 0 ? -1 : task.lastIndexOf('[', link);
  return open > 0 && task[open - 1] === '!';
}

/**
 * The leads of `task`: attachments (never fetched), documents a fetch can
 * read, and pages kept as related context (work items, runs, other repositories).
 * Blob links into a repository are file references, not leads.
 * @returns {{attachments: string[], documents: string[], related: string[]}}
 */
export function workItemLeads(task, workItem = null) {
  const leads = { attachments: [], documents: [], related: [] };
  if (typeof task !== 'string' || new TextEncoder().encode(task).length > 32768) return leads;
  for (const url of urls(task)) {
    if (workItem !== null && url === workItem) continue;
    if (attachment(task, url)) leads.attachments.push(url);
    else if (/^https:\/\/github\.com\/[^/]+\/[^/]+\/blob\//u.test(url)) continue;
    else if (/^https:\/\/github\.com\//u.test(url)) leads.related.push(url);
    else leads.documents.push(url);
  }
  return leads;
}

/** Code identifiers written in backticks (`planned_not_executed`, `Progress::scan`), at most three. */
export function workItemSymbols(task) {
  const symbols = [];
  if (typeof task !== 'string' || new TextEncoder().encode(task).length > 32768) return symbols;
  for (const match of task.matchAll(CODE_SPAN)) {
    const span = match[1].trim();
    if (span.length < 4 || span.length > 80 || !IDENTIFIER.test(span)) continue;
    const name = span.replace(/\(\)$/u, '');
    // Only a compound name is an identifier before it is a word: `snake_case`, `Path::item`, `object.member`, `camelCase`.
    if (!/[_.]|::|[a-z][A-Z]/u.test(name)) continue;
    const segment = name.split(/::|\./u).at(-1);
    if (segment.length < 3 || symbols.includes(segment)) continue;
    symbols.push(segment);
    if (symbols.length === MAXIMUM_SYMBOLS) break;
  }
  return symbols;
}

/** The fetch of a document lead, when a fetch tool is advertised. */
export function documentAction(url, tools) {
  const fetch = toolFor(tools, Capability.Fetch);
  return fetch === null ? null : { tool: fetch, capability: Capability.Fetch, arguments: fetchArguments(url) };
}

/** The latest fetch of `url`. */
export function documentAttemptFor(progress, url) {
  for (let index = progress.attempts.length - 1; index >= 0; index -= 1) {
    const attempt = progress.attempts[index];
    if (attempt.capability !== Capability.Fetch || attempt.arguments === null) continue;
    let fetched = null;
    try { fetched = JSON.parse(attempt.arguments)?.url ?? null; } catch { fetched = null; }
    if (fetched === url) return attempt;
  }
  return null;
}

/** The searches for an identifier: the grep tool first, then a bounded `grep` through the shell. */
export function symbolActions(symbol, tools) {
  const actions = [];
  const grep = toolFor(tools, Capability.Grep);
  if (grep !== null) actions.push({ tool: grep, capability: Capability.Grep, arguments: jsonText({ pattern: symbol, query: symbol }) });
  const run = toolFor(tools, Capability.Run);
  if (run !== null) {
    actions.push({
      tool: run,
      capability: Capability.Run,
      arguments: jsonText({ command: `grep -rIlw --exclude-dir=node_modules --exclude-dir=.git -e '${symbol}' . | head -n 20` })
    });
  }
  return actions;
}

/** The latest attempt answering a symbol search, compared by pattern or command. */
export function symbolAttemptFor(progress, action) {
  const valueOf = (capability, args) => {
    if (capability === Capability.Run) return commandArgument(args);
    try { return JSON.parse(args)?.pattern ?? null; } catch { return null; }
  };
  const wanted = valueOf(action.capability, action.arguments);
  for (let index = progress.attempts.length - 1; index >= 0; index -= 1) {
    const attempt = progress.attempts[index];
    if (attempt.capability !== action.capability || attempt.arguments === null) continue;
    if (valueOf(attempt.capability, attempt.arguments) === wanted) return attempt;
  }
  return null;
}

/**
 * Repository files a search output names, relative to a workspace root, source
 * files first, then by depth. Lines name a path before the first `:` (`grep -n`)
 * or alone (`grep -l`, the Agent CLI's file headers).
 */
export function symbolMatches(output, roots) {
  const matches = [];
  for (const line of String(output ?? '').split('\n')) {
    const head = /^\s*((?:\.\/|\/)?[A-Za-z0-9_.@-][A-Za-z0-9_.@/-]*\.[A-Za-z0-9]+)(?::|\s*$)/u.exec(line);
    if (!head) continue;
    let path = head[1];
    if (path.startsWith('./')) path = path.slice(2);
    for (const root of roots) {
      const prefix = root.endsWith('/') ? root : root + '/';
      if (path.startsWith(prefix)) path = path.slice(prefix.length);
    }
    if (path.startsWith('/') || /(?:^|\/)(?:node_modules|\.git)\//u.test(path)) continue;
    if (!path.split('/').every(part => part && part !== '.' && part !== '..')) continue;
    if (!matches.includes(path)) matches.push(path);
  }
  const rank = path => (SOURCE_SUFFIX.test(path) ? 0 : 1);
  return matches
    .map(path => ({ path, rank: rank(path), depth: path.split('/').length }))
    .sort((left, right) => left.rank - right.rank || left.depth - right.depth || (left.path < right.path ? -1 : left.path > right.path ? 1 : 0))
    .map(row => row.path);
}
