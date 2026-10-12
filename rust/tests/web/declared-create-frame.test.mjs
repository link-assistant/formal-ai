import {test,before} from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {ownedDeclaredCreateFrame} from '../../../js/agentic/planner/owned_goals.mjs';
before(async()=>{await installNodeHost(new WorkerHost());});
const cases=[
 ['create file note.txt containing hello',{target:'note.txt',content:'hello'}],
 ['add file note.txt containing hello',{target:'note.txt',content:'hello'}],
 ['create file result.txt containing "append overwrite prepend"',{target:'result.txt',content:'append overwrite prepend'}],
 ['Do not write. Create file note.txt containing hello',null],
 ['Read first.txt before creating file note.txt containing hello',null],
 ['Append the line "hello" to note.txt.',null],
 ['Prepend "hello" to note.txt.',null],
 ['Overwrite file note.txt containing hello',null],
 ['Create file note.txt containing "hello". Deploy it.',null],
 ['Create file note.txt containing "hello". Do something unknown.',null],
 ['add to the file note.txt containing hello',null],
 ['Create file note.txt containing "hello',null]
];
for(const [request,expected] of cases) {
  test('complete declared creation ownership: '+request,()=>{
    assert.deepEqual(ownedDeclaredCreateFrame(request),expected);
  });
}
