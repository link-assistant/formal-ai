#!/usr/bin/env node

/**
 * Detect code changes for CI/CD pipeline
 *
 * This script detects what types of files have changed between two commits
 * and outputs the results for use in GitHub Actions workflow conditions.
 *
 * Key behavior:
 * - For PRs: compares PR head against base branch
 * - For pushes: compares HEAD against HEAD^
 * - Excludes certain folders and file types from "code changes" detection
 *
 * Excluded from code changes (don't require changesets):
 * - Markdown files (*.md) in any folder
 * - .changeset/ folder (changeset metadata)
 * - docs/ folder (documentation)
 * - dev/log/ folder (development logs)
 * - experiments/ folder (experimental scripts)
 * - examples/ folder (example scripts)
 *
 * Usage:
 *   bun run scripts/detect-code-changes.mjs
 *
 * Environment variables (set by GitHub Actions):
 *   - GITHUB_EVENT_NAME: 'pull_request' or 'push'
 *   - GITHUB_BASE_SHA: Base commit SHA for PR
 *   - GITHUB_HEAD_SHA: Head commit SHA for PR
 *
 * Outputs (written to GITHUB_OUTPUT):
 *   - any-code-changed: 'true' if any code files changed after applying the exclusion policy
 */

import { execSync } from 'child_process';
import { appendFileSync } from 'fs';

const CSHARP_ROOT = String(process.env.CSHARP_ROOT ?? '')
  .trim()
  .replaceAll('\\', '/')
  .replace(/\/+$/u, '');

function prefixCsharpRoot(folder) {
  return CSHARP_ROOT && CSHARP_ROOT !== '.' ? `${CSHARP_ROOT}/${folder}` : folder;
}

/**
 * Execute a shell command and return trimmed output
 * @param {string} command - The command to execute
 * @returns {string} - The trimmed command output
 */
function exec(command) {
  try {
    return execSync(command, { encoding: 'utf-8' }).trim();
  } catch (error) {
    console.error(`Error executing command: ${command}`);
    console.error(error.message);
    return '';
  }
}

/**
 * Write output to GitHub Actions output file
 * @param {string} name - Output name
 * @param {string} value - Output value
 */
function setOutput(name, value) {
  const outputFile = process.env.GITHUB_OUTPUT;
  if (outputFile) {
    appendFileSync(outputFile, `${name}=${value}\n`);
  }
  console.log(`${name}=${value}`);
}

/**
 * Get the list of changed files between two commits
 * @returns {string[]} Array of changed file paths
 */
function getChangedFiles() {
  const eventName = process.env.GITHUB_EVENT_NAME || 'local';

  if (eventName === 'pull_request') {
    const baseSha = process.env.GITHUB_BASE_SHA;
    const headSha = process.env.GITHUB_HEAD_SHA;

    if (baseSha && headSha) {
      console.log(`Comparing PR: ${baseSha}...${headSha}`);
      try {
        // Ensure we have the base commit
        try {
          execSync(`git cat-file -e ${baseSha}`, { stdio: 'ignore' });
        } catch {
          console.log('Base commit not available locally, attempting fetch...');
          execSync(`git fetch origin ${baseSha}`, { stdio: 'inherit' });
        }
        const output = exec(`git diff --name-only ${baseSha} ${headSha}`);
        return output ? output.split('\n').filter(Boolean) : [];
      } catch (error) {
        console.error(`Git diff failed: ${error.message}`);
      }
    }
  }

  // For push events or fallback
  console.log('Comparing HEAD^ to HEAD');
  try {
    const output = exec('git diff --name-only HEAD^ HEAD');
    return output ? output.split('\n').filter(Boolean) : [];
  } catch {
    // If HEAD^ doesn't exist (first commit), list all files in HEAD
    console.log('HEAD^ not available, listing all files in HEAD');
    const output = exec('git ls-tree --name-only -r HEAD');
    return output ? output.split('\n').filter(Boolean) : [];
  }
}

/**
 * Check if a file should be excluded from code changes detection
 * @param {string} filePath - The file path to check
 * @returns {boolean} True if the file should be excluded
 */
export function isExcludedFromCodeChanges(filePath) {
  // Exclude markdown files in any folder
  if (filePath.endsWith('.md')) {
    return true;
  }

  // Exclude specific folders from code changes
  const excludedFolders = [
    '.changeset/',
    'docs/',
    'dev/log/',
    'experiments/',
    'examples/',
    prefixCsharpRoot('.changeset/'),
    prefixCsharpRoot('docs/'),
    prefixCsharpRoot('dev/log/'),
    prefixCsharpRoot('experiments/'),
    prefixCsharpRoot('examples/'),
  ];

  for (const folder of excludedFolders) {
    if (filePath.startsWith(folder)) {
      return true;
    }
  }

  return false;
}

/**
 * Compute the outputs used by change-gated workflow jobs.
 * @param {string[]} changedFiles - Changed repository-relative paths
 * @returns {{ 'any-code-changed': string }} GitHub Actions output values
 */
export function detectChangeOutputs(changedFiles) {
  const includedFiles = changedFiles.filter(
    (file) => !isExcludedFromCodeChanges(file)
  );
  const codePattern = /\.(cs|csproj|sln|props|mjs|json|yml|yaml)$|\.github\/workflows\//;
  const codeChanged = includedFiles.some((file) => codePattern.test(file));

  return { 'any-code-changed': codeChanged ? 'true' : 'false' };
}

/**
 * Main function to detect changes
 */
function detectChanges() {
  console.log('Detecting file changes for CI/CD...\n');

  const changedFiles = getChangedFiles();

  console.log('Changed files:');
  if (changedFiles.length === 0) {
    console.log('  (none)');
  } else {
    changedFiles.forEach((file) => console.log(`  ${file}`));
  }
  console.log('');

  // Apply the exclusion policy before computing every job-gating output.
  const codeChangedFiles = changedFiles.filter(
    (file) => !isExcludedFromCodeChanges(file)
  );

  console.log('\nFiles considered as code changes:');
  if (codeChangedFiles.length === 0) {
    console.log('  (none)');
  } else {
    codeChangedFiles.forEach((file) => console.log(`  ${file}`));
  }
  console.log('');

  const outputs = detectChangeOutputs(changedFiles);
  setOutput('any-code-changed', outputs['any-code-changed']);

  console.log('\nChange detection completed.');
}

// Run the detection
if (import.meta.main) {
  detectChanges();
}
