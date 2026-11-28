#!/usr/bin/env node
// JavaScript twin of scripts/check-changelog-fragment.rs: same output, same
// exit codes. A pull request that changes source files must add a changelog
// fragment in its own diff, not lean on leftover fragments of earlier ones.
//
// Usage: node scripts/check-changelog-fragment.mjs
// GITHUB_BASE_REF names the base branch (default "main").
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';

const MAX_COMMAND_OUTPUT_BYTES = 16 * 1024 * 1024;

function exec(command, args) {
  const result = spawnSync(command, args, { maxBuffer: MAX_COMMAND_OUTPUT_BYTES });
  if (result.error) {
    const reason = result.error.code === 'ENOBUFS'
      ? `output exceeded ${MAX_COMMAND_OUTPUT_BYTES} bytes` : result.error.message;
    console.error(`Failed to execute ${command} ${debugList(args)}: ${reason}`);
    process.exit(1);
  }
  if (result.status !== 0) {
    console.error(`Error executing ${command} ${debugList(args)}`);
    console.error(result.stderr.toString('utf8'));
    process.exit(1);
  }
  return result.stdout;
}

function rustRoot() {
  if (process.env.RUST_ROOT) return process.env.RUST_ROOT;
  if (existsSync('./Cargo.toml')) return '.';
  if (existsSync('./rust/Cargo.toml')) return 'rust';
  return '.';
}

export function parseGitPaths(bytes) {
  const output = new TextDecoder('utf-8', {fatal:true, ignoreBOM:true}).decode(bytes);
  if (output === '') return [];
  if (!output.endsWith('\0')) throw new Error('Git path inventory is not NUL terminated');
  const paths = output.slice(0, -1).split('\0');
  if (paths.some(path => path === '')) throw new Error('Git path inventory contains an empty path');
  return paths;
}

function changedFiles(addedOnly = false) {
  const base = process.env.GITHUB_BASE_REF ?? 'main';
  console.error(`Comparing against origin/${base}...HEAD`);
  // Exact renames compare object IDs; similarity detection would fetch missing blobs.
  const selection = addedOnly ? ['--find-renames=100%', '--diff-filter=A'] : ['--no-renames'];
  const output = exec('git', ['diff', ...selection, '--name-only', '-z', `origin/${base}...HEAD`]);
  try { return parseGitPaths(output); }
  catch (error) {
    console.error(`Invalid Git path inventory: ${error.message}`);
    process.exit(1);
  }
}

const displayPath = (file) => file.replaceAll('\r', '\\r').replaceAll('\n', '\\n');
const debugList = (items) => `[${items.map((item) => JSON.stringify(item)).join(', ')}]`;
const escape = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

export function isSourceFile(path, root) {
  const prefix = escape(root === '.' ? '' : `${root}/`);
  return [`^${prefix}src/`, `^${prefix}tests/`, `^${prefix}scripts/`, '^scripts/', `^${prefix}Cargo\\.toml$`]
    .some((pattern) => new RegExp(pattern).test(path));
}

export function isChangelogFragment(path, root) {
  const dir = root === '.' ? 'changelog.d/' : `${root}/changelog.d/`;
  const slash = path.lastIndexOf('/'), parent = path.slice(0, slash + 1), name = path.slice(slash + 1);
  return (parent === dir || parent === 'changelog.d/')
    && name.endsWith('.md') && name !== 'README.md';
}

function main() {
  console.log('Checking for changelog fragment in PR diff...\n');
  const root = rustRoot();
  if (root !== '.') console.log(`Detected multi-language repository (Rust root: ${root})`);
  const files = changedFiles();
  if (files.length === 0) {
    console.log('No changed files found');
    process.exit(0);
  }
  console.log('Changed files:');
  for (const file of files) console.log(`  ${displayPath(file)}`);
  console.log();
  const sources = files.filter((file) => isSourceFile(file, root));
  console.log(`Source files changed: ${sources.length}`);
  for (const file of sources) console.log(`  ${displayPath(file)}`);
  console.log();
  const fragments = changedFiles(true).filter((file) => isChangelogFragment(file, root));
  console.log(`Changelog fragments added: ${fragments.length}`);
  for (const file of fragments) console.log(`  ${displayPath(file)}`);
  console.log();
  if (sources.length > 0 && fragments.length === 0) {
    console.error('::error::No changelog fragment found in this PR. Please add a changelog entry in changelog.d/');
    console.error();
    console.error('To create a changelog fragment:');
    console.error('  Create a new .md file in changelog.d/ with your changes');
    console.error();
    console.error('See changelog.d/README.md for more information.');
    process.exit(1);
  }
  console.log(`Changelog check passed (source files changed: ${sources.length}, fragments added: ${fragments.length})`);
}

if (import.meta.url === `file://${process.argv[1]}`) main();
