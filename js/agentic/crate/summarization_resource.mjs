// `crate::summarization::resource` (rust/src/summarization/resource.rs):
// recursive repository-resource formalization and summarization. It
// generalizes the file summarizer (summarization_file.mjs) from one file to any
// repository resource: a file or a directory tree of arbitrary depth, by the
// meta-algorithm loop - decompose into children, summarize each child
// (recursing into subdirectories), compose the child summaries behind an
// aggregate identity sentence. Recursion depth is bounded by the summarization
// mode ladder (`oneStepShorter`): a `Full` folder describes its children in
// `Standard`, theirs in `Short`, everything deeper as a `Topic` label.
//
// Everything here is a pure function of an input tree plus a config; no
// filesystem access happens in this module.
//
// Representation: a `RepositoryEntry` is `{kind: 'file', path, content}` or
// `{kind: 'directory', path, children}`; a formalized resource is
// `{kind: 'file', file}` or `{kind: 'directory', directory}` where `file` is a
// `RepositoryFileFormalization` and `directory` is a
// `RepositoryDirectoryFormalization`.

import { agenticMessage } from '../messages.mjs';
import {
  fileLinksNotation, fileSummary, formalizeRepositoryFile, pushField,
} from './summarization_file.mjs';
import { SummarizationMode, isLabelOnly, labelForMode, oneStepShorter, withMode } from './summarization.mjs';

/** Mirrors `RepositoryEntry::file` in rust/src/summarization/resource.rs. */
export function repositoryFile(path, content) {
  return { kind: 'file', path, content };
}

/** Mirrors `RepositoryEntry::directory` in rust/src/summarization/resource.rs. */
export function repositoryDirectory(path, children) {
  return { kind: 'directory', path, children };
}

/** Mirrors `RepositoryEntry::path` in rust/src/summarization/resource.rs. */
export function entryPath(entry) {
  return entry.path;
}

/** Mirrors `RepositoryResourceFormalization::path` in rust/src/summarization/resource.rs. */
export function resourcePath(resource) {
  return resource.kind === 'file' ? resource.file.path : resource.directory.path;
}

/** Mirrors `RepositoryResourceFormalization::is_directory` in rust/src/summarization/resource.rs. */
export function resourceIsDirectory(resource) {
  return resource.kind === 'directory';
}

/**
 * Mirrors `RepositoryResourceFormalization::summary` in
 * rust/src/summarization/resource.rs.
 * @param {object} resource a formalized resource
 * @param {object} config a `SummarizationConfig`
 */
export function resourceSummary(resource, config) {
  return resource.kind === 'file' ? fileSummary(resource.file, config) : directorySummary(resource.directory, config);
}

/**
 * Mirrors `RepositoryResourceFormalization::links_notation` in
 * rust/src/summarization/resource.rs.
 * @param {object} resource a formalized resource
 */
export function resourceLinksNotation(resource) {
  return resource.kind === 'file' ? fileLinksNotation(resource.file) : directoryLinksNotation(resource.directory);
}

/**
 * Mirrors `const fn pluralize` in rust/src/summarization/resource.rs.
 * @param {number} count
 * @param {string} singular
 * @param {string} plural
 * @returns {string}
 */
function pluralize(count, singular, plural) {
  return count === 1 ? singular : plural;
}

/**
 * Mirrors `fn child_summary_cap` in rust/src/summarization/resource.rs: how
 * many direct children a directory summary lists in prose, per mode.
 * @param {string} mode
 */
export function childSummaryCap(mode) {
  switch (mode) {
    case SummarizationMode.Identifier: case SummarizationMode.Topic: return 0;
    case SummarizationMode.Short: return 2;
    case SummarizationMode.Standard: return 4;
    default: return Number.POSITIVE_INFINITY;
  }
}

/** Mirrors `RepositoryDirectoryFormalization::identity_sentence` in rust/src/summarization/resource.rs. */
function identitySentence(directory) {
  return agenticMessage('summarization_directory_identity', {
    path: directory.path,
    direct_files: directory.direct_file_count,
    direct_files_noun: pluralize(directory.direct_file_count, 'file', 'files'),
    direct_directories: directory.direct_directory_count,
    direct_directories_noun: pluralize(directory.direct_directory_count, 'subdirectory', 'subdirectories'),
    lines: directory.total_line_count,
    lines_noun: pluralize(directory.total_line_count, 'line', 'lines'),
    files: directory.total_file_count,
    files_noun: pluralize(directory.total_file_count, 'file', 'files'),
  });
}

