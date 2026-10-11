import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync, readdirSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {projectLexemeImportShard, registeredSharedLexemeShard} from '../../../scripts/lib/lexeme-import-notation.mjs';
import {expandSharedLexemeFields} from '../../../scripts/lib/notation-shared-lexeme-fields.mjs';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const rules = readFileSync(root + 'data/meta/notation-rules.lino', 'utf8');

test('every committed importer shard is exactly reproduced by its registered source projection', () => {
  const names = readdirSync(root + 'data/seed').filter(name => name.startsWith('meanings-lexicon-import-')).sort();
  assert.equal(names.length, 5);
  for (const name of names) {
    const committed = readFileSync(root + 'data/seed/' + name, 'utf8');
    const expanded = expandSharedLexemeFields(committed);
    assert.equal(projectLexemeImportShard(rules, name, expanded), committed, name);
    assert.equal(expandSharedLexemeFields(projectLexemeImportShard(rules, name, expanded)), expanded);
  }
});

test('family registration selects arbitrary source names without filename exceptions or state leakage', () => {
  const source = 'notation-rules\n  family arbitrary\n    rule shared-lexeme-fields\n    files data/seed/new-shard.lino\n  family other\n    files data/seed/untouched.lino\n';
  assert.equal(registeredSharedLexemeShard(source, 'new-shard.lino'), true);
  assert.equal(registeredSharedLexemeShard(source, 'untouched.lino'), false);
  assert.equal(registeredSharedLexemeShard(source, 'unknown.lino'), false);
});

test('unsupported output namespaces and wildcard registration refuse', () => {
  for (const source of ['data/meta/file.lino', 'data/seed/*.lino']) {
    const rules = 'notation-rules\n  family arbitrary\n    rule shared-lexeme-fields\n    files ' + source + '\n';
    assert.throws(() => registeredSharedLexemeShard(rules, 'file.lino'));
  }
});

test('native production applies projection only after unchanged shard geometry', () => {
  const source = readFileSync(root + 'rust/src/lexeme_import.rs', 'utf8');
  assert.match(source, /let block = render_block\(lexeme\);/u);
  assert.match(source, /let block_lines = block.lines\(\).count\(\);/u);
  assert.match(source, /content = notation::project\(&file_name, content\);\s*Shard \{ file_name, content \}/u);
});

test('native notation source remains inside the published crate with an exact canonical mirror', () => {
  const module = readFileSync(root + 'rust/src/lexeme_import/notation.rs', 'utf8');
  assert.match(module, /include_str!\("\.\.\/\.\.\/embedded\/data\/meta\/notation-rules\.lino"\)/u);
  assert.equal(readFileSync(root + 'rust/embedded/data/meta/notation-rules.lino', 'utf8'), rules);
  assert.match(readFileSync(root + 'rust/Cargo.toml', 'utf8'), /"\/embedded\/\*\*"/u);
});
