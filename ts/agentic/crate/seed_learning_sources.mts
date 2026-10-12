// The learnable-source registry (issue #499): `LearningSource`,
// `LearningSources::match_directive` and `learning_sources` from
// rust/src/seed.rs, over data/seed/learning-sources.lino.

import { cached, readText } from '../host.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

/**
 * Mirrors `fn learning_sources` in rust/src/seed.rs: `{sources: [{id,
 * capability, host, keywords}], directive_cues}`.
 */
export function learningSources() {
  return cached('seed-learning-sources', () => {
    const registry = { sources: [], directive_cues: [] };
    const root = parseRoot(readText('data/seed/learning-sources.lino')).children[0];
    for (const child of root?.children || []) {
      if (child.name === 'source') {
        registry.sources.push({
          id: child.value,
          capability: findChildValue(child, 'capability'),
          host: findChildValue(child, 'host'),
          keywords: (child.children || []).filter((entry) => entry.name === 'keyword').map((entry) => entry.value),
        });
      } else if (child.name === 'directive') {
        for (const entry of (child.children || []).filter((cue) => cue.name === 'cue')) registry.directive_cues.push(entry.value);
      }
    }
    return registry;
  });
}

/**
 * Mirrors `LearningSources::match_directive` in rust/src/seed.rs: the source a
 * lowercased prompt teaches the engine to learn from, or null. Needs both a
 * directive cue and a reference to the source (its host or a keyword).
 * @param {{sources: Array<object>, directive_cues: Array<string>}} registry
 * @param {string} lowercased
 */
export function matchDirective(registry, lowercased) {
  if (!registry.directive_cues.some((cue) => lowercased.includes(cue))) return null;
  return registry.sources.find((source) => (source.host !== '' && lowercased.includes(source.host))
    || source.keywords.some((keyword) => lowercased.includes(keyword))) ?? null;
}
