// Shared lexical proof only; source is observed and never evaluated.
import { tokenize, tokenizeWithSpans } from './crate/es_tokenizer.mjs';
import { validateProgram } from './host.mjs';
import { normalizeTrailingLayout } from './portable/source_layout.mjs';

function lexicalIntervals(trees, output) {
  for (const tree of trees) {
    if (tree.$ === 'group') {
      output.push([tree.span.start, tree.span.start + 1], [tree.span.end - 1, tree.span.end]);
      lexicalIntervals(tree.trees, output);
    } else {
      // Whole templates stay opaque, including their interpolation bodies.
      output.push([tree.span.start, tree.span.end]);
    }
  }
}

function protectedCharacters(source, intervals) {
  const characters = Array.from(source);
  const protectedValues = [];
  const encoder = new TextEncoder();
  const bytes = encoder.encode(source);
  const decoder = new TextDecoder('utf8');
  const ordered = intervals.sort((left, right) => left[0] - right[0]);
  let intervalIndex = 0;
  let byteOffset = 0;
  let gapStart = 0;
  let gapEnd = ordered[0]?.[0] ?? bytes.length;
  let gapProtected = !/^[\t\n\r ]*$/u.test(decoder.decode(bytes.subarray(gapStart, gapEnd)));
  for (const character of characters) {
    while (intervalIndex < ordered.length && byteOffset >= ordered[intervalIndex][1]) {
      gapStart = ordered[intervalIndex][1];
      intervalIndex += 1;
      gapEnd = ordered[intervalIndex]?.[0] ?? bytes.length;
      gapProtected = !/^[\t\n\r ]*$/u.test(decoder.decode(bytes.subarray(gapStart, gapEnd)));
    }
    const interval = ordered[intervalIndex];
    const tokenProtected = interval !== undefined && byteOffset >= interval[0] && byteOffset < interval[1];
    protectedValues.push(tokenProtected || gapProtected);
    byteOffset += encoder.encode(character).length;
  }
  return { characters, protectedValues };
}

export function deriveSourceMaintenance(property, language, source) {
  if (property !== 'trailing-horizontal-whitespace' || !['javascript', 'typescript'].includes(language)) {
    return { gap: 'SourceMaintenancePropertyUnsupported' };
  }
  if (typeof source !== 'string' || new TextEncoder().encode(source).length > 131072) {
    return { gap: 'SourceMaintenanceInputLimit' };
  }
  // Existing host syntax capability is a refusal guard, not the transformation proof.
  const syntax = validateProgram(language, source);
  if (syntax === null) return { gap: 'SourceMaintenanceValidatorUnavailable' };
  if (syntax !== true) return { gap: 'SourceMaintenanceInvalidSource' };
  try {
    const intervals = [];
    lexicalIntervals(tokenizeWithSpans(source), intervals);
    const observation = protectedCharacters(source, intervals);
    const result = normalizeTrailingLayout(observation.characters, observation.protectedValues);
    if (result.$ === 'refused') return { gap: result.reason };
    if (JSON.stringify(tokenize(source)) !== JSON.stringify(tokenize(result.source))) {
      return { gap: 'SourceMaintenanceTokenMismatch' };
    }
    if (JSON.stringify(source.match(/[\r\n]/gu)) !== JSON.stringify(result.source.match(/[\r\n]/gu))) {
      return { gap: 'SourceMaintenanceLinebreakMismatch' };
    }
    return { source: result.source, removedBytes: result.removed };
  } catch {
    return { gap: 'SourceMaintenanceUnsupportedLexicalSource' };
  }
}
