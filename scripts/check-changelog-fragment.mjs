#!/usr/bin/env node
// JavaScript twin of scripts/check-changelog-fragment.rs: same output, same
// exit codes. A pull request that changes source files must add a changelog
// fragment in its own diff, not lean on leftover fragments of earlier ones.
//
// Usage: node scripts/check-changelog-fragment.mjs
// GITHUB_BASE_REF names the base branch (default "main").
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';

function exec(command, args) {
  const result = spawnSync(command, args, { encoding: 'utf8' });
  if (result.error) {
    console.error(`Failed to execute ${command} ${debugList(args)}: ${result.error.message}`);
    return '';
  }
  if (result.status !== 0) {
    console.error(`Error executing ${command} ${debugList(args)}`);
    console.error(result.stderr);
    return '';
  }
  return result.stdout.trim();
}

function rustRoot() {
  if (process.env.RUST_ROOT) return process.env.RUST_ROOT;
  if (existsSync('./Cargo.toml')) return '.';
  if (existsSync('./rust/Cargo.toml')) return 'rust';
  return '.';
}

function changedFiles() {
  const base = process.env.GITHUB_BASE_REF ?? 'main';
  console.error(`Comparing against origin/${base}...HEAD`);
  const output = exec('git', ['diff', '--name-only', `origin/${base}...HEAD`]);
  return output === '' ? [] : output.split('\n').filter(Boolean);
}

const debugList = (items) => `[${items.map((item) => JSON.stringify(item)).join(', ')}]`;
const escape = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

export function isSourceFile(path, root) {
  const prefix = escape(root === '.' ? '' : `${root}/`);
  return [`^${prefix}src/`, `^${prefix}tests/`, `^${prefix}?scripts/`, `^${prefix}Cargo\\.toml$`]
    .some((pattern) => new RegExp(pattern).test(path));
}

export function isChangelogFragment(path, root) {
  const dir = root === '.' ? 'changelog.d/' : `${root}/changelog.d/`;
  return (path.startsWith(dir) || path.startsWith('changelog.d/'))
    && path.endsWith('.md') && !path.endsWith('README.md');
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
  for (const file of files) console.log(`  ${file}`);
  console.log();
  const sources = files.filter((file) => isSourceFile(file, root));
  console.log(`Source files changed: ${sources.length}`);
  for (const file of sources) console.log(`  ${file}`);
  console.log();
  const fragments = files.filter((file) => isChangelogFragment(file, root));
  console.log(`Changelog fragments added: ${fragments.length}`);
  for (const file of fragments) console.log(`  ${file}`);
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
