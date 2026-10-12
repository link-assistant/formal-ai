// A Markdown section as the scope of an additive edit (PR #1188 G35):
// `Insert the line 'x' at the end of the section '## Usage' in README.md`.
// The section runs from its heading to the line before the next heading of
// the same or a higher level (fenced code is not a heading). The JavaScript
// twin of rust/src/agentic_coding/markdown_section.rs.

import { mentionsRole } from './write_lexicon.mjs';
import { namedTargetAndPayloads } from './workspace_line_operation.mjs';

const FENCE = '```';

/** Mirrors `fn heading_level`: the level of a Markdown ATX heading line, or null. */
export function headingLevel(line) {
  const trimmed = line.trim();
  let level = 0;
  while (level < trimmed.length && trimmed[level] === '#') level += 1;
  if (level === 0 || level > 6) return null;
  return level === trimmed.length || /\s/u.test(trimmed[level]) ? level : null;
}

/**
 * Mirrors `fn section_scope`: `{target, heading, text}` -- the seeded
 * `file_section_noun` outside the request's quotes, two quoted payloads of
 * which one is a Markdown heading (the later, when both are) and the other
 * the text placed -- or null.
 * @param {string} task
 * @param {string} outside the lowered request outside its quotes
 */
export function sectionScope(task, outside) {
  if (!mentionsRole('file_section_noun', outside)) return null;
  const named = namedTargetAndPayloads(task);
  if (!named || named.payloads.length !== 2) return null;
  const headings = named.payloads.filter((payload) => headingLevel(payload) !== null);
  if (headings.length === 0) return null;
  const heading = headings[headings.length - 1];
  const text = named.payloads.find((payload) => payload !== heading);
  return text === undefined ? null : { target: named.target, heading, text };
}

/**
 * Mirrors `fn inserted_in_section`: `{updated, anchor}` -- `source` with
 * `text` placed after the section's last non-blank line (`atEnd`) or after
 * its heading and the blank line under it, and the line it now follows -- or
 * null when the heading is not one line of the file.
 */
export function insertedInSection(source, heading, text, atEnd) {
  const lines = source.split('\n');
  const found = lines.flatMap((line, index) => (line.trim() === heading.trim() ? [index] : []));
  if (found.length !== 1) return null;
  const [start] = found;
  const level = headingLevel(lines[start]);
  let end = lines.length;
  let fenced = false;
  for (let index = start + 1; index < lines.length; index += 1) {
    if (lines[index].trimStart().startsWith(FENCE)) fenced = !fenced;
    const nested = fenced ? null : headingLevel(lines[index]);
    if (nested !== null && nested <= level) {
      end = index;
      break;
    }
  }
  let after = start;
  if (atEnd) {
    for (let index = end - 1; index > start; index -= 1) {
      if (lines[index].trim() !== '') {
        after = index;
        break;
      }
    }
  } else if (start + 1 < end && lines[start + 1].trim() === '') {
    after = start + 1;
  }
  const updated = [...lines.slice(0, after + 1), ...text.split('\n'), ...lines.slice(after + 1)].join('\n');
  return { updated, anchor: atEnd ? lines[after] : lines[start] };
}
