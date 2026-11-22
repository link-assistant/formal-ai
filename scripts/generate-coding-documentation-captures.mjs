#!/usr/bin/env node
// Projects publisher references from the documentation replay manifest.
// The recorded input paths and hashes remain in the test tree. The runtime
// registry contains only the information needed to acquire a fresh source.
//
// Usage:
//   node scripts/generate-coding-documentation-captures.mjs --write
//   node scripts/generate-coding-documentation-captures.mjs --check

import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

import { REPO_ROOT, createWorkerContext, evaluate, plain } from '../js/server/worker-host.mjs';

export const SEED_PATH = 'data/seed/coding-documentation-captures.lino';
export const EMBEDDED_PATH = 'rust/embedded/data/seed/coding-documentation-captures.lino';
export const REPLAY_PATH = 'rust/tests/fixtures/coding-discovery/documentation-replays.lino';
const SOURCE_FIELDS = ['language', 'language_name', 'task', 'rediscovery_query', 'url', 'mime'];
const REPLAY_FIELDS = [...SOURCE_FIELDS, 'fixture', 'sha256'];
const HEADER = '# Publisher page references for coding discovery. The runtime acquires page bytes on demand.';

/** A value in the seed dialect both roots read: doubled quotes, \\n, \\\\. */
export function quoteValue(text) {
  return `"${String(text).replace(/\\/g, '\\\\').replace(/"/g, '""').replace(/\n/g, '\\n')}"`;
}

/** A slug stays bare; anything else is quoted. */
function bare(value) {
  return /^[a-z_]+$/.test(value) ? value : quoteValue(value);
}

/** The authored header of every capture record in a seed text, in order. */
export function captureHeaders(context, text) {
  const tree = plain(evaluate(context, `parseLinoTree(${JSON.stringify(text)})`));
  const root = tree.children.find((node) => node.name === 'coding_documentation_captures');
  return (root?.children || [])
    .filter((node) => node.name === 'capture')
    .map((node) => Object.fromEntries(REPLAY_FIELDS.map((field) => [field, node.children.find((child) => child.name === field)?.value ?? ''])));
}

/** The code blocks the page formalizer reads from one capture's fixture. */
export function captureBlocks(context, header) {
  const bytes = readFileSync(path.join(REPO_ROOT, header.fixture));
  const digest = createHash('sha256').update(bytes).digest('hex');
  if (digest !== header.sha256) throw new Error(`${header.fixture}: sha256 ${digest} is not the recorded ${header.sha256}`);
  const call = `formalizePage(${JSON.stringify(bytes.toString('utf8'))}, ${JSON.stringify(header.mime)}, ${JSON.stringify(header.url)})`;
  return plain(evaluate(context, call)).blocks
    .filter((block) => block.kind === 'code_block')
    .map((block) => ({ language: block.language || 'unknown', text: block.text }));
}

/** Project the source declarations, excluding all test fixture data. */
export function renderSeed(headers) {
  const lines = [HEADER, 'coding_documentation_captures', '  version "2"'];
  for (const header of headers) {
    lines.push('  capture');
    for (const field of SOURCE_FIELDS) {
      const value = header[field];
      if (value === '' && field === 'language_name') continue;
      lines.push(`    ${field} ${field === 'task' ? bare(value) : quoteValue(value)}`);
    }
  }
  return `${lines.join('\n')}\n`;
}

async function main() {
  const mode = process.argv[2];
  if (mode !== '--write' && mode !== '--check') {
    console.error('usage: generate-coding-documentation-captures.mjs --write|--check');
    process.exit(2);
  }
  const context = createWorkerContext();
  await evaluate(context, 'loadSeed()');
  const fixtures = readFileSync(path.join(REPO_ROOT, REPLAY_PATH), 'utf8');
  const headers = captureHeaders(context, fixtures);
  for (const header of headers) captureBlocks(context, header);
  const rendered = renderSeed(headers);
  if (mode === '--write') {
    writeFileSync(path.join(REPO_ROOT, SEED_PATH), rendered);
    writeFileSync(path.join(REPO_ROOT, EMBEDDED_PATH), rendered);
    return;
  }
  const current = readFileSync(path.join(REPO_ROOT, SEED_PATH), 'utf8');
  const embedded = readFileSync(path.join(REPO_ROOT, EMBEDDED_PATH), 'utf8');
  if (rendered !== current || embedded !== current) {
    console.error(`${SEED_PATH} drifted from the replay source declarations; run with --write`);
    process.exit(1);
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await main();
