// Issue #1550 (acceptance criterion f): README.md and docs/meta-algorithm.md
// describe what the code does. Both documents carry the same two tables between
// `<!-- meta-procedure:start -->` and `<!-- meta-procedure:end -->`:
//
// - the stage table names, for every stage of the picture, the trace event the
//   procedure emits and the functions of each runtime that emit it;
// - the entry table names the call chain from each runtime's solver entry to
//   the procedure, where every function calls the next, and the recursive
//   executor one function of the chain runs.
//
// `procedureDocDivergences` returns one message per divergence: the tables
// differ between the documents, an event is not the procedure's (in order), a
// named function is not defined, none of a row's functions emits the row's
// event, the event never occurs in the executed parity fixture, or a link of
// the entry chain does not call the next one.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

export const DOCUMENTS = ['README.md', 'docs/meta-algorithm.md'];
const START = '<!-- meta-procedure:start -->';
const END = '<!-- meta-procedure:end -->';
const RUNTIMES = {
  Rust: {
    files: [
      'rust/src/solver.rs',
      'rust/src/meta_executor.rs',
      'rust/src/meta_executor/production.rs',
      'rust/src/meta_executor/selection.rs',
      'rust/src/least_action.rs',
      'rust/src/recursive_execution.rs',
    ],
    kinds: { file: 'rust/src/meta_executor.rs', marker: 'pub const EVENT_KINDS' },
  },
  JavaScript: {
    files: [
      'js/worker/formal_ai_worker_solve.js',
      'js/worker/formal_ai_worker_meta_composite.js',
      'js/agentic/crate/meta_executor.mjs',
      'js/agentic/crate/least_action.mjs',
      'js/agentic/crate/recursive_execution.mjs',
    ],
    kinds: { file: 'js/agentic/crate/meta_executor.mjs', marker: 'export const META_EXEC_EVENT_KINDS' },
  },
};
const PARITY_FIXTURE = 'rust/tests/fixtures/meta-executor-parity.json';

