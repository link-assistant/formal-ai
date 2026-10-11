import assert from 'node:assert/strict';
import {factorSharedLexemeFields, expandSharedLexemeFields} from './notation-shared-lexeme-fields.mjs';

/** A source-format choice, never a runtime or publication authority. */
export function registeredSharedLexemeShard(rules, fileName) {
  let sharedFamily = false;
  let selected = false;
  for (const line of rules.split('\n')) {
    if (line.startsWith('  family ')) sharedFamily = false;
    else if (line.startsWith('    rule ')) {
      sharedFamily = line.slice('    rule '.length) === 'shared-lexeme-fields';
    } else if (sharedFamily && line.startsWith('    files ')) {
      const source = line.slice('    files '.length);
      assert.ok(source.startsWith('data/seed/'), 'shared lexeme output must name a seed source');
      const relative = source.slice('data/seed/'.length);
      assert.ok(!/[*?\[\]]/u.test(relative), 'explicit shared source required');
      selected ||= relative === fileName;
    }
  }
  return selected;
}

export function projectLexemeImportShard(rules, fileName, content) {
  if (!registeredSharedLexemeShard(rules, fileName)) return content;
  const projected = factorSharedLexemeFields(content).text;
  assert.equal(expandSharedLexemeFields(projected), content, 'full generated source inverse');
  return projected;
}
