// A work item's text is split into requirements before any evidence is
// collected, so a refusal or a gap can name the requirement it concerns
// (issue #1560). The text is read as Markdown blocks: a paragraph, a list
// item (without its marker) or a fenced block is one candidate; a heading or a
// rule closes a block and is not a requirement. Each block is then split at enumeration cues by the
// shared clause splitter. Spans are UTF-8 byte offsets into the whole text.
// Native twin: rust/src/agentic_coding/work_item_requirements.rs.
import { clausesWithSpans } from './crate/obligation_ledger.mjs';
import { trim, utf8Len } from './write_str.mjs';

const FENCE = /^(?:```|~~~)/u;
const HEADING = /^#{1,6}(?:\s|$)/u;
const RULE = /^(?:(?:-\s*){3,}|(?:\*\s*){3,}|(?:_\s*){3,})$/u;
const LIST_ITEM = /^[ \t]*(?:[-*+]|[0-9]{1,9}[.)])[ \t]+/u;

/** Mirrors `fn work_item_blocks`: `[start, end]` UTF-8 byte ranges of the Markdown blocks. */
function workItemBlocks(text) {
  const blocks = [];
  let open = null,
    fenced = false,
    offset = 0;
  const close = () => {
    if (open !== null) blocks.push(open);
    open = null;
  };
  for (const raw of text.split('\n')) {
    const start = offset,
      end = start + utf8Len(raw);
    offset = end + 1;
    const line = trim(raw);
    if (fenced) {
      open[1] = end;
      if (FENCE.test(line)) fenced = false;
      continue;
    }
    if (FENCE.test(line)) {
      if (open === null) open = [start, end];
      open[1] = end;
      fenced = true;
      continue;
    }
    if (line === '' || HEADING.test(line) || RULE.test(line)) {
      close();
      continue;
    }
    // A list item opens its own block after its marker, so `1. First` is
    // not cut at the cue that follows the marker's full stop.
    const marker = LIST_ITEM.exec(raw);
    if (marker !== null) {
      close();
      open = [start + utf8Len(marker[0]), end];
    }
    if (open === null) open = [start, end];
    open[1] = end;
  }
  close();
  return blocks;
}

/**
 * Mirrors `fn work_item_requirements`: `[text, [start, end]]` in UTF-8 bytes,
 * the same shape as `clausesWithSpans`.
 */
export function workItemRequirements(text) {
  const bytes = new TextEncoder().encode(text),
    decoder = new TextDecoder();
  const out = [];
  for (const [start, end] of workItemBlocks(text)) {
    const block = decoder.decode(bytes.subarray(start, end));
    for (const [clause, [from, to]] of clausesWithSpans(block)) {
      if (clause !== '') out.push([clause, [start + from, start + to]]);
    }
  }
  return out.length ? out : [[trim(text), [0, bytes.length]]];
}

/**
 * Mirrors `fn requirement_naming`: the index of the first requirement whose
 * text names `path`, or `null` when none does.
 */
export function requirementNaming(requirements, path) {
  if (typeof path !== 'string' || path === '') return null;
  const index = requirements.findIndex(requirement => requirement.text.includes(path));
  return index < 0 ? null : index;
}
