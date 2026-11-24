// Supplied diagnostic harness; zero independent Formal AI authorship credit.
// Compare the two English causal seed lexemes with local file transport; no external requests.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { gzipSync } from 'node:zlib';
import { createWorkerContext, evaluate, plain, REPO_ROOT } from '../../rust/tests/web/support/browser-runtime.mjs';

const seedPath = 'data/seed/meanings-summarization.lino';
const seed = readFileSync(seedPath, 'utf8');
const withoutCausalLexemes = seed.replace('    lexeme en "because"\n', '').replace('    lexeme en "because of"\n', '');
assert.notEqual(withoutCausalLexemes, seed, 'the controlled English lexeme exists');
const lines = readFileSync('data/benchmarks/web-formalization/en.lino', 'utf8').split('\n');
const paragraphs = [];
for (let i = lines.indexOf('  title "Moon"') + 1; i < lines.length && lines[i].startsWith('  '); i += 1) {
  const match = /^ {2}paragraph "(.*)"$/u.exec(lines[i]);
  if (match) paragraphs.push(match[1]);
}
assert.equal(paragraphs.length > 0, true);
const reports = {};
for (const [label, seedText] of [['current', seed], ['withoutCausalLexemes', withoutCausalLexemes]]) {
  const worker = createWorkerContext({ fetch: async (url) => {
    const address = new URL(String(url), 'http://localhost/');
    assert.equal(address.origin, 'http://localhost', 'fixture makes no external request');
    const pathname = address.pathname.replace(/^\/+/u, '');
    const onDisk = pathname.startsWith('seed/') ? path.join(REPO_ROOT, 'data', pathname) : path.join(REPO_ROOT, 'js', pathname);
    try {
      const text = onDisk === path.join(REPO_ROOT, seedPath) ? seedText : readFileSync(onDisk, 'utf8');
      return { ok: true, status: 200, text: async () => text };
    } catch { return { ok: false, status: 404, text: async () => '' }; }
  }});
  await evaluate(worker, 'loadSeed()');
  reports[label] = plain(await evaluate(worker, `crateModule("crate/page_formalization.mjs").formalizePage(${JSON.stringify(paragraphs.join('\n'))}, "en")`));
}
const counts = (report) => ({sentences: report.sentences.length, ...Object.fromEntries(['covered', 'statements', 'factual', 'survived', 'terms', 'unknown'].map(key => [key, report[key]]))});
const unknownTerms = (report) => report.sentences.flatMap(sentence => sentence.unknown);
console.log(JSON.stringify({counts: Object.fromEntries(Object.entries(reports).map(([key, report]) => [key, counts(report)])), unknownSamples: unknownTerms(reports.withoutCausalLexemes).slice(0, 3), becauseSentences: Object.fromEntries(Object.entries(reports).map(([key, report]) => [key, report.sentences.filter(sentence => /because/iu.test(sentence.text)).map(sentence => ({text: sentence.text, unknown: sentence.unknown}))]))}, null, 2));
const before = unknownTerms(reports.withoutCausalLexemes);
const after = unknownTerms(reports.current);
assert.deepEqual(before.filter(term => term !== 'because'), after);
assert.equal(before.filter(term => term === 'because').length, 1);
assert.deepEqual(counts(reports.withoutCausalLexemes), {...counts(reports.current), unknown: reports.current.unknown + 1});
const raw = Buffer.from(JSON.stringify({schema: 'issue-1580-moon-lexeme-probe/v1', seedPath, seedSha256: createHash('sha256').update(seed).digest('hex'), removedLines: ['    lexeme en "because"', '    lexeme en "because of"'], counts: Object.fromEntries(Object.entries(reports).map(([key, report]) => [key, counts(report)])), changedUnknownTerms: ['because'], actualNetworkRequests: 0, reports}, null, 2) + '\n');
const base = 'experiments/issue-1580/evidence/checks/moon-lexeme-probe';
mkdirSync(base, {recursive: true});
writeFileSync(`${base}/comparison.json.gz`, gzipSync(raw), {flag: 'wx'});
console.log(JSON.stringify({counts: Object.fromEntries(Object.entries(reports).map(([key, report]) => [key, counts(report)])), changedUnknownTerms: ['because'], rawSha256: createHash('sha256').update(raw).digest('hex')}, null, 2));
