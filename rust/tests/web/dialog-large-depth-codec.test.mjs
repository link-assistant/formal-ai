import assert from 'node:assert/strict';
import {test} from 'node:test';
import {encodeDialogRecord,decodeDialogRecord} from '../../../js/server/dialog-codec.mjs';
import {encodeTyped,decodeTyped} from '../../../js/logging/typed-codec.mjs';
import {encodeWord} from '../../../js/logging/lino-codec.mjs';
import {f64} from '../../../js/logging/json-numbers.mjs';
const frame=body=>'(formal-ai-log-v1 (dialog-tree '+encodeWord(body)+'))';
function nested(depth){let value='leaf';for(let i=0;i<depth;i+=1)value=[value];return value;}
test('accepted large and 65-deep dialog records preserve every value and numeric type',()=>{
 const record={messages:[{role:'user',content:'x'.repeat(2*1024*1024),nested:nested(65)}],integer:9007199254740993n,float:f64(-0)};
 const line=encodeDialogRecord(record);
 assert(line.startsWith('(formal-ai-log-v1 (dialog-tree '));
 assert(Buffer.byteLength(line)>1024*1024);
 assert.deepEqual(decodeDialogRecord(line),record);
 let depth=0,maximum=0;for(const c of line){if(c==='(')maximum=Math.max(maximum,++depth);if(c===')')depth-=1;}
 assert.equal(maximum,2);assert.equal(depth,0);
 assert.throws(()=>decodeTyped(line),/record_size_limit/);
 assert.throws(()=>encodeTyped(nested(65)),/depth_limit/);
});
test('legacy shallow typed records retain exact bytes and large shallow dialogs remain readable',()=>{
 const value={text:'legacy',items:[false,null,1]};
 const line=encodeTyped(value,{preserveJsonNumbers:true});
 assert.equal(encodeDialogRecord(value),line);assert.deepEqual(decodeDialogRecord(line),value);
 const large={text:'x'.repeat(2*1024*1024)};
 assert.deepEqual(decodeDialogRecord(encodeDialogRecord(large)),large);
 assert.throws(()=>decodeTyped(encodeDialogRecord(large)),/record_size_limit/);
});
test('dialog framing rejects alternate encodings, malformed nodes, duplicate keys and invalid numbers',()=>{
 for(const line of [frame('(string shallow)'),frame('(array (number NaN))'),frame('(object (field a (null)) (field a (null)))'),frame('(array (null)) (null)'),'(formal-ai-log-v1 (dialog-tree %GG))','(formal-ai-log-v1 (dialog-tree (null)))'])assert.throws(()=>decodeDialogRecord(line));
 const line=encodeDialogRecord(nested(65));assert.throws(()=>decodeTyped(line));
 assert.throws(()=>decodeDialogRecord(line+' '));assert.throws(()=>decodeDialogRecord(line+'\n'));
});
