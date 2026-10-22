// Authorized host for the shared declared complete-CST parameter rule.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {execFileSync} from 'node:child_process';
import {createHash,randomBytes} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {sourceCstTransformRequest} from '../js/agentic/source_cst_transform.mjs';
import {sourceCstFrontends} from './lib/source-cst-frontends.mjs';
import {sourceToLinks,sourceFromLinks,ownedSourceLinks,sourceOwner,writeSourceLinks} from '../js/meta/readable-source.js';
import {sourceCstRuleForObservedParameter,applyQualifiedSourceCstRule,writeQualifiedSourceCstRule,readQualifiedSourceCstRule,verifyQualifiedSourceCstRule} from './lib/source-cst-documentation-transform.mjs';
import {withSourceCstCommentContext} from './lib/source-cst-comment-host.mjs';
const digest=value=>createHash('sha256').update(value).digest('hex');
function observedOwner(root,owner){
 const physical=path.join(root,owner);
 if(fs.realpathSync(physical)!==physical)throw Error('SourceOwnerSymlinkRefused');
 const metadata=fs.lstatSync(physical);
 if(!metadata.isFile()||metadata.nlink!==1||metadata.size>1024*1024)throw Error('SourceOwnerBoundsRefused');
 const tracked=execFileSync('git',['ls-files','--error-unmatch','--',owner],{cwd:root,encoding:'utf8',maxBuffer:16384}).trim();
 if(tracked!==owner)throw Error('SourceOwnerNotTracked');
 const installedRoot=fs.realpathSync(new URL('../',import.meta.url));
 if(root===installedRoot&&/^(?:ts\/|rust\/src\/generated\/)/u.test(owner))throw Error('GeneratedOwnerRequiresCanonicalGenerator');
 const bytes=fs.readFileSync(physical),source=bytes.toString('utf8');
 if(!Buffer.from(source).equals(bytes))throw Error('SourceEncodingRefused');
 if(/(?:@generated|DO NOT EDIT|Auto-generated)/u.test(source.slice(0,4096)))throw Error('GeneratedOwnerRequiresCanonicalGenerator');
 return {physical,metadata,source};
}
function save(directory,name,content){fs.writeFileSync(path.join(directory,name),content,{flag:'wx',mode:0o400});}
/** Stage complete bytes before replacing the exact observed physical owner. */
function replaceObserved(observed,before,after,writeChunk){
 const original=fs.openSync(observed.physical,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
 const staged=observed.physical+'.formal-ai-'+randomBytes(12).toString('hex');
 let descriptor=null,renamed=false;
 const unchanged=()=>{
  const current=fs.fstatSync(original),physical=fs.lstatSync(observed.physical);
  if(physical.dev!==current.dev||physical.ino!==current.ino||!physical.isFile()||physical.nlink!==1||current.dev!==observed.metadata.dev||current.ino!==observed.metadata.ino)throw Error('SourceOwnerIdentityChanged');
  if(fs.readFileSync(observed.physical,'utf8')!==before)throw Error('SourcePreimageChanged');
 };
 try{
  unchanged();
  descriptor=fs.openSync(staged,fs.constants.O_WRONLY|fs.constants.O_CREAT|fs.constants.O_EXCL|fs.constants.O_NOFOLLOW,observed.metadata.mode&0o777);
  const bytes=Buffer.from(after);let offset=0;
  while(offset<bytes.length){
   const remaining=bytes.length-offset;
   const written=writeChunk(descriptor,bytes,offset,remaining,offset);
   if(!Number.isSafeInteger(written)||written<=0||written>remaining)throw Error('SourceWriteProgressInvalid');
   offset+=written;
  }
  fs.fsyncSync(descriptor);fs.closeSync(descriptor);descriptor=null;
  if(!fs.readFileSync(staged).equals(bytes))throw Error('SourceStagedWriteMismatch');
  unchanged();fs.renameSync(staged,observed.physical);renamed=true;
 }finally{
  if(descriptor!==null)fs.closeSync(descriptor);
  fs.closeSync(original);
  if(!renamed&&fs.existsSync(staged))fs.unlinkSync(staged);
 }
}
export async function executeSourceCstTransform({root=process.cwd(),request,expected,inverseDirectory=null,writeChunk=fs.writeSync}){
 root=fs.realpathSync(root);
 let definition,prior;
 if(inverseDirectory!==null){
 prior=JSON.parse(fs.readFileSync(path.join(inverseDirectory,'intent.json'),'utf8'));
 if(prior.root!==root||prior.schema!=='source-cst-host-intent/v1')throw Error('InverseRootMismatch');
 request=prior.request;definition=sourceCstTransformRequest(request);
 }else definition=sourceCstTransformRequest(request);
 if(!definition||!/^([a-f0-9]{64})$/u.test(expected??''))throw Error('QualifiedObservedRequestRequired');
 const observed=observedOwner(root,definition.owner);
 if(digest(observed.source)!==expected)throw Error('SourcePreimageChanged');
 const language=definition.owner.endsWith('.rs')?'Rust':/\.(?:mts|ts)$/u.test(definition.owner)?'TypeScript':'JavaScript';
 const runtime=await sourceCstFrontends(undefined,[language]);
 try{return withSourceCstCommentContext(()=>{
 const frontend=runtime.forSource(definition.owner,language);
 const before=ownedSourceLinks(definition.owner,sourceToLinks(observed.source,language,frontend));
 let rule;
 if(inverseDirectory===null)rule=sourceCstRuleForObservedParameter(before,definition);
 else {
 const inverseFile=path.join(inverseDirectory,'inverse.lino');
 if(fs.statSync(inverseFile).size>64*1024*1024)throw Error('InverseEvidenceBoundsRefused');
 const parsed=readQualifiedSourceCstRule(fs.readFileSync(inverseFile,'utf8'));
 const receipt=JSON.parse(fs.readFileSync(path.join(inverseDirectory,'receipt.json'),'utf8'));
 if(receipt.root!==root||receipt.request!==request||receipt.afterSha256!==expected||sourceOwner(parsed.preimage)!==definition.owner||parsed.definition.declaration!==definition.declaration||parsed.definition.binding!==definition.target||parsed.definition.target!==definition.binding)throw Error('InverseRequestMismatch');
 rule=parsed.network;
 }
 const result=applyQualifiedSourceCstRule(before,rule,frontend),after=sourceFromLinks(result.document);
 if(Buffer.byteLength(after)>1024*1024)throw Error('TransformedSourceBoundsRefused');
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-cst-transform-'));
 save(directory,'intent.json',JSON.stringify({schema:'source-cst-host-intent/v1',root,request,owner:definition.owner,expected,device:observed.metadata.dev,inode:observed.metadata.ino,authorshipCredit:0})+'\n');
 if(result.documentaryModel)save(directory,'documentation.json',JSON.stringify(result.documentaryModel,null,2)+'\n');
 save(directory,'before.lino',writeSourceLinks(before));save(directory,'rule.lino',writeQualifiedSourceCstRule(rule));
 save(directory,'after.lino',writeSourceLinks(result.document));save(directory,'inverse.lino',writeQualifiedSourceCstRule(result.inverse));
 try{replaceObserved(observed,observed.source,after,writeChunk);}
 catch(error){
  save(directory,'failure.json',JSON.stringify({schema:'source-cst-host-failure/v1',stage:'staged-write',message:error.message,sourceUnchanged:fs.existsSync(observed.physical)&&digest(fs.readFileSync(observed.physical))===expected,authorshipCredit:0})+'\n');
  error.evidenceDirectory=directory;throw error;
 }
 const physical=fs.readFileSync(observed.physical,'utf8');
 const reparsed=ownedSourceLinks(definition.owner,sourceToLinks(physical,language,frontend));
 verifyQualifiedSourceCstRule(before,reparsed,rule,frontend);
 if(sourceFromLinks(applyQualifiedSourceCstRule(reparsed,result.inverse,frontend).document)!==observed.source)throw Error('PhysicalInverseMismatch');
 const receipt={schema:'source-cst-host-receipt/v1',transformKind:rule[0],root,owner:definition.owner,request,beforeSha256:expected,afterSha256:digest(physical),evidenceDirectory:directory,physicalWriteVerified:true,verification:result.verification,authorshipCredit:0};
 save(directory,'receipt.json',JSON.stringify(receipt,null,2)+'\n');return receipt;
 });}finally{runtime.close();}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(path.resolve(process.argv[1])).href){
 try{
 const argumentsList=process.argv.slice(2),options={};
 if(argumentsList.length!==4)throw Error('ExpectedRequestAndObservedDigest');
 for(let index=0;index<argumentsList.length;index+=2){const name=argumentsList[index];if(!['--request','--expected','--inverse'].includes(name)||Object.hasOwn(options,name))throw Error('InvalidArgument');options[name]=argumentsList[index+1];}
 console.log(JSON.stringify(await executeSourceCstTransform({request:options['--request'],expected:options['--expected'],inverseDirectory:options['--inverse']??null})));
 }catch(error){console.error('SourceCstTransformRefused: '+error.message);process.exitCode=1;}
}
