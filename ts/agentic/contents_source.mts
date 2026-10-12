// Another file's contents as the text an additive edit places (PR #1188 G50,
// G51): `Append the contents of a.txt to the end of b.txt`, `Insert the
// contents of rows.txt after the line 'x' in f.lino`. The seeded
// `file_contents_source_cue` beside a path names the source file; once it is
// read, the request is restated with the source's lines under it as a fenced
// block, which the additive-edit composers (end insertion, positional insert)
// already place verbatim. The JavaScript twin of
// rust/src/agentic_coding/contents_source.rs.

import { cleanPathToken, looksLikeFilePath, safeRelativePath, tokens } from './write_request.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { wordsForRole } from './write_lexicon.mjs';

const ROLE = 'file_contents_source_cue';
const FENCE = '```';

/**
 * Mirrors `fn contents_source`: `{path, start, end}` -- the source path a
 * seeded contents cue names (the path right after the cue, or right before it
 * in a postpositional language) and the span of cue and path in the request's
 * first line -- or null. A cue inside a quoted literal is text, not a source.
 * @param {string} task
 */
export function contentsSource(task) {
  const breakAt = task.indexOf('\n');
  const line = breakAt < 0 ? task : task.slice(0, breakAt);
  // Offsets in the lowered line are the line's own while lowering keeps its length.
  const lowered = line.toLowerCase().length === line.length ? line.toLowerCase() : line;
  const quoted = quotedSegmentSpans(line);
  const words = tokens(line);
  const isPath = (token) => token !== undefined && looksLikeFilePath(cleanPathToken(token.text))
    && safeRelativePath(cleanPathToken(token.text));
  for (const cue of wordsForRole(ROLE)) {
    const surface = cue.toLowerCase();
    if (surface === '') continue;
    for (let at = lowered.indexOf(surface); at >= 0; at = lowered.indexOf(surface, at + 1)) {
      const end = at + surface.length;
      if (quoted.some((segment) => at >= segment.start && at < segment.end)) continue;
      const following = words.find((token) => token.start >= end);
      const preceding = words.filter((token) => token.end <= at).pop();
      const onlySpaceBetween = (from, to) => line.slice(from, to).trim() === '';
      if (isPath(following) && onlySpaceBetween(end, following.start)) {
        return { path: cleanPathToken(following.text), start: at, end: following.start + following.text.length };
      }
      if (isPath(preceding) && onlySpaceBetween(preceding.end, at)) {
        return { path: cleanPathToken(preceding.text), start: preceding.start, end };
      }
    }
  }
  return null;
}

/**
 * Mirrors `fn with_contents`: the request's first line without the cue and
 * the source path, ending in a colon, and the source's lines under it as a
 * fenced block (one final line break is the file's, not a line).
 * @param {string} task
 * @param {{start: number, end: number}} source
 * @param {string} contents
 */
export function withContents(task, source, contents) {
  const breakAt = task.indexOf('\n');
  const line = breakAt < 0 ? task : task.slice(0, breakAt);
  const head = `${line.slice(0, source.start)}${line.slice(source.end)}`.trimEnd().replace(/[.!。]$/u, '');
  const body = contents.endsWith('\n') ? contents.slice(0, -1) : contents;
  return `${head}:\n${FENCE}\n${body}\n${FENCE}`;
}
