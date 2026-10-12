#!/usr/bin/env node
// Adds missing lexeme languages to a seed meanings file from a table, by rule
// (PR #1188, LEXEMES; the twin of add-translations.mjs). Each table row
//
//   <meaning><TAB><surface><TAB><surface>...
//
// becomes one `lexeme <language>` block with a `surface` / `text` entry per
// cell, placed after the meaning's last existing lexeme block. A cell may
// carry extra surface fields after ` ;; ` (`suma ;; action add`); without
// them, the new surface inherits the fields every surface of the meaning's
// reference lexeme shares (its first present language in en, ru, hi, zh, es
// order), so `part_of_speech noun` or `action add` follow the existing forms.
// A cell is taken verbatim, so a padded surface such as ` web ` keeps its
// spaces. A meaning that already has the language is left alone, so the command can be
// re-run. The seed file is then copied to every byte-identical mirror.
//
// `--missing` prints a table skeleton instead: for each meaning that lacks the
// language, a comment with the surfaces the other languages list and an empty
// row to fill in.
//
// Usage:
//   node experiments/formal_ai_subagent/add-lexemes.mjs <seed.lino> <language> <table.tsv>
//   node experiments/formal_ai_subagent/add-lexemes.mjs --missing <seed.lino> <language>
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { basename } from 'node:path';

const MIRRORS = ['rust/embedded/data/seed', 'js/seed', 'packages/formal-ai-engine/assets/seed'];
const LANGUAGES = ['en', 'ru', 'hi', 'zh', 'es'];
const FIELD_SEPARATOR = ' ;; ';

function indentOf(line) {
  return line.length - line.trimStart().length;
}

function isContent(line) {
  const trimmed = line.trim();
  return trimmed !== '' && !trimmed.startsWith('#');
}

/** The index of the last line of the subtree that starts at `start`. */
function subtreeEnd(lines, start) {
  const indent = indentOf(lines[start]);
  let end = start;
  for (let index = start + 1; index < lines.length; index += 1) {
    if (!isContent(lines[index])) continue;
    if (indentOf(lines[index]) <= indent) break;
    end = index;
  }
  return end;
}

/** The value of a `text` line: its quoted or bare word, without a trailing comment. */
function textValue(trimmed) {
  const rest = trimmed.slice('text '.length);
  if (rest.startsWith('"')) return rest.slice(1, rest.indexOf('"', 1));
  return rest.split(' #')[0].trim();
}

/** The surfaces of one lexeme block: their text and extra field lines. */
function lexemeSurfaces(lines, start, end) {
  const surfaces = [];
  for (let index = start + 1; index <= end; index += 1) {
    const trimmed = lines[index].trim();
    if (trimmed === 'surface') surfaces.push({ text: '', fields: [] });
    else if (surfaces.length && trimmed.startsWith('text ')) surfaces.at(-1).text = textValue(trimmed);
    else if (surfaces.length && isContent(lines[index])) surfaces.at(-1).fields.push(trimmed);
  }
  return surfaces;
}

/** Every lexeme owner of a seed file: its label, indents and lexeme blocks. */
function lexemeOwners(lines) {
  const owners = new Map();
  const stack = [];
  for (const [index, line] of lines.entries()) {
    if (!isContent(line)) continue;
    const indent = indentOf(line);
    while (stack.length && stack.at(-1).indent >= indent) stack.pop();
    const words = line.trim().split(/\s+/);
    if (words[0] === 'lexeme' && LANGUAGES.includes(words[1]) && stack.length) {
      const owner = stack.at(-1);
      if (!owners.has(owner.index)) owners.set(owner.index, { label: owner.label, lexemes: [] });
      const end = subtreeEnd(lines, index);
      owners.get(owner.index).lexemes.push({
        language: words[1],
        indent,
        start: index,
        end,
        surfaces: lexemeSurfaces(lines, index, end),
      });
    }
    stack.push({ index, indent, label: line.trim() });
  }
  return [...owners.values()];
}

/** The fields every surface of a lexeme shares, in their first surface's order. */
function sharedFields(lexeme) {
  const [first, ...rest] = lexeme.surfaces;
  if (!first) return [];
  return first.fields.filter((field) => rest.every((surface) => surface.fields.includes(field)));
}

