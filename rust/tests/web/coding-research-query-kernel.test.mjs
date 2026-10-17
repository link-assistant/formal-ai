import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {codingResearchQuery} from '../../../js/agentic/coding_research.mjs';
import {quotedSegmentSpans} from '../../../js/agentic/crate/normal_markov.mjs';
import {codingResearchQueryKernel} from '../../../js/agentic/portable/coding_research_query.mjs';
await installNodeHost(new WorkerHost());
const fixtures=JSON.parse(readFileSync(new URL('../fixtures/coding-research-query.json',import.meta.url),'utf8'));
for(const fixture of fixtures){
  test('canonical query: '+fixture.label,()=>{
    let clause=fixture.clause;
    for(const quote of quotedSegmentSpans(clause).reverse()){clause=clause.slice(0,quote.start)+' '+clause.slice(quote.end);}
    assert.equal(codingResearchQueryKernel('javascript',Array.from(clause),fixture.name,fixture.parameters),fixture.expected);
    assert.equal(codingResearchQuery({...fixture,language:'javascript'}),fixture.expected);
  });
}
