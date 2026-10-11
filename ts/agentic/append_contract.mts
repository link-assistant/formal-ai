// Explicit provider schema and receipt contracts; ordinary Write remains replacement.
import { evidenceWindowStart } from './planner/continuation.mjs';
import { jsonText, planOne } from './plan.mjs';
import { utf8Len } from './crate/rust_str.mjs';
import { readText } from './host.mjs';
import { parseRoot } from './crate/seed_parser.mjs';

export const APPEND_CONTRACT = 'atomic-record-append/v1';
export const APPEND_MODE = 'atomic_record_append';

/** Mirrors general_change_plan_record_header_from: exact single-line seed schema. */
export function generalChangePlanRecordHeaderFrom(text) {
  if (typeof text !== 'string') return null;
  const parsed = parseRoot(text);
  const root = parsed.children[0];
  if (parsed.children.length !== 1 || root?.name !== 'agentic_tool_capabilities') return null;
  const records = (root?.children ?? []).filter(node => node.name === 'record-schema'
    && node.value === 'general-change-plan');
  if (records.length !== 1) return null;
  const headers = (records[0].children ?? []).filter(node => node.name === 'header');
  if (headers.length !== 1 || !/^[a-z][a-z0-9_]*$/u.test(headers[0].value)) return null;
  return headers[0].value + '\n';
}

export function generalChangePlanRecordHeader() {
  try {
    return generalChangePlanRecordHeaderFrom(readText('data/seed/agentic-tool-capabilities.lino'));
  } catch {
    return null;
  }
}

export function declaredAppendTool(definition) {
  if (!definition || typeof definition !== 'object') return null;
  const tool = Object.hasOwn(definition, 'function') ? definition.function : definition;
  const properties = tool?.parameters?.properties;
  if (typeof tool?.name !== 'string' || tool['x-formal-ai-contract'] !== APPEND_CONTRACT
    || tool['x-formal-ai-receipt-contract'] !== APPEND_CONTRACT
    || tool.parameters?.type !== 'object'
    || !Array.isArray(tool.parameters.required)
    || !tool.parameters.required.every(key => ['path', 'content', 'record_id', 'append_mode', 'append_request_id'].includes(key))
    || properties?.append_mode?.const !== APPEND_MODE
    || !['path', 'content', 'record_id', 'append_request_id'].every(key => properties?.[key]?.type === 'string')) return null;
  return tool.name;
}

export function projectAppendContracts(messages, definitions = []) {
  const projected = messages.map(message => {
    const { append_contract: ignored, ...rest } = message;
    return rest;
  });
  const contracts = Array.isArray(definitions)
    ? definitions.filter(definition => {
      const name = declaredAppendTool(definition);
      if (name === null) return false;
      return definitions.filter(candidate => {
        const tool = candidate && Object.hasOwn(candidate, 'function')
          ? candidate.function : candidate;
        return tool?.name === name;
      }).length === 1;
    }) : [];
  const user = projected.findLastIndex(message => message.role === 'user');
  if (user >= 0 && contracts.length) projected[user].append_contract = contracts;
  return projected;
}

export function appendDefinition(name = 'write') {
  return { type: 'function', function: { name,
    'x-formal-ai-contract': APPEND_CONTRACT,
    'x-formal-ai-receipt-contract': APPEND_CONTRACT,
    parameters: { type: 'object', properties: {
      path: { type: 'string' }, content: { type: 'string' }, record_id: { type: 'string' }, append_request_id: { type: 'string' },
      append_mode: { type: 'string', const: APPEND_MODE },
    }, required: ['path', 'content'] },
  } };
}

export function appendReceiptValid(receipt, appendArguments) {
  if (!receipt || receipt.schema !== APPEND_CONTRACT || receipt.complete !== true
    || receipt.success !== true || receipt.path !== appendArguments.path
    || receipt.append_request_id !== appendArguments.append_request_id || receipt.record_id !== appendArguments.record_id || receipt.content !== appendArguments.content
    || typeof receipt.before !== 'string' || typeof receipt.after !== 'string') return false;
  if ([receipt.before, receipt.after].some(text => [...text].some(character => {
    const point = character.codePointAt(0);
    return point >= 0xd800 && point <= 0xdfff;
  }))) return false;
  if (receipt.before_bytes !== utf8Len(receipt.before) || receipt.after_bytes !== utf8Len(receipt.after)) return false;
  const separator = receipt.before !== '' && !receipt.before.endsWith('\n') ? '\n' : '';
  if (receipt.operation === 'appended') {
    return !receipt.before.split('\n').includes(appendArguments.record_id)
      && receipt.after === receipt.before + separator + appendArguments.content;
  }
  if (receipt.operation !== 'already_present' || receipt.after !== receipt.before
    || receipt.before.split('\n').filter(line => line === appendArguments.record_id).length !== 1) return false;
  const header = generalChangePlanRecordHeader();
  if (header === null) return false;
  const start = receipt.before.indexOf(appendArguments.content);
  return start >= 0 && (start === 0 || receipt.before[start - 1] === '\n')
    && (receipt.before.length === start + appendArguments.content.length
      || receipt.before.slice(start + appendArguments.content.length).startsWith(header));
}

export function appendRecordStep(messages, toolNames, path, content, recordId) {
  const start = evidenceWindowStart(messages);
  const current = messages.slice(start);
  const user = messages.findLast(message => message.role === 'user');
  const contracts = user?.append_contract;
  if (!Array.isArray(contracts)) return null;
  const names = contracts.map(declaredAppendTool).filter(name => name !== null && toolNames.includes(name));
  if (names.length !== 1) return null;
  const tool = names[0];
  const base = { path, content, record_id: recordId, append_mode: APPEND_MODE };
  const appendArguments = { ...base, append_request_id: recordId + '/' + messages.length };
  let matched = null;
  let matchedArguments = appendArguments;
  for (let index = 0; index < current.length; index++) {
    const message = current[index];
    if (message.role !== 'assistant') continue;
    for (const call of message.tool_calls ?? []) {
      if (call.function?.name !== tool) continue;
      let supplied;
      try { supplied = JSON.parse(call.function.arguments); } catch { continue; }
      if (!supplied || Object.keys(supplied).length !== Object.keys(appendArguments).length
        || !Object.keys(base).every(key => supplied[key] === base[key])
        || supplied.append_request_id !== recordId + '/' + (start + index)) continue;
      const results = current.slice(index + 1).filter(result => result.role === 'tool'
        && result.tool_call_id === call.id && result.name === tool);
      matchedArguments = supplied;
      matched = results.length === 1 ? results[0] : false;
    }
  }
  if (matched === null) return { kind: 'pending', plan: planOne(tool, jsonText(appendArguments)) };
  return matched && !matched.is_error && !matched.isError && appendReceiptValid(matched.append_receipt, matchedArguments)
    ? { kind: 'observed' } : { kind: 'refused' };
}
