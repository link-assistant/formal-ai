#!/usr/bin/env node
// The js -> rust leg of the js-first cycle (R994, R1000, R1012), through the
// upstream translator: link-foundation/meta-language's self-translation
// (`selfTranslate(source, 'JavaScript', 'Rust')`, merged in meta-language
// PR #196 on 2026-10-06 and not yet released), pinned to one exact commit of
// its main branch.
//
// Every module of the JavaScript root in scope (the agentic crate twins,
// js/agentic/crate/*.mjs, and the meta reasoner, js/worker/
// formal_ai_worker_meta_*.js) is translated item by item. Each top-level item
// is either translated (meta-language emitted Rust for it) or carried (the
// portable core cannot express it yet). The result is committed in two parts:
//
//   rust/tests/fixtures/js-rust-translation/<root>/<module>.rs
//       the projection of the upstream output: its provenance header and
//       prelude, every translated block byte for byte (marker, `// |` source
//       lines, emitted Rust), and for every carried item its marker line plus
//       one `// formal-ai:refusal <construct>` line naming the construct the
//       portable core refused (read from upstream `translateProgram`'s
//       diagnostic). Carried source lines are elided: the source is the
//       JavaScript module itself, whose SHA-256 the header records.
//   data/meta/js-rust-translation.lino
//       the pin, the totals, one row per module and the census of refusals.
//       The ratchet: a module's translated count never falls, so the
//       translated totals only grow and the refused set only shrinks.
//
// Upstream is never vendored: CI checks it out at the pinned commit
// (layered-ci.yml, the js-rust job) and `--fetch` downloads the same commit's
// tarball for a local run.
//
// Usage:
//   node scripts/translate-js-rust.mjs --verify
//       seconds, no upstream: every committed projection and ledger row is
//       for the current JavaScript source, and the CI pin matches the ledger
//   node scripts/translate-js-rust.mjs --check  (--meta-language DIR | --fetch) [--compile]
//       re-translate everything; the ledger and projections must be byte-equal,
//       and with --compile every projection compiles with rustc and the
//       calls of calls.lino give the expected result in translated Rust
//   node scripts/translate-js-rust.mjs --write  (--meta-language DIR | --fetch) [MODULE...]
//       regenerate (all modules, or only the named ones); refuses a regression
//   node scripts/translate-js-rust.mjs --why MODULE (--meta-language DIR | --fetch)
//       print every carried item of one module with the full upstream diagnostic
//
// FORMAL_AI_META_LANGUAGE names the checkout instead of --meta-language.

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { availableParallelism, tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { Worker, isMainThread, parentPort, workerData } from 'node:worker_threads';

const REPO = join(dirname(fileURLToPath(import.meta.url)), '..');
export const LEDGER_FILE = 'data/meta/js-rust-translation.lino';
export const PROJECTION_DIR = 'rust/tests/fixtures/js-rust-translation';
export const CALLS_FILE = `${PROJECTION_DIR}/calls.lino`;
const WORKFLOW_FILE = '.github/workflows/layered-ci.yml';
const UPSTREAM = 'link-foundation/meta-language';
/** The roots in scope: a directory, the file names it contributes, and the projection subdirectory. */
export const SCOPES = Object.freeze([
  Object.freeze({ dir: 'js/agentic/crate', name: /^[a-z0-9_]+\.mjs$/u, root: 'crate' }),
  Object.freeze({ dir: 'js/worker', name: /^formal_ai_worker_meta_[a-z0-9_]+\.js$/u, root: 'worker' }),
]);
const HEADER = '// meta-language:self-translation:v1 ';
const CARRIED = '// meta-language:carried ';
const TRANSLATED = '// meta-language:translated ';
const PRELUDE_BEGIN = '// meta-language:prelude begin';
const PRELUDE_END = '// meta-language:prelude end';
const SOURCE_LINE = '// |';
const REFUSAL = '// formal-ai:refusal ';
const PROJECTION_NOTE = '// formal-ai:projection translated blocks verbatim; carried items keep their marker and refused construct (scripts/translate-js-rust.mjs)';
// Upstream reasons that come with a portable-core diagnostic worth naming.
const DIAGNOSED = new Set(['unsupported', 'syntax', 'type']);

const sha256 = (text) => createHash('sha256').update(text, 'utf8').digest('hex');

// ---------------------------------------------------------------- scope

/**
 * The repository-relative paths of every tracked module in scope, sorted.
 * Untracked files are left out, so a measurement never names a file the
 * commit does not hold.
 * @param {string} repo
 * @returns {string[]}
 */
export function scopeModules(repo = REPO) {
  let tracked;
  try {
    tracked = execFileSync('git', ['ls-files', '--', ...SCOPES.map((scope) => scope.dir)], { cwd: repo, encoding: 'utf8' })
      .split('\n').filter(Boolean);
  } catch {
    tracked = SCOPES.flatMap((scope) => readdirSync(join(repo, scope.dir)).map((name) => `${scope.dir}/${name}`));
  }
  return tracked.filter((path) => scopeOf(path) !== null).sort();
}

/** The scope a module belongs to, or null. */
function scopeOf(path) {
  const slash = path.lastIndexOf('/');
  return SCOPES.find((scope) => scope.dir === path.slice(0, slash) && scope.name.test(path.slice(slash + 1))) ?? null;
}

/**
 * Where a module's projection is committed.
 * @param {string} path
 * @returns {string}
 */
export function projectionPath(path) {
  const scope = scopeOf(path);
  if (!scope) throw new Error(`${path} is outside the js -> rust scope`);
  const name = path.slice(path.lastIndexOf('/') + 1).replace(/\.m?js$/u, '.rs');
  return `${PROJECTION_DIR}/${scope.root}/${name}`;
}

// ---------------------------------------------------------------- projection

/**
 * The blocks of an upstream self-translation, in order: the header line, the
 * prelude lines, and one record per translated or carried item. A translated
 * block's code is the shortest run of lines after its source lines whose
 * SHA-256 is the one its marker records, so copied comments never join it.
 * @param {string} code
 */
export function upstreamBlocks(code) {
  const lines = code.split('\n');
  const blocks = [];
  let header = null;
  let prelude = [];
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (line.startsWith(HEADER)) {
      header = line;
    } else if (line === PRELUDE_BEGIN) {
      const end = lines.indexOf(PRELUDE_END, index);
      prelude = lines.slice(index, end + 1);
      index = end;
    } else if (line.startsWith(CARRIED) || line.startsWith(TRANSLATED)) {
      let last = index;
      while (lines[last + 1]?.startsWith(SOURCE_LINE)) last += 1;
      const source = lines.slice(index + 1, last + 1).map((text) => text.slice(SOURCE_LINE.length).replace(/^ /u, ''));
      if (line.startsWith(CARRIED)) {
        const match = /^\/\/ meta-language:carried (\S+) (\S+) \((.*)\)$/u.exec(line);
        blocks.push({ kind: 'carried', marker: line, term: match?.[2] ?? '', reason: match?.[3] ?? '', source: source.join('\n') });
        index = last;
      } else {
        const hash = /sha256=([0-9a-f]{64})/u.exec(line)?.[1];
        let end = last + 1;
        while (end <= lines.length && sha256(lines.slice(last + 1, end).join('\n')) !== hash) end += 1;
        if (end > lines.length) throw new Error(`translated block at line ${index + 1} has no code matching its hash`);
        blocks.push({ kind: 'translated', text: lines.slice(index, end).join('\n') });
        index = end - 1;
      }
    }
  }
  if (!header) throw new Error('the upstream output has no self-translation header');
  return { header, prelude, blocks };
}

