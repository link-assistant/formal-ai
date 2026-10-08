// Issue #1188 R1188-U23 (owner, 2026-10-08, a strict requirement): every
// regular code file in js/, ts/ and rust/ is human-readable multi-line code;
// only distribution bundles and untouchable generated output may be exempt,
// each listed with its reason. scripts/check-readable-code.mjs is the gate
// (data/meta/ci-gates/check-readable-code.lino); scripts/translate-es.mjs no
// longer writes a ts twin as one line of spaced tokens.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  EXEMPTIONS_FILE, MAXIMUM_CODE_CHARACTERS, checkReadableCode, codeCharactersPerLine, familyOf,
  parseExemptions, readabilityProblems,
} from '../../../scripts/check-readable-code.mjs';
import { renderSource, tokenize, translateJsToTs } from '../../../scripts/translate-es.mjs';
import { breakLongLines } from '../../../experiments/formal_ai_subagent/break-long-lines.mjs';

const REPOSITORY = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');

test('a ts twin rendered as one line of spaced tokens is refused; the layout-carrying twin passes', () => {
  const source = readFileSync(join(REPOSITORY, 'js/agentic/crate/es_tokenizer.mjs'), 'utf8');
  const oneLine = renderSource(tokenize(source));
  assert.equal(oneLine.includes('\n'), false, 'the canonical rendering is a single line');
  const problems = readabilityProblems(oneLine, 'script');
  assert.ok(problems.some((problem) => problem.startsWith('packed:')), problems.join('; '));
  assert.ok(problems.some((problem) => problem.startsWith('long line:')), problems.join('; '));
  assert.deepEqual(readabilityProblems(translateJsToTs(source), 'script'), []);
});

test('string, template and comment text is data, not code; the code around it is counted', () => {
  const prose = 'x'.repeat(MAXIMUM_CODE_CHARACTERS * 2);
  // The rest of each file: enough short lines that only the long line is measured.
  const rest = 'a;\n'.repeat(8);
  assert.deepEqual(readabilityProblems(`const prompt = "${prose}";\n${rest}`, 'script'), []);
  assert.deepEqual(readabilityProblems(`// ${prose}\nconst a = 1;\n${rest}`, 'script'), []);
  assert.deepEqual(readabilityProblems(`let s = r#"${prose}"#;\nlet c = '"';\n${rest}`, 'rust'), []);
  assert.deepEqual(readabilityProblems(`text = """\n${prose}\n"""\n${rest}`, 'python'), []);
  assert.deepEqual(readabilityProblems(`echo '${prose}'\n${rest}`, 'shell'), []);
  // One packed line of the same text is still refused: a file needs lines for its size.
  assert.match(readabilityProblems(`const prompt = "${prose}";\n`, 'script')[0], /^packed: 1 line\(s\)/u);
  // A template's interpolation is code; its text is not.
  assert.deepEqual(codeCharactersPerLine('const t = `ab ${x + 1} cd`;', 'script'), [21]);
  // Rust lifetimes are code, character literals are not.
  assert.deepEqual(codeCharactersPerLine("fn f<'a>(c: char) -> bool { c == 'x' }", 'rust'), [35]);
  const longCode = `const list = [${Array.from({ length: 80 }, (_, index) => `item${index}`).join(', ')}];\n${rest}`;
  const problems = readabilityProblems(longCode, 'script');
  assert.equal(problems.length, 1);
  assert.match(problems[0], /^long line: 1 line\(s\) over 300 code characters, at 1 \(\d+\)$/u);
});

test('the measured roots and code extensions', () => {
  assert.equal(familyOf('ts/agentic/planner.mts'), 'script');
  assert.equal(familyOf('js/app/app.jsx'), 'script');
  assert.equal(familyOf('rust/src/es_meta.rs'), 'rust');
  assert.equal(familyOf('scripts/check-closure-audit.py'), 'python');
  assert.equal(familyOf('scripts/run-prebuilt-tests.sh'), 'shell');
  assert.equal(familyOf('data/meta/readable-code-exemptions.lino'), null);
  assert.equal(familyOf('js/vendor/tree-sitter/tree-sitter-rust.wasm'), null);
});

test('every exemption names a reason, and a stale or reasonless exemption fails', () => {
  const committed = parseExemptions(readFileSync(join(REPOSITORY, EXEMPTIONS_FILE), 'utf8'));
  assert.ok(committed.length > 0);
  for (const exemption of committed) assert.ok(exemption.reason.length > 40, `${exemption.path} has a reason`);
  assert.ok(committed.some((exemption) => exemption.reason.includes('github.com/link-foundation/meta-language/issues/')),
    'the translation projections name the upstream issue');

  const repository = mkdtempSync(join(tmpdir(), 'readable-code-'));
  try {
    mkdirSync(join(repository, 'data/meta'), { recursive: true });
    mkdirSync(join(repository, 'js'), { recursive: true });
    writeFileSync(join(repository, 'js/readable.js'), 'export const a = 1;\n');
    writeFileSync(join(repository, 'js/packed.js'), `${'let a = 1; '.repeat(60)}\n`);
    writeFileSync(join(repository, EXEMPTIONS_FILE), [
      'readable-code-exemption js/readable.js',
      '  reason "stale: the file is readable"',
      'readable-code-exemption js/gone.js',
      '',
    ].join('\n'));
    execFileSync('git', ['init', '--quiet'], { cwd: repository });
    execFileSync('git', ['add', '.'], { cwd: repository });
    const { report, failed } = checkReadableCode(repository);
    assert.equal(failed, true);
    assert.ok(report.some((line) => line.startsWith('js/packed.js: ')), report.join('\n'));
    assert.ok(report.some((line) => line.includes('js/readable.js is exempt but readable')), report.join('\n'));
    assert.ok(report.some((line) => line.includes('js/gone.js has no reason')), report.join('\n'));
  } finally {
    rmSync(repository, { recursive: true, force: true });
  }
});

test('the JSX line breaker splits only at element and attribute boundaries, once', () => {
  const attributes = Array.from({ length: 12 }, (_, index) => `data-field-${index}={value${index}}`).join(' ');
  const line = `  return <section className="panel" ${attributes}><h2>{title}</h2><p>{body}</p><span className="note">it's fine</span></section>;`;
  const broken = breakLongLines(`${line}\n`, 'script');
  assert.notEqual(broken, `${line}\n`);
  assert.deepEqual(readabilityProblems(broken, 'script'), []);
  assert.equal(broken.replace(/\n\s*/gu, ' ').replace(/> </gu, '><').replace(/> \{/gu, '>{').replace(/\} </gu, '}<'), `${line}\n`.replace(/\n$/u, ' '));
  assert.equal(breakLongLines(broken, 'script'), broken, 'a broken text is left as it is');
});

test('the committed repository passes the gate', () => {
  const { report, failed } = checkReadableCode(REPOSITORY);
  assert.equal(failed, false, report.join('\n'));
});
