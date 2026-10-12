#!/usr/bin/env node
// Fill the five-language response contract (en, ru, hi, zh, es) by rule.
//
// The response seed (data/seed/multilingual-responses*.lino) states one
// `response` block per intent and language. This tool turns the missing rows
// into work items, applies reviewed translations next to the intent's existing
// rows, and moves whole intent families into a category file when a seed file
// would pass the size limit. Every pass is mechanical and repeatable, so a
// bulk translation is a rule application, not a hand edit.
//
// Usage:
//   node experiments/formal_ai_subagent/fill-response-languages.mjs --extract <directory> [--chunk 40]
//       Write work items (one JSON file per chunk): intent, file, missing
//       languages and every existing text of the intent.
//   node experiments/formal_ai_subagent/fill-response-languages.mjs --apply <translations.json>...
//       Insert translations ({ intent, language, text } with `text` written as
//       the seed writes it between the quotes) after the intent's last row.
//       Each text must keep the English row's slots and its leading and
//       trailing line breaks; a refused item is reported and skipped.
//   node experiments/formal_ai_subagent/fill-response-languages.mjs --validate <translations.json>...
//       The same checks as --apply, writing nothing.
//   node experiments/formal_ai_subagent/fill-response-languages.mjs --move <from.lino> <to.lino> <intent-regexp> [header comment]
//       Move every block whose intent matches into a category file (created
//       with the header comment when it does not exist yet).
//   node experiments/formal_ai_subagent/fill-response-languages.mjs --check
//       Report each response file's line count against the 1500-line limit
//       and the intents that still lack a target language.
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const SEED_DIRECTORY = 'data/seed';
const SEED_PREFIX = 'multilingual-responses';
const LANGUAGES = ['en', 'ru', 'hi', 'zh', 'es'];
const LINE_LIMIT = 1500;
const ROOT = 'multilingual_responses';

const responseFiles = () =>
  readdirSync(SEED_DIRECTORY)
    .filter((name) => name.startsWith(SEED_PREFIX) && name.endsWith('.lino'))
    .sort();

const valueOf = (line, key) => line.trim().slice(key.length + 1).trim();

// One `response` block: its line span and the fields the contract reads.
export function parseBlocks(lines) {
  const blocks = [];
  for (let index = 0; index < lines.length; index += 1) {
    const match = /^  response (\S+)\s*$/.exec(lines[index]);
    if (!match) continue;
    const block = { name: match[1], start: index, end: index + 1, intent: '', language: '', text: '', fields: [] };
    while (block.end < lines.length && /^ {4,}\S/.test(lines[block.end])) {
      const line = lines[block.end];
      const key = line.trim().split(' ')[0];
      if (line.startsWith('    ') && !line.startsWith('     ')) {
        block.fields.push(key);
        if (key === 'intent') block.intent = valueOf(line, key).replace(/^"|"$/g, '');
        if (key === 'language') block.language = valueOf(line, key).replace(/^"|"$/g, '');
        if (key === 'text') block.text = valueOf(line, key);
      }
      block.end += 1;
    }
    blocks.push(block);
    index = block.end - 1;
  }
  return blocks;
}

function loadSeed() {
  const files = new Map();
  for (const name of responseFiles()) {
    const lines = readFileSync(join(SEED_DIRECTORY, name), 'utf8').split('\n');
    files.set(name, { lines, blocks: parseBlocks(lines) });
  }
  return files;
}

function intentsOf(files) {
  const intents = new Map();
  for (const [file, { blocks }] of files) {
    for (const block of blocks) {
      if (!block.intent) continue;
      if (!intents.has(block.intent)) intents.set(block.intent, { files: new Set(), rows: new Map() });
      const entry = intents.get(block.intent);
      entry.files.add(file);
      entry.rows.set(block.language, block);
    }
  }
  return intents;
}

const missingLanguages = (rows) =>
  LANGUAGES.some((language) => rows.has(language)) ? LANGUAGES.filter((language) => !rows.has(language)) : [];

// The quoted body of a `text` value as the seed writes it.
const quotedBody = (raw) => (raw.startsWith('"') && raw.endsWith('"') && raw.length > 1 ? raw.slice(1, -1) : null);
const slots = (body) => (body.match(/\{[A-Za-z0-9_]+\}/g) ?? []).sort().join(' ');
const leadingBreaks = (body) => /^(?:\\n|\s)*/.exec(body)[0];
const trailingBreaks = (body) => /(?:\\n|\s)*$/.exec(body)[0];

// Why a translation cannot stand next to the English row, or '' when it can.
export function refusal(english, translated) {
  const englishBody = quotedBody(english);
  if (englishBody === null) return 'the English row is not a double-quoted text';
  if (typeof translated !== 'string' || translated.trim() === '') return 'empty text';
  if (/(^|[^\\])"/.test(translated)) return 'a bare double quote (write \\" or use typographic quotes)';
  if (/[\r\n]/.test(translated)) return 'a raw line break (write \\n as the seed does)';
  if (slots(englishBody) !== slots(translated)) return `slots differ: ${slots(englishBody)} vs ${slots(translated)}`;
  if (leadingBreaks(englishBody) !== leadingBreaks(translated)) return 'leading whitespace differs';
  if (trailingBreaks(englishBody) !== trailingBreaks(translated)) return 'trailing whitespace differs';
  return '';
}