/**
 * Normalizes an upstream refusal to the construct it names, without the
 * identifiers, literals and offsets of the one occurrence, so equal
 * constructs count together.
 * @param {string} reason the upstream carried reason
 * @param {string|null} message the portable-core diagnostic, when there is one
 * @returns {string}
 */
export function refusalClass(reason, message) {
  if (!DIAGNOSED.has(reason) || !message) return reason;
  const text = message.replace(/ at \d+\.\.\d+$/u, '');
  // A JSDoc type may itself hold ': ' (`{{ text: string }}`), so it is matched before the split.
  if (text.startsWith('JSDoc type {')) return 'JSDoc type {…}';
  if (reason === 'type') {
    if (/^unknown name crate\./u.test(text)) return 'type: unknown name (a sibling item or an import)';
    return `type: ${text.replace(/'[^']*'|"[^"]*"|`[^`]*`/gu, '…').replace(/\b[a-z]\w*[A-Z]\w*\b/gu, '…')}`;
  }
  if (reason === 'syntax') return `syntax: ${text.replace(/'[^']*'|"[^"]*"|`[^`]*`/gu, '…').replace(/\s+\S*\d\S*$/u, '')}`;
  const construct = text.slice(0, text.indexOf(': ') < 0 ? text.length : text.indexOf(': '));
  const rules = [
    [/^import \{.*\}$/u, 'import { … }'],
    [/^import from .*$/u, "import from '…'"],
    [/^JSDoc type \{.*\}$/u, 'JSDoc type {…}'],
    [/^function value .*$/u, 'function value …'],
    [/^(\.\w+\(\)) with \d+ arguments$/u, '$1 with … arguments'],
    [/^(assignment of constant|assignment of|default reading parameter|function expression|namespace property|namespace member|field type of|call of async function|@typedef|label|labelled|parameter|let|throw new|console) .+$/u, '$1 …'],
  ];
  for (const [pattern, replacement] of rules) if (pattern.test(construct)) return construct.replace(pattern, replacement);
  return construct.replace(/'[^']*'|"[^"]*"|`[^`]*`/gu, '…');
}

/**
 * The committed projection of one module's upstream output.
 * @param {{ header: string, prelude: string[], blocks: object[] }} parsed
 * @returns {string}
 */
export function renderProjection(parsed) {
  const parts = [[parsed.header, PROJECTION_NOTE].join('\n')];
  if (parsed.prelude.length) parts.push(parsed.prelude.join('\n'));
  for (const block of parsed.blocks) {
    parts.push(block.kind === 'translated' ? block.text : `${block.marker}\n${REFUSAL}${block.refusal}`);
  }
  return `${parts.join('\n\n')}\n`;
}

/**
 * Reads a committed projection back: its header fields and its blocks.
 * @param {string} text
 */
export function readProjection(text) {
  const { header, blocks } = upstreamBlocks(text);
  const fields = Object.fromEntries([...header.matchAll(/(\w+)=(\S+)/gu)].map((match) => [match[1], match[2]]));
  const translated = blocks.filter((block) => block.kind === 'translated');
  const refusals = [...text.matchAll(/^\/\/ formal-ai:refusal (.*)$/gmu)].map((match) => match[1]);
  return {
    header,
    sha256: fields.sha256 ?? null,
    bytes: Number(fields.bytes ?? Number.NaN),
    translated: translated.length,
    inferred: translated.filter((block) => inferredSignature(block.text)).length,
    carried: refusals.length,
    refusals,
  };
}

/**
 * Whether a translated block's JavaScript leaves a parameter without a JSDoc
 * `@param` type, so meta-language inferred its Rust type from the item's own
 * body (where nothing constrains a parameter, it becomes a Number). Read from
 * the block's `// |` source lines: every parameter list of a function
 * declaration, an arrow function bound to a name, or a namespace method.
 * @param {string} block
 * @returns {boolean}
 */
export function inferredSignature(block) {
  const source = block.split('\n').filter((line) => line.startsWith(SOURCE_LINE)).map((line) => line.slice(SOURCE_LINE.length)).join('\n');
  const declared = new Set([...source.matchAll(/@param\s+\{[^}]*\}\s+\[?([A-Za-z_$][\w$]*)/gu)].map((match) => match[1]));
  const lists = [
    ...source.matchAll(/\bfunction\s*\*?\s*[\w$]*\s*\(([^)]*)\)/gu),
    ...source.matchAll(/=\s*(?:async\s+)?\(([^)]*)\)\s*=>/gu),
    ...source.matchAll(/=\s*(?:async\s+)?([A-Za-z_$][\w$]*)\s*=>/gu),
    ...source.matchAll(/^\s+(?!if\b|for\b|while\b|switch\b|return\b)[A-Za-z_$][\w$]*\s*\(([^)]*)\)\s*\{/gmu),
  ];
  const params = lists.flatMap((match) => match[1].split(',').map((param) => param.replace(/=.*$/su, '').trim()).filter(Boolean));
  return params.some((param) => !declared.has(param));
}

