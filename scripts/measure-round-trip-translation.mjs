#!/usr/bin/env node
// R1188-U19: measure round-trip translation (source -> meta -> target ->
// meta -> source) over all 20 ordered pairs of en, ru, hi, zh and es, and
// hold the result to data/meta/round-trip-translation-ratchet.lino.
//
// The corpus, data/benchmarks/round-trip-translation/sentences.lino, is built
// once by --write-corpus from two sources the repository already holds:
//
//   * the seed's own sentences: the multilingual responses written in all
//     five languages, without placeholders or markup, the first
//     `SEED_INTENTS` intents in name order (one sentence per language each);
//   * the cached page sentences: the first `PAGE_SENTENCES` sentences of every
//     page in data/benchmarks/web-formalization/.
//
// The measures, overall and per ordered pair:
//
//   sentence-survival   sentences whose every word is known and whose meanings
//                       all come back the same in the target and in the source
//   term-survival       terms whose meaning comes back the same, over all the
//                       content terms of the source sentences, known or not
//
// The term-survival denominator is fixed by the corpus, not by the lexicon
// (PR #1188, LEXEMES): over known terms only, a word the lexicon newly knows
// grew the denominator and could lower the share even while more terms
// survived. Now a newly known term can only add survivors. The old share over
// known terms is still printed on each pair's line, but not ratcheted.
//
// Usage:
//   node scripts/measure-round-trip-translation.mjs                 measure and check the ratchet
//   node scripts/measure-round-trip-translation.mjs --write         also raise the ratchet
//   node scripts/measure-round-trip-translation.mjs --write-corpus  rebuild the corpus file

import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { formatLinoValueVerbatim } from '../js/agentic/crate/links_format.mjs';
import { REPO_ROOT, childValue, parseLino, readRepoFile } from '../js/server/lino.mjs';
import {
  LANGUAGES, enforceRatchet, installTextHost, readPages, recordsNamed, share,
} from './lib/text-capability-measures.mjs';

export const RATCHET_FILE = 'data/meta/round-trip-translation-ratchet.lino';
export const CORPUS_FILE = 'data/benchmarks/round-trip-translation/sentences.lino';

/** How many seed response intents (each one sentence per language) the corpus takes. */
const SEED_INTENTS = 40;

/** How many leading sentences of each cached page the corpus takes. */
const PAGE_SENTENCES = 3;

