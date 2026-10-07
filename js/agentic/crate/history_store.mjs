// The subprocess and storage half of repository history: a port of
// rust/src/history_context/cursor.rs and the `git`-running parts of
// rust/src/history_context/commits.rs (issue #1180 R11).
//
// Like the derivation store (derivation.mjs), everything outside the process
// comes through an injected `io`, so this file never imports `node:*` and the
// planner stays host-agnostic:
//
//   { runGit(repoRoot, args) -> string   (throws when git fails),
//     readText(path) -> string|null      (null when the file is missing),
//     writeText(path, text), createDirAll(path),
//     listFiles(dir) -> [{name, text}]   (regular files, any order) }
//
// `nodeHistoryIo` builds that object from the `node:fs`, `node:path` and
// `node:child_process` modules a node caller passes in. The browser worker has
// no repository to run git in and passes none.
//
// One residual difference from the native importer: the per-path symbol diff
// carries the named top-level items (`diffNamedItems`, both censuses) but not
// the tree-sitter node-kind histogram nor the ES token count, which need the
// meta-language parse the JavaScript roots do not have.

import { diffNamedItems, formalizeCommit, parseLogOutput } from './history_context.mjs';
import { importCiRuns, importIssuesAndPullsWithWatermark } from './history_github.mjs';
import { MemoryStore, escapeValue, memoryEvent } from './memory.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

/** Mirrors `const CURSOR_ROOT`. */
export const CURSOR_ROOT = 'repository_history_cursor';
/** Mirrors `const LOG_FORMAT`. */
export const LOG_FORMAT = '\u001e%H\u001f%an\u001f%cI\u001f%s\u001f%b\u001f';
/** Mirrors the fallback slug of `repository_slug`. */
export const LOCAL_REPOSITORY = 'local-repository';

/**
 * An `io` over node's own modules.
 * @param {{fs: object, path: object, childProcess: object}} modules
 */
export function nodeHistoryIo({ fs, path, childProcess }) {
  return {
    runGit(repoRoot, args) {
      return childProcess.execFileSync('git', args, {
        cwd: repoRoot,
        encoding: 'utf8',
        maxBuffer: 1 << 30,
        stdio: ['ignore', 'pipe', 'pipe'],
      });
    },
    readText(file) {
      return fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : null;
    },
    writeText(file, text) {
      fs.writeFileSync(file, text);
    },
    createDirAll(dir) {
      fs.mkdirSync(dir, { recursive: true });
    },
    listFiles(dir) {
      return fs
        .readdirSync(dir, { withFileTypes: true })
        .filter((entry) => !entry.isDirectory())
        .map((entry) => ({ name: entry.name, text: fs.readFileSync(path.join(dir, entry.name), 'utf8') }));
    },
    join: (...parts) => path.join(...parts),
    dirname: (file) => path.dirname(file),
  };
}

const join = (io, ...parts) => (io.join ? io.join(...parts) : parts.join('/'));
const dirname = (io, file) => (io.dirname ? io.dirname(file) : file.slice(0, Math.max(0, file.lastIndexOf('/'))));

/** `run_git` with failures read as absence, as `blob_at` and `merge_subject_for` do. */
function gitOrNull(io, repoRoot, args) {
  try {
    return io.runGit(repoRoot, args);
  } catch {
    return null;
  }
}

/** Mirrors `merge_subject_for`: the oldest merge that delivered `sha`. */
export function mergeSubjectFor(io, repoRoot, sha) {
  const out = gitOrNull(io, repoRoot, ['log', '--merges', '--ancestry-path', '--format=%s', `${sha}..HEAD`]);
  if (out === null) return null;
  const lines = out
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  return lines.length > 0 ? lines[lines.length - 1] : null;
}

/** Mirrors `blob_at`. */
function blobAt(io, repoRoot, rev, file) {
  return gitOrNull(io, repoRoot, ['show', `${rev}:${file}`]);
}

