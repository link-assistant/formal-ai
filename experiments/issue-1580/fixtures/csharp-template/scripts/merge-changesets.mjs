#!/usr/bin/env bun

/**
 * Merge multiple changeset files into a single changeset
 *
 * Key behavior:
 * - Combines all pending changesets into a single changeset file
 * - Uses the highest version bump type (major > minor > patch)
 * - Preserves all descriptions in chronological order (by file modification time)
 * - Removes the individual changeset files after merging
 * - Does nothing if there's only one or no changesets
 *
 * This script is run before versioning to ensure a clean release
 * even when multiple PRs have merged before a release cycle.
 *
 * IMPORTANT: Update the package name below to match your csproj
 */

import { readdirSync, writeFileSync, unlinkSync, statSync } from 'fs';
import { parseChangesetFile } from './parse-changeset.mjs';
import { join } from 'path';
import { printUntrusted } from './print-untrusted.mjs';

// Package name must match the package name in the changeset files
const PACKAGE_NAME = 'MyPackage';
const CHANGESET_DIR = '.changeset';

// Version bump type priority (higher number = higher priority)
const BUMP_PRIORITY = {
  patch: 1,
  minor: 2,
  major: 3,
};

/**
 * Generate a random changeset file name (similar to what @changesets/cli does)
 * @returns {string}
 */
function generateChangesetName() {
  const adjectives = [
    'bright',
    'calm',
    'cool',
    'cyan',
    'dark',
    'fast',
    'gold',
    'good',
    'green',
    'happy',
    'kind',
    'loud',
    'neat',
    'nice',
    'pink',
    'proud',
    'quick',
    'red',
    'rich',
    'safe',
    'shy',
    'soft',
    'sweet',
    'tall',
    'warm',
    'wise',
    'young',
  ];
  const nouns = [
    'apple',
    'bird',
    'book',
    'car',
    'cat',
    'cloud',
    'desk',
    'dog',
    'door',
    'fish',
    'flower',
    'frog',
    'grass',
    'house',
    'key',
    'lake',
    'leaf',
    'moon',
    'mouse',
    'owl',
    'park',
    'rain',
    'river',
    'rock',
    'sea',
    'star',
    'sun',
    'tree',
    'wave',
    'wind',
  ];

  const randomAdjective = adjectives[Math.floor(Math.random() * adjectives.length)];
  const randomNoun = nouns[Math.floor(Math.random() * nouns.length)];

  return `${randomAdjective}-${randomNoun}`;
}

/**
 * Parse a changeset file and extract its metadata
 * @param {string} filePath
 * @returns {{type: string, description: string, mtime: Date} | null}
 */
function parseChangeset(filePath) {
  return {
    ...parseChangesetFile(filePath, PACKAGE_NAME),
    mtime: statSync(filePath).mtime,
  };
}

/**
 * Get the highest priority bump type
 * @param {string[]} types
 * @returns {string}
 */
function getHighestBumpType(types) {
  let highest = 'patch';
  for (const type of types) {
    if (BUMP_PRIORITY[type] > BUMP_PRIORITY[highest]) {
      highest = type;
    }
  }
  return highest;
}

/**
 * Create a merged changeset file
 * @param {string} type
 * @param {string[]} descriptions
 * @returns {string}
 */
function createMergedChangeset(type, descriptions) {
  const combinedDescription = descriptions.join('\n\n');

  return `---
'${PACKAGE_NAME}': ${type}
---

${combinedDescription}
`;
}

function main() {
  console.log('Checking for multiple changesets to merge...');

  // Get all changeset files
  const changesetFiles = readdirSync(CHANGESET_DIR).filter(
    (file) => file.endsWith('.md') && file !== 'README.md' && file !== 'config.json',
  );

  console.log(`Found ${changesetFiles.length} changeset file(s)`);

  // Validate even a single pending file before deciding no merge is needed.
  for (const file of changesetFiles) parseChangeset(join(CHANGESET_DIR, file));
  if (changesetFiles.length <= 1) {
    console.log('No merging needed (0 or 1 changeset found)');
    return;
  }

  console.log('Multiple changesets found, merging...');
  changesetFiles.forEach((file) => console.log(`  - ${file}`));

  // Parse all changesets
  const parsedChangesets = [];
  for (const file of changesetFiles) {
    const filePath = join(CHANGESET_DIR, file);
    const parsed = parseChangeset(filePath);
    if (parsed) {
      parsedChangesets.push({
        file,
        filePath,
        ...parsed,
      });
    }
  }

  if (parsedChangesets.length === 0) {
    console.error('Error: No valid changesets could be parsed');
    process.exit(1);
  }

  // Sort by modification time (oldest first) to preserve chronological order
  parsedChangesets.sort((a, b) => a.mtime.getTime() - b.mtime.getTime());

  // Determine the highest bump type
  const bumpTypes = parsedChangesets.map((c) => c.type);
  const highestBumpType = getHighestBumpType(bumpTypes);

  console.log(`\nMerge summary:`);
  console.log(`  Bump types found: ${[...new Set(bumpTypes)].join(', ')}`);
  console.log(`  Using highest: ${highestBumpType}`);

  // Collect descriptions in chronological order
  const descriptions = parsedChangesets
    .filter((c) => c.description)
    .map((c) => c.description);

  console.log(`  Descriptions to merge: ${descriptions.length}`);

  // Create merged changeset content
  const mergedContent = createMergedChangeset(highestBumpType, descriptions);

  // Generate a unique name for the merged changeset
  const mergedFileName = `merged-${generateChangesetName()}.md`;
  const mergedFilePath = join(CHANGESET_DIR, mergedFileName);

  // Write the merged changeset
  writeFileSync(mergedFilePath, mergedContent);
  console.log(`\nCreated merged changeset: ${mergedFileName}`);

  // Remove the original changeset files
  console.log('\nRemoving original changeset files:');
  for (const changeset of parsedChangesets) {
    unlinkSync(changeset.filePath);
    console.log(`  Removed: ${changeset.file}`);
  }

  console.log('\nChangeset merge completed successfully');
  console.log('\nMerged changeset content:');
  printUntrusted(mergedContent);
}

main();
