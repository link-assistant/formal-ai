#!/usr/bin/env node
// One pinned scanner/configuration, sequential inputs, no extracted files.
import {accessSync,constants,readFileSync,realpathSync,statSync,existsSync} from 'node:fs';
import {delimiter,dirname,join,resolve} from 'node:path';
import {pathToFileURL,fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {randomUUID} from 'node:crypto';
import {MAX_DECOMPRESSED_BYTES,scanCompressed} from './lib/compressed-secrets.mjs';

function packageVersion(file,name) {
  for(let directory=dirname(file);;) {
    const manifest=join(directory,'package.json');
    if(existsSync(manifest)) {
      const value=JSON.parse(readFileSync(manifest,'utf8'));
      if(value.name===name)return value.version;
    }
    const parent=dirname(directory);if(parent===directory)throw Error('Scanner package identity unavailable');
    directory=parent;
  }
}
export function scannerModule(version,searchPath=process.env.PATH??'') {
  let binary;
  for(const directory of searchPath.split(delimiter)) {
    if(!directory)continue;
    const candidate=join(directory,process.platform==='win32'?'secretlint.cmd':'secretlint');
    try {accessSync(candidate,constants.X_OK);binary=realpathSync(candidate);break;}catch{}
  }
  if(!binary)throw Error('Pinned secretlint executable unavailable');
  const require=createRequire(pathToFileURL(binary));
  const engine=require.resolve('@secretlint/node');
  const builtin=require.resolve('@secretlint/core/builtin');
  for(const [file,name] of [[binary,'secretlint'],[engine,'@secretlint/node'],[builtin,'@secretlint/core']]) {
    if(packageVersion(file,name)!==version)throw Error('Pinned scanner package version mismatch: '+name);
  }
  return pathToFileURL(engine).href;
}

export async function createScanner(version,searchPath) {
  const module=scannerModule(version,searchPath);
  const profiler=createRequire(module).resolve('@secretlint/profiler');
  if(packageVersion(profiler,'@secretlint/profiler')!==version)throw Error('Pinned scanner profiler version mismatch');
  const {secretLintProfiler}=await import(pathToFileURL(profiler).href);
  // Match secretlint's CLI default. The API otherwise retains per-rule/file marks.
  secretLintProfiler.setEnabled(false);
  const {createEngine}=await import(module);
  return createEngine({cwd:process.cwd(),formatter:'stylish',color:false,maskSecrets:true});
}

// NUL transport keeps filenames out of npm's generated shell command/argument limits.
export async function* filenames(stream) {
  let pending=Buffer.alloc(0),count=0;
  for await(const chunk of stream) {
    pending=Buffer.concat([pending,Buffer.from(chunk)]);
    let end;
    while((end=pending.indexOf(0))!==-1) {
      if(end===0||end>32768)throw Error('Invalid filename in scanner inventory');
      const name=pending.subarray(0,end).toString('utf8');pending=pending.subarray(end+1);
      count++;yield name;
    }
    if(pending.length>32768)throw Error('Scanner inventory filename size limit exceeded');
  }
  if(pending.length)throw Error('Unterminated scanner inventory');
  if(!count)throw Error('Empty scanner inventory; no scan performed');
}

export async function checkFiles(files,engine,write=value=>process.stdout.write(value)) {
  let count=0,compressed=0,failed=false;
  const accept=result=> {
    if(result.output)write(result.output+'\n');
    if(!result.ok)failed=true;
  };
  for await(const file of files) {
    const status=statSync(file);if(!status.isFile())throw Error('Selected scanner input is not a file');
    count++;
    if(!/\.(gz|tgz)$/iu.test(file)) {
      accept(await engine.executeOnFiles({filePathList:[resolve(file)]}));continue;
    }
    compressed++;
    if(status.size>MAX_DECOMPRESSED_BYTES)throw Error('Compressed input exceeds 128 MiB scan limit');
    if(process.env.FORMAL_AI_SECRETS_DEBUG==='1')console.error('[check-secrets] archive '+JSON.stringify(file));
    const entries=[];
    scanCompressed(file,readFileSync(file),(name,bytes)=>entries.push({name,bytes}));
    for(const {name,bytes} of entries) {
      if(!bytes.length)continue; // Empty, successfully decoded content contains no credential.
      accept(await engine.executeOnContent({content:bytes.toString('utf8'),filePath:resolve(name)}));
    }
  }
  if(!count)throw Error('Empty scanner inventory; no scan performed');
  return {ok:!failed,files:count,compressedFiles:compressed};
}

if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const token='secret-scan-'+randomUUID();
  console.log('::stop-commands::'+token);
  try {
    const version=process.env.SECRETLINT_VERSION??'14.0.0';
    const engine=await createScanner(version);
    const result=await checkFiles(filenames(process.stdin),engine);
    console.log(JSON.stringify({secretsScan:result}));
    if(!result.ok)process.exitCode=1;
  } catch(error) {
    console.error(JSON.stringify({secretsScan:'failed',reason:String(error.message)}));
    process.exitCode=1;
  } finally {console.log('::'+token+'::');}
}