function extract(directory, chunkSize) {
  const files = loadSeed();
  const items = [];
  for (const [intent, { files: owners, rows }] of intentsOf(files)) {
    const missing = missingLanguages(rows);
    if (missing.length === 0) continue;
    const texts = Object.fromEntries(LANGUAGES.filter((language) => rows.has(language)).map((language) => [language, rows.get(language).text]));
    items.push({ intent, file: [...owners].join(','), missing, texts });
  }
  items.sort((left, right) => left.file.localeCompare(right.file) || left.intent.localeCompare(right.intent));
  mkdirSync(directory, { recursive: true });
  let chunk = 0;
  for (let index = 0; index < items.length; index += chunkSize) {
    chunk += 1;
    writeFileSync(join(directory, `work-${String(chunk).padStart(2, '0')}.json`), `${JSON.stringify(items.slice(index, index + chunkSize), null, 2)}\n`);
  }
  console.log(`${items.length} intents in ${chunk} chunks under ${directory}`);
}

function apply(paths, { write = true } = {}) {
  const files = loadSeed();
  const intents = intentsOf(files);
  const names = new Set([...files.values()].flatMap(({ blocks }) => blocks.map((block) => block.name)));
  const insertions = new Map();
  let applied = 0;
  const refused = [];
  for (const path of paths) {
    for (const item of JSON.parse(readFileSync(path, 'utf8'))) {
      const entry = intents.get(item.intent);
      const where = `${path}: ${item.intent} ${item.language}`;
      if (!entry) { refused.push(`${where}: unknown intent`); continue; }
      if (!LANGUAGES.includes(item.language) || item.language === 'en') { refused.push(`${where}: not a target language`); continue; }
      if (entry.rows.has(item.language)) { refused.push(`${where}: already present`); continue; }
      const english = entry.rows.get('en');
      if (!english) { refused.push(`${where}: no English row to follow`); continue; }
      const why = refusal(english.text, item.text);
      if (why) { refused.push(`${where}: ${why}`); continue; }
      const name = english.name.endsWith('_en') ? `${english.name.slice(0, -3)}_${item.language}` : `${english.name}_${item.language}`;
      if (names.has(name)) { refused.push(`${where}: response name ${name} is taken`); continue; }
      names.add(name);
      const block = { name, intent: item.intent, language: item.language, lines: [`  response ${name}`, `    intent ${item.intent}`, `    language ${item.language}`, `    text "${item.text}"`] };
      entry.rows.set(item.language, block);
      if (!insertions.has(item.intent)) insertions.set(item.intent, []);
      insertions.get(item.intent).push(block);
      applied += 1;
    }
  }
  for (const [file, { lines, blocks }] of write ? files : []) {
    const after = new Map();
    for (const block of blocks) if (insertions.has(block.intent)) after.set(block.intent, block.end);
    if (after.size === 0) continue;
    const order = [...after].sort((left, right) => right[1] - left[1]);
    for (const [intent, end] of order) {
      const added = insertions.get(intent).sort((left, right) => LANGUAGES.indexOf(left.language) - LANGUAGES.indexOf(right.language));
      lines.splice(end, 0, ...added.flatMap((block) => block.lines));
    }
    writeFileSync(join(SEED_DIRECTORY, file), lines.join('\n'));
  }
  for (const line of refused) console.error(`refused ${line}`);
  console.log(`${write ? 'applied' : 'valid'} ${applied} translations, refused ${refused.length}`);
  if (refused.length > 0) process.exitCode = 1;
}

function move(from, to, pattern, header) {
  const expression = new RegExp(pattern);
  const source = readFileSync(join(SEED_DIRECTORY, from), 'utf8').split('\n');
  const moved = [];
  const kept = [];
  const blocks = parseBlocks(source);
  let cursor = 0;
  for (const block of blocks) {
    kept.push(...source.slice(cursor, block.start));
    (expression.test(block.intent) ? moved : kept).push(...source.slice(block.start, block.end));
    cursor = block.end;
  }
  kept.push(...source.slice(cursor));
  if (moved.length === 0) throw new Error(`no intent of ${from} matches ${pattern}`);
  const targetPath = join(SEED_DIRECTORY, to);
  const target = existsSync(targetPath)
    ? readFileSync(targetPath, 'utf8').replace(/\n*$/, '\n')
    : `${(header ?? '').split('\\n').filter(Boolean).map((line) => `# ${line}`.trimEnd()).join('\n')}\n\n${ROOT}\n`;
  writeFileSync(targetPath, `${target}${moved.join('\n')}\n`);
  writeFileSync(join(SEED_DIRECTORY, from), kept.join('\n'));
  console.log(`moved ${moved.length} lines matching ${pattern} from ${from} to ${to}`);
}

function check() {
  const files = loadSeed();
  let over = 0;
  for (const [file, { lines }] of files) {
    const count = lines.length - (lines.at(-1) === '' ? 1 : 0);
    if (count > LINE_LIMIT) { over += 1; console.error(`${file}: ${count} lines (limit ${LINE_LIMIT})`); }
  }
  const missing = [...intentsOf(files)].filter(([, { rows }]) => missingLanguages(rows).length > 0);
  console.log(`${missing.length} intents still lack a target language; ${over} response files over ${LINE_LIMIT} lines`);
  for (const [intent, { rows }] of missing) console.log(`  ${intent}: ${missingLanguages(rows).join(',')}`);
  if (over > 0) process.exitCode = 1;
}

function main() {
  const [mode, ...rest] = process.argv.slice(2);
  if (mode === '--extract') {
    const chunkIndex = rest.indexOf('--chunk');
    extract(rest[0], chunkIndex >= 0 ? Number(rest[chunkIndex + 1]) : 40);
  } else if (mode === '--apply') apply(rest);
  else if (mode === '--validate') apply(rest, { write: false });
  else if (mode === '--move') move(rest[0], rest[1], rest[2], rest[3]);
  else check();
}

if (import.meta.url === `file://${process.argv[1]}`) main();