/** A `text` value, quoted when it is not one bare word. */
function textLiteral(text) {
  return /^[^\s"'()#:;]+$/u.test(text) ? text : `"${text}"`;
}

/** The lines of one new lexeme block. */
function lexemeBlock(owner, language, cells) {
  const reference = LANGUAGES
    .map((code) => owner.lexemes.find((lexeme) => lexeme.language === code))
    .find(Boolean);
  const pad = ' '.repeat(reference.indent);
  const inherited = sharedFields(reference);
  const block = [`${pad}lexeme ${language}`];
  for (const cell of cells) {
    const [text, ...explicit] = cell.split(FIELD_SEPARATOR);
    block.push(`${pad}  surface`, `${pad}    text ${textLiteral(text)}`);
    const fields = explicit.length ? explicit.map((field) => field.trim()) : inherited;
    for (const field of fields) block.push(`${pad}    ${field}`);
  }
  return block;
}

function readTable(path) {
  return readFileSync(path, 'utf8')
    .split('\n')
    .filter((row) => row.trim() !== '' && !row.startsWith('#'))
    .map((row) => row.split('\t'))
    .map(([meaning, ...cells]) => ({ meaning, cells: cells.filter((cell) => cell.trim() !== '') }));
}

/** Copy the edited seed to each mirror that held the file's previous bytes. */
function mirror(seed, previous) {
  const updated = [];
  const diverged = [];
  for (const directory of MIRRORS) {
    const target = `${directory}/${basename(seed)}`;
    if (!existsSync(target)) continue;
    if (readFileSync(target, 'utf8') === previous) {
      copyFileSync(seed, target);
      updated.push(directory);
    } else {
      diverged.push(target);
    }
  }
  return { updated, diverged };
}

function addLexemes(seed, language, tablePath) {
  const previous = readFileSync(seed, 'utf8');
  const lines = previous.split('\n');
  const owners = new Map(lexemeOwners(lines).map((owner) => [owner.label, owner]));
  const insertions = [];
  const skipped = [];
  for (const { meaning, cells } of readTable(tablePath)) {
    const owner = owners.get(meaning);
    if (!owner) {
      skipped.push(`${meaning} (no such lexeme owner)`);
    } else if (owner.lexemes.some((lexeme) => lexeme.language === language)) {
      skipped.push(`${meaning} (has ${language})`);
    } else if (cells.length === 0) {
      skipped.push(`${meaning} (no surfaces)`);
    } else {
      const after = Math.max(...owner.lexemes.map((lexeme) => lexeme.end));
      insertions.push({ after, lines: lexemeBlock(owner, language, cells) });
    }
  }
  for (const insertion of insertions.sort((left, right) => right.after - left.after)) {
    lines.splice(insertion.after + 1, 0, ...insertion.lines);
  }
  writeFileSync(seed, lines.join('\n'));
  const { updated, diverged } = mirror(seed, previous);
  console.log(
    `${seed}: added ${insertions.length} ${language} lexeme(s); mirrored to ${updated.length} location(s); `
      + `skipped ${skipped.length}${skipped.length ? `: ${skipped.join(', ')}` : ''}`,
  );
  if (diverged.length) console.log(`  not mirrored (the mirror differed before the edit): ${diverged.join(', ')}`);
}

function printMissing(seed, language) {
  const lines = readFileSync(seed, 'utf8').split('\n');
  for (const owner of lexemeOwners(lines)) {
    if (owner.lexemes.some((lexeme) => lexeme.language === language)) continue;
    const known = owner.lexemes
      .map((lexeme) => {
        const shared = sharedFields(lexeme);
        const surfaces = lexeme.surfaces.map((surface) => {
          const own = surface.fields.filter((field) => !shared.includes(field));
          return own.length ? `${surface.text}${FIELD_SEPARATOR}${own.join(FIELD_SEPARATOR)}` : surface.text;
        });
        return `${lexeme.language}: ${surfaces.join(' | ')}`;
      })
      .join(' ; ');
    console.log(`# ${known}`);
    console.log(`${owner.label}\t`);
  }
}

function main() {
  const argv = process.argv.slice(2);
  if (argv[0] === '--missing' && argv.length === 3) {
    printMissing(argv[1], argv[2]);
    return;
  }
  const [seed, language, table] = argv;
  if (!seed || !LANGUAGES.includes(language) || !table) {
    console.error('usage: add-lexemes.mjs <seed.lino> <language> <table.tsv> | --missing <seed.lino> <language>');
    process.exit(2);
  }
  addLexemes(seed, language, table);
}

main();