/**
 * Mirrors `RepositoryDirectoryFormalization::summary` in
 * rust/src/summarization/resource.rs: the decompose -> summarize -> compose
 * loop, children one mode shorter than their parent.
 * @param {object} directory a `RepositoryDirectoryFormalization`
 * @param {object} config a `SummarizationConfig`
 */
export function directorySummary(directory, config) {
  const identity = identitySentence(directory);
  if (isLabelOnly(config.mode)) return labelForMode(config.mode, identity);
  const parts = [identity];
  const childConfig = withMode(config, oneStepShorter(config.mode));
  const cap = childSummaryCap(config.mode);
  const childSummaries = [];
  for (const child of directory.children.slice(0, cap)) {
    const summary = resourceSummary(child, childConfig);
    if (summary !== '') childSummaries.push(summary);
  }
  const hidden = Math.max(0, directory.children.length - cap);
  if (childSummaries.length > 0) parts.push(`Contents: ${childSummaries.join(' ')}`);
  if (hidden > 0) parts.push(agenticMessage('summarization_omitted_entries', { hidden, noun: pluralize(hidden, 'entry', 'entries') }));
  return parts.join(' ');
}

/**
 * Mirrors `RepositoryDirectoryFormalization::links_notation` in
 * rust/src/summarization/resource.rs: a `repository_directory` block of paths,
 * counts and per-child kind.
 * @param {object} directory a `RepositoryDirectoryFormalization`
 */
export function directoryLinksNotation(directory) {
  let out = 'repository_directory\n';
  out += pushField(1, 'path', directory.path);
  out += pushField(1, 'direct_file_count', String(directory.direct_file_count));
  out += pushField(1, 'direct_directory_count', String(directory.direct_directory_count));
  out += pushField(1, 'total_file_count', String(directory.total_file_count));
  out += pushField(1, 'total_directory_count', String(directory.total_directory_count));
  out += pushField(1, 'total_line_count', String(directory.total_line_count));
  out += pushField(1, 'total_byte_count', String(directory.total_byte_count));
  for (const child of directory.children) {
    if (child.kind === 'file') out += pushField(1, 'file', child.file.path);
    else out += pushField(1, 'directory', child.directory.path);
  }
  return out.replace(/\p{White_Space}+$/u, '');
}

/**
 * Mirrors `fn formalize_repository_resource` in
 * rust/src/summarization/resource.rs: a file or directory formalized recursively.
 * @param {object} entry a `RepositoryEntry`
 */
export function formalizeRepositoryResource(entry) {
  if (entry.kind === 'file') return { kind: 'file', file: formalizeRepositoryFile(entry.path, entry.content) };
  return { kind: 'directory', directory: formalizeRepositoryDirectory(entry.path, entry.children) };
}

/**
 * Mirrors `fn formalize_repository_directory` in
 * rust/src/summarization/resource.rs: the ordered children formalized and the
 * recursive file, directory, line and byte aggregates summed.
 * @param {string} path
 * @param {Array<object>} children `RepositoryEntry` values
 */
export function formalizeRepositoryDirectory(path, children) {
  const formalizedChildren = children.map(formalizeRepositoryResource);
  let directFileCount = 0;
  let directDirectoryCount = 0;
  let totalFileCount = 0;
  let totalDirectoryCount = 0;
  let totalLineCount = 0;
  let totalByteCount = 0;
  for (const child of formalizedChildren) {
    if (child.kind === 'file') {
      directFileCount += 1;
      totalFileCount += 1;
      totalLineCount += child.file.line_count;
      totalByteCount += child.file.byte_count;
    } else {
      const directory = child.directory;
      directDirectoryCount += 1;
      totalDirectoryCount += 1 + directory.total_directory_count;
      totalFileCount += directory.total_file_count;
      totalLineCount += directory.total_line_count;
      totalByteCount += directory.total_byte_count;
    }
  }
  return {
    path,
    direct_file_count: directFileCount,
    direct_directory_count: directDirectoryCount,
    total_file_count: totalFileCount,
    total_directory_count: totalDirectoryCount,
    total_line_count: totalLineCount,
    total_byte_count: totalByteCount,
    children: formalizedChildren,
  };
}

/**
 * Mirrors `fn summarize_repository_resource` in
 * rust/src/summarization/resource.rs: the general entry point that subsumes
 * `summarizeRepositoryFile`.
 * @param {object} entry a `RepositoryEntry`
 * @param {object} config a `SummarizationConfig`
 */
export function summarizeRepositoryResource(entry, config) {
  return resourceSummary(formalizeRepositoryResource(entry), config);
}
