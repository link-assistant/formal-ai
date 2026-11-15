// Resolving the file names a work item mentions against the repository (issue #1558).
// Native twin: rust/src/agentic_coding/work_item_references.rs.
//
// A name in issue text is a reference, not a path: `hive-delivery-attempt.mjs`
// written as link text names `scripts/hive-delivery-attempt.mjs`, `Node.js`
// names a runtime, and a link into another repository names nothing the
// workspace holds. Each reference is resolved in this order: the exact path,
// then a unique match by file name through the search tool, then candidates
// ranked by how many words around the reference their directories share.
// Every step is derived from the transcript, so an action the client already
// answered is never planned again; a failed read moves the reference to the
// search, and a reference nothing resolves becomes an explicit unresolved item.

import { splitWhitespace } from './crate/rust_str.mjs';
import { commandArgument } from './tool_result.mjs';
import { sourceReadPath } from './progress.mjs';
import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { jsonText } from './plan.mjs';

/**
 * File references are evidence candidates, never shell commands or write authority.
 * With `dotSlash`, a leading `./` is dropped instead of rejecting the token.
 */
export function workItemEvidencePaths(task, dotSlash = false) {
  if (new TextEncoder().encode(task).length > 32768) return [];
  const paths = [];
  for (const word of splitWhitespace(task).flatMap(part => part.split(/[`'"()[\],;]+/u))) {
    let path = word.replace(/^[`'"(\[]+|[`'"),.;:\]]+$/gu, '');
    if (dotSlash && path.startsWith('./')) path = path.slice(2);
    if (!/^[A-Za-z0-9_-][A-Za-z0-9_./-]*\.(?:md|toml|json|rs|mjs|js|ts|yml|yaml|lino)$/u.test(path) || path.split('/').some(part => !part || part === '.' || part === '..') || paths.includes(path)) continue;
    paths.push(path);
    if (paths.length === 8) break;
  }
  return paths;
}

/** Names that are runtimes or products before they are files (`Node.js`). */
const PRODUCT_NAMES = ['Node.js', 'Deno.js', 'Bun.js', 'Next.js', 'Nuxt.js', 'Vue.js', 'React.js', 'Express.js', 'Three.js', 'D3.js', 'Ember.js', 'Backbone.js', 'Socket.io.js'];
/** At most this many ranked candidates are read for one reference. */
export const MAXIMUM_CANDIDATE_READS = 3;
const MAXIMUM_REFERENCES = 8;
const CONTEXT_WORDS = 8;
const BLOB_LINK = /https:\/\/github\.com\/([A-Za-z0-9_.-]+)\/([A-Za-z0-9_.-]+)\/blob\/[^/\s)\]>"'`]+\/([^\s)\]>"'`#?]+)/gu;

/** `owner/repository` of a GitHub issue or pull request URL, lowercased. */
export function workItemRepository(workItem) {
  const match = /^https:\/\/github\.com\/([A-Za-z0-9_.-]+)\/([A-Za-z0-9_.-]+)\/(?:issues|pull)\/\d+/u.exec(workItem ?? '');
  return match ? `${match[1]}/${match[2]}`.toLowerCase() : null;
}

const baseName = path => path.slice(path.lastIndexOf('/') + 1);
const validRelative = path => /^[A-Za-z0-9_-][A-Za-z0-9_./-]*$/u.test(path)
  && path.split('/').every(part => part && part !== '.' && part !== '..');

/** A capitalized `Name.js` with no directory is a product name before it is a file. */
export function productName(name) {
  return !name.includes('/') && (PRODUCT_NAMES.includes(name) || /^[A-Z][A-Za-z0-9]*\.js$/u.test(name));
}

/** Lowercase word set of the text around `offset`, for ranking same-named candidates. */
function contextWords(task, offset, length) {
  const before = splitWhitespace(task.slice(0, offset)).slice(-CONTEXT_WORDS);
  const after = splitWhitespace(task.slice(offset + length)).slice(0, CONTEXT_WORDS);
  const words = [];
  for (const word of [...before, ...after]) {
    for (const part of word.toLowerCase().split(/[^a-z0-9]+/u)) {
      if (part.length >= 3 && !words.includes(part)) words.push(part);
    }
  }
  return words;
}

/**
 * The file references of `task`, in order of first mention.
 * @returns {Array<{name: string, target: string, path: string|null, repository: string|null, external: boolean, context: string[]}>}
 */
export function workItemReferences(task, workItem = null) {
  const tokens = workItemEvidencePaths(task, true);
  if (!tokens.length) return [];
  const home = workItemRepository(workItem);
  const links = [];
  for (const match of task.matchAll(BLOB_LINK)) {
    let path;
    try { path = decodeURIComponent(match[3]); } catch { continue; }
    if (!validRelative(path)) continue;
    const repository = `${match[1]}/${match[2]}`.toLowerCase();
    links.push({ offset: match.index, length: match[0].length, path, repository });
  }
  const mentions = links.map(link => ({ offset: link.offset, length: link.length, name: link.path, link }));
  let cursor = 0;
  for (const token of tokens) {
    let offset = task.indexOf(token, cursor);
    if (offset < 0) offset = Math.max(task.indexOf(token), 0);
    else cursor = offset + token.length;
    mentions.push({ offset, length: token.length, name: token, link: null });
  }
  mentions.sort((left, right) => left.offset - right.offset);
  const references = [];
  for (const mention of mentions) {
    const name = mention.name;
    // Link text names the linked file: `[name.mjs](…/blob/…/dir/name.mjs)`.
    const link = mention.link ?? links.find(candidate => candidate.path === name || candidate.path.endsWith('/' + name)) ?? null;
    const target = link ? link.path : name;
    const repository = link ? link.repository : null;
    const known = references.find(reference => reference.repository === repository
      && (reference.target === target || reference.name === baseName(target) && (!target.includes('/') || !reference.target.includes('/'))));
    if (known) {
      if (target.includes('/') && !known.target.includes('/')) {
        known.target = target;
        if (!known.external) known.path = target;
      }
      continue;
    }
    if (references.length === MAXIMUM_REFERENCES) break;
    const external = repository !== null && home !== null && repository !== home;
    references.push({
      name: mention.link ? baseName(target) : name,
      target,
      path: external ? null : target,
      repository,
      external,
      context: contextWords(task, mention.offset, mention.length)
    });
  }
  return references;
}

/** Roots a relative path is joined with when the client binds it to an absolute path. */
export function workspaceRoots(workspace, hostRoot) {
  return [workspace, hostRoot].filter((root, index, roots) => typeof root === 'string' && root.startsWith('/') && roots.indexOf(root) === index);
}

const joinRoot = (root, path) => (root.endsWith('/') ? root + path : `${root}/${path}`);

/** Whether a read of `attempted` read the relative `path` (exactly, `./path`, or `<root>/path`). */
export function samePath(attempted, path, roots) {
  if (attempted === path || attempted === './' + path) return true;
  return roots.some(root => attempted === joinRoot(root, path));
}

/** A path from a search result, made relative to a workspace root when it lies under one. */
function relativeMatch(line, roots) {
  let path = line.trim();
  if (path.startsWith('./')) path = path.slice(2);
  for (const root of roots) {
    const prefix = root.endsWith('/') ? root : root + '/';
    if (path.startsWith(prefix)) return path.slice(prefix.length);
  }
  return path;
}

/** Search-result lines naming a file called like `reference`, outside dependency and VCS directories. */
export function searchMatches(output, reference, roots) {
  const name = baseName(reference.target);
  const matches = [];
  for (const line of output.split('\n')) {
    const path = relativeMatch(line, roots);
    if (!path || baseName(path) !== name || /(?:^|\/)(?:node_modules|\.git)\//u.test(path)) continue;
    if (!(validRelative(path) || path.startsWith('/') && path.split('/').slice(1).every(part => part && part !== '.' && part !== '..'))) continue;
    if (!matches.includes(path)) matches.push(path);
  }
  return matches;
}

/** Candidates ordered by suffix match, then words shared with the reference's context, then depth. */
export function rankedCandidates(matches, reference) {
  const score = path => {
    const suffix = reference.target.includes('/') && (path === reference.target || path.endsWith('/' + reference.target)) ? 1000 : 0;
    const directories = path.split('/').slice(0, -1).join('/').toLowerCase().split(/[^a-z0-9]+/u).filter(Boolean);
    return suffix + directories.filter((part, index) => directories.indexOf(part) === index && reference.context.includes(part)).length;
  };
  return matches
    .map(path => ({ path, score: score(path), depth: path.split('/').length }))
    .sort((left, right) => right.score - left.score || left.depth - right.depth || (left.path < right.path ? -1 : left.path > right.path ? 1 : 0))
    .map(row => row.path);
}

/** The search action for `reference`: the glob tool first, then a bounded `find` through the shell. */
export function searchActions(reference, tools) {
  const name = baseName(reference.target);
  const actions = [];
  const glob = toolFor(tools, Capability.Glob);
  if (glob !== null) actions.push({ tool: glob, capability: Capability.Glob, arguments: jsonText({ pattern: '**/' + name }) });
  const run = toolFor(tools, Capability.Run);
  if (run !== null) {
    actions.push({
      tool: run,
      capability: Capability.Run,
      arguments: jsonText({ command: `find . -type f -name '${name}' -not -path '*/node_modules/*' -not -path '*/.git/*' | head -n 100` })
    });
  }
  return actions;
}

/** The latest attempt answering `action`, compared by pattern or command, not by byte-identical arguments. */
export function attemptFor(progress, action) {
  const wanted = action.capability === Capability.Run ? commandArgument(action.arguments) : JSON.parse(action.arguments).pattern;
  for (let index = progress.attempts.length - 1; index >= 0; index -= 1) {
    const attempt = progress.attempts[index];
    if (attempt.capability !== action.capability || attempt.arguments === null) continue;
    let value = null;
    if (action.capability === Capability.Run) value = commandArgument(attempt.arguments);
    else {
      try { value = JSON.parse(attempt.arguments)?.pattern ?? null; } catch { value = null; }
    }
    if (value === wanted) return attempt;
  }
  return null;
}

/** The latest read attempt of `path` under any binding of its workspace roots. */
export function readAttemptFor(progress, path, roots) {
  for (let index = progress.attempts.length - 1; index >= 0; index -= 1) {
    const attempt = progress.attempts[index];
    if (attempt.capability !== Capability.Read || attempt.arguments === null) continue;
    const attempted = sourceReadPath(attempt.arguments);
    if (attempted !== null && samePath(attempted, path, roots)) return attempt;
  }
  return null;
}

/** Whether a read answered "no such file" (Agent CLI, POSIX tools, Node). */
export function notFoundRead(text) {
  return /^\s*(?:Error: )?(?:File not found|ENOENT|No such file or directory)\b/iu.test(text ?? '')
    || /^\s*(?:cat|head|sed): [^\n]*: No such file or directory/u.test(text ?? '');
}

/** `failed` when the read produced no source, `complete` or `incomplete` when it did. */
export function readOutcome(attempt) {
  const observed = attempt.source_read ?? null;
  if (!attempt.succeeded || observed === null || observed.error !== null || observed.absent) return 'failed';
  if (!observed.complete && notFoundRead(attempt.detail)) return 'failed';
  return observed.complete && typeof observed.source === 'string' ? 'complete' : 'incomplete';
}
