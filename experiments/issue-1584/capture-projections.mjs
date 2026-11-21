// Supplied evidence recorder. Declared lexical identity is not general semantics.
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {installHost} from '../../js/agentic/host.mjs';
import {parseLino,readRepoFile} from '../../js/server/lino.mjs';
import {cnlPolicy,formalizeOutput,outputLinksNotation,renderOutput} from '../../js/agentic/crate/cnl_output.mjs';

installHost({parseLino,readText:readRepoFile,realm:{}});
const policy=cnlPolicy();
const filename=process.argv[2];
if(!filename)throw Error('Provide a JSONL output filename');
const descriptor=fs.openSync(filename,'w');
const source='js/agentic/crate/cnl_output.mjs';
const identity={head:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),
  node:process.version,source,sha256:createHash('sha256').update(fs.readFileSync(source)).digest('hex'),
  policy_sha256:createHash('sha256').update(fs.readFileSync('data/seed/cnl-output.lino')).digest('hex')};
let count=0,drift=0,compliant=0;
const densities=['standard','primes','top-1','top-100','top-200','top-500','top-1000','max-precision'];
for(const lexeme of policy.lexemes)for(const form of lexeme.forms)for(const profile of policy.profiles)for(const density of densities){
  if(++count>1000)throw Error('Bounded evidence case limit');
  const document=formalizeOutput(form.text,lexeme.language);
  const rendered=renderOutput(document,{output_standard:profile.id,lexical_density:density});
  const expected=outputLinksNotation(document);
  const actual=outputLinksNotation(formalizeOutput(rendered.text,lexeme.language));
  const same_declared_lexical_identity=expected===actual;
  if(!same_declared_lexical_identity)drift++;
  if(rendered.report.compliant)compliant++;
  fs.writeSync(descriptor,JSON.stringify({case:count,identity,language:lexeme.language,input:form.text,
    rendered,round_trip_graph:actual,same_declared_lexical_identity,
    independent_general_semantic_validation:false})+'\n');
}
fs.closeSync(descriptor);
console.log(JSON.stringify({count,declared_identity_drift:drift,compliant,filename,
  complete_issue_matrix:false,independent_general_semantic_validation:false}));
if(drift)process.exitCode=1;
