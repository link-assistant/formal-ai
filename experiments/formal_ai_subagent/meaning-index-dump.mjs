#!/usr/bin/env node
// Dumps the text-formalization meaning index (surface -> meaning) of each
// language, one `language<TAB>surface<TAB>meaning` line, so two trees can be
// diffed to see which surfaces a seed change moved to another meaning.
//
// Usage: node experiments/formal_ai_subagent/meaning-index-dump.mjs [language...]
import { installTextHost } from '../../scripts/lib/text-capability-measures.mjs';

installTextHost();
const { meaningIndex } = await import('../../js/agentic/crate/text_formalization.mjs');
const languages = process.argv.slice(2).length ? process.argv.slice(2) : ['en', 'ru', 'hi', 'zh', 'es'];
for (const language of languages) {
  for (const [surface, slug] of [...meaningIndex(language)].sort()) console.log(`${language}\t${surface}\t${slug}`);
}
