// JavaScript twin of scripts/lib/requirements-traceability.rs (issue #1580).
import {joinPath,readOrEmpty,readToString,ioErrorDisplay} from './requirements-rust-compat.mjs';
export const TRACEABILITY='docs/requirements-traceability.md';
export const TRACEABILITY_CONTINUATION='docs/requirements-traceability-continuation.md';
const link='(requirements-traceability-continuation.md)';

/** Read only explicitly indexed parts; legacy single-file fixtures stay valid. */
export function traceabilityPaths(root) {
  const head=readOrEmpty(joinPath(root,TRACEABILITY));
  return head.includes(link)?[TRACEABILITY,TRACEABILITY_CONTINUATION]:[TRACEABILITY];
}
export function readTraceability(root) {
  const head=readOrEmpty(joinPath(root,TRACEABILITY));
  if(!head.includes(link))return head;
  const file=joinPath(root,TRACEABILITY_CONTINUATION);
  try{return head+'\n'+readToString(file);}
  catch(error){throw new Error(`cannot read ${file}: ${ioErrorDisplay(error)}`);}
}
