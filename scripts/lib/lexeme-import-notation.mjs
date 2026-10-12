import assert from 'node:assert/strict';
import {parseLino, childrenNamed} from '../../js/server/lino.mjs';
import {factorSharedLexemeFields, expandSharedLexemeFields} from './notation-shared-lexeme-fields.mjs';

/** A source-format choice, never a runtime or publication authority. */
export function registeredSharedLexemeShard(rules, fileName) {
  const document = parseLino(rules);
  const roots = document.name === 'notation-rules' ? [document] : childrenNamed(document, 'notation-rules');
  let selected = false;
  for (const root of roots) for (const family of childrenNamed(root, 'family')) {
    let sharedFamily = false;
    for (const field of family.children) {
      if (field.name === 'rule') sharedFamily = field.id === 'shared-lexeme-fields';
      else if (sharedFamily && field.name === 'files') {
        assert.ok(field.id.startsWith('data/seed/'), 'shared lexeme output must name a seed source');
        const relative = field.id.slice('data/seed/'.length);
        assert.ok(!/[*?\[\]]/u.test(relative), 'explicit shared source required');
        selected ||= relative === fileName;
      }
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
