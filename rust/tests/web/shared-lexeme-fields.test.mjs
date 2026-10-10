import assert from 'node:assert/strict';
import {readFileSync}from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import test from 'node:test';
const root=path.resolve(import.meta.dirname,'../../..');
const context={self:{}};
vm.runInNewContext(readFileSync(path.join(root,'js/seed_loader.js'),'utf8'),context);
const parse=context.self.FormalAiSeed.parse;
const surface=(word,fields)=>'      surface\n        text '+word+'\n'+fields;
const fields='        part_of_speech noun\n        grammatical_number singular\n';
const legacy='meanings\n  sample\n    lexeme en\n'+surface('first',fields)+'    lexeme es\n'+surface('second',fields);
const shared='meanings\n  sample\n    shared-lexeme-fields lexical\n      part_of_speech noun\n      grammatical_number singular\n'
 +'    lexeme en\n'+surface('first','        use-lexeme-fields lexical\n')
 +'    lexeme es\n'+surface('second','        use-lexeme-fields lexical\n');
const shape=node=>({name:node.name,id:node.id,value:node.value,children:Array.from(node.children,shape)});
test('scoped shared fields preserve all effective fields and order, including combined roots',()=>{
 assert.deepEqual(shape(parse(shared)),shape(parse(legacy)));
 assert.deepEqual(shape(parse(legacy+shared)),shape(parse(legacy+legacy)));
});
test('each expansion owns independent mutable nodes',()=>{
 const meaning=parse(shared).children[0];
 const lexemes=meaning.children.filter(node=>node.name==='lexeme');
 const first=lexemes[0].children[0].children[1];
 const second=lexemes[1].children[0].children[1];
 assert.notEqual(first,second);first.id='changed';assert.equal(second.id,'noun');
});
test('unresolved, ambiguous, nested, aliased, conflicting and effect-bearing fields refuse',()=>{
 const invalid=[
  shared.replace('shared-lexeme-fields lexical','shared-lexeme-fields'),
  shared.replace('shared-lexeme-fields lexical','shared-lexeme-fields lexical\n    shared-lexeme-fields other\n      part_of_speech noun'),
  shared.replace('use-lexeme-fields lexical','use-lexeme-fields missing'),
  shared.replace('use-lexeme-fields lexical','use-lexeme-fields lexical\n          use-lexeme-fields lexical'),
  shared.replace('use-lexeme-fields lexical','use-lexeme-fields lexical\n        use-lexeme-fields lexical'),
  shared.replace('part_of_speech noun','unknown-field noun'),
  shared.replace('part_of_speech noun','effect mutation'),
  shared.replace('part_of_speech noun','part_of_speech noun\n        nested child'),
  shared.replace('part_of_speech noun','part_of_speech noun\n      part_of_speech noun'),
  shared.replace('use-lexeme-fields lexical','part_of_speech noun\n        use-lexeme-fields lexical'),
  shared.replace('use-lexeme-fields lexical','use_lexeme_fields lexical'),
  shared.replace('lexeme es','lexeme en'),
  shared.replace('meanings\n','other-root\n'),
  shared.replace('use-lexeme-fields lexical','text missing-reference'),
  shared.replace('      part_of_speech noun','      text alias'),
 ];
 for(const candidate of invalid)assert.throws(()=>parse(candidate));
});
test('before and after declaration placement preserve parsed and raw source boundaries',()=>{
 const declaration='    shared-lexeme-fields local\n      part_of_speech noun\n';
 const refs='    lexeme en\n'+surface('first','        use-lexeme-fields local\n')
  +'    lexeme es\n'+surface('second','        use-lexeme-fields local\n');
 const expected='meanings\n  sample\n'+refs.replaceAll('use-lexeme-fields local','part_of_speech noun');
 for(const body of [declaration+refs,refs+declaration,
  declaration.replace('      part_of_speech noun','\n# declaration comment\n      part_of_speech noun')+refs]){
  const authored='meanings\n  sample\n'+body;
  const bundle='seed_bundle\n  file "seed/shared.lino"\n'+authored.split('\n').slice(0,-1).map(line=>'    '+line).join('\n')+'\n';
  const runtime=context.self.FormalAiSeed.loadFromBundle(bundle);
  assert.deepEqual(shape(parse(authored)),shape(parse(expected)));
  assert.deepEqual(shape(parse(runtime.raw['seed/shared.lino'])),shape(parse(expected)));
  assert.ok(!runtime.raw['seed/shared.lino'].includes('use-lexeme-fields'));
  assert.equal(runtime.sourceRaw['seed/shared.lino'],authored+'\n');
 }
});
test('reserved owners, unknown identities and foreign roots never gain scope authority',()=>{
 for(const invalid of [shared.replace('  sample\n','  meanings\n'),
  shared.replace('shared-lexeme-fields lexical','shared-lexeme-fields lexical extra'),
  shared.replace('use-lexeme-fields lexical','use-lexeme-fields "lexical"'),
  shared+'meanings\n  foreign\n    lexeme en\n'+surface('other','        use-lexeme-fields lexical\n')]){
  assert.throws(()=>parse(invalid));
 }
});
test('inline comments retain one identity while extra, quoted and Unicode operands refuse',()=>{
 const commented=shared.replace('shared-lexeme-fields lexical','shared-lexeme-fields lexical # declaration')
  .replaceAll('use-lexeme-fields lexical','use-lexeme-fields lexical # reference');
 const bundle='seed_bundle\n  file "seed/shared.lino"\n'
  +commented.split('\n').slice(0,-1).map(line=>'    '+line).join('\n')+'\n';
 assert.deepEqual(shape(parse(commented)),shape(parse(legacy)));
 const runtime=context.self.FormalAiSeed.loadFromBundle(bundle);
 assert.deepEqual(shape(parse(runtime.raw['seed/shared.lino'])),shape(parse(legacy)));
 assert.equal(runtime.sourceRaw['seed/shared.lino'],commented+'\n');
 for(const head of ['shared-lexeme-fields','use-lexeme-fields']){
  for(const identity of ['lexical extra','"lexical"','lexicál','Lexical']){
   const invalid=shared.replace(head+' lexical',head+' '+identity);
   assert.throws(()=>parse(invalid));
   const invalidBundle='seed_bundle\n  file "seed/shared.lino"\n'
    +invalid.split('\n').slice(0,-1).map(line=>'    '+line).join('\n')+'\n';
   assert.throws(()=>context.self.FormalAiSeed.loadFromBundle(invalidBundle));
  }
 }
});