// ---------------------------------------------------------------- ledger

/**
 * Renders the ledger from the measured modules.
 * @param {string} commit
 * @param {{ path: string, translated: number, carried: number, refusals: string[] }[]} modules
 * @returns {string}
 */
export function renderLedger(commit, modules) {
  const sum = (field) => modules.reduce((total, module) => total + module[field], 0);
  const census = new Map();
  for (const module of modules) for (const refusal of module.refusals) census.set(refusal, (census.get(refusal) ?? 0) + 1);
  const quote = (text) => `"${text.replace(/"/gu, "'")}"`;
  const lines = [
    '# The js -> rust leg of the js-first cycle (R994, R1000, R1012), measured with',
    '# link-foundation/meta-language self-translation at the pinned main commit.',
    '#',
    '# Generated by `node scripts/translate-js-rust.mjs --write`; never edited by',
    '# hand. Each module row counts its top-level items (with their doc comments)',
    '# that meta-language translated to Rust and the ones it carried. The ratchet:',
    '# a module\'s `translated` never falls (`--write` refuses it), so',
    '# `translated_items` and `translated_modules` only grow and',
    '# `refused_modules` (modules with nothing translated) only shrinks. The',
    '# projections in rust/tests/fixtures/js-rust-translation/ hold the Rust; the',
    '# refusal census counts the constructs that keep the carried items out.',
    'js_rust_translation',
    `  upstream ${UPSTREAM}`,
    `  commit ${commit}`,
    `  modules ${modules.length}`,
    `  translated_items ${sum('translated')}`,
    `  inferred_signatures ${sum('inferred')}`,
    `  carried_items ${sum('carried')}`,
    `  translated_modules ${modules.filter((module) => module.translated > 0).length}`,
    `  fully_translated_modules ${modules.filter((module) => module.translated > 0 && module.carried === 0).length}`,
    `  refused_modules ${modules.filter((module) => module.translated === 0).length}`,
    ...modules.map((module) => `module ${module.path} translated ${module.translated} inferred ${module.inferred} carried ${module.carried}`),
    ...[...census].sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1)).map(([refusal, count]) => `refusal ${quote(refusal)} ${count}`),
  ];
  return `${lines.join('\n')}\n`;
}

