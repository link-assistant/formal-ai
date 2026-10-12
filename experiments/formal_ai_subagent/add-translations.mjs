#!/usr/bin/env node
// Adds response translations to a seed response file from a table, by rule:
// each row `intent<TAB>"text"` becomes a `response response_<intent>_<language>`
// block placed after the intent's last existing block. A language the intent
// already has is left alone, so the command can be re-run. The seed file is
// then copied to its byte-identical mirrors.
//
// Usage: node experiments/formal_ai_subagent/add-translations.mjs <seed.lino> <language> <table.tsv>
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { basename } from 'node:path';

const MIRRORS = ['rust/embedded/data/seed', 'js/seed'];

function responseBlocks(lines) {
  const blocks = [];
  for (const [index, line] of lines.entries()) {
    if (line.startsWith('  response ')) blocks.push({ start: index, end: index, intent: '', language: '' });
    else if (blocks.length && line.startsWith('    ') && blocks.at(-1).end === index - 1) {
      const block = blocks.at(-1);
      block.end = index;
      const [key, ...rest] = line.trim().split(' ');
      if (key === 'intent') block.intent = rest.join(' ').replaceAll('"', '');
      if (key === 'language') block.language = rest.join(' ').replaceAll('"', '');
    }
  }
  return blocks;
}

function main() {
  const [seed, language, table] = process.argv.slice(2);
  if (!seed || !language || !table) {
    console.error('usage: add-translations.mjs <seed.lino> <language> <table.tsv>');
    process.exit(2);
  }
  const lines = readFileSync(seed, 'utf8').split('\n');
  const blocks = responseBlocks(lines);
  const rows = readFileSync(table, 'utf8').split('\n').filter(Boolean).map((row) => row.split('\t'));
  const insertions = [];
  const skipped = [];
  for (const [intent, text] of rows) {
    const ofIntent = blocks.filter((block) => block.intent === intent);
    if (ofIntent.length === 0) { skipped.push(`${intent} (no such intent)`); continue; }
    if (ofIntent.some((block) => block.language === language)) { skipped.push(`${intent} (has ${language})`); continue; }
    insertions.push({ after: Math.max(...ofIntent.map((block) => block.end)), lines: [
      `  response response_${intent}_${language}`, `    intent ${intent}`, `    language ${language}`, `    text ${text}`,
    ] });
  }
  for (const insertion of insertions.sort((left, right) => right.after - left.after)) {
    lines.splice(insertion.after + 1, 0, ...insertion.lines);
  }
  writeFileSync(seed, lines.join('\n'));
  for (const mirror of MIRRORS) {
    const target = `${mirror}/${basename(seed)}`;
    if (existsSync(target)) copyFileSync(seed, target);
  }
  console.log(`${seed}: added ${insertions.length} ${language} response(s); skipped ${skipped.length}${skipped.length ? `: ${skipped.join(', ')}` : ''}`);
}

main();
