// Structural meaning selection from rust/src/coding/concept_discovery.rs.
// The seed declares reusable structures and language surfaces, never task answers.
import { cached, childValue, childrenNamed, readText } from '../host.mjs';
import { parseLinoRoot } from '../write_lino.mjs';
import { normalizePrompt } from './engine.mjs';
import { evidencedIn, meaning, meaningsWithRole, words } from './seed_meanings.mjs';
import { splitWhitespace, trim } from './rust_str.mjs';

export function structuralMeanings() {
  return cached('formalization-structural-meanings', () => [
    'meanings-coding-structure.lino', 'meanings-coding-structure-2.lino',
  ].flatMap(name => childrenNamed(parseLinoRoot(readText(`data/seed/${name}`)), 'meanings')
    .flatMap(container => container.children.map(node => ({
      id: node.name === 'meaning' ? node.id : node.name,
      idiom: childValue(node, 'idiom'), grounding: childValue(node, 'grounding'),
    })))));
}

function slottedSurfacePresent(normalized, surface) {
  const at = surface.indexOf('…');
  if (at < 0) return false;
  const before = splitWhitespace(trim(normalizePrompt(surface.slice(0, at))));
  const after = splitWhitespace(trim(normalizePrompt(surface.slice(at + 1))));
  if (!before.length || !after.length) return false;
  const tokens = splitWhitespace(normalized);
  const matches = (start, expected) => expected.every((token, offset) => tokens[start + offset] === token);
  for (let start = 0; start + before.length <= tokens.length; start += 1) {
    if (!matches(start, before)) continue;
    for (let width = 1; width <= 6; width += 1) {
      if (matches(start + before.length + width, after)) return true;
    }
  }
  return false;
}

function editDistanceAtMostOne(left, right) {
  if (left === right) return true;
  if (Math.abs(left.length - right.length) > 1) return false;
  const [shorter, longer] = left.length <= right.length ? [left, right] : [right, left];
  let short = 0, long = 0, edits = 0;
  while (short < shorter.length && long < longer.length) {
    if (shorter[short] === longer[long]) { short += 1; long += 1; continue; }
    edits += 1;
    if (edits > 1) return false;
    if (shorter.length === longer.length) short += 1;
    long += 1;
  }
  return edits + Number(long < longer.length) <= 1;
}

function surfaceMatches(normalized, surface) {
  const expected = normalizePrompt(surface);
  return (expected !== '' && ` ${normalized} `.includes(` ${expected} `))
    || slottedSurfacePresent(normalized, surface)
    || (expected.length >= 8 && /^[A-Za-z]+$/u.test(expected)
      && splitWhitespace(normalized).some(token => /^[A-Za-z]+$/u.test(token)
        && editDistanceAtMostOne(token, expected)));
}

/** Same precedence and reduction/quantifier conflict rules as native discovery. */
export function structuresFor(normalized) {
  const matched = new Set(meaningsWithRole('coding_structure')
    .filter(candidate => evidencedIn(candidate, normalized)
      || words(candidate).some(surface => surfaceMatches(normalized, surface)))
    .map(candidate => candidate.slug));
  if (matched.has('pairwise_distinct')) matched.delete('distinct_elements');
  if (matched.has('quantifier_any')) matched.delete('quantifier_all');
  if ([...matched].some(slug => meaning(slug)?.defined_by.includes('reduction'))) matched.delete('quantifier_all');
  if (matched.has('range_inclusive')) { matched.delete('reduce_count'); matched.delete('membership'); }
  return structuralMeanings().filter(structure => matched.has(structure.id));
}