/**
 * Parses the ledger.
 * @param {string} text
 */
export function parseLedger(text) {
  const field = (name) => {
    const match = new RegExp(`^  ${name} (\\S+)$`, 'mu').exec(text);
    if (!match) throw new Error(`${LEDGER_FILE}: ${name} missing`);
    return match[1];
  };
  const modules = [...text.matchAll(/^module (\S+) translated (\d+) inferred (\d+) carried (\d+)$/gmu)]
    .map((match) => ({ path: match[1], translated: Number(match[2]), inferred: Number(match[3]), carried: Number(match[4]) }));
  const refusals = [...text.matchAll(/^refusal "(.*)" (\d+)$/gmu)].map((match) => ({ refusal: match[1], count: Number(match[2]) }));
  const totals = Object.fromEntries(['modules', 'translated_items', 'inferred_signatures', 'carried_items', 'translated_modules', 'fully_translated_modules', 'refused_modules']
    .map((name) => [name, Number(field(name))]));
  return { upstream: field('upstream'), commit: field('commit'), totals, modules, refusals };
}

/**
 * The modules whose translated count fell against the committed ledger.
 * Modules that left the scope are not regressions: their source is gone.
 * @param {{ path: string, translated: number }[]} before
 * @param {{ path: string, translated: number }[]} after
 * @returns {string[]}
 */
export function regressions(before, after) {
  const now = new Map(after.map((module) => [module.path, module]));
  return before.flatMap((module) => {
    const current = now.get(module.path);
    if (!current) return [];
    return [
      ...(current.translated < module.translated ? [`${module.path}: translated ${module.translated} -> ${current.translated}`] : []),
      ...(current.inferred > module.inferred ? [`${module.path}: inferred signatures ${module.inferred} -> ${current.inferred} (give every parameter a JSDoc @param type)`] : []),
    ];
  });
}

/**
 * The commit the js-rust job checks meta-language out at, read from the workflow.
 * @param {string} workflow
 * @returns {string|null}
 */
export function workflowPin(workflow) {
  const match = /repository: link-foundation\/meta-language\n\s+ref: ([0-9a-f]{40})/u.exec(workflow);
  return match?.[1] ?? null;
}

// ---------------------------------------------------------------- verify (no upstream)

/**
 * Checks the committed state against the current sources, without the
 * upstream translator: every module in scope has a ledger row and a
 * projection recorded from its current bytes, the rows match their
 * projections, the totals are the sums of the rows, and the CI pin is the
 * ledger's commit. Returns the problems found.
 * @param {string} repo
 * @returns {string[]}
 */
export function verify(repo = REPO) {
  const problems = [];
  const ledger = parseLedger(readFileSync(join(repo, LEDGER_FILE), 'utf8'));
  const pin = workflowPin(readFileSync(join(repo, WORKFLOW_FILE), 'utf8'));
  if (pin !== ledger.commit) problems.push(`${WORKFLOW_FILE} checks out meta-language at ${pin}, the ledger is measured at ${ledger.commit}`);
  const modules = scopeModules(repo);
  const rows = new Map(ledger.modules.map((module) => [module.path, module]));
  for (const path of modules) if (!rows.has(path)) problems.push(`${path} has no ledger row`);
  const expected = new Set();
  const census = new Map();
  for (const row of ledger.modules) {
    if (!modules.includes(row.path)) {
      problems.push(`${row.path} has a ledger row but is not a tracked module in scope`);
      continue;
    }
    const projection = projectionPath(row.path);
    expected.add(projection);
    if (!existsSync(join(repo, projection))) {
      problems.push(`${projection} is missing`);
      continue;
    }
    const read = readProjection(readFileSync(join(repo, projection), 'utf8'));
    const source = readFileSync(join(repo, row.path));
    if (read.sha256 !== createHash('sha256').update(source).digest('hex') || read.bytes !== source.length) {
      problems.push(`${row.path} changed since its translation`);
    }
    if (read.translated !== row.translated || read.inferred !== row.inferred || read.carried !== row.carried) {
      problems.push(`${projection} holds ${read.translated} translated (${read.inferred} inferred) and ${read.carried} carried items, the ledger ${row.translated} (${row.inferred}) and ${row.carried}`);
    }
    for (const refusal of read.refusals) census.set(refusal, (census.get(refusal) ?? 0) + 1);
  }
  for (const scope of SCOPES) {
    const dir = join(repo, PROJECTION_DIR, scope.root);
    for (const name of existsSync(dir) ? readdirSync(dir) : []) {
      const path = `${PROJECTION_DIR}/${scope.root}/${name}`;
      if (!expected.has(path)) problems.push(`${path} is not the projection of a module in scope`);
    }
  }
  const rendered = parseLedger(renderLedger(ledger.commit, ledger.modules.map((row) => ({
    ...row, refusals: readProjectionRefusals(repo, row.path),
  }))));
  for (const [name, value] of Object.entries(rendered.totals)) {
    if (ledger.totals[name] !== value) problems.push(`${LEDGER_FILE}: ${name} is ${ledger.totals[name]}, the rows sum to ${value}`);
  }
  const censusText = JSON.stringify([...census].sort());
  if (censusText !== JSON.stringify(ledger.refusals.map((row) => [row.refusal, row.count]).sort())) {
    problems.push(`${LEDGER_FILE}: the refusal census is not the count of the projections' refusal lines`);
  }
  return problems;
}

