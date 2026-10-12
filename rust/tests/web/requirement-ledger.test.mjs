import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import {parseRequirementLedger} from '../../../scripts/lib/requirement-ledger.mjs';
import {requirementRows, renderShard, quoted} from '../../../scripts/generate-requirement-status.mjs';
import {parseLino} from '../../../js/server/lino.mjs';
const root=path.resolve(import.meta.dirname,'../../..');
const first={id:'R1188-First',shard:'docs/owned.md',issue:'1188',verdict:'partial',
  delivered:'some "quote"',automatedTest:'rust/tests/a.rs',manual:'not yet confirmed'};
const second={...first,id:'R1188-Second',manual:'confirmed'};
const expanded=rows=>'requirement_status_ledger_shard\n'+rows.map(row=>'  requirement\n'
  +[['id',row.id],['shard',row.shard],['issue',row.issue],['verdict',row.verdict],
    ['delivered',row.delivered],['automated_test',row.automatedTest],['manual',row.manual]]
    .map(([name,value])=>'    '+name+' '+quoted(value)+'\n').join('')).join('');

test('complete current ledger records preserve every producer field',()=>{
 const directory=path.join(root,'data/meta/requirement-status-ledger');
 const records=fs.readdirSync(directory).filter(name=>name.endsWith('.lino')).sort()
   .flatMap(name=>parseRequirementLedger(fs.readFileSync(path.join(directory,name),'utf8')).records);
 const actual=new Map(records.map(row=>[row.id,row]));
 const expected=requirementRows(root);
 assert.equal(records.length,expected.length);
 assert.equal(actual.size,records.length);
 for(const row of expected)assert.deepEqual(actual.get(row.id),{
  id:row.id,shard:row.shard,issue:row.issue,verdict:row.verdict,delivered:row.delivered,
  automatedTest:row.automatedTest,manual:row.manual,
 });
});

test('shared defaults and explicit records conserve fields, order and separate owners',()=>{
 const shared=renderShard([first,second]);
 const long=expanded([first,second]);
 const shortRecords=parseRequirementLedger(shared).records;
 assert.deepEqual(shortRecords,parseRequirementLedger(long).records);
 assert.deepEqual(shortRecords,[first,second]);
 shortRecords[0].shard='changed';
 assert.equal(shortRecords[1].shard,second.shard);
});

test('canonical quoted operands and explicit shard overrides use the parsed tree',()=>{
 const source='requirement_status_ledger_shard\n  shard "docs/base ""quote"".md"\n'
  +'  manual "not yet confirmed"\n  requirement\n    id "R1188-First"\n'
  +'    shard "docs/override.md"\n    verdict "partial"\n';
 const parsed=parseRequirementLedger(source);
 assert.equal(parsed.records[0].shard,'docs/override.md');
 assert.equal(parsed.records[0].manual,'not yet confirmed');
 assert.deepEqual(parsed.tree,parseLino(source));
 assert.equal(parsed.sourceRaw,source);
 const withoutOverride=source.replace('    shard "docs/override.md"\n','');
 assert.equal(parseRequirementLedger(withoutOverride).records[0].shard,'docs/base "quote".md');
});

test('unknown legacy metadata stays separate; checked scopes and conflicting fields refuse',()=>{
 const source=renderShard([first,second]);
 const unknown=source+'    unknown_metadata "still unresolved"\n';
 assert.throws(()=>parseRequirementLedger(unknown));
 const legacy=parseRequirementLedger(unknown,{unknownFields:'preserve'});
 assert.deepEqual(legacy.records,parseRequirementLedger(source).records);
 assert.equal(legacy.ignoredFields[0].node.id,'still unresolved');
 assert.deepEqual(legacy.tree,parseLino(unknown));
 const invalid=[source.replace('requirement_status_ledger_shard','foreign_root'),
  source.replace('  requirement\n','  requirement alias\n'),
  source.replace('    id "R1188-First"','    id "R1188-First"\n    id "R1188-Other"'),
  source.replace('    verdict "partial"','    holder\n      verdict "partial"'),
  source+'  manual "changed"\n',source+'    effect "mutation"\n',
  source+'requirement_status_ledger_shard\n',
 ];
 for(const input of invalid)assert.throws(()=>parseRequirementLedger(input));
});
