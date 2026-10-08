// PR #1188 (SCRIPTS-B): the JavaScript twins of the five rust-script checks
// agents hit most -- check-file-size, check-debt-ratchet,
// check-hardcoded-language, check-minimal-core-boundary and
// check-worker-line-budget -- print what the Rust originals print. Each case
// builds a small tree with one deliberate violation and pins the exact text
// the Rust source formats for it (stdout, stderr and exit code), then the
// clean case. CI also runs both halves on the real tree and diffs them
// (data/meta/ci-gates/check-*-js-twin.lino, scripts/lib/checks-twin-parity.mjs).

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  canonicalizeNormal, canonicalizeRaw, extractLiterals, isMultiWordPhrase, isUserFacingLiteral,
  isUserFacingProse, parseAllowlist, renderAllowlist,
} from '../../../scripts/check-hardcoded-language.mjs';
import { fileLimit, growingPathsFromNumstat } from '../../../scripts/check-file-size.mjs';
import { announcesCorrection, checkAgainstPrevious, parseRatchet } from '../../../scripts/check-debt-ratchet.mjs';
import { leadingSummary, shardName } from '../../../scripts/check-worker-line-budget.mjs';
import { compareRuns } from '../../../scripts/lib/checks-twin-parity.mjs';
import {
  compareStrings, debugString, extension, lineCount, lines, parseUnsigned, trim,
} from '../../../scripts/lib/checks-rust-compat.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');

/** A temporary tree holding `files` (path -> text), outside any git work tree. */
function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), 'scripts-js-twins-checks-'));
  for (const [path, text] of Object.entries(files)) write(root, path, text);
  return root;
}

function write(root, path, text) {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), text);
}

const numbered = (count) => Array.from({ length: count }, (_, index) => `// line ${index + 1}\n`).join('');

/** Run a twin from `cwd` with the CI-only variables cleared. */
function run(script, args, cwd) {
  const env = { ...process.env };
  for (const name of ['FILE_SIZE_WARNING_BASE', 'GITHUB_BASE_REF', 'GIT_DIR', 'GIT_WORK_TREE']) delete env[name];
  const result = spawnSync(process.execPath, [join(ROOT, 'scripts', script), ...args], { cwd, env, encoding: 'utf8' });
  return { status: result.status, stdout: result.stdout, stderr: result.stderr };
}

describe('Rust standard-library semantics the twins share', () => {
  test('str::lines and its count drop one trailing newline and a carriage return', () => {
    assert.deepEqual(lines('a\r\nb\n'), ['a', 'b']);
    assert.deepEqual(lines(''), []);
    assert.deepEqual(lines('\n'), ['']);
    assert.equal(lineCount('a\nb'), 2);
    assert.equal(lineCount('a\nb\n'), 2);
    assert.equal(lineCount(''), 0);
  });

  test('trim is Unicode White_Space, not JavaScript \\s', () => {
    assert.equal(trim('\u0085 a \u00a0'), 'a');
    assert.equal(trim('\ufeffa'), '\ufeffa');
  });

  test('strings order by code point, as UTF-8 bytes do', () => {
    assert.ok(compareStrings('\uffff', '\u{1f600}') < 0);
    assert.ok(compareStrings('a', 'ab') < 0);
  });

  test('Path::extension, {:?} and ParseIntError read as Rust prints them', () => {
    assert.equal(extension('a/.gitignore'), null);
    assert.equal(extension('a/b.tar.gz'), 'gz');
    assert.equal(debugString('a"b\\c\nd\u00a0'), '"a\\"b\\\\c\\nd\\u{a0}"');
    assert.deepEqual(parseUnsigned('+7'), { value: 7 });
    assert.deepEqual(parseUnsigned(''), { error: 'cannot parse integer from empty string' });
    assert.deepEqual(parseUnsigned('-1'), { error: 'invalid digit found in string' });
    assert.deepEqual(parseUnsigned('18446744073709551616'), { error: 'number too large to fit in target type' });
  });

  test('the parity runner reports exit code and stream differences', () => {
    const same = { status: 0, stdout: 'a\n', stderr: '' };
    assert.deepEqual(compareRuns(same, { ...same }), []);
    assert.deepEqual(compareRuns(same, { ...same, status: 1, stdout: 'b\n' }), [
      'exit code: rust 0, js 1',
      'stdout differs at line 1:\n  rust: "a"\n  js:   "b"',
    ]);
  });
});

