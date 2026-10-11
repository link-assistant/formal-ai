// Preserve the original native all-table equality and real source-registry input.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { before, test } from 'node:test';
import { WorkerHost, REPO_ROOT } from '../../../js/server/worker-host.mjs';
import { tokenize } from '../../../scripts/lib/rust-specification-cases.mjs';
import { parseLino } from '../../../js/server/lino.mjs';
let host;
before(async () => { host=new WorkerHost(); await host.boot(); });
const read = file => readFileSync(path.join(REPO_ROOT,file),'utf8');
const staticTable = (source,name) => [...source.split(`${name}:`)[1].split('];')[0].matchAll(/\("([^"]*)", "([^"]*)"\)/gu)].map(match=>[tokenize(`"${match[1]}"`)[0].text,tokenize(`"${match[2]}"`)[0].text]);

test('the native source registry carries actual canonical number-word bytes in both inventories',()=>{
  const registry=read('rust/tests/source/seed/embedded.rs');
  const constants=new Map([...registry.matchAll(/pub const (\w+): &str =\s*include_str!\("([^"]+)"\)/gu)].map(match=>[match[1],path.resolve(REPO_ROOT,'rust/tests/source/seed',match[2])]));
  const meaningNames=[...registry.split('pub const MEANING_FILES:')[1].matchAll(/\b(MEANINGS_\w+_LINO)\b/gu)].map(match=>match[1]);
  assert.equal(constants.get('MEANINGS_NUMBER_WORDS_LINO'),path.join(REPO_ROOT,'data/seed/meanings-number-words.lino'));
  assert.ok(meaningNames.includes('MEANINGS_NUMBER_WORDS_LINO'));
  assert.match(registry,/\(\s*"data\/seed\/meanings-number-words\.lino",\s*MEANINGS_NUMBER_WORDS_LINO\s*,?\s*\)/u);
  const meanings=meaningNames.flatMap(name=>parseLino(readFileSync(constants.get(name),'utf8')).children);
  for(let value=0;value<=10;value++){
    const numeric=meanings.filter(meaning=>meaning.children.some(field=>field.name==='role'&&field.value==='cardinal_number_word')
      &&meaning.children.filter(field=>field.name==='lexeme').some(lexeme=>lexeme.children.some(surface=>surface.children.some(field=>field.name==='text'&&field.value===String(value)))));
    assert.equal(numeric.length,1,`canonical cardinal ${value} survives the actual source registry`);
  }
});

test('all original no_std token and phrase tables match the actual multilingual worker lexicon',async()=>{
  const tables=await host.run('arithmeticNormalizationTables()');
  const production=read('rust/src/arithmetic_word_tables.rs');
  const snapshot=read('rust/tests/source/arithmetic_word_tables.rs');
  for(const [name,key] of [['WORD_VALUE_TOKENS','tokens'],['WORD_VALUE_PHRASES','phrases']]){
    assert.deepEqual(tables[key],staticTable(production,name));
    assert.deepEqual(tables[key],staticTable(snapshot,name));
  }
});