/** Mirrors `diff_symbols` over the named-item half (see the header). */
export function diffSymbols(io, repoRoot, sha, file, rules) {
  const name = file.toLowerCase();
  const census = rules.censusSuffixes.some((suffix) => name.endsWith(suffix))
    ? 'ast_census'
    : rules.esSuffixes.some((suffix) => name.endsWith(suffix))
      ? 'es_meta_extract'
      : null;
  if (census === null) return [];
  const before = blobAt(io, repoRoot, `${sha}^`, file) ?? '';
  const after = blobAt(io, repoRoot, sha, file) ?? '';
  return diffNamedItems(file, census, before, after, rules);
}

/** Mirrors `import_commits_impl`. */
function importCommitsImpl(io, repoRoot, sinceSha, file, rules) {
  const args = ['log', '--no-show-signature', `--format=${LOG_FORMAT}`, '--name-only'];
  if (file !== null) args.push('--follow');
  args.push(sinceSha ? `${sinceSha}..HEAD` : 'HEAD');
  if (file !== null) args.push('--', file);
  return parseLogOutput(io.runGit(repoRoot, args)).map((raw) => {
    const symbols = raw.changedPaths.flatMap((changed) => diffSymbols(io, repoRoot, raw.sha, changed, rules));
    return formalizeCommit(raw, symbols, mergeSubjectFor(io, repoRoot, raw.sha), rules);
  });
}

/** Mirrors `import_commits`: commits newer than `sinceSha`, chronological. */
export function importCommits(io, repoRoot, sinceSha, rules) {
  return importCommitsImpl(io, repoRoot, sinceSha, null, rules);
}

/** Mirrors `import_commits_for_path`: one path's `git log --follow`. */
export function importCommitsForPath(io, repoRoot, sinceSha, file, rules) {
  return importCommitsImpl(io, repoRoot, sinceSha, file, rules);
}

/** A history event (camelCase, as the importers return it) as a stored `MemoryEvent`. */
export function toMemoryEvent(event) {
  return memoryEvent({
    id: event.id,
    kind: event.kind ?? null,
    role: event.role ?? null,
    intent: event.intent ?? null,
    content: event.content ?? null,
    sent_at: event.sentAt ?? null,
    conversation_id: event.conversationId ?? null,
    evidence: [...(event.evidence || [])],
  });
}

/**
 * Mirrors `write_repository_history`: append the events whose ids the store
 * does not hold yet, and return how many were appended.
 */
export function writeRepositoryHistory(io, events, storePath) {
  const store = new MemoryStore();
  const text = io.readText(storePath);
  if (text !== null) store.replaceFromLinksNotation(text);
  const existing = new Set(store.events().map((event) => event.id));
  let appended = 0;
  for (const event of events) {
    if (existing.has(event.id)) continue;
    store.events().push(toMemoryEvent(event));
    appended += 1;
  }
  if (io.createDirAll) io.createDirAll(dirname(io, storePath));
  io.writeText(storePath, store.exportLinksNotation());
  return appended;
}

/** Mirrors `RepositoryHistoryCursor::read`; a missing file is a fresh start. */
export function readCursor(io, cursorPath) {
  const fresh = { lastCommitSha: null, lastIssuePrUpdatedAt: null, lastCiRunDatabaseId: null };
  const text = io.readText(cursorPath);
  if (text === null) return fresh;
  const node = parseRoot(text).children.find((child) => child.name === CURSOR_ROOT);
  if (!node) return fresh;
  const optional = (name) => findChildValue(node, name) || null;
  const run = findChildValue(node, 'last_ci_run_database_id');
  return {
    lastCommitSha: optional('last_commit_sha'),
    lastIssuePrUpdatedAt: optional('last_issue_pr_updated_at'),
    lastCiRunDatabaseId: /^[0-9]+$/.test(run) ? Number(run) : null,
  };
}

