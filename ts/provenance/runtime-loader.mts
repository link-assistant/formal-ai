// Node18-compatible ESM hooks; source is the actual value returned to Node.
import {createRequire} from 'node:module';
const require=createRequire(import.meta.url);
const observer=require(process.env.FORMAL_AI_RUNTIME_BOOTSTRAP);
export async function resolve(specifier,context,nextResolve){
 const result=await nextResolve(specifier,context);
 observer.record({kind:'esm-resolve',specifier,parentURL:context.parentURL??null,url:result.url});
 return result;
}
export async function load(url,context,nextLoad){
 const result=await nextLoad(url,context);
 if(result.source!==undefined&&result.source!==null){
  observer.source(url,result.format,result.source,'actual-esm-load-return');
 }else{
  observer.record({kind:'source-not-returned',url,format:result.format});
 }
 return result;
}
