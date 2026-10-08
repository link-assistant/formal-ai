#!/usr/bin/env node
// R1188-U18: measure page formalization over the cached Wikipedia intros in
// data/benchmarks/web-formalization/ (fetched by
// scripts/fetch-web-formalization-corpus.mjs) and hold the result to
// data/meta/web-formalization-ratchet.lino.
//
// Every sentence of every page goes through js/agentic/crate/page_formalization.mjs.
// The measures, overall and per language:
//
//   sentence-coverage   sentences with at least one statement of two or more
//                       terms and no unknown term, over all sentences
//   fact-survival       statements with a known term whose deformalization
//                       reads back to the same known terms, over those statements
//   known-term-share    terms that name a meaning, a name or a number, over all terms
//   pages, sentences    the corpus size, so the corpus cannot shrink unnoticed
//
// Usage:
//   node scripts/measure-web-formalization.mjs           measure and check the ratchet
//   node scripts/measure-web-formalization.mjs --write   also raise the ratchet
//   node scripts/measure-web-formalization.mjs --pages   also print one line per page

import { pathToFileURL } from 'node:url';

import { LANGUAGES, enforceRatchet, installTextHost, readPages, share } from './lib/text-capability-measures.mjs';

export const RATCHET_FILE = 'data/meta/web-formalization-ratchet.lino';

/** The summed counts of `formalizePage` over `pages`. */
function totals(reports) {
  const sum = { pages: reports.length, sentences: 0, covered: 0, factual: 0, survived: 0, terms: 0, unknown: 0 };
  for (const report of reports) {
    sum.sentences += report.sentences.length;
    for (const key of ['covered', 'factual', 'survived', 'terms', 'unknown']) sum[key] += report[key];
  }
  return sum;
}

/** The named measures of one total, each name prefixed with `prefix`. */
function measuresOf(prefix, total) {
  return [
    [`${prefix}pages`, total.pages],
    [`${prefix}sentences`, total.sentences],
    [`${prefix}sentence-coverage`, share(total.covered, total.sentences)],
    [`${prefix}fact-survival`, share(total.survived, total.factual)],
    [`${prefix}known-term-share`, share(total.terms - total.unknown, total.terms)],
  ];
}

/** Measure the whole corpus: `{measures, lines}`. */
export async function measureWebFormalization({ perPage = false } = {}) {
  installTextHost();
  const { formalizePage } = await import('../js/agentic/crate/page_formalization.mjs');
  const all = [];
  const perLanguage = [];
  const lines = [];
  for (const language of LANGUAGES) {
    const reports = readPages(language).map((page) => {
      const report = formalizePage(page.text, language);
      if (perPage) {
        lines.push(`${language} ${page.title}: ${report.covered}/${report.sentences.length} sentences covered, `
          + `${report.survived}/${report.factual} facts survive, ${report.unknown}/${report.terms} terms unknown`);
      }
      return report;
    });
    all.push(...reports);
    const total = totals(reports);
    perLanguage.push(...measuresOf(`${language}-`, total));
    lines.push(`${language}: ${total.covered}/${total.sentences} sentences covered, `
      + `${total.survived}/${total.factual} facts survive, ${total.unknown}/${total.terms} terms unknown`);
  }
  const total = totals(all);
  lines.push(`all: ${total.covered}/${total.sentences} sentences covered, `
    + `${total.survived}/${total.factual} facts survive, ${total.unknown}/${total.terms} terms unknown`);
  return { measures: [...measuresOf('', total), ...perLanguage], lines };
}

async function main() {
  const argv = process.argv.slice(2);
  const { measures, lines } = await measureWebFormalization({ perPage: argv.includes('--pages') });
  for (const line of lines) console.log(line);
  enforceRatchet(RATCHET_FILE, {
    comments: [
      'R1188-U18 page formalization ratchet. Written by',
      '`node scripts/measure-web-formalization.mjs --write`; the CI gate',
      'data/meta/ci-gates/check-web-formalization.lino fails when a value falls.',
      'Shares are rounded down to three decimals; counts are exact.',
    ],
    name: 'web-formalization-ratchet',
    fields: [['source', 'scripts/measure-web-formalization.mjs'], ['corpus', 'data/benchmarks/web-formalization']],
    measures,
  }, argv);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await main();