/** Characters that mark a seed response as a template or markup, not a plain sentence. */
const NOT_PLAIN = /[{}<>`*#|\n]/u;

/** The longest seed response the corpus takes, in characters. */
const LONGEST_RESPONSE = 120;

/** The corpus sentences: `{language, origin, text}`. */
export function readCorpus() {
  const root = parseLino(readRepoFile(CORPUS_FILE));
  return recordsNamed(root, 'round-trip-sentence').map((record) => ({
    language: childValue(record, 'language'),
    origin: childValue(record, 'origin'),
    text: childValue(record, 'text'),
  }));
}

/** Build the corpus sentences from the seed responses and the cached pages. */
async function buildCorpus() {
  installTextHost();
  const { multilingualResponses } = await import('../js/agentic/crate/seed.mjs');
  const { sentences } = await import('../js/agentic/crate/formalization_segment.mjs');
  const byIntent = new Map();
  for (const response of multilingualResponses()) {
    if (NOT_PLAIN.test(response.text) || Array.from(response.text).length > LONGEST_RESPONSE) continue;
    if (!byIntent.has(response.intent)) byIntent.set(response.intent, new Map());
    const texts = byIntent.get(response.intent);
    if (!texts.has(response.language)) texts.set(response.language, response.text);
  }
  const intents = [...byIntent.keys()]
    .filter((intent) => LANGUAGES.every((language) => byIntent.get(intent).has(language)))
    .sort()
    .slice(0, SEED_INTENTS);
  const out = [];
  for (const intent of intents) {
    for (const language of LANGUAGES) {
      out.push({ language, origin: `seed-response ${intent}`, text: byIntent.get(intent).get(language) });
    }
  }
  for (const language of LANGUAGES) {
    for (const page of readPages(language)) {
      for (const sentence of sentences(page.text).slice(0, PAGE_SENTENCES)) {
        out.push({ language, origin: `page ${page.url}`, text: sentence.text });
      }
    }
  }
  return out;
}

function writeCorpus(corpus) {
  const lines = [
    '# R1188-U19 round-trip translation corpus. Built by',
    '# `node scripts/measure-round-trip-translation.mjs --write-corpus` from the seed',
    '# responses written in all five languages and the leading sentences of the',
    '# cached pages in data/benchmarks/web-formalization/.',
  ];
  for (const sentence of corpus) {
    lines.push('round-trip-sentence');
    lines.push(`  language ${sentence.language}`);
    lines.push(`  origin ${formatLinoValueVerbatim(sentence.origin)}`);
    lines.push(`  text ${formatLinoValueVerbatim(sentence.text)}`);
  }
  const file = join(REPO_ROOT, CORPUS_FILE);
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, `${lines.join('\n')}\n`);
}

/** Measure every ordered pair: `{measures, lines}`. */
export async function measureRoundTripTranslation() {
  installTextHost();
  const { roundTrip } = await import('../js/agentic/crate/round_trip_translation.mjs');
  const corpus = readCorpus();
  const { formalizeText, contentIds } = await import('../js/agentic/crate/text_formalization.mjs');
  const contentTerms = (text, language) => formalizeText(text, language)
    .reduce((sum, statement) => sum + contentIds(statement).length, 0);
  const all = { sentences: 0, survived: 0, content: 0, known: 0, surviving: 0 };
  const pairs = [];
  const lines = [];
  for (const source of LANGUAGES) {
    const texts = corpus.filter((sentence) => sentence.language === source).map((sentence) => sentence.text);
    for (const target of LANGUAGES) {
      if (target === source) continue;
      const pair = { sentences: 0, survived: 0, content: 0, known: 0, surviving: 0 };
      for (const text of texts) {
        const trip = roundTrip(text, source, target);
        pair.sentences += 1;
        if (trip.survives) pair.survived += 1;
        pair.content += contentTerms(text, source);
        pair.known += trip.knownTerms;
        pair.surviving += trip.survivingTerms;
      }
      for (const key of Object.keys(all)) all[key] += pair[key];
      pairs.push([`${source}-${target}-sentence-survival`, share(pair.survived, pair.sentences)]);
      pairs.push([`${source}-${target}-term-survival`, share(pair.surviving, pair.content)]);
      lines.push(`${source}->${target}: ${pair.survived}/${pair.sentences} sentences survive, `
        + `${pair.surviving}/${pair.content} content terms survive (${pair.surviving}/${pair.known} known)`);
    }
  }
  lines.push(`all pairs: ${all.survived}/${all.sentences} sentences survive, `
    + `${all.surviving}/${all.content} content terms survive (${all.surviving}/${all.known} known)`);
  const measures = [
    ['sentences', corpus.length],
    ['sentence-survival', share(all.survived, all.sentences)],
    ['term-survival', share(all.surviving, all.content)],
    ...pairs,
  ];
  return { measures, lines };
}

async function main() {
  const argv = process.argv.slice(2);
  if (argv.includes('--write-corpus')) {
    const corpus = await buildCorpus();
    writeCorpus(corpus);
    console.log(`wrote ${CORPUS_FILE}: ${corpus.length} sentences`);
    return;
  }
  const { measures, lines } = await measureRoundTripTranslation();
  for (const line of lines) console.log(line);
  enforceRatchet(RATCHET_FILE, {
    comments: [
      'R1188-U19 round-trip translation ratchet, all 20 ordered pairs of en, ru, hi,',
      'zh and es. Written by `node scripts/measure-round-trip-translation.mjs --write`;',
      'the CI gate data/meta/ci-gates/check-round-trip-translation.lino fails when a',
      'value falls. Shares are rounded down to three decimals; counts are exact.',
      'Re-recorded once for PR #1188 (the LEXEMES batch): term survival is now',
      'counted over all content terms of the source sentences, known or not, so a',
      'word the lexicon newly knows can only add survivors. Over known terms only,',
      'the Spanish lexemes lowered the share while survivors rose (es->ru 167 to',
      '210). The same batch ranks a surface shared by several meanings by the',
      'meanings\' total surfaces across all languages, which still separates them',
      'once every meaning is written in all five languages. Its Spanish file-edit',
      'lead (por, con, a), which Spanish edit requests need, gives the lead all five',
      'languages, so it now holds zh 为 over content_assignment and zh->hi keeps 346',
      'of 353 survivors: hi से reads back as line_range_connector.',
    ],
    name: 'round-trip-translation-ratchet',
    fields: [['source', 'scripts/measure-round-trip-translation.mjs'], ['corpus', CORPUS_FILE]],
    measures,
  }, argv);
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await main();
