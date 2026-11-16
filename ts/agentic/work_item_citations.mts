// The obligations of a work item, apart from what it only cites (issue #1562).
//
// A bug report about another tool cites the evidence it was read from: the
// address of a test repository such as `test-hello-world-…`, a source file in
// a code span, a table of evidence, a quoted log. None of those spans asks for
// anything. `obligationText` keeps the work item's own words and blanks every
// citation span with spaces of the same UTF-8 byte length, so a byte span of
// the obligation text is the byte span of the same words in the work item. The
// citations are: fenced code blocks, block quotations (`>`), table rows
// (`|`), inline code spans, Markdown links and images (`[text](target)`),
// autolinks (`<scheme://…>`) and bare addresses (a token holding `://` or
// starting with `www.`). A quoted phrase inside a sentence is the work item's
// own naming of what it asks for and is kept.
// Native twin: rust/src/agentic_coding/work_item_citations.rs.

import { programContractAnswer } from './crate/coding_program_contract.mjs';
import { traceRoute } from './planner/continuation.mjs';

const FENCES = ['```', '~~~'];

/** Mirrors `SCHEME_SEPARATOR`: the separator that marks an address. */
const SCHEME_SEPARATOR = '://';

/** Mirrors `WEB_HOST`: the host prefix of an address written without a scheme. */
const WEB_HOST = 'www.';

/** Mirrors `IMAGE_OPENER`: the opener of a Markdown image. */
const IMAGE_OPENER = '![';

const ENCODER = new TextEncoder();

/** Mirrors `fn blank`: one space per UTF-8 byte of `span`, so byte offsets are kept. */
function blank(span) {
  return ' '.repeat(ENCODER.encode(span).length);
}

/** Mirrors `fn link_end`: the length of a Markdown link or image at the start of `rest`, or null. */
function linkEnd(rest) {
  const open = rest.indexOf('[');
  if (open < 0) return null;
  let depth = 0;
  let close = null;
  for (let offset = open; offset < rest.length; offset += 1) {
    if (rest[offset] === '[') depth += 1;
    else if (rest[offset] === ']') {
      depth = Math.max(0, depth - 1);
      if (depth === 0) {
        close = offset;
        break;
      }
    }
  }
  if (close === null || rest[close + 1] !== '(') return null;
  const end = rest.indexOf(')', close + 2);
  return end < 0 ? null : end + 1;
}

/** Mirrors `fn citation_end`: the end of the citation starting at `start`, or `start`. */
function citationEnd(line, start) {
  const rest = line.slice(start);
  if (rest.startsWith('`')) {
    const run = rest.length - rest.replace(/^`+/u, '').length;
    const close = rest.indexOf(rest.slice(0, run), run);
    return close < 0 ? start : start + close + run;
  }
  if (rest.startsWith('[') || rest.startsWith(IMAGE_OPENER)) {
    const end = linkEnd(rest);
    if (end !== null) return start + end;
  }
  if (rest.startsWith('<')) {
    const close = rest.indexOf('>');
    if (close > 0 && rest.slice(1, close).includes(SCHEME_SEPARATOR) && !/\s/u.test(rest.slice(1, close))) return start + close + 1;
  }
  const previous = start > 0 ? line[start - 1] : null;
  if (previous === null || /\s/u.test(previous) || '([{<"\''.includes(previous)) {
    const space = rest.search(/\s/u);
    const token = space < 0 ? rest : rest.slice(0, space);
    if (token.includes(SCHEME_SEPARATOR) || token.toLowerCase().startsWith(WEB_HOST)) return start + token.length;
  }
  return start;
}

/** Mirrors `fn without_inline_citations`. */
function withoutInlineCitations(line) {
  let kept = '';
  let index = 0;
  while (index < line.length) {
    const end = citationEnd(line, index);
    if (end > index) {
      kept += blank(line.slice(index, end));
      index = end;
      continue;
    }
    const character = String.fromCodePoint(line.codePointAt(index));
    kept += character;
    index += character.length;
  }
  return kept;
}

/** Mirrors `fn obligation_text`: the work item with every citation span blanked. */
export function obligationText(text) {
  const source = String(text);
  let kept = '';
  let fenced = false;
  for (const line of source.match(/[^\n]*\n|[^\n]+$/gu) ?? []) {
    const ending = line.endsWith('\r\n') ? '\r\n' : line.endsWith('\n') ? '\n' : '';
    const body = line.slice(0, line.length - ending.length);
    const trimmed = body.trimStart();
    const fence = FENCES.some(marker => trimmed.startsWith(marker));
    if (fence || fenced || trimmed.startsWith('>') || trimmed.startsWith('|')) {
      if (fence) fenced = !fenced;
      kept += blank(body);
    } else {
      kept += withoutInlineCitations(body);
    }
    kept += ending;
  }
  return kept;
}

/** Mirrors `fn cites_anything`. */
export function citesAnything(text) {
  return obligationText(text) !== String(text);
}

/**
 * Mirrors `fn is_citation`: whether the UTF-8 byte span `[start, end]` of the
 * work item holds only citations, given its obligation text.
 */
export function isCitation(obligations, [start, end]) {
  const bytes = ENCODER.encode(obligations);
  return end <= bytes.length && start < end && bytes.subarray(start, end).every(byte => byte === 0x20 || byte === 0x0a || byte === 0x0d || byte === 0x09);
}

/**
 * Mirrors `fn unbind_cited_recipe`: drop the recipe when no obligation of the
 * work item names it and return the path it would have written, or null.
 */
export function unbindCitedRecipe(answer, objective, solve = programContractAnswer) {
  const recipe = answer?.execution_recipe;
  if (!recipe) return null;
  const obligations = obligationText(objective);
  if (obligations === objective) return null;
  const obliged = solve(obligations)?.execution_recipe ?? null;
  const bound = obliged !== null && obliged.path === recipe.path && obliged.language === recipe.language;
  traceRoute('work_item_recipe_binding', { path: recipe.path, language: recipe.language, bound });
  if (bound) return null;
  answer.execution_recipe = null;
  return recipe.path;
}
