// Finite declaration claims, not runtime behavior or probabilistic truth.
import {cached,parseLino,readText} from '../host.mjs';
import {quotedSegmentSpans,quoteFault} from '../crate/normal_markov.mjs';
import {sentences} from '../shell_command_policy.mjs';
import {observedSourceJson} from '../source_manifest_facts.mjs';
import {sha256Hex} from '../crate/source_fetch.mjs';

const grammarPath='data/meta/source-scalar-questions.lino';
function questionGrammar() {
 return cached('source-scalar-questions',()=>{
  const root=parseLino(readText(grammarPath));
  const node=root.name==='source-scalar-questions'?root:(root.children??[]).find(item=>item.name==='source-scalar-questions');
  const values=name=>(node?.children??[]).filter(item=>item.name===name).map(item=>String(item.id??item.value??''));
  if(values('schema')[0]!=='source-scalar-questions/v1')return null;
  return {copulas:values('copula'),subjects:values('subject'),negations:values('negation'),ends:values('terminator')};
 });
}
const bytes=text=>new TextEncoder().encode(text).length;
const scalar=value=>value===null||['string','boolean','number'].includes(typeof value);
const span=(request,start,end)=>[bytes(request.slice(0,start)),bytes(request.slice(0,end))];

/** Own an entire question before admitting literal/polarity operands. */
export function sourceScalarQuestion(request,key) {
 const grammar=questionGrammar();
 if(!grammar||typeof request!=='string'||bytes(request)>16384)return null;
 const characters=request.split('');
 for(const quoted of quotedSegmentSpans(request))
  for(let index=quoted.start;index<quoted.end;index++)characters[index]=' ';
 const masked=characters.join('');
 const candidates=sentences(masked).filter(sentence=>{
  const text=masked.slice(sentence.span.start,sentence.span.end).toLowerCase();
  return grammar.subjects.some(subject=>text.includes(subject.toLowerCase()));
 });
 if(!candidates.length)return null;
 const unknown={claim:null};
 if(candidates.length!==1||quoteFault(request)!==null)return unknown;
 const sentence=candidates[0],raw=request.slice(sentence.span.start,sentence.span.end);
 const leading=raw.length-raw.trimStart().length;
 const text=raw.trim(),start=sentence.span.start+leading;
 const claims=[];
 for(const copula of grammar.copulas)for(const subject of grammar.subjects) {
  const prefix=copula+' '+subject+' '+key+' ';
  if(!text.toLowerCase().startsWith(prefix.toLowerCase()))continue;
  // JSON keys remain case-sensitive despite case-insensitive instruction words.
  const keyStart=start+copula.length+1+subject.length+1;
  if(request.slice(keyStart,keyStart+key.length)!==key)continue;
  for(const ending of grammar.ends) {
   if(!text.endsWith(ending))continue;
   const operandStart=start+prefix.length;
   const rest=text.slice(prefix.length,-ending.length);
   for(const marker of ['',...grammar.negations]) {
    const lead=marker?marker+' ':'';
    if(!rest.toLowerCase().startsWith(lead.toLowerCase()))continue;
    const valueText=rest.slice(lead.length).trim();
    const valueStart=operandStart+lead.length+rest.slice(lead.length).indexOf(valueText);
    let expected;
    try{expected=observedSourceJson(valueText);}catch{continue;}
    if(!scalar(expected))continue;
    claims.push({key,expected,polarity:marker?'negative':'positive',
     requestIdentity:sha256Hex(request),
     claimSpan:span(request,start,start+text.length),
     keySpan:span(request,keyStart,keyStart+key.length),
     expectedSpan:span(request,valueStart,valueStart+valueText.length),
     polaritySpan:marker?span(request,operandStart,operandStart+marker.length):span(request,start,start+text.length)});
   }
  }
 }
 return claims.length===1?{claim:claims[0]}:unknown;
}
function scalarEvidence(value) {
 return typeof value==='string'&&bytes(value)>512
  ?{type:'string',bytes:bytes(value),sha256:sha256Hex(value)}:value;
}

/** Current qualified complete Read is the only source authority for judgment. */
export function sourceScalarAssessment(request,path,key,progress) {
 const question=sourceScalarQuestion(request,key);
 if(question===null)return null;
 const claim=question.claim;
 const report={schema:'source-scalar-assessment/v1',status:'unknown',
  scope:'declared-json-scalar-only',claim:claim?{...claim,expected:scalarEvidence(claim.expected)}:null,
  evidence:[],unresolved:[]};
 const read=progress.freshSourceReadFor(path);
 if(read===null){report.unresolved.push('fresh-complete-source-read-unavailable');return report;}
 let document;
 try{document=observedSourceJson(read.source);}catch{
  report.unresolved.push('strict-json-observation-unavailable');return report;
 }
 if(document===null||typeof document!=='object'||Array.isArray(document)||!Object.hasOwn(document,key)){
  report.unresolved.push('declared-key-unavailable');return report;
 }
 const actual=document[key];
 if(!scalar(actual)){report.unresolved.push('declared-value-not-scalar');return report;}
 report.evidence.push({owner:path,sourceIdentity:sha256Hex(read.source),
  documentSpan:[0,bytes(read.source)],spanScope:'complete-document',key,actual:scalarEvidence(actual)});
 if(claim===null){report.unresolved.push('comparison-frame-not-owned');return report;}
 const equal=typeof actual===typeof claim.expected&&actual===claim.expected;
 report.status=(claim.polarity==='positive'?equal:!equal)?'supported':'contradicted';
 return report;
}
