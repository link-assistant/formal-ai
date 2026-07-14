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
 * Usage:
 *   node experiments/issue_711_rebuild_changelog.mjs --write
 *   node experiments/issue_711_rebuild_changelog.mjs --check
 *   node experiments/issue_711_rebuild_changelog.mjs --ref origin/main
 */

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";

const INITIAL_COMMIT = "6f8d4a8a05770adfd2fe33fdf3c6c586efb103af";
const CHANGELOG_PATH = "CHANGELOG.md";
const MAP_PATH = "docs/case-studies/issue-711/fragment-release-map.tsv";
const HEADER = `# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->`;

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
  return git(["log", "--first-parent", "--reverse", "--format=%H%x09%cI%x09%s", ref])
    .trim()
    .split("\n")
    .map((line) => line.split("\t"))
    .filter(([, , subject]) => /^chore: release v\d+\.\d+\.\d+/.test(subject))
    .map(([commit, committedAt, subject]) => ({
      commit,
      date: committedAt.slice(0, 10),
      version: subject.match(/^chore: release v(\d+\.\d+\.\d+)/)[1],
    }));
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
    for (const path of treeFiles(release.commit)) {
      if (seen.has(path)) continue;
      fragments.push({ path, body: stripFrontmatter(fileAt(release.commit, path)), commit: release.commit });
      assignments.push({ path, version: release.version, commit: release.commit });
      seen.add(path);
    }
    if (fragments.length > 0) groups.set(release.version, { ...release, fragments });
  }

  const sections = [...groups.values()]
    .sort((a, b) => compareVersions(b.version, a.version))
    .map((group) => {
      const body = group.fragments === null
        ? group.body
        : group.fragments.sort((a, b) => a.path.localeCompare(b.path)).map(({ body }) => body).join("\n\n");
      return `## [${group.version}] - ${group.date}\n\n${body}`;
    });

  const changelog = `${HEADER}\n\n${sections.join("\n\n")}\n`;
  const map = [
    "fragment\tfirst_release\tfirst_release_commit",
    ...assignments
      .sort((a, b) => a.path.localeCompare(b.path))
      .map(({ path, version, commit }) => `${path}\t${version}\t${commit}`),
    "",
  ].join("\n");

  if (assignments.length !== 391) {
    throw new Error(`Expected 391 released fragments, reconstructed ${assignments.length}`);
  }
  return { changelog, map, assignments, groups };
}

const ref = argument("ref", "origin/main");
const result = reconstruct(ref);

if (process.argv.includes("--write")) {
  writeFileSync(CHANGELOG_PATH, result.changelog);
  writeFileSync(MAP_PATH, result.map);
} else if (process.argv.includes("--check")) {
  if (readFileSync(CHANGELOG_PATH, "utf8") !== result.changelog) {
    throw new Error("CHANGELOG.md differs from reconstructed Git history");
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
