// Dialog records share the diagnostic codec with explicit serde number custody.
// Diagnostic callers retain their finite/safe JavaScript number refusal policy.
import {encodeTyped,decodeTyped} from './structured-log.mjs';
export const encodeDialogRecord=record=>encodeTyped(record,{preserveJsonNumbers:true});
export const decodeDialogRecord=line=>decodeTyped(line,{preserveJsonNumbers:true});
