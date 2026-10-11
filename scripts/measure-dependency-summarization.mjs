#!/usr/bin/env node
// R1188-U21: measure dependency summarization over the corpus named in
// data/benchmarks/dependency-summarization/corpus.lino (the cached Wikipedia
// intros and a fixed list of cached issue bodies), and hold the result to
// data/meta/text-summarization-ratchet.lino.
//
// Every document goes through js/agentic/crate/dependency_summarization.mjs.
// The measures, overall and for the pages and the issues apart:
//
//   compression          1 - summary characters / source characters
//   key-fact-retention   the gold facts the summary keeps, over all gold facts;
//                        the gold of a document is its first sentence's subject
//                        and the terms that define it
//   duplicates-removed   statements dropped because an earlier statement
//                        already holds their formal content
//   documents            the corpus size
//
// Usage:
//   node scripts/measure-dependency-summarization.mjs            measure and check the ratchet
//   node scripts/measure-dependency-summarization.mjs --write    also raise the ratchet
//   node scripts/measure-dependency-summarization.mjs --show N   print the summary of document N

import { pathToFileURL } from 'node:url';

import { childValue, childrenNamed, parseLino, readRepoFile } from '../js/server/lino.mjs';
import { LANGUAGES, enforceRatchet, installTextHost, readPages, recordsNamed, share } from './lib/text-capability-measures.mjs';

export const RATCHET_FILE = 'data/meta/text-summarization-ratchet.lino';
export const CORPUS_FILE = 'data/benchmarks/dependency-summarization/corpus.lino';

/**
 * The prose of a Markdown issue body: fenced code, tables, block quotes and
 * HTML tags left out, a link or image kept as its text, list and heading
 * markers removed, runs of spaces made one.
 */
export function proseOf(markdown) {
  const out = [];
  let fenced = false;
  for (const raw of markdown.split(/\r?\n/u)) {
    const line = raw.trim();
    if (line.startsWith('```') || line.startsWith('~~~')) {
      fenced = !fenced;
      continue;
    }
    if (fenced || line.startsWith('|') || line.startsWith('>')) continue;
    const prose = line
      .replace(/<[^>]*>/gu, ' ')
      .replace(/!?\[([^\]]*)\]\([^)]*\)/gu, '$1')
      .replace(/https?:\/\/\S+/gu, ' ')
      .replace(/^(?:#+|[-*+]|\d+[.)])\s+/u, '')
      .replace(/[`*_]+/gu, '')
      .replace(/\s+/gu, ' ')
      .trim();
    if (prose !== '') out.push(prose);
  }
  return out.join('\n');
}

/** The corpus documents: `{kind, name, language, text}`. */
export async function readDocuments() {
  installTextHost();
  const { detect } = await import('../js/agentic/crate/language.mjs');
  const record = recordsNamed(parseLino(readRepoFile(CORPUS_FILE)), 'dependency-summarization-corpus')[0];
  const documents = [];
  if (childValue(record, 'pages') !== '') {
    for (const language of LANGUAGES) {
      for (const page of readPages(language)) documents.push({ kind: 'pages', name: page.url, language, text: page.text });
    }
  }
  for (const issue of childrenNamed(record, 'issue')) {
    const text = proseOf(readRepoFile(issue.value));
    const detected = detect(text);
    documents.push({ kind: 'issues', name: issue.value, language: LANGUAGES.includes(detected) ? detected : 'en', text });
  }
  return documents;
}

const emptyTotal = () => ({ documents: 0, source: 0, summary: 0, facts: 0, retained: 0, duplicates: 0 });

function measuresOf(prefix, total) {
  return [
    [`${prefix}documents`, total.documents],
    [`${prefix}compression`, share(total.source - total.summary, total.source)],
    [`${prefix}key-fact-retention`, share(total.retained, total.facts)],
    [`${prefix}duplicates-removed`, total.duplicates],
  ];
}

/** Measure the corpus: `{measures, lines, summaries}`. */
export async function measureDependencySummarization() {
  const documents = await readDocuments();
  const { keyFacts, retainedFacts, summarizeByDependency } = await import('../js/agentic/crate/dependency_summarization.mjs');
  const totals = { all: emptyTotal(), pages: emptyTotal(), issues: emptyTotal() };
  const summaries = [];
  for (const document of documents) {
    const summary = summarizeByDependency(document.text, document.language);
    const facts = keyFacts(document.text, document.language);
    const retained = retainedFacts(summary, facts);
    summaries.push({ document, summary, facts, retained });
    for (const total of [totals.all, totals[document.kind]]) {
      total.documents += 1;
      total.source += Array.from(document.text).length;
      total.summary += Array.from(summary.text).length;
      total.facts += facts.length;
      total.retained += retained;
      total.duplicates += summary.duplicates;
    }
  }
  const lines = Object.entries(totals).map(([kind, total]) => `${kind}: ${total.documents} documents, `
    + `summary ${total.summary}/${total.source} characters, ${total.retained}/${total.facts} key facts kept, `
    + `${total.duplicates} duplicates removed`);
  const measures = [
    ...measuresOf('', totals.all),
    ...measuresOf('pages-', totals.pages),
    ...measuresOf('issues-', totals.issues),
  ];
  return { measures, lines, summaries };
}

async function main() {
  const argv = process.argv.slice(2);
  const { measures, lines, summaries } = await measureDependencySummarization();
  for (const line of lines) console.log(line);
  const show = argv.indexOf('--show');
  if (show >= 0) {
    const entry = summaries[Number(argv[show + 1])];
    console.log(`\n${entry.document.name} (${entry.document.language})\n${entry.summary.text}\n`);
  }
  enforceRatchet(RATCHET_FILE, {
    comments: [
      'R1188-U21 dependency summarization ratchet. Written by',
      '`node scripts/measure-dependency-summarization.mjs --write`; the CI gate',
      'data/meta/ci-gates/check-text-summarization.lino fails when a value falls.',
      'Shares are rounded down to three decimals; counts are exact.',
      'Re-recorded for PR #1188 (the LEXEMES batch): the Spanish lexemes and the',
      'total-surface ranking of shared surfaces let the summary keep more key',
      'facts (retention 0.644 to 0.650) at a slightly longer summary, so the',
      'compression floors fell by 0.002 to 0.005; the trade-off is recorded here.',
    ],
    name: 'text-summarization-ratchet',
    fields: [['source', 'scripts/measure-dependency-summarization.mjs'], ['corpus', CORPUS_FILE]],
    measures,
  }, argv);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await main();
