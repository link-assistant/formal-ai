#!/usr/bin/env node

/**
 * Rebuild CHANGELOG.md from the release trees in Git history.
 *
 * The release collector accidentally retained fragments, so the existing
 * changelog cannot be used as a release-to-fragment map. A fragment belongs to
 * the first release tree in which it appears. The initial import is the one
 * exception: its fragments already belonged to releases 0.2.0 through 0.11.0,
 * so their original sections are matched by exact fragment body.
 *
 * CHANGELOG.md keeps only the newest releases. No maintained file may exceed
 * 1500 lines, so older releases roll into `docs/changelog/archive-NN.md`. The
 * archive is packed oldest first, so a full archive file never changes again:
 * each release only moves the oldest sections of CHANGELOG.md into the newest
 * archive file, which opens a new one once it is full. CHANGELOG.md links every
 * archive file above the insert marker.
 *
 * Usage:
 *   node experiments/issue_711_rebuild_changelog.mjs --write
 *   node experiments/issue_711_rebuild_changelog.mjs --write --pending-release 1.2.3 --pending-date 2026-07-17
 *   node experiments/issue_711_rebuild_changelog.mjs --check
 *   node experiments/issue_711_rebuild_changelog.mjs --ref origin/main
 */

import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { posix } from "node:path";
import { pathToFileURL } from "node:url";

const INITIAL_COMMIT = "6f8d4a8a05770adfd2fe33fdf3c6c586efb103af";
const CHANGELOG_PATH = "CHANGELOG.md";
const MAP_PATH = "docs/case-studies/issue-711/fragment-release-map.tsv";
export const ARCHIVE_DIR = "docs/changelog";
const INSERT_MARKER = "<!-- changelog-insert-here -->";
const INTRO = `# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).`;
// Lines of release sections CHANGELOG.md keeps; the newest release is always
// kept whatever its size. The header and the archive list stay well inside
// the remaining headroom under the 1400-line warning of check-file-size.
export const RECENT_SECTION_LINES = 1200;
// The most lines one archive file holds, header included.
export const ARCHIVE_LINE_LIMIT = 1400;
const ARCHIVE_HEADER_LINES = 4;

function argument(name, fallback = undefined) {
  const index = process.argv.indexOf(`--${name}`);
  return index === -1 ? fallback : process.argv[index + 1];
}

function git(args, options = {}) {
  return execFileSync("git", args, {
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    ...options,
  });
}

function treeFiles(commit) {
  return git(["ls-tree", "-r", "--name-only", commit, "--", "changelog.d"])
    .trim()
    .split("\n")
    .filter((path) => path.endsWith(".md") && !path.endsWith("/README.md"))
    .sort();
}

function releaseTreeFiles(commit) {
  // Current releases consume their fragments in the release commit. Read both
  // sides of that transition so reconstruction continues to work for releases
  // made after fragment cleanup was introduced, while preserving historical
  // releases whose fragments remained in the committed tree.
  const before = treeFiles(`${commit}^`);
  const after = treeFiles(commit);
  const afterSet = new Set(after);
  return [...new Set([...before, ...after])].sort().map((path) => ({
    path,
    source: afterSet.has(path) ? commit : `${commit}^`,
  }));
}

function fileAt(commit, path) {
  return git(["show", `${commit}:${path}`]);
}

function stripFrontmatter(content) {
  const match = content.match(/^---\s*\n.*?\n---\s*\n([\s\S]*)$/);
  return (match ? match[1] : content).trim();
}

function versionParts(version) {
  return version.split(".").map(Number);
}

function compareVersions(left, right) {
  const a = versionParts(left);
  const b = versionParts(right);
  return a[0] - b[0] || a[1] - b[1] || a[2] - b[2];
}