function readProjectionRefusals(repo, path) {
  const file = join(repo, projectionPath(path));
  return existsSync(file) ? readProjection(readFileSync(file, 'utf8')).refusals : [];
}

// ---------------------------------------------------------------- upstream

/**
 * The meta-language checkout to translate with, verified to be the pinned
 * commit: a git checkout's HEAD, or the marker `--fetch` writes.
 * @param {string[]} argv
 * @param {string} commit
 * @returns {string}
 */
function upstreamDirectory(argv, commit) {
  const named = argv.includes('--meta-language') ? argv[argv.indexOf('--meta-language') + 1] : process.env.FORMAL_AI_META_LANGUAGE;
  const dir = argv.includes('--fetch') ? fetchUpstream(commit) : named;
  if (!dir) throw new Error('name the meta-language checkout with --meta-language DIR (or FORMAL_AI_META_LANGUAGE), or pass --fetch');
  const marker = join(dir, '.formal-ai-meta-language-commit');
  const head = existsSync(join(dir, '.git'))
    ? execFileSync('git', ['-C', dir, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim()
    : (existsSync(marker) ? readFileSync(marker, 'utf8').trim() : null);
  if (head !== commit) throw new Error(`${dir} is meta-language ${head ?? '(unknown commit)'}; the ledger pins ${commit}`);
  if (!existsSync(join(dir, 'js', 'node_modules'))) {
    execFileSync('npm', ['ci', '--prefix', join(dir, 'js'), '--omit=dev', '--ignore-scripts', '--no-audit', '--no-fund'], { stdio: 'inherit' });
  }
  return dir;
}

/** Downloads the pinned commit's tarball once into the temporary directory. */
function fetchUpstream(commit) {
  const dir = join(tmpdir(), 'formal-ai-meta-language', commit);
  if (existsSync(join(dir, '.formal-ai-meta-language-commit'))) return dir;
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  const archive = join(dir, '..', `${commit}.tar.gz`);
  execFileSync('curl', ['-fsSL', '-o', archive, `https://codeload.github.com/${UPSTREAM}/tar.gz/${commit}`], { stdio: 'inherit' });
  execFileSync('tar', ['-xzf', archive, '-C', dir, '--strip-components=1'], { stdio: 'inherit' });
  rmSync(archive, { force: true });
  execFileSync('npm', ['ci', '--prefix', join(dir, 'js'), '--omit=dev', '--ignore-scripts', '--no-audit', '--no-fund'], { stdio: 'inherit' });
  writeFileSync(join(dir, '.formal-ai-meta-language-commit'), `${commit}\n`);
  return dir;
}

/**
 * Translates the modules on a pool of worker threads, largest first.
 * @param {string} dir the meta-language checkout
 * @param {string[]} paths
 * @param {number} jobs
 * @returns {Promise<Map<string, { code: string, details: (string|null)[] }>>}
 */
function translateAll(dir, paths, jobs) {
  const queue = paths.map((path) => ({ path, text: readFileSync(join(REPO, path), 'utf8') }))
    .sort((a, b) => b.text.length - a.text.length);
  const results = new Map();
  const total = queue.length;
  return new Promise((resolve, reject) => {
    let running = 0;
    const start = () => {
      const worker = new Worker(fileURLToPath(import.meta.url), { workerData: { role: 'translate', dir } });
      running += 1;
      const next = () => {
        const task = queue.shift();
        if (task) worker.postMessage(task);
        else worker.terminate();
      };
      worker.on('message', (message) => {
        results.set(message.path, message);
        process.stderr.write(`translated ${results.size}/${total} ${message.path} (${message.ms} ms)\n`);
        next();
      });
      worker.on('error', reject);
      worker.on('exit', () => {
        running -= 1;
        if (running === 0) {
          if (results.size === total) resolve(results);
          else reject(new Error(`${total - results.size} module(s) were not translated`));
        }
      });
      next();
    };
    for (let index = 0; index < Math.max(1, Math.min(jobs, total)); index += 1) start();
  });
}

async function workerMain() {
  const api = await import(pathToFileURL(join(workerData.dir, 'js', 'src', 'index.js')).href);
  parentPort.on('message', ({ path, text }) => {
    const started = Date.now();
    const { code } = api.selfTranslate(text, 'JavaScript', 'Rust');
    // The portable-core diagnostic of each carried item, from the public
    // translateProgram API: selfTranslate keeps only its kind.
    const details = upstreamBlocks(code).blocks.map((block) => (block.kind === 'carried' && DIAGNOSED.has(block.reason)
      ? (api.translateProgram(block.source, 'JavaScript', 'Rust').diagnostic?.message ?? null)
      : null));
    parentPort.postMessage({ path, code, details, ms: Date.now() - started });
  });
}

/**
 * The measured module and its projection from one upstream translation.
 * @param {string} path
 * @param {{ code: string, details: (string|null)[] }} result
 */
export function measure(path, result) {
  const parsed = upstreamBlocks(result.code);
  parsed.blocks.forEach((block, index) => {
    if (block.kind === 'carried') block.refusal = refusalClass(block.reason, result.details[index]);
  });
  const carried = parsed.blocks.filter((block) => block.kind === 'carried');
  return {
    path,
    translated: parsed.blocks.length - carried.length,
    inferred: parsed.blocks.filter((block) => block.kind === 'translated' && inferredSignature(block.text)).length,
    carried: carried.length,
    refusals: carried.map((block) => block.refusal),
    projection: renderProjection(parsed),
  };
}

// ---------------------------------------------------------------- equivalence

/**
 * Parses calls.lino: `(call <module> <javascript function> <rust function>
 * (arguments (<type> <value>)…) (result (<type> <value>)))`. Values are
 * percent-encoded; types are `str`, `f64`, `bool` and `f64s` (a
 * comma-separated list of numbers).
 * @param {string} text
 */
export function parseCalls(text) {
  return text.split('\n').filter((line) => line.startsWith('(call ')).map((line) => {
    const match = /^\(call (\S+) (\S+) (\S+) \(arguments((?: \(\w+ [^()\s]*\))*)\) \(result \((\w+) ([^()\s]*)\)\)\)$/u.exec(line);
    if (!match) throw new Error(`${CALLS_FILE}: malformed call ${line}`);
    const args = [...match[4].matchAll(/\((\w+) ([^()\s]*)\)/gu)].map((arg) => callValue(arg[1], arg[2]));
    return { module: match[1], javascript: match[2], rust: match[3], args, result: callValue(match[5], match[6]) };
  });
}

function callValue(type, raw) {
  const text = decodeURIComponent(raw);
  if (type === 'str') return { type, value: text };
  if (type === 'f64') return { type, value: Number(text) };
  if (type === 'bool') return { type, value: text === 'true' };
  if (type === 'f64s') return { type, value: text === '' ? [] : text.split(',').map(Number) };
  throw new Error(`${CALLS_FILE}: unknown value type ${type}`);
}

/**
 * Reads a Rust string literal's value (the escapes meta-language emits).
 * @param {string} literal
 * @returns {string}
 */
export function rustStringValue(literal) {
  const SIMPLE = { n: '\n', r: '\r', t: '\t', '0': '\0', '\\': '\\', '"': '"', "'": "'" };
  return literal.slice(1, -1).replace(/\\(u\{([0-9a-fA-F]+)\}|x([0-9a-fA-F]{2})|.)/gsu, (_, escape, unicode, hex) => {
    if (unicode) return String.fromCodePoint(Number.parseInt(unicode, 16));
    if (hex) return String.fromCharCode(Number.parseInt(hex, 16));
    if (!(escape in SIMPLE)) throw new Error(`unknown Rust escape \\${escape}`);
    return SIMPLE[escape];
  });
}

/**
 * The exported constants a projection translated: `export const NAME = …`
 * items whose Rust is `pub const NAME: &str | f64 | bool = <literal>;`, with
 * the value the Rust literal denotes.
 * @param {string} projection
 * @returns {{ name: string, type: string, value: string|number|boolean }[]}
 */
export function translatedConstants(projection) {
  return upstreamBlocks(projection).blocks
    .filter((block) => block.kind === 'translated' && block.text.startsWith(`${TRANSLATED}JavaScript export_statement `))
    .flatMap((block) => [...block.text.matchAll(/^pub const ([A-Z][A-Z0-9_]*): (&str|f64|bool) = (.+);$/gmu)])
    .map(([, name, type, literal]) => ({
      name,
      type,
      value: type === '&str' ? rustStringValue(literal) : type === 'bool' ? literal === 'true' : Number(literal.replace(/f64$/u, '')),
    }));
}

/** A Rust string literal for `text`. */
function rustString(text) {
  return `"${[...text].map((char) => {
    const code = char.codePointAt(0);
    if (char === '\\' || char === '"') return `\\${char}`;
    if (char === '\n') return '\\n';
    if (char === '\t') return '\\t';
    if (code < 0x20 || code > 0x7e) return `\\u{${code.toString(16)}}`;
    return char;
  }).join('')}"`;
}

const rustNumber = (value) => `${Number.isInteger(value) ? `${value}.0` : value}_f64`;

function rustValue(value, type) {
  if (value.type === 'str') return type === 'String' ? `String::from(${rustString(value.value)})` : rustString(value.value);
  if (value.type === 'f64') return rustNumber(value.value);
  if (value.type === 'f64s') return `vec![${value.value.map(rustNumber).join(', ')}]`;
  return String(value.value);
}

/**
 * The `main` that holds a projection's translated Rust to its JavaScript:
 * every translated exported constant equals the value the JavaScript module
 * exports, and every call of calls.lino returns its recorded result.
 * Returns the statements and the problems found while writing them.
 */
async function equivalenceMain(path, projection, calls) {
  const problems = [];
  const statements = [];
  if (path.endsWith('.mjs')) {
    const exported = await import(pathToFileURL(join(REPO, path)).href);
    for (const constant of translatedConstants(projection)) {
      if (!(constant.name in exported)) continue;
      const value = exported[constant.name];
      const kind = { '&str': 'string', f64: 'number', bool: 'boolean' }[constant.type];
      if (typeof value !== kind) {
        problems.push(`${path}: ${constant.name} is a ${typeof value} in JavaScript and ${constant.type} in the translation`);
      } else if (!Number.isNaN(value)) {
        const literal = kind === 'string' ? rustString(value) : kind === 'number' ? rustNumber(value) : String(value);
        statements.push(`    assert_eq!(${constant.name}, ${literal}, ${rustString(constant.name)});`);
      }
    }
  }
  for (const call of calls.filter((entry) => entry.module === path)) {
    const signature = new RegExp(`^pub fn ${call.rust}\\(([^)]*)\\) -> (\\S+) \\{$`, 'mu').exec(projection);
    if (!signature) {
      problems.push(`${CALLS_FILE}: ${call.rust} is not a translated function of ${path}`);
      continue;
    }
    const types = signature[1] ? signature[1].split(', ').map((param) => param.slice(param.indexOf(': ') + 2)) : [];
    const args = call.args.map((arg, index) => rustValue(arg, types[index])).join(', ');
    const label = `${call.rust}(${call.args.map((arg) => JSON.stringify(arg.value)).join(', ')})`;
    statements.push(`    assert_eq!(${call.rust}(${args}), ${rustValue(call.result, '&str')}, ${rustString(label)});`);
  }
  return { statements, problems };
}

/**
 * Compiles every projection with rustc (a library, metadata only), then
 * each one with equivalence assertions once more as a program and runs it.
 * Returns the problems found.
 * @param {[string, string][]} projections module path and projection text
 */
async function compileProjections(projections) {
  const problems = [];
  const scratch = mkdtempSync(join(tmpdir(), 'formal-ai-js-rust-'));
  const calls = parseCalls(readFileSync(join(REPO, CALLS_FILE), 'utf8'));
  let compiled = 0;
  let asserted = 0;
  try {
    for (const [path, text] of projections) {
      const file = join(scratch, projectionPath(path).replaceAll('/', '_'));
      writeFileSync(file, text);
      try {
        execFileSync('rustc', ['--edition', '2024', '--crate-type', 'lib', '--emit=metadata', '-o', `${file}.rmeta`, file], { stdio: 'pipe' });
        compiled += 1;
      } catch (error) {
        problems.push(`${projectionPath(path)} does not compile:\n${error.stderr}`);
        continue;
      }
      const main = await equivalenceMain(path, text, calls);
      problems.push(...main.problems);
      if (main.statements.length === 0) continue;
      writeFileSync(`${file}.main.rs`, `${text}\nfn main() {\n${main.statements.join('\n')}\n}\n`);
      try {
        execFileSync('rustc', ['--edition', '2024', '--crate-name', 'translated', '-o', `${file}.bin`, `${file}.main.rs`], { stdio: 'pipe' });
        execFileSync(`${file}.bin`, { stdio: 'pipe' });
        asserted += main.statements.length;
      } catch (error) {
        problems.push(`the translated Rust of ${path} disagrees with its JavaScript:\n${error.stderr ?? error.message}`);
      }
    }
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
  console.log(`rustc: ${compiled}/${projections.length} projections compile; ${asserted} constant and call assertions hold in translated Rust`);
  return problems;
}

// ---------------------------------------------------------------- main

function option(argv, name, fallback) {
  return argv.includes(name) ? argv[argv.indexOf(name) + 1] : fallback;
}

async function main(argv) {
  const committed = parseLedger(readFileSync(join(REPO, LEDGER_FILE), 'utf8'));
  if (argv.includes('--verify')) {
    const problems = verify();
    for (const problem of problems) console.error(`::error::${problem}`);
    if (problems.length) {
      const stale = problems.map((problem) => /^(\S+) (?:changed since its translation|has no ledger row)$/u.exec(problem)?.[1]).filter(Boolean);
      console.error(`regenerate with: node scripts/translate-js-rust.mjs --write --fetch ${stale.join(' ')}`.trimEnd());
      return 1;
    }
    const { totals } = committed;
    console.log(`js -> rust: ${totals.translated_items} items translated, ${totals.carried_items} carried, ${totals.translated_modules}/${totals.modules} modules translate (${totals.fully_translated_modules} fully), ${totals.refused_modules} refused; meta-language ${committed.commit}`);
    return 0;
  }
  if (argv.includes('--compile') && !argv.includes('--check')) {
    // The committed projections alone: rustc, no upstream checkout.
    const problems = await compileProjections(scopeModules().map((path) => [path, readFileSync(join(REPO, projectionPath(path)), 'utf8')]));
    for (const problem of problems) console.error(`::error::${problem}`);
    return problems.length ? 1 : 0;
  }
  const dir = upstreamDirectory(argv, committed.commit);
  const jobs = Number(option(argv, '--jobs', Math.min(4, availableParallelism())));
  if (argv.includes('--why')) {
    const path = option(argv, '--why');
    const result = (await translateAll(dir, [path], 1)).get(path);
    const parsed = upstreamBlocks(result.code);
    parsed.blocks.forEach((block, index) => {
      if (block.kind === 'carried') console.log(`${block.term} (${block.reason}) ${result.details[index] ?? ''}\n  ${block.source.split('\n').find((line) => !/^\s*(\/\/|\/\*|\*)/u.test(line)) ?? ''}`);
    });
    return 0;
  }
  const scope = scopeModules();
  const named = argv.filter((arg, index) => !arg.startsWith('--') && !['--meta-language', '--jobs'].includes(argv[index - 1]));
  const write = argv.includes('--write');
  const targets = write && named.length ? named : scope;
  for (const path of targets) if (!scope.includes(path)) throw new Error(`${path} is not a tracked module in scope`);
  const results = await translateAll(dir, targets, jobs);
  const measured = new Map(targets.map((path) => [path, measure(path, results.get(path))]));
  // Modules not re-translated keep their committed measurement.
  const modules = scope.map((path) => {
    if (measured.has(path)) return measured.get(path);
    const row = committed.modules.find((entry) => entry.path === path);
    if (!row) throw new Error(`${path} has no committed measurement; name it: --write ${path}`);
    return { ...row, refusals: readProjectionRefusals(REPO, path), projection: readFileSync(join(REPO, projectionPath(path)), 'utf8') };
  });
  const ledger = renderLedger(committed.commit, modules);
  const fell = regressions(committed.modules, modules);
  if (fell.length) {
    for (const line of fell) console.error(`::error::translation regressed: ${line}`);
    console.error('a module written in the portable subset must stay in it (R1000); restore the translated items');
    return 1;
  }
  if (write) {
    for (const module of modules) {
      const file = join(REPO, projectionPath(module.path));
      mkdirSync(dirname(file), { recursive: true });
      writeFileSync(file, module.projection);
    }
    for (const scopeEntry of SCOPES) {
      const sub = join(REPO, PROJECTION_DIR, scopeEntry.root);
      for (const name of existsSync(sub) ? readdirSync(sub) : []) {
        const path = `${PROJECTION_DIR}/${scopeEntry.root}/${name}`;
        if (!modules.some((module) => projectionPath(module.path) === path)) rmSync(join(REPO, path));
      }
    }
    writeFileSync(join(REPO, LEDGER_FILE), ledger);
  } else {
    const problems = [];
    if (ledger !== readFileSync(join(REPO, LEDGER_FILE), 'utf8')) problems.push(`${LEDGER_FILE} differs from the translation`);
    for (const module of modules) {
      const file = join(REPO, projectionPath(module.path));
      if (!existsSync(file) || readFileSync(file, 'utf8') !== module.projection) problems.push(`${projectionPath(module.path)} differs from the translation`);
    }
    problems.push(...verify());
    if (argv.includes('--compile')) problems.push(...await compileProjections(modules.map((module) => [module.path, module.projection])));
    for (const problem of problems) console.error(`::error::${problem}`);
    if (problems.length) {
      console.error(`regenerate with: node scripts/translate-js-rust.mjs --write --fetch (meta-language ${committed.commit})`);
      return 1;
    }
  }
  const totals = parseLedger(ledger).totals;
  console.log(`js -> rust: ${totals.translated_items} items translated, ${totals.carried_items} carried, ${totals.translated_modules}/${totals.modules} modules translate (${totals.fully_translated_modules} fully), ${totals.refused_modules} refused`);
  return 0;
}

if (!isMainThread && workerData?.role === 'translate') {
  await workerMain();
} else if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = await main(process.argv.slice(2));
}
