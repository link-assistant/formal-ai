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
let sequence=0,totalBytes=0;
function record(entry){
 const line=JSON.stringify({sequence:sequence++,...entry})+'\n';
 if(Buffer.byteLength(line)>65536)throw Error('RuntimeObservationRecordTooLarge');
 fileSystem.appendFileSync(journal,line,{mode:0o600});
}
function source(url,format,body,boundary){
 const bytes=Buffer.isBuffer(body)?body:Buffer.from(body,'utf8');
 if(bytes.length>4*1024*1024)throw Error('RuntimeModuleSourceTooLarge');
 totalBytes+=bytes.length;
 if(totalBytes>16*1024*1024)throw Error('RuntimeSourceBudgetExceeded');
 const digest=crypto.createHash('sha256').update(bytes).digest('hex');
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
process.once('exit',code=>record({kind:'process-exit',code}));
module.exports={record,source};