/** Mirrors `RepositoryHistoryCursor::write`. */
export function cursorText(cursor) {
  const row = (name, value) => `  ${name} "${escapeValue(String(value))}"\n`;
  let out = `${CURSOR_ROOT}\n`;
  if (cursor.lastCommitSha) out += row('last_commit_sha', cursor.lastCommitSha);
  if (cursor.lastIssuePrUpdatedAt) out += row('last_issue_pr_updated_at', cursor.lastIssuePrUpdatedAt);
  if (cursor.lastCiRunDatabaseId !== null && cursor.lastCiRunDatabaseId !== undefined) {
    out += row('last_ci_run_database_id', cursor.lastCiRunDatabaseId);
  }
  return out;
}

/** Writes the cursor document, creating parent directories. */
export function writeCursor(io, cursor, cursorPath) {
  if (io.createDirAll) io.createDirAll(dirname(io, cursorPath));
  io.writeText(cursorPath, cursorText(cursor));
}

/**
 * Mirrors `RepositoryHistoryCursor::advance`: the last commit wins and run
 * ids take the maximum.
 */
export function advanceCursor(cursor, events, rules) {
  const recordFor = (kind) => rules.records.find((record) => record.kind === kind);
  const commit = recordFor('commit');
  const ci = recordFor('ci_run');
  for (const event of events) {
    if (commit && event.kind === commit.kind && event.id.startsWith(commit.idPrefix)) {
      cursor.lastCommitSha = event.id.slice(commit.idPrefix.length);
    } else if (ci && event.kind === ci.kind && event.id.startsWith(ci.idPrefix)) {
      const number = event.id.slice(ci.idPrefix.length);
      if (/^[0-9]+$/.test(number)) {
        const value = Number(number);
        cursor.lastCiRunDatabaseId =
          cursor.lastCiRunDatabaseId === null ? value : Math.max(cursor.lastCiRunDatabaseId, value);
      }
    }
  }
  return cursor;
}

/** Mirrors `repository_slug`: `owner-name` from the origin remote. */
export function repositorySlug(io, repoRoot) {
  const url = (gitOrNull(io, repoRoot, ['config', '--get', 'remote.origin.url']) ?? '').trim();
  if (url.length === 0) return LOCAL_REPOSITORY;
  const withoutGit = url.endsWith('.git') ? url.slice(0, -4) : url;
  const segments = withoutGit.split(/[/:]/).filter((segment) => segment.length > 0);
  return segments.length >= 2
    ? `${segments[segments.length - 2]}-${segments[segments.length - 1]}`
    : LOCAL_REPOSITORY;
}

/** Mirrors `store_paths`: the events store and the cursor of one repository. */
export function storePaths(io, memoryDir, repoRoot) {
  const base = join(io, memoryDir, 'repository-history', repositorySlug(io, repoRoot));
  return { storePath: join(io, base, 'events.lino'), cursorPath: join(io, base, 'cursor.lino') };
}

/**
 * Mirrors `import_incremental`: commits since the cursor, then (with a
 * github-logs directory) issues, pull requests, reviews and Actions runs,
 * appended to the store. Returns the number of events appended.
 */
export function importIncremental(io, repoRoot, logsDir, memoryDir, rules) {
  const { storePath, cursorPath } = storePaths(io, memoryDir, repoRoot);
  const cursor = readCursor(io, cursorPath);
  const events = importCommits(io, repoRoot, cursor.lastCommitSha, rules);
  let issueWatermark = cursor.lastIssuePrUpdatedAt;
  if (logsDir !== null && logsDir !== undefined) {
    const files = io.listFiles(logsDir);
    const imported = importIssuesAndPullsWithWatermark(files, cursor.lastIssuePrUpdatedAt, rules);
    events.push(...imported.events);
    if (imported.watermark !== null) issueWatermark = imported.watermark;
    events.push(...importCiRuns(files, cursor.lastCiRunDatabaseId, rules));
  }
  const appended = writeRepositoryHistory(io, events, storePath);
  advanceCursor(cursor, events, rules);
  cursor.lastIssuePrUpdatedAt = issueWatermark;
  writeCursor(io, cursor, cursorPath);
  return appended;
}
