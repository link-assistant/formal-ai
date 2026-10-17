// Dialog records preserve original number types and accepted size/depth.
import {encodeDialogTyped,decodeDialogTyped} from '../logging/typed-codec.mjs';
export const encodeDialogRecord=encodeDialogTyped;
export const decodeDialogRecord=decodeDialogTyped;
