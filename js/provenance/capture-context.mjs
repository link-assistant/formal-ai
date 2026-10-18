import {AsyncLocalStorage} from 'node:async_hooks';
const contexts=new AsyncLocalStorage();
export function activeDriveCapture(){
 const current=contexts.getStore();
 return current!==null&&typeof current==='object'&&'active' in current&&current.active===true?current:null;
}
export function withEvidenceContext(context,run){return contexts.run(context,run);}