const escape = (text) => text.replace(/[.*+?^${}()|[\]\\]/gu, '\\$&');
const codes = (cell) => [...cell.matchAll(/`([^`]+)`/gu)].map((match) => match[1]);

/** The text between the markers of one document, or null. */
export function procedureBlock(text) {
  const start = text.indexOf(START);
  const end = text.indexOf(END, start);
  return start < 0 || end < 0 ? null : text.slice(start + START.length, end).trim();
}

/** The rows of every Markdown table of a block, by the header's first cell. */
function tables(block) {
  const result = {};
  let current = null;
  for (const line of block.split('\n')) {
    if (!line.startsWith('|')) {
      current = null;
      continue;
    }
    const cells = line.slice(1, line.endsWith('|') ? -1 : undefined).split('|').map((cell) => cell.trim());
    if (cells.every((cell) => /^-+$/u.test(cell))) continue;
    if (!current) {
      current = cells[0];
      result[current] = { header: cells, rows: [] };
    } else {
      result[current].rows.push(cells);
    }
  }
  return result;
}

/**
 * Every body of a definition of `name` in `source`: a Rust `fn`, an `impl`
 * block of a type, a JavaScript `function` or a `const` arrow.
 */
function definitionBodies(source, name) {
  const pattern = new RegExp(
    `^([ \\t]*)(?:pub(?:\\([^)]*\\))? )?(?:export )?(?:async )?(?:fn ${escape(name)}\\b|impl\\b[^\\n{]*\\b${escape(name)}\\b[^\\n{]*\\{|function ${escape(name)}\\(|const ${escape(name)} = )`,
    'gmu',
  );
  const bodies = [];
  for (const match of source.matchAll(pattern)) {
    const from = match.index;
    const end = match[0].includes(`const ${name} = `)
      ? source.indexOf(';\n', from)
      : source.indexOf(`\n${match[1]}}`, from);
    bodies.push(source.slice(from, end < 0 ? undefined : end));
  }
  return bodies;
}

function eventKinds(root, { file, marker }) {
  const source = readFileSync(join(root, file), 'utf8');
  const from = source.indexOf(marker);
  if (from < 0) return [];
  const open = source.indexOf('[', source.indexOf('=', from));
  const list = source.slice(open, source.indexOf(']', open));
  return [...list.matchAll(/['"](meta_exec:[a-z_]+)['"]/gu)].map((match) => match[1]);
}

function executedKinds(root) {
  const fixture = JSON.parse(readFileSync(join(root, PARITY_FIXTURE), 'utf8'));
  const kinds = new Set();
  for (const entry of fixture.cases) {
    for (const event of entry.expected?.events ?? []) kinds.add(event.split(' ')[0]);
  }
  return kinds;
}

/**
 * The divergences between the documents and the code.
 * @param {string} root repository root
 * @param {Record<string, string>} [texts] document texts by path (defaults to the files)
 * @returns {string[]}
 */
export function procedureDocDivergences(root, texts = {}) {
  const messages = [];
  const read = (file) => texts[file] ?? readFileSync(join(root, file), 'utf8');
  const blocks = DOCUMENTS.map((file) => [file, procedureBlock(read(file))]);
  for (const [file, block] of blocks) {
    if (block === null) messages.push(`${file}: no ${START} ... ${END} block`);
  }
  if (messages.length) return messages;
  if (blocks[0][1] !== blocks[1][1]) messages.push(`${DOCUMENTS[0]} and ${DOCUMENTS[1]} describe the procedure differently`);

  const sources = Object.fromEntries(Object.entries(RUNTIMES).map(([runtime, { files }]) => [
    runtime,
    files.map((file) => readFileSync(join(root, file), 'utf8')).join('\n'),
  ]));
  const bodiesOf = (runtime, name) => definitionBodies(sources[runtime], name);
  const executed = executedKinds(root);
  const found = tables(blocks[0][1]);
  const stages = found['Diagram stage'];
  const entry = found.Runtime;
  if (!stages) messages.push('the stage table (first header cell "Diagram stage") is missing');
  if (!entry) messages.push('the entry table (first header cell "Runtime") is missing');
  if (!stages || !entry) return messages;

  const documented = stages.rows.map((row) => codes(row[1])[0]);
  for (const [runtime, spec] of Object.entries(RUNTIMES)) {
    const kinds = eventKinds(root, spec.kinds);
    if (JSON.stringify(kinds) !== JSON.stringify(documented)) {
      messages.push(`the stage table lists ${documented.join(', ')}; ${runtime} emits ${kinds.join(', ')}`);
    }
    const column = stages.header.indexOf(runtime);
    if (column < 0) {
      messages.push(`the stage table has no ${runtime} column`);
      continue;
    }
    for (const row of stages.rows) {
      const event = codes(row[1])[0];
      const names = codes(row[column]);
      const bodies = names.map((name) => [name, bodiesOf(runtime, name)]);
      for (const [name, found] of bodies) {
        if (!found.length) messages.push(`${runtime}: \`${name}\` (stage ${row[0]}) is not defined`);
      }
      if (!bodies.some(([, found]) => found.some((body) => body.includes(`'${event}'`) || body.includes(`"${event}"`)))) {
        messages.push(`${runtime}: none of ${names.join(', ')} emits ${event}`);
      }
    }
  }
  for (const event of documented) {
    if (!executed.has(event)) messages.push(`${event} never occurs in the executed fixture ${PARITY_FIXTURE}`);
  }

  for (const row of entry.rows) {
    const runtime = row[0];
    if (!sources[runtime]) {
      messages.push(`the entry table names the unknown runtime ${runtime}`);
      continue;
    }
    const chain = codes(row[1]);
    if (chain.length < 2) messages.push(`${runtime}: the entry chain names fewer than two functions`);
    for (let index = 0; index + 1 < chain.length; index += 1) {
      const [caller, callee] = [chain[index], chain[index + 1]];
      const bodies = bodiesOf(runtime, caller);
      if (!bodies.length) {
        messages.push(`${runtime}: \`${caller}\` of the entry chain is not defined`);
      } else if (!bodies.some((body) => new RegExp(`\\b${escape(callee)}\\b`, 'u').test(body.slice(body.indexOf(caller) + caller.length)))) {
        messages.push(`${runtime}: \`${caller}\` does not call \`${callee}\``);
      }
    }
    for (const executor of codes(row[2] ?? '')) {
      if (!bodiesOf(runtime, executor).length) messages.push(`${runtime}: the executor \`${executor}\` is not defined`);
      const calls = chain.some((name) => bodiesOf(runtime, name)
        .some((body) => new RegExp(`\\b${escape(executor)}\\(`, 'u').test(body)));
      if (!calls) messages.push(`${runtime}: no function of the entry chain runs \`${executor}\``);
    }
  }
  return messages;
}
