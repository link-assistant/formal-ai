#!/usr/bin/env node
// Writes data/seed/coding-documentation-captures.lino (issue #1165, R1165-1).
//
// The seed is pre-cached source data: one record per documentation page that
// was captured byte for byte under rust/tests/fixtures/coding-discovery/
// captured/. A record's header (language, task, rediscovery query, URL, mime,
// fixture path and SHA-256) is authored; its `block` rows are derived here,
// by running the browser engine's page formalizer (formalizePage in
// js/worker/formal_ai_worker_web_formalize.js) over the fixture after its
// SHA-256 is checked. No program is written by hand: a write_program cache
// miss recomposes its program from these blocks at answer time.
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
const HEADER_FIELDS = ['language', 'language_name', 'task', 'rediscovery_query', 'url', 'mime', 'fixture', 'sha256'];
/** Header fields a capture may leave out: the name the page gives its language. */
const OPTIONAL_FIELDS = ['language_name'];

const HEADER = `# Documentation captures for coding discovery (issue #1165, R1165-1).
#
# Pre-cached source data, not solutions. Each capture is one documentation
# page fetched byte for byte into the named fixture; its SHA-256 pins those
# bytes, and its block rows are the code blocks the page formalizer reads
# from them, in page order, with the language each block declares. The
# header fields are authored; the blocks are written by
# scripts/generate-coding-documentation-captures.mjs and pinned against the
# fixture in both roots. A write_program cache miss rediscovers its program
# from these blocks at answer time: the page's example is decomposed, the
# task's expected output is bound into its literal slot, and the result is
# verified before it is answered or cached (see documentation_route in
# data/seed/program-cache-policy.lino).
`;

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
    .map((node) => Object.fromEntries(HEADER_FIELDS.map((field) => [field, node.children.find((child) => child.name === field)?.value ?? ''])));
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

/** The whole seed for `headers`, blocks re-derived from the fixtures. */
export function renderSeed(context, headers) {
  const lines = [HEADER.trimEnd(), 'coding_documentation_captures', '  version "1"'];
  for (const header of headers) {
    lines.push('  capture');
    for (const field of HEADER_FIELDS) {
      const value = header[field];
      if (value === '' && OPTIONAL_FIELDS.includes(field)) continue;
      // Only the task stays bare: every other header value is page or
      // catalog data (a language slug included), quoted as data.
      lines.push(`    ${field} ${field === 'task' ? bare(value) : quoteValue(value)}`);
    }
    for (const block of captureBlocks(context, header)) {
      // The page's own tag is literal page content, so it stays quoted.
      lines.push('    block', `      language ${quoteValue(block.language)}`, `      code ${quoteValue(block.text)}`);
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
  const current = readFileSync(path.join(REPO_ROOT, SEED_PATH), 'utf8');
  const rendered = renderSeed(context, captureHeaders(context, current));
  if (mode === '--write') {
    writeFileSync(path.join(REPO_ROOT, SEED_PATH), rendered);
    writeFileSync(path.join(REPO_ROOT, EMBEDDED_PATH), rendered);
    return;
  }
  const embedded = readFileSync(path.join(REPO_ROOT, EMBEDDED_PATH), 'utf8');
  if (rendered !== current || embedded !== current) {
    console.error(`${SEED_PATH} drifted from its fixtures; run with --write`);
    process.exit(1);
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await main();
