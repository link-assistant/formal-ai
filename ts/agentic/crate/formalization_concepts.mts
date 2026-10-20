// Retrieved sense -> attributed concept / grounded relation, matching
// rust/src/formalization/concepts.rs. Glosses select structure; they never emit code.
import { cached, childValue, childrenNamed, readText } from '../host.mjs';
import { parseLinoRoot } from '../write_lino.mjs';
import { normalizePrompt } from './engine.mjs';
import { stableId } from './engine_stable_identifier.mjs';
import { structuresFor } from './formalization_structures.mjs';
import { isWordBoundary } from './source_walk.mjs';
import { byteOrder, charCount, isAlphanumeric, trim, trimStart } from './rust_str.mjs';

function surfacesOf(node) {
  return childrenNamed(node, 'lexeme').flatMap(lexeme => childrenNamed(lexeme, 'surface')
    .map(surface => [lexeme.id, childValue(surface, 'text')])
    .filter(([, text]) => text !== ''));
}

function relationSeed() {
  return cached('formalization-relation-seed', () => parseLinoRoot(readText('data/seed/formalization-relations.lino')));
}

export function relations() {
  return cached('formalization-relations', () => childrenNamed(relationSeed(), 'relations')
    .flatMap(container => container.children.map(node => ({
      slug: node.name, kind: childValue(node, 'kind'), inverse: childValue(node, 'inverse'),
      grounding: childValue(node, 'grounding'),
      cues: surfacesOf(node).sort((left, right) => charCount(right[1]) - charCount(left[1])),
    }))));
}

function determiners() {
  return cached('formalization-determiners', () => childrenNamed(relationSeed(), 'determiners')
    .flatMap(surfacesOf).map(([, text]) => text)
    .sort((left, right) => charCount(right) - charCount(left)));
}

/** Lower one Unicode scalar at a time, preserving original source boundaries. */
function foldedSource(text) {
  let folded = '', position = 0;
  const boundaries = new Map([[0, 0]]);
  for (const character of text) {
    folded += character.toLowerCase();
    position += character.length;
    boundaries.set(folded.length, position);
  }
  return { folded, boundaries };
}

/** Returns original UTF-16 span; a case expansion cannot invent a source boundary. */
export function findCue(text, cue) {
  const { folded, boundaries } = foldedSource(text);
  const needle = Array.from(cue, character => character.toLowerCase()).join('');
  if (!needle) return null;
  let from = 0;
  while (from <= folded.length - needle.length) {
    const start = folded.indexOf(needle, from);
    if (start < 0) return null;
    const end = start + needle.length;
    const before = Array.from(folded.slice(0, start)).at(-1);
    const after = Array.from(folded.slice(end))[0];
    if (boundaries.has(start) && boundaries.has(end)
      && (before === undefined || isWordBoundary(before))
      && (after === undefined || isWordBoundary(after))) return [boundaries.get(start), boundaries.get(end)];
    from = end;
  }
  return null;
}

export function conceptId(lemma) {
  let id = 'concept:', previousDash = false;
  for (const character of trim(lemma)) {
    if (isAlphanumeric(character)) { id += character.toLowerCase(); previousDash = false; }
    else if (!previousDash) { id += '-'; previousDash = true; }
  }
  return id.replace(/-+$/u, '');
}

function headTerm(text) {
  let rest = trim(text);
  for (const determiner of determiners()) {
    const prefix = `${determiner} `;
    if (rest.toLowerCase().startsWith(prefix)) { rest = trimStart(rest.slice(prefix.length)); break; }
  }
  let token = '';
  for (const character of rest) {
    if (isWordBoundary(character)) { if (token) break; }
    else token += character;
  }
  return token ? token.toLowerCase() : null;
}

function differentiaeOf(gloss, genus) {
  const at = findCue(gloss, genus);
  const after = at ? gloss.slice(at[1]) : gloss;
  const out = [];
  for (const definition of relations()) {
    if (definition.slug === 'is_a') continue;
    for (const [, cue] of definition.cues) {
      const found = findCue(after, cue);
      if (!found) continue;
      const tail = trim(after.slice(found[1]));
      if (tail && !out.includes(tail)) out.push(tail);
    }
  }
  if (!out.length && trim(after)) out.push(trim(after));
  return out;
}

export function readGloss(gloss) {
  for (const definition of relations()) {
    if (definition.slug !== 'is_a') continue;
    for (const [, cue] of definition.cues) {
      const found = findCue(gloss, cue);
      const head = found ? headTerm(gloss.slice(found[1])) : null;
      if (head) return [head, differentiaeOf(gloss, head)];
    }
  }
  const head = headTerm(gloss);
  return [head, head ? differentiaeOf(gloss, head) : []];
}

export function conceptFromSense(sense) {
  const lemma = trim(sense.lemma), gloss = trim(sense.gloss);
  if (!lemma || !gloss) return null;
  const [genus, differentiae] = readGloss(gloss);
  return {
    id: conceptId(lemma), label: lemma, language: sense.language, gloss, genus, differentiae,
    structures: structuresFor(normalizePrompt(gloss)).map(structure => structure.id),
    source_id: sense.source_id, source_url: sense.source_url, sha256: sense.sha256,
    license_name: sense.license_name, depth: sense.depth,
  };
}

function groundedTerms(graph, text) {
  return [...graph.concepts, ...graph.entities].flatMap(term => {
    const found = findCue(text, term.label);
    return found ? [[found[0], term.id]] : [];
  }).sort((left, right) => left[0] - right[0]).map(([, id]) => id);
}

export function relationsIn(clause, graph) {
  const out = [];
  for (const definition of relations()) {
    for (const [language, cue] of definition.cues) {
      const found = findCue(clause.text, cue);
      if (!found) continue;
      const subject = groundedTerms(graph, clause.text.slice(0, found[0])).at(-1);
      const object = groundedTerms(graph, clause.text.slice(found[1]))[0];
      if (!subject || !object) continue;
      const id = stableId('relation', [definition.slug, subject, object].join('\x1f'));
      if (out.some(relation => relation.id === id)) continue;
      out.push({ id, kind: definition.slug, subject, object, cue,
        source_span: `${graph.doc_id}@${clause.start}:${clause.end}`, language });
    }
  }
  return out.sort((left, right) => byteOrder(left.id, right.id));
}

/** Stable evidence identity over the same four fields as ConceptSense::content_id. */
export function conceptSenseContentId(sense) {
  return stableId('sense', [sense.sha256, sense.lemma, sense.language, sense.gloss].join('\x1f'));
}
