import test from 'node:test';
import assert from 'node:assert/strict';
import {wellFormedText} from '../../../js/logging/text-lines.mjs';
import {encodeTyped,decodeTyped} from '../../../js/logging/typed-codec.mjs';
import {provenancePrimitives} from '../../../js/provenance/session.mjs';
import {sourceSnapshot} from '../../../js/provenance/snapshot.mjs';
const cases=[
 ['',true],['ordinary text',true],['\u0000\r\n\t',true],['\uD800',false],['\uDBFF',false],['\uDC00',false],['\uDFFF',false],
 ['\uD800\uDC00',true],['\uDBFF\uDFFF',true],['\uD800\uD800',false],['\uDC00\uD800',false],['x\uD800y',false],
 ['Հայերեն हिन्दी 中文 العربية 🚀',true],['\uFFFD',true],['\uFEFF',true]
];
test('declared UTF16 boundaries reject lone surrogates and preserve scalar strings',()=>{
 for(const[value,expected]of cases)assert.equal(wellFormedText(value),expected,JSON.stringify(value));
 for(const value of [null,undefined,false,0,{},[],new String('boxed')])assert.equal(wellFormedText(value),false);
});
test('every Unicode code point agrees with independent scalar boundaries and the native method when available',()=>{
 const native=String.prototype.isWellFormed;let checked=0;
 for(let point=0;point<=0x10ffff;point++){
  const value=String.fromCodePoint(point),expected=point<0xd800||point>0xdfff;
  assert.equal(wellFormedText(value),expected,String(point));
  if(typeof native==='function')assert.equal(wellFormedText(value),native.call(value));
  checked++;
 }
 assert.equal(checked,0x110000);
});
test('actual typed codec/session/snapshot APIs work without String.prototype.isWellFormed',()=>{
 const descriptor=Object.getOwnPropertyDescriptor(String.prototype,'isWellFormed');
 delete String.prototype.isWellFormed;
 try{
  const text='данные\nनमस्ते 🚀';assert.equal(decodeTyped(encodeTyped(text)),text);
  assert.throws(()=>encodeTyped('\uD800'),/unsupported_text/);
  const primitives=provenancePrimitives({encodeTyped,decodeTyped});
  const session=primitives.createSession({sessionId:'held-out-🚀',contextId:'scope',initialMemory:Buffer.from(text),runtimeId:process.version,sourceId:'source'});
  session.append('terminal',{reason:'complete'});assert.deepEqual(primitives.verifyChain(session.lines(),session.anchor()),session.anchor());
  assert.throws(()=>primitives.createSession({sessionId:'\uD800',contextId:'scope',initialMemory:Buffer.alloc(0),runtimeId:'runtime',sourceId:'source'}),/identifier_required/);
  const snapshot=sourceSnapshot([{path:'js/नया.js',content:Buffer.from(text)}],[['javascript','.js']]);assert.equal(snapshot.languages[0].file_count,1);
  assert.throws(()=>sourceSnapshot([{path:'js/\uD800.js',content:Buffer.alloc(0)}],[['javascript','.js']]),/invalid_source_file/);
 }finally{if(descriptor)Object.defineProperty(String.prototype,'isWellFormed',descriptor);}
});