test('authored scope is validated before every browser expansion',()=>{
 const concise='meanings\n  sample\n'
  +'    shared-lexeme-fields lexical\n      part_of_speech noun\n'
  +'    lexeme en first\n      use-lexeme-fields lexical\n'
  +'    lexeme es second\n      use-lexeme-fields lexical\n';
 const invalid=[concise,
  concise.replace('lexeme es second','lexeme es\n      surface\n        text second'),
  shared.replace('use-lexeme-fields lexical','use-lexeme-fields missing'),
  shared.replace('part_of_speech noun','unknown-field noun'),
  shared.replace('use-lexeme-fields lexical','use-lexeme-fields lexical extra'),
  shared.replace('use-lexeme-fields lexical','use-lexeme-fields lexical\n        use-lexeme-fields lexical'),
  shared.replace('part_of_speech noun','use-lexeme-fields lexical'),
  shared+'meanings\n  foreign\n    lexeme en other\n      use-lexeme-fields lexical\n'];
 for(const source of invalid){
  assert.throws(()=>parse(source));
  const bundle='seed_bundle\n  file \"seed/shared.lino\"\n'
   +source.split('\n').slice(0,-1).map(line=>'    '+line).join('\n')+'\n';
  assert.throws(()=>context.self.FormalAiSeed.loadFromBundle(bundle));
 }
 const legacyConcise='meanings\n  sample\n    lexeme en first\n    lexeme es second\n';
 assert.doesNotThrow(()=>parse(legacyConcise));
 const bundle='seed_bundle\n  file \"seed/legacy.lino\"\n'
  +legacyConcise.split('\n').slice(0,-1).map(line=>'    '+line).join('\n')+'\n';
 assert.doesNotThrow(()=>context.self.FormalAiSeed.loadFromBundle(bundle));
});
