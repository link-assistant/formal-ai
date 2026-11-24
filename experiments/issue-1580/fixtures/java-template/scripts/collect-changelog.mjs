#!/usr/bin/env node
/**
 * Changelog collection script.
 *
 * Collects all changesets from .changeset/ and merges them
 * into CHANGELOG.md under a new version entry.
 *
 * Usage:
 *   bun scripts/collect-changelog.mjs [--version <version>] [--dry-run]
 */

import { parseArgs as parseCommandLine } from 'node:util';

import { readFileSync, writeFileSync, readdirSync, unlinkSync, existsSync, statSync } from 'fs';
import { join, basename } from 'path';
import { printUntrusted } from './github-actions-log.mjs';

// Package name - update this when forking the template
const PACKAGE_NAME = 'my-package';

/**
 * Parse command line arguments.
 */
function parseArgs() {
  const { values } = parseCommandLine({
    options: {
      version: { type: 'string' },
      'dry-run': { type: 'boolean', default: false },
    },
  });
  const version = values.version;
  const dryRun = values['dry-run'];

  return { dryRun, version };
}

/**
 * Get the current version from pom.xml.
 * @param {string} pomPath - Path to pom.xml
 * @returns {string} Current version
 */
function getCurrentVersion(pomPath) {
  const content = readFileSync(pomPath, 'utf-8');
  const match = content.match(/<project[^>]*>[\s\S]*?<version>([^<]+)<\/version>/);
  if (!match) {
    throw new Error('Could not find version in pom.xml');
  }
  return match[1];
}

/**
 * Find all changeset files.
 * @param {string} changesetDir - Path to .changeset directory
 * @returns {Array<{path: string, mtime: Date}>} Array of changeset file info
 */
function findChangesets(changesetDir) {
  if (!existsSync(changesetDir)) {
    return [];
  }

  return readdirSync(changesetDir)
    .filter((file) =>
      file.endsWith('.md') &&
      file !== 'README.md'
    )
    .map((file) => {
      const filePath = join(changesetDir, file);
      const stats = statSync(filePath);
      return { path: filePath, mtime: stats.mtime };
    })
    .sort((a, b) => a.mtime - b.mtime); // Sort by modification time (oldest first)
}

/**
 * Parse a changeset file and extract description.
 * @param {string} filePath - Path to changeset file
 * @returns {string|null} Description or null if invalid
 */
function parseChangeset(filePath) {
  try {
    const content = readFileSync(filePath, 'utf-8');

    // Match frontmatter
    const frontmatterMatch = content.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
    if (!frontmatterMatch) {
      return null;
    }

    return frontmatterMatch[2].trim();
  } catch {
    return null;
  }
}

/**
 * Get current date in YYYY-MM-DD format.
 * @returns {string} Formatted date
 */
function getCurrentDate() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/**
 * Update CHANGELOG.md with new version entry.
 * @param {string} changelogPath - Path to CHANGELOG.md
 * @param {string} version - Version number
 * @param {string} content - Changelog content
 */
function updateChangelog(changelogPath, version, content) {
  let changelog = '';

  if (existsSync(changelogPath)) {
    changelog = readFileSync(changelogPath, 'utf-8');
  } else {
    changelog = `# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

`;
  }

  const date = getCurrentDate();
  const newEntry = `## [${version}] - ${date}\n\n${content}\n\n`;

  // Find the insertion point (after the header, before first version entry)
  const insertMarker = '<!-- changelog-insert-here -->';
  if (changelog.includes(insertMarker)) {
    changelog = changelog.replace(insertMarker, `${insertMarker}\n\n${newEntry}`);
  } else {
    // Insert after the header section
    const headerEndMatch = changelog.match(/^(# Changelog[\s\S]*?\n)\n/m);
    if (headerEndMatch) {
      const headerEnd = headerEndMatch.index + headerEndMatch[0].length;
      changelog =
        changelog.slice(0, headerEnd) + newEntry + changelog.slice(headerEnd);
    } else {
      changelog = newEntry + changelog;
    }
  }

  writeFileSync(changelogPath, changelog);
}

/**
 * Delete processed changeset files.
 * @param {Array<{path: string}>} changesetFiles - Array of changeset file info
 */
function deleteChangesets(changesetFiles) {
  for (const { path } of changesetFiles) {
    unlinkSync(path);
    console.log('Deleted changeset:');
    printUntrusted(basename(path));
  }
}

/**
 * Main function.
 */
function main() {
  const { dryRun, version: providedVersion } = parseArgs();
  const projectRoot = process.cwd();
  const pomPath = join(projectRoot, 'pom.xml');
  const changesetDir = join(projectRoot, '.changeset');
  const changelogPath = join(projectRoot, 'CHANGELOG.md');

  const version = providedVersion || getCurrentVersion(pomPath);
  console.log(`Collecting changesets for version ${version}...`);

  const changesetFiles = findChangesets(changesetDir);

  if (changesetFiles.length === 0) {
    console.log('No changesets found.');
    return;
  }

  console.log(`Found ${changesetFiles.length} changeset(s):`);

  // Parse and combine all changesets
  const descriptions = [];
  for (const { path } of changesetFiles) {
    printUntrusted(basename(path));
    const description = parseChangeset(path);
    if (description) {
      descriptions.push(description);
    }
  }

  const combinedContent = descriptions.join('\n\n');

  if (dryRun) {
    console.log('\n[DRY RUN] Would add the following to CHANGELOG.md:');
    console.log(`\n## [${version}] - ${getCurrentDate()}\n`);
    printUntrusted(combinedContent);
    console.log('\n[DRY RUN] No changes made.');
    return;
  }

  updateChangelog(changelogPath, version, combinedContent);
  console.log(`Updated CHANGELOG.md with version ${version}`);

  deleteChangesets(changesetFiles);

  console.log('\nChangelog collection complete!');
}

main();
