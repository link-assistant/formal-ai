// Canonical CommonJS preload source. The launcher copies these exact bytes to .cjs.
// Private transport observations are not diagnostic logs or authorship authority.
const fileSystem=require('node:fs');
const path=require('node:path');
const crypto=require('node:crypto');
const Module=require('node:module');
const {threadId}=require('node:worker_threads');
const configuration=JSON.parse(fileSystem.readFileSync(process.env.FORMAL_AI_RUNTIME_CONFIG,'utf8'));
const directory=fileSystem.realpathSync(configuration.transportDirectory);
const journal=path.join(directory,'cjs-'+threadId+'.jsonl');
const objects=path.join(directory,'objects');
fileSystem.mkdirSync(objects,{recursive:true,mode:0o700});
let sequence=0,totalBytes=0;const archivedDigests=new Set();
function record(entry){
 const line=JSON.stringify({sequence:sequence++,...entry})+'\n';
 if(Buffer.byteLength(line)>65536)throw Error('RuntimeObservationRecordTooLarge');
 fileSystem.appendFileSync(journal,line,{mode:0o600});
}
function source(url,format,body,boundary){
 const bytes=Buffer.isBuffer(body)?body:Buffer.from(body,'utf8');
 if(bytes.length>4*1024*1024)throw Error('RuntimeModuleSourceTooLarge');
 const digest=crypto.createHash('sha256').update(bytes).digest('hex');
 if(!archivedDigests.has(digest)){
  totalBytes+=bytes.length;if(totalBytes>16*1024*1024)throw Error('RuntimeSourceBudgetExceeded');
  archivedDigests.add(digest);
 }
 const file='objects/'+digest+'.src';
 try{fileSystem.writeFileSync(path.join(directory,file),bytes,{flag:'wx',mode:0o600});}
 catch(error){if(error.code!=='EEXIST')throw error;}
 record({kind:'source',url,format,file,sha256:digest,size:bytes.length,boundary});
}
record({kind:'bootstrap',runtime:process.version,threadId,argv:process.execArgv});
const originalCompile=Module.prototype._compile;
Module.prototype._compile=function(content,filename){
 source(require('node:url').pathToFileURL(filename).href,'commonjs',content,'actual-commonjs-compile-input');
 return originalCompile.apply(this,arguments);
};
const originalJson=Module._extensions['.json'];
Module._extensions['.json']=function(module,filename){
 const originalRead=fileSystem.readFileSync;
 fileSystem.readFileSync=function(file,...argumentsList){
  const bytes=originalRead.call(this,file,...argumentsList);
  if(String(file)===filename)source(require('node:url').pathToFileURL(filename).href,'json',bytes,'actual-commonjs-json-read');
  return bytes;
 };
 try{return originalJson(module,filename);}finally{fileSystem.readFileSync=originalRead;}
};
const originalResolve=Module._resolveFilename;
Module._resolveFilename=function(request,parent){
 const resolved=originalResolve.apply(this,arguments);
 record({kind:'resolve',request,parent:parent?.filename??null,resolved,boundary:'actual-commonjs-resolution'});
 return resolved;
};
// Observe actual synchronous repository read returns, including failed reads.
// This is input custody, never a claim that read bytes were compiled or executed.
if(typeof configuration.repository==='string'){
 const repository=fileSystem.realpathSync(configuration.repository),originalRead=fileSystem.readFileSync;
 const urlTools=require('node:url');
 fileSystem.readFileSync=function(file,...argumentsList){
  let filename=null;
  if(typeof file==='string')filename=path.resolve(file);
  else if(file instanceof URL&&file.protocol==='file:')filename=urlTools.fileURLToPath(file);
  const owned=filename!==null&&(filename===repository||filename.startsWith(repository+path.sep))
   &&filename!==directory&&!filename.startsWith(directory+path.sep);
  const options=argumentsList[0],encoding=typeof options==='string'?options
   :options===undefined||options===null?null:'non-string-options-not-inspected';
  let result;
  try{result=originalRead.call(this,file,...argumentsList);}
  catch(error){
   if(owned)record({kind:'file-read-failure',url:urlTools.pathToFileURL(filename).href,encoding,
    code:typeof error.code==='string'?error.code:null,boundary:'actual-repository-file-read-failure'});
   throw error;
  }
  if(owned){
   const representation=typeof result==='string'?'string':'buffer';
   source(urlTools.pathToFileURL(filename).href,'repository-read-'+representation,result,'actual-repository-file-read-return');
   record({kind:'file-read-result',url:urlTools.pathToFileURL(filename).href,encoding,representation,
    boundary:'actual-repository-file-read-return'});
  }
  return result;
 };
 Module.syncBuiltinESMExports();
}
process.once('exit',code=>record({kind:'process-exit',code}));
module.exports={record,source};
