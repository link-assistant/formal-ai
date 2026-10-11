// `crate::seed::concepts` (rust/src/seed.rs `ConceptRecord`, `LocalizedConcept`):
// the concept seed data/seed/concepts.lino as records `{slug, term, category,
// aliases, contexts, context_links, wikidata, summary, source, source_kind,
// localized}` with `localized` rows `{language, term, aliases, summary, source,
// source_kind}`.

import { cached, childValue, parseLino, readText } from '../host.mjs';
import { splitPipeList } from './seed.mjs';

/** Mirrors `CONCEPTS_LINO`'s path in rust/src/seed.rs. */
export const CONCEPTS_FILE = 'data/seed/concepts.lino';

/**
 * Mirrors `fn concepts` in rust/src/seed.rs: every `concept_*` entry with a
 * term and a summary, in declaration order.
 * @returns {Array<object>}
 */
export function seedConcepts() {
  return cached('seed-concepts', () => {
    const tree = parseLino(readText(CONCEPTS_FILE));
    const entries = tree.name === '' ? tree.children : [tree];
    const out = [];
    for (const entry of entries) {
      if (!entry.name.startsWith('concept_')) continue;
      const summary = childValue(entry, 'summary');
      const term = childValue(entry, 'term');
      if (term === '' || summary === '') continue;
      const localized = [];
      for (const child of (entry.children || []).filter((candidate) => candidate.name === 'localized')) {
        if (child.id === '' || child.id === undefined) continue;
        localized.push({
          language: child.id,
          term: childValue(child, 'term'),
          aliases: splitPipeList(childValue(child, 'aliases')),
          summary: childValue(child, 'summary'),
          source: childValue(child, 'source'),
          source_kind: childValue(child, 'source_kind'),
        });
      }
      out.push({
        slug: entry.name,
        term,
        category: childValue(entry, 'category'),
        aliases: splitPipeList(childValue(entry, 'aliases')),
        contexts: splitPipeList(childValue(entry, 'contexts')),
        context_links: splitPipeList(childValue(entry, 'context_links')),
        wikidata: childValue(entry, 'wikidata'),
        summary,
        source: childValue(entry, 'source'),
        source_kind: childValue(entry, 'source_kind'),
        localized,
      });
    }
    return out;
  });
}