function initialSections() {
  const original = fileAt(INITIAL_COMMIT, CHANGELOG_PATH);
  const headings = [...original.matchAll(/^## \[([^\]]+)] - (\d{4}-[0-9X]{2}-[0-9X]{2})$/gm)];
  return headings.map((heading, index) => ({
    version: heading[1],
    date: heading[2],
    body: original
      .slice(heading.index + heading[0].length, headings[index + 1]?.index ?? original.length)
      .trim(),
  }));
}

function releaseCommits(ref) {
  const releases = git(["log", "--format=%H%x09%cI%x09%s", ref])
    .trim()
    .split("\n")
    .map((line) => line.split("\t"))
    .filter(([, , subject]) => /^chore: release v\d+\.\d+\.\d+/.test(subject))
    .map(([commit, committedAt, subject]) => ({
      commit,
      date: committedAt.slice(0, 10),
      version: subject.match(/^chore: release v(\d+\.\d+\.\d+)/)[1],
    }));

  const byVersion = new Map(releases.map((release) => [release.version, release]));
  return [...byVersion.values()].sort((a, b) => compareVersions(a.version, b.version));
}

function reconstruct(ref) {
  const initial = initialSections();
  const groups = new Map();
  const assignments = [];
  const seen = new Set();

  // 0.1.0 predates the fragment system and is preserved verbatim.
  const baseline = initial.find(({ version }) => version === "0.1.0");
  if (!baseline) throw new Error("Initial changelog has no 0.1.0 baseline");
  groups.set(baseline.version, { ...baseline, fragments: null });

  const chronologicalInitial = initial
    .filter(({ version }) => version !== "0.1.0")
    .sort((a, b) => compareVersions(a.version, b.version));

  for (const path of treeFiles(INITIAL_COMMIT)) {
    const body = stripFrontmatter(fileAt(INITIAL_COMMIT, path));
    const section = chronologicalInitial.find(({ body: sectionBody }) =>
      sectionBody.includes(body),
    );
    if (!section) throw new Error(`Cannot map initial fragment ${path}`);
    const group = groups.get(section.version) ?? {
      version: section.version,
      date: section.date,
      fragments: [],
    };
    group.fragments.push({ path, body, commit: INITIAL_COMMIT });
    groups.set(section.version, group);
    assignments.push({ path, version: section.version, commit: INITIAL_COMMIT });
    seen.add(path);
  }

  for (const release of releaseCommits(ref)) {
    const fragments = [];
    for (const { path, source } of releaseTreeFiles(release.commit)) {
      if (seen.has(path)) continue;
      fragments.push({ path, body: stripFrontmatter(fileAt(source, path)), commit: release.commit });
      assignments.push({ path, version: release.version, commit: release.commit });
      seen.add(path);
    }
    if (fragments.length > 0) groups.set(release.version, { ...release, fragments });
  }

  if (assignments.length === 0) {
    throw new Error(`No released fragments found in ${ref}`);
  }
  return { assignments, groups };
}

export function addPendingRelease(result, release) {
  const seen = new Set(result.assignments.map(({ path }) => path));
  const fragments = release.fragments
    .filter(({ path }) => !seen.has(path))
    .sort((a, b) => a.path.localeCompare(b.path));
  if (fragments.length === 0) {
    throw new Error(`No pending fragments found for ${release.version}`);
  }
  if (result.groups.has(release.version)) {
    throw new Error(`Release ${release.version} already exists in reconstructed history`);
  }
  result.groups.set(release.version, { ...release, fragments });
  for (const { path } of fragments) {
    result.assignments.push({ path, version: release.version });
  }
  return result;
}

function deletedFragments(ref) {
  const output = git([
    "diff", "--diff-filter=D", "--name-only", ref, "--", "changelog.d",
  ]).trim();
  if (!output) return [];
  return output
    .split("\n")
    .filter((path) => path.endsWith(".md") && !path.endsWith("/README.md"))
    .sort()
    .map((path) => ({ path, body: stripFrontmatter(fileAt(ref, path)) }));
}

function lineCount(text) {
  return text.split("\n").length;
}

function isDocumentRelative(target) {
  return !(
    target === "" ||
    target.startsWith("#") ||
    target.startsWith("/") ||
    target.includes("://") ||
    target.startsWith("mailto:")
  );
}

// A root-relative link target as a document in the directory `base` writes it.
export function relativeTo(base, target) {
  if (!isDocumentRelative(target)) return target;
  const baseParts = base.split("/");
  const parts = posix.normalize(target).split("/");
  let shared = 0;
  while (
    shared < baseParts.length &&
    shared < parts.length - 1 &&
    baseParts[shared] === parts[shared]
  ) {
    shared += 1;
  }
  return [...Array(baseParts.length - shared).fill(".."), ...parts.slice(shared)].join("/");
}

// Rebase every relative path link in a release section written for the
// repository root. Only plain path targets are touched, so a `](` inside code
// or prose that is not a link target is left alone.
export function rebaseLinks(body, base) {
  return body.replace(/\]\(([A-Za-z0-9_.\-/]+(?:#[^)\s]*)?)\)/g, (match, target) =>
    isDocumentRelative(target) ? `](${relativeTo(base, target)})` : match,
  );
}

function sectionText(group) {
  const body = group.fragments === null
    ? group.body
    : group.fragments.sort((a, b) => a.path.localeCompare(b.path)).map(({ body }) => body).join("\n\n");
  return `## [${group.version}] - ${group.date}\n\n${body}`;
}

// Split sections (newest first) into the ones CHANGELOG.md keeps and the
// archive files, packed oldest first so a full archive file is never rewritten.
export function splitHistory(sections) {
  let recentLines = 0;
  let recentCount = 0;
  for (const section of sections) {
    const lines = lineCount(section.text) + (recentCount > 0 ? 1 : 0);
    if (recentCount > 0 && recentLines + lines > RECENT_SECTION_LINES) break;
    recentLines += lines;
    recentCount += 1;
  }
  const recent = sections.slice(0, recentCount);
  const archived = sections.slice(recentCount).reverse();
  const archives = [];
  let lines = 0;
  for (const section of archived) {
    const size = lineCount(section.text);
    const current = archives.at(-1);
    if (current && lines + 1 + size <= ARCHIVE_LINE_LIMIT) {
      current.push(section);
      lines += 1 + size;
    } else {
      archives.push([section]);
      lines = ARCHIVE_HEADER_LINES + size;
    }
  }
  return { recent, archives };
}

function archivePath(index) {
  return `${ARCHIVE_DIR}/archive-${String(index + 1).padStart(2, "0")}.md`;
}

function archiveRange(archive) {
  const [oldest, newest] = [archive[0].version, archive.at(-1).version];
  return oldest === newest ? oldest : `${oldest} to ${newest}`;
}

function renderArchive(archive) {
  const newestFirst = [...archive].reverse();
  const header = `# Changelog archive: ${archiveRange(archive)}\n\n` +
    "Releases in this range, newest first. Newer releases are in " +
    "[CHANGELOG.md](../../CHANGELOG.md).";
  const sections = newestFirst.map(({ text }) => rebaseLinks(text, ARCHIVE_DIR));
  return `${header}\n\n${sections.join("\n\n")}\n`;
}

export function renderReconstruction(groups, assignments) {
  const sections = [...groups.values()]
    .sort((a, b) => compareVersions(b.version, a.version))
    .map((group) => ({ version: group.version, text: sectionText(group) }));

  const { recent, archives } = splitHistory(sections);
  let header = INTRO;
  if (archives.length > 0) {
    const links = archives
      .map((archive, index) => `- [${archiveRange(archive)}](${archivePath(index)})`)
      .reverse();
    header += "\n\nOlder releases are archived under `docs/changelog/` so that no file\n" +
      `exceeds the repository's 1500-line cap (newest first):\n\n${links.join("\n")}`;
  }
  header += `\n\n${INSERT_MARKER}`;

  const changelog = `${header}\n\n${recent.map(({ text }) => text).join("\n\n")}\n`;
  const archiveFiles = archives.map((archive, index) => ({
    path: archivePath(index),
    content: renderArchive(archive),
  }));
  const map = [
    "fragment\tfirst_release",
    ...assignments
      .sort((a, b) => a.path.localeCompare(b.path))
      .map(({ path, version }) => `${path}\t${version}`),
    "",
  ].join("\n");

  return { changelog, archives: archiveFiles, map, assignments, groups };
}

// The archive files on disk, by repository-relative path.
function existingArchives() {
  if (!existsSync(ARCHIVE_DIR)) return [];
  return readdirSync(ARCHIVE_DIR)
    .filter((name) => /^archive-\d+\.md$/.test(name))
    .sort()
    .map((name) => `${ARCHIVE_DIR}/${name}`);
}

function main() {
  const ref = argument("ref", "origin/main");
  const reconstruction = reconstruct(ref);
  const pendingVersion = argument("pending-release");
  if (pendingVersion) {
    addPendingRelease(reconstruction, {
      version: pendingVersion,
      date: argument("pending-date", new Date().toISOString().slice(0, 10)),
      fragments: deletedFragments(ref),
    });
  }
  const result = renderReconstruction(
    reconstruction.groups,
    reconstruction.assignments,
  );

  if (process.argv.includes("--write")) {
    writeFileSync(CHANGELOG_PATH, result.changelog);
    for (const stale of existingArchives()) unlinkSync(stale);
    if (result.archives.length > 0) mkdirSync(ARCHIVE_DIR, { recursive: true });
    for (const { path, content } of result.archives) writeFileSync(path, content);
    writeFileSync(MAP_PATH, result.map);
  } else if (process.argv.includes("--check")) {
    if (readFileSync(CHANGELOG_PATH, "utf8") !== result.changelog) {
      throw new Error("CHANGELOG.md differs from reconstructed Git history");
    }
    const expected = new Set(result.archives.map(({ path }) => path));
    for (const path of existingArchives()) {
      if (!expected.has(path)) {
        throw new Error(`${path} is a stale changelog archive; rerun with --write`);
      }
    }
    for (const { path, content } of result.archives) {
      if (!existsSync(path) || readFileSync(path, "utf8") !== content) {
        throw new Error(`${path} differs from reconstructed Git history`);
      }
    }
    if (readFileSync(MAP_PATH, "utf8") !== result.map) {
      throw new Error(`${MAP_PATH} differs from reconstructed Git history`);
    }
  } else {
    process.stdout.write(result.changelog);
  }

  process.stderr.write(
    `Reconstructed ${result.assignments.length} fragments across ${result.groups.size} non-empty releases from ${ref}.\n`,
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
