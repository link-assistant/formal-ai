// Transcript binding is distinct from provider execution, approval and effects.
import { Capability } from './capability.mjs';
import { classifyTool } from './capability_router.mjs';
import { plainText } from './content.mjs';
import { failureMessage } from './tool_result.mjs';

function immutable(value) {
  if (value !== null && typeof value === 'object') {
    for (const item of Object.values(value)) immutable(item);
    Object.freeze(value);
  }
  return value;
}

function supportedArgumentValue(value, ancestors = new Set()) {
  if (value === null || typeof value === 'boolean') return true;
  if (typeof value === 'string') {
    return [...value].every(character => {
      const point = character.codePointAt(0);
      return point < 0xd800 || point > 0xdfff;
    });
  }
  if (typeof value === 'number') {
    return Number.isFinite(value) && (!Number.isInteger(value) || Number.isSafeInteger(value));
  }
  if (typeof value !== 'object' || ancestors.has(value)) return false;
  const array = Array.isArray(value);
  const prototype = Object.getPrototypeOf(value);
  if (array ? prototype !== Array.prototype : prototype !== Object.prototype && prototype !== null) return false;
  const keys = Reflect.ownKeys(value);
  if (keys.some(key => typeof key !== 'string')) return false;
  const descriptors = Object.getOwnPropertyDescriptors(value);
  if (array && (keys.length !== value.length + 1 || keys.some(key => key !== 'length'
    && (!/^(0|[1-9][0-9]*)$/u.test(key) || Number(key) >= value.length)))) return false;
  ancestors.add(value);
  const supported = keys.every(key => {
    if (array && key === 'length') return true;
    const descriptor = descriptors[key];
    return descriptor.enumerable && Object.hasOwn(descriptor, 'value')
      && supportedArgumentValue(key, ancestors) && supportedArgumentValue(descriptor.value, ancestors);
  });
  ancestors.delete(value);
  return supported;
}

function equalSupportedArguments(left, right) {
  if (left === right) return true;
  if (left === null || right === null || typeof left !== typeof right || typeof left !== 'object') return false;
  if (Array.isArray(left) !== Array.isArray(right)) return false;
  const keys = Object.keys(left), others = Object.keys(right);
  return keys.length === others.length
    && keys.every(key => Object.hasOwn(right, key) && equalSupportedArguments(left[key], right[key]));
}

/** Mirrors `argument_values_equal` in rust/src/agentic_coding/qualified_tool_observation.rs.
 * Closed JSON values only: finite fractions and exact safe integers; signed zero is zero.
 * Objects contain enumerable own data properties; null-prototype maps share JSON object semantics.
 * This comparison grants transcript identity only, never provider execution or effect authority.
 */
export function argumentValuesEqual(left, right) {
  if (!supportedArgumentValue(left) || !supportedArgumentValue(right)) return false;
  return equalSupportedArguments(left, right);
}

/** Mirrors `scan` in rust/src/agentic_coding/qualified_tool_observation.rs.
 * Copies declarations before inspecting receipts; no caller metadata creates execution authority. */
export function qualifiedTranscriptFrames(messages,start) {
  const frames=[];
  const declaredIdentifiers = new Set();
  for(let index=start;index<messages.length;index++) {
    const message=messages[index];
    if(typeof message.role!=='string')continue;
    if(message.role.toLowerCase()==='assistant') {
      for(const call of message.tool_calls??[]) {
        if(typeof call.id!=='string' || call.id==='' || typeof call.function?.name!=='string'
          || typeof call.function.arguments!=='string')continue;
        const capability=classifyTool(call.function.name);
        if(capability===null)continue;
        const priorDeclaration = declaredIdentifiers.has(call.id);
        declaredIdentifiers.add(call.id);
        if (priorDeclaration) for (const frame of frames) if (frame.callId === call.id) {
          frame.binding = "contradicted"; frame.succeeded = false; frame.duplicate = true;
        }
        let argumentsValue;
        try {argumentsValue=JSON.parse(call.function.arguments);} catch {continue;}
        if (!argumentValuesEqual(argumentsValue, argumentsValue)) continue;
        const duplicate=priorDeclaration || frames.some(frame=>frame.callId===call.id);
        if(duplicate)for(const frame of frames)if(frame.callId===call.id)frame.binding='contradicted';
        frames.push({capability,declaredTool:call.function.name,callId:call.id,
          arguments:call.function.arguments,argumentsValue:immutable(argumentsValue),binding:duplicate?'contradicted':'unknown',
          duplicate,receiptPresent:false,succeeded:false,detail:'',provider:'Unknown',approval:'Unknown',effects:'Unknown'});
      }
      continue;
    }
    if(message.role.toLowerCase()!=='tool' || typeof message.tool_call_id!=='string')continue;
    const matches=frames.filter(frame=>frame.callId===message.tool_call_id);
    if(matches.length!==1) {
      for(const frame of matches){frame.binding='contradicted';frame.succeeded=false;frame.duplicate=true;}
      continue;
    }
    const frame=matches[0];
    if(frame.duplicate)continue;
    if(frame.receiptPresent){frame.binding='contradicted';frame.succeeded=false;frame.duplicate=true;continue;}
    const raw=plainText(message.content);
    frame.receiptPresent=true;frame.detail=raw;
    frame.binding=message.name===null || message.name===undefined?'unknown':
      typeof message.name==='string' && message.name===frame.declaredTool?'exact':'contradicted';
    frame.succeeded=frame.binding==='exact'
      && failureMessage(raw,Boolean(message.is_error||message.isError),frame.capability!==Capability.Run)===null;
  }
  return immutable(frames);
}