describe('check-file-size.mjs', () => {
  test('limits follow the Rust table', () => {
    assert.equal(fileLimit('js/app/main.jsx').label, 'Maintained');
    assert.equal(fileLimit('vscode/package-lock.json'), null);
    assert.equal(fileLimit('js/vendor.bundle.js'), null);
    assert.equal(fileLimit('data/cache/wikidata/lexeme/L3302.json'), null);
    assert.equal(fileLimit('/r/js/worker/formal_ai_worker_seed_responses_and_language.js').label, 'Worker JavaScript');
    assert.equal(fileLimit('/r/.github/workflows/release.yml').label, 'GitHub Actions workflow');
    assert.deepEqual(growingPathsFromNumstat('12\t0\tsrc/growing.rs\n3\t3\tsrc/same.rs\n1\t8\tsrc/shrinking.rs\n-\t-\ta.png\n'), ['src/growing.rs']);
  });

  test('a warning, a violation and embedded worker data print as the Rust script prints them', () => {
    const root = fixture({
      'src/near_limit.rs': numbered(901),
      'src/over_limit.rs': numbered(1001),
      'js/worker/formal_ai_worker_seed_responses_and_language.js': 'const MEANINGS_LINO = [\n  "meaning fact",\n].join("\\n");\n',
      'docs/case-studies/issue-561/release.yml': numbered(1501),
      'dev/log/issues/798/big.rs': numbered(1001),
      'target/debug/big.rs': numbered(1001),
    });
    try {
      const result = run('check-file-size.mjs', [], root);
      assert.equal(result.status, 1);
      assert.equal(result.stderr, '');
      assert.equal(result.stdout, [
        '',
        'Checking file line limits: Rust 1000, every other maintained text format 1500...',
        '',
        '::warning file=src/near_limit.rs::Rust file has 901 lines (approaching limit of 1000). Consider extracting content to keep at or below 900 lines and prevent review and merge conflicts.',
        'WARNING: src/near_limit.rs has 901 lines (approaching Rust limit of 1000, warning threshold: 900)',
        '',
        'The following files are approaching their configured line limits:',
        '  src/near_limit.rs',
        '',
        'Consider extracting code to prevent concurrent PR merge limit violations.',
        '',
        'Found files exceeding the line limit:',
        '',
        '  src/over_limit.rs: 1001 lines (exceeds Rust limit of 1000)',
        '',
        'Please refactor or split these files to stay under their limits',
        '',
        'Found embedded Links Notation data in worker JavaScript:',
        '',
        '  js/worker/formal_ai_worker_seed_responses_and_language.js:1: Worker JavaScript must load Links Notation data from data/seed via seed_loader.js, not embed _LINO arrays or template literals.',
        '',
        'Move worker seed data to data/seed and load it through seed_loader.js',
        '',
        '',
      ].join('\n'));
      rmSync(join(root, 'src/over_limit.rs'));
      rmSync(join(root, 'js'), { recursive: true });
      const clean = run('check-file-size.mjs', [], root);
      assert.equal(clean.status, 0);
      assert.ok(clean.stdout.endsWith('All checked files are within their line limits\n\n'));
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

describe('check-hardcoded-language.mjs', () => {
  test('the prose heuristics match the Rust unit tests', () => {
    assert.ok(isUserFacingProse("Sorry, I can't do that."));
    assert.ok(isUserFacingProse('Test passed. I\'m here.'));
    for (const text of ['greeting', 'Done.', 'en ru hi zh', '{} {}.', '{informal} -> {formal}.']) {
      assert.ok(!isUserFacingProse(text), text);
    }
    assert.ok(!isMultiWordPhrase('find {path}'));
    const detected = (source) => extractLiterals(source).filter(isUserFacingLiteral).map((literal) => literal.text);
    assert.deepEqual(detected('fn f() -> String { format!("Try again") }\nfn g(o: &mut String) { o.push_str("Please wait") }\nfn h() -> &\'static str { return "Need details"; }\n'),
      ['Try again', 'Please wait', 'Need details']);
    assert.deepEqual(detected('let i = "internal display name";\nlet c = format!("fn main() {}");\nlet l = format!("en ru hi zh");\n'), []);
    assert.deepEqual(extractLiterals('let s = "Base case here \\\n            resolves it.";').map((l) => l.text), ['Base case here resolves it.']);
    assert.equal(canonicalizeNormal('please {stem} '), 'please {stem}\\x20');
    assert.equal(canonicalizeRaw('status: '), 'status:\\x20');
    assert.deepEqual(extractLiterals('// "Sorry, I can\'t do that."\nlet x = 1;'), []);
    assert.equal(extractLiterals('let s = r#"Sorry, I can\'t do that."#;').length, 1);
    const entries = [{ file: 'rust/src/a.rs', text: 'Alpha sentence here.' }, { file: 'rust/src/b.rs', text: 'Beta sentence with \\n break.' }];
    assert.deepEqual(parseAllowlist(renderAllowlist(entries)), entries);
  });

  test('a new literal, --write and a stale row print as the Rust script prints them', () => {
    const header = renderAllowlist([]);
    const root = fixture({
      'rust/src/solver.rs': 'pub fn reply() -> &\'static str {\n    "Sorry, I can\'t do that."\n}\n',
      'scripts/hardcoded-language-allowlist.txt': header,
    });
    try {
      const fresh = run('check-hardcoded-language.mjs', [], root);
      assert.equal(fresh.status, 1);
      assert.equal(fresh.stdout, [
        '',
        'Checking rust/src/ for hardcoded user-facing natural language (R379)...',
        '',
        'Detected prose literals: 1 | allowlisted: 0',
        '',
        'New hardcoded user-facing strings found in rust/src/ (not in the allowlist):',
        '',
        '::error file=rust/src/solver.rs::Hardcoded user-facing string: "Sorry, I can\'t do that.". Move it into data/seed/ (R379).',
        '  rust/src/solver.rs: "Sorry, I can\'t do that."',
        '',
        'Move each string into grounded meanings under data/seed/ and look it',
        'up (e.g. via seed::response_for), or — only if it is genuinely part of',
        'the existing debt — regenerate the allowlist with --write.',
        '',
        '',
      ].join('\n'));

      const written = run('check-hardcoded-language.mjs', ['--write'], root);
      assert.equal(written.stdout, 'Wrote 1 entry to scripts/hardcoded-language-allowlist.txt\n');
      assert.equal(readFileSync(join(root, 'scripts/hardcoded-language-allowlist.txt'), 'utf8'),
        `${header}rust/src/solver.rs\tSorry, I can't do that.\n`);
      const synced = run('check-hardcoded-language.mjs', [], root);
      assert.equal(synced.status, 0);
      assert.match(synced.stdout, /No new hardcoded natural language; allowlist is in sync \(1 entries\)\.\n\n$/);

      write(root, 'rust/src/solver.rs', 'pub fn n() -> u8 { 1 }\n');
      const stale = run('check-hardcoded-language.mjs', [], root);
      assert.equal(stale.status, 1);
      assert.ok(stale.stdout.endsWith([
        'Stale allowlist rows (no longer present in rust/src/ — prune them):',
        '',
        '  rust/src/solver.rs: "Sorry, I can\'t do that."',
        '',
        'These strings were migrated or changed. Regenerate the allowlist with:',
        'rust-script scripts/check-hardcoded-language.rs --write',
        '',
        '',
      ].join('\n')), stale.stdout);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

/** The smallest tree every debt-ratchet measure can read. */
function ratchetTree(ceilings) {
  return fixture({
    'data/meta/debt-ratchet.lino': ceilings.map(([name, value]) => `ceiling\n  measure "${name}"\n  value ${value}\n`).join(''),
    'data/meta/core-boundary-ledger.lino': '  source rust/src/solver_handlers/a.rs\n    disposition migrate\n  source rust/src/solver_handlers/mod.rs\n    disposition migrate\n',
    'data/meta/handler-migration-ledger.lino': 'row\n  status pending\n',
    'scripts/hardcoded-language-allowlist.txt': '# header\n\nrust/src/a.rs\tA row here.\n',
    'rust/src/solver_dispatch.rs': 'const HANDLER_FUNCTIONS: &[(&str, F)] = &[\n    ("a", try_a),\n    ("b", other),\n];\n',
    'rust/src/intent_formalization/prompt_relevants.rs': '"handler:a"\n',
    'rust/src/meta_method_dispatch.rs': 'fn f() {}\n',
    'rust/src/lib.rs': 'fn f(x: &str) -> bool { x.contains("a") || x.starts_with("b") }\n',
    'js/worker/formal_ai_worker_seed_responses_and_language.js': 'function synchronousHandlerCandidates() { return [{ name: "a" }]; }\n',
    'rust/tests/unit/docs_requirements.rs': '\n',
    'experiments/issue_1028_agent_cli_ladder/rules/r.lino': '\n',
    'data/seed/s.lino': 'meaning a\n  lexeme en\n  lexeme ru\n',
  });
}

describe('check-debt-ratchet.mjs', () => {
  test('the ceiling and base rules match the Rust unit logic', () => {
    const previous = parseRatchet('ceiling\n  measure "a"\n  value 3\nceiling\n  measure "b"\n  value 1\n');
    const raised = parseRatchet('ceiling\n  measure "a"\n  value 4\n  note "corrected undercount of 3"\nceiling\n  measure "b"\n  value 2\n');
    assert.ok(announcesCorrection(raised, 'a', 3));
    assert.deepEqual(checkAgainstPrevious(previous, raised), [
      'b: ceiling raised from 1 to 2; a ceiling can only move down. If the old value counted the wrong set, say so in this measure\'s `note` as "corrected undercount" and name the 1 it corrects (plan 00 §6.2)',
    ]);
    assert.throws(() => parseRatchet('value 3\n'), { message: '`value 3` has no `measure` above it' });
  });

  test('a measure above its ceiling and one below it print as the Rust script prints them', () => {
    const root = ratchetTree([['literal_predicates', 1], ['handler_files', 2], ['language_parity_gaps', 1], ['try_dispatch_entries', 1]]);
    try {
      const result = run('check-debt-ratchet.mjs', ['--repo', root, '--base', 'HEAD'], root);
      assert.equal(result.status, 1);
      const out = result.stdout.split('\n');
      assert.deepEqual(out.slice(0, 5), [
        'debt ratchet (data/meta/debt-ratchet.lino):',
        '  handler_files: measured 1 / ceiling 2',
        '  language_parity_gaps: measured 1 / ceiling 1',
        '  literal_predicates: measured 2 / ceiling 1',
        '  try_dispatch_entries: measured 1 / ceiling 1',
      ]);
      // Outside a git work tree the base cannot be read; the Rust script says so and goes on.
      assert.match(out[5], /^ {2}\(skipping the base comparison: git \["show", "HEAD:data\/meta\/debt-ratchet\.lino"\] failed: /);
      assert.deepEqual(out.slice(6), [
        '::error file=data/meta/debt-ratchet.lino::handler_files: improved from 2 to 1; lower the reviewed ceiling in data/meta/debt-ratchet.lino in this commit',
        '::error file=data/meta/debt-ratchet.lino::literal_predicates: measured 2, ceiling 1; move the behaviour into data/seed or data/meta rules instead of raising the ceiling (issue #1085 D1)',
        '',
      ]);
      assert.equal(result.stderr, 'check-debt-ratchet: 2 debt ratchet failure(s)\n');

      write(root, 'data/meta/debt-ratchet.lino', 'ceiling\n  measure "literal_predicates"\n  value 2\n');
      const held = run('check-debt-ratchet.mjs', ['--repo', root, '--base', 'HEAD'], root);
      assert.equal(held.status, 0);
      assert.ok(held.stdout.endsWith('debt ratchet holds\n'));

      const unbased = run('check-debt-ratchet.mjs', ['--repo', root], root);
      assert.equal(unbased.status, 1);
      assert.equal(unbased.stderr, 'check-debt-ratchet: --base <rev> is required: without it the ceilings are compared against nothing and a raised ceiling passes. Use --base origin/main locally; CI supplies GITHUB_BASE_REF.\n');
      assert.equal(run('check-debt-ratchet.mjs', ['--bogus'], root).stderr, 'check-debt-ratchet: unknown argument: --bogus\n');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

const BOUNDARY_LEDGER = [
  'core_boundary_ledger',
  '  source_file_count_max 1',
  '  source_lines_max 2',
  '  outside_core_file_count_max 1',
  '  outside_core_lines_max 2',
  '  source rust/src/solver_handlers/a.rs',
  '    disposition migrate',
  '    baseline_lines 2',
  '    data_target "data/seed/a.lino"',
  '    reason "fixture"',
  '',
].join('\n');

describe('check-minimal-core-boundary.mjs', () => {
  test('a grown handler and an unledgered one print as the Rust script prints them', () => {
    const root = fixture({
      'data/meta/core-boundary-ledger.lino': BOUNDARY_LEDGER,
      'rust/src/solver_handlers/a.rs': 'fn a() {}\nfn b() {}\n',
      'rust/src/solver_handlers/modules.rs': 'mod a;\n',
    });
    try {
      const clean = run('check-minimal-core-boundary.mjs', [], root);
      assert.deepEqual(clean, { status: 0, stdout: 'minimal-core boundary: 1 handler sources, 2 outside-core lines\n', stderr: '' });

      write(root, 'rust/src/solver_handlers/a.rs', 'fn a() {}\nfn b() {}\nfn c() {}\n');
      write(root, 'rust/src/solver_handlers/nested/b.rs', 'fn b() {}\n');
      const grown = run('check-minimal-core-boundary.mjs', [], root);
      assert.equal(grown.status, 1);
      assert.equal(grown.stdout, '');
      assert.equal(grown.stderr, [
        'minimal-core boundary audit failed:',
        'rust/src/solver_handlers/a.rs grew from 2 to 3 lines',
        'unledgered handler source rust/src/solver_handlers/nested/b.rs',
        'source_file_count_max grew from 1 to 2',
        'source_lines_max grew from 2 to 3',
        'outside_core_lines_max grew from 2 to 3',
        '',
      ].join('\n'));

      write(root, 'data/meta/core-boundary-ledger.lino', BOUNDARY_LEDGER.replace('baseline_lines 2', 'baseline_lines x'));
      assert.equal(run('check-minimal-core-boundary.mjs', [], root).stderr,
        'minimal-core boundary audit failed:\ninvalid baseline_lines value "x": invalid digit found in string\n');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

describe('check-worker-line-budget.mjs', () => {
  const shard = (ceiling) => `worker_module_budget\n  module "formal_ai_worker_seed_responses_and_language.js"\n  ceiling ${ceiling}\n  rationale "Glue for tests."\n`;

  test('a module summary and shard name read as the Rust script reads them', () => {
    assert.equal(leadingSummary('// Worker module 0. Glue for "tests".\n// More.\n\ncode\n'), "Glue for 'tests'. More.");
    assert.equal(shardName('a.js.js'), 'a.lino');
  });

  test('a regrown module, --write and a clean run print as the Rust script prints them', () => {
    const root = fixture({
      'js/worker/formal_ai_worker_seed_responses_and_language.js': '// Worker module 0. Glue for tests.\nx\n',
      'data/meta/worker-line-budget/formal_ai_worker_seed_responses_and_language.lino': shard(2),
      'data/meta/worker-line-budget/formal_ai_worker_gone.lino': 'worker_module_budget\n  module "formal_ai_worker_gone.js"\n  ceiling 1\n  rationale "Gone."\n',
    });
    try {
      write(root, 'js/worker/formal_ai_worker_seed_responses_and_language.js', '// Worker module 0. Glue for tests.\nx\ny\n');
      const grown = run('check-worker-line-budget.mjs', [], root);
      assert.equal(grown.status, 1);
      assert.equal(grown.stdout, [
        '',
        'Checking the UI-glue line budget for the split JavaScript worker...',
        '',
        'Worker JavaScript line counts (js/worker/*.js):',
        '       3 /      2  js/worker/formal_ai_worker_seed_responses_and_language.js',
        '',
        '  total: 3 lines (summed ceilings 3, target 3000)',
        '',
        '::error::js/worker/formal_ai_worker_seed_responses_and_language.js grew to 3 lines, past its recorded ceiling of 2. Move logic into the Rust→WASM worker (js/wasm-worker) instead of growing the mirror, or re-baseline this one module with `--write` and explain the growth in data/meta/worker-line-budget/formal_ai_worker_seed_responses_and_language.lino',
        '::error::data/meta/worker-line-budget/formal_ai_worker_gone.lino budgets `formal_ai_worker_gone.js`, which no longer exists; delete the shard',
        '',
        '2 module budget violation(s). The mirror cannot silently regrow.',
        '',
        '',
      ].join('\n'));

      const rebaselined = run('check-worker-line-budget.mjs', ['--write'], root);
      assert.equal(rebaselined.stdout, [
        '',
        'Checking the UI-glue line budget for the split JavaScript worker...',
        '',
        '  rebaselined  formal_ai_worker_seed_responses_and_language.js -> 3 lines',
        '  removed      formal_ai_worker_gone.js (no longer in the mirror)',
        '',
        'Budget shards re-baselined. Review the diff and explain any growth.',
        '',
        '',
      ].join('\n'));
      assert.equal(readFileSync(join(root, 'data/meta/worker-line-budget/formal_ai_worker_seed_responses_and_language.lino'), 'utf8'), shard(3));

      const clean = run('check-worker-line-budget.mjs', [], root);
      assert.equal(clean.status, 0);
      assert.ok(clean.stdout.endsWith('\n  total: 3 lines (summed ceilings 3, target 3000)\n\nWorker JavaScript is at or below the 3000-line UI-glue target.\n\n'));
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
