#!/usr/bin/env node
// Lists the meaning terms a round trip loses (source -> en -> source), with the
// target surface chosen and the meaning that surface reads back as, so a seed
// change that lowers term survival can be traced to the colliding meanings.
//
// Usage: node experiments/formal_ai_subagent/round-trip-losses.mjs [source] [target]
import { installTextHost } from '../../scripts/lib/text-capability-measures.mjs';
import { readCorpus } from '../../scripts/measure-round-trip-translation.mjs';

installTextHost();
const { formalizeText, statementTerms, resolveSurface, WordKind } = await import('../../js/agentic/crate/text_formalization.mjs');
const { bestSurface } = await import('../../js/agentic/crate/round_trip_translation.mjs');

const languages = ['en', 'ru', 'hi', 'zh', 'es'];
const [onlySource, target = 'en'] = process.argv.slice(2);
const losses = new Map();
for (const source of languages) {
  if (source === target || (onlySource && source !== onlySource)) continue;
  for (const sentence of readCorpus().filter((entry) => entry.language === source)) {
    for (const statement of formalizeText(sentence.text, source)) {
      for (const term of statementTerms(statement)) {
        if (term.kind !== WordKind.Meaning) continue;
        const surface = bestSurface(term, source, target);
        if (surface === null) continue;
        const back = resolveSurface(surface, target);
        if (back === term.id) continue;
        const key = `${source} ${term.surface} [${term.id}] -> ${target} "${surface}" -> [${back}]`;
        losses.set(key, (losses.get(key) ?? 0) + 1);
      }
    }
  }
}
for (const [key, count] of [...losses].sort((left, right) => right[1] - left[1])) console.log(`${count}\t${key}`);
