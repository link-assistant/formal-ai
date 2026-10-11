// Tool-definition readers and the agentic permission gate
// (rust/src/protocol_policy.rs, rust/src/protocol.rs
// `ChatCompletionRequest::requested_tool_names`).

import { agenticMessage } from './messages.mjs';

const isObject = (value) => Boolean(value) && typeof value === 'object' && !Array.isArray(value);

/** `hosted_tool_type_name`. */
function hostedToolTypeName(object) {
  const kind = object.type;
  if (typeof kind !== 'string') return null;
  if (kind === 'web_search' || kind === 'web_search_preview') return 'web_search';
  if (kind === 'file_search') return 'file_search';
  if (kind === 'computer_use' || kind === 'computer_use_preview') return 'computer_use';
  if (kind === 'code_interpreter') return 'code_interpreter';
  if (kind.startsWith('web_search_')) return 'web_search';
  return null;
}

/** Mirrors `fn tool_definition_name` (also `tool_choice_function_name`). */
export function toolDefinitionName(value) {
  if (!isObject(value)) return null;
  // `function.name` when present (whatever its type), else top-level `name`;
  // a non-string either way falls back to the hosted tool's type.
  const nested = isObject(value.function) ? value.function.name : undefined;
  const name = nested !== undefined ? nested : value.name;
  return typeof name === 'string' ? name : hostedToolTypeName(value);
}

/** Mirrors `fn tool_choice_function_name`. */
export const toolChoiceFunctionName = toolDefinitionName;

/** `qualify_tool_name`. */
function qualifyToolName(prefix, name) {
  if (prefix === null) return name;
  const separator = prefix.endsWith('__') ? '' : '__';
  return name.startsWith(`${prefix}${separator}`) ? name : `${prefix}${separator}${name}`;
}

function appendQualified(value, prefix, names) {
  if (!isObject(value)) return;
  if (value.type === 'namespace') {
    if (typeof value.name !== 'string') return;
    const qualified = qualifyToolName(prefix, value.name);
    if (Array.isArray(value.tools)) {
      for (const child of value.tools) appendQualified(child, qualified, names);
    }
    return;
  }
  const name = toolDefinitionName(value);
  if (name !== null) names.push(qualifyToolName(prefix, name));
}

/** Mirrors `fn tool_definition_names`. */
export function toolDefinitionNames(value) {
  const names = [];
  appendQualified(value, null, names);
  return names;
}

function findQualified(value, prefix, toolName) {
  if (!isObject(value)) return null;
  if (value.type === 'namespace') {
    if (typeof value.name !== 'string' || !Array.isArray(value.tools)) return null;
    const qualified = qualifyToolName(prefix, value.name);
    for (const child of value.tools) {
      const found = findQualified(child, qualified, toolName);
      if (found) return found;
    }
    return null;
  }
  const name = toolDefinitionName(value);
  return name !== null && qualifyToolName(prefix, name) === toolName ? value : null;
}

/** Mirrors `fn find_tool_definition`. */
export function findToolDefinition(definitions, toolName) {
  for (const definition of definitions) {
    const found = findQualified(definition, null, toolName);
    if (found) return found;
  }
  return null;
}

function namespaceIdentity(value, prefix, toolName) {
  if (!isObject(value) || value.type !== 'namespace' || typeof value.name !== 'string') return null;
  if (!Array.isArray(value.tools)) return null;
  const qualified = qualifyToolName(prefix, value.name);
  for (const child of value.tools) {
    const nested = namespaceIdentity(child, qualified, toolName);
    if (nested) return nested;
    const childName = toolDefinitionName(child);
    if (childName === null) continue;
    if (qualifyToolName(qualified, childName) === toolName) return [childName, qualified];
  }
  return null;
}

/** Mirrors `fn response_tool_call_identity`: `[name, namespace|null]`. */
export function responseToolCallIdentity(definitions, toolName) {
  for (const definition of definitions) {
    const identity = namespaceIdentity(definition, null, toolName);
    if (identity) return identity;
  }
  return [toolName, null];
}

/** Mirrors `fn is_hosted_tool_definition`. */
export function isHostedToolDefinition(value, capability) {
  return isObject(value) && hostedToolTypeName(value) === capability;
}

/** Mirrors `fn matches_tool_choice_none`. */
export function matchesToolChoiceNone(value) {
  if (value === null || value === undefined) return true;
  if (typeof value === 'string') return value.toLowerCase() === 'none';
  if (isObject(value)) return typeof value.type === 'string' && value.type.toLowerCase() === 'none';
  return false;
}

/** Mirrors `ChatCompletionRequest::requested_tool_names` (sorted, deduped). */
export function requestedToolNames(request) {
  const names = [];
  const choice = request.tool_choice ?? null;
  const call = request.function_call ?? null;
  if (choice !== null) {
    const name = toolChoiceFunctionName(choice);
    if (name !== null) names.push(name);
  }
  if (call !== null) {
    const name = toolChoiceFunctionName(call);
    if (name !== null) names.push(name);
  }
  if (!(choice !== null && matchesToolChoiceNone(choice))) {
    for (const tool of request.tools || []) names.push(...toolDefinitionNames(tool));
  }
  if (!(call !== null && matchesToolChoiceNone(call))) {
    for (const fn of request.functions || []) names.push(...toolDefinitionNames(fn));
  }
  return [...new Set(names)].sort(byteCompare);
}

function byteCompare(left, right) {
  const a = Array.from(left, (character) => character.codePointAt(0));
  const b = Array.from(right, (character) => character.codePointAt(0));
  for (let index = 0; index < Math.min(a.length, b.length); index += 1) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return a.length - b.length;
}

/**
 * The capabilities the default associative packages grant
 * (rust/src/associative_package.rs `default_associative_packages`, every
 * `with_permission` row).
 */
const DEFAULT_GRANTS = new Set([
  'tool:calculator', 'tool:web_search', 'tool:javascript_execution', 'tool:concept_lookup',
  'tool:write_program', 'tool:web_fetch', 'tool:write_file', 'tool:run_command',
  ...['search', 'fetch', 'read', 'write', 'edit', 'run', 'grep', 'glob', 'list_dir', 'todo',
    'subagent', 'read_many', 'multi_edit', 'ask_user'].map((name) => `tool:capability:${name}`),
  ...['fs.read', 'fs.write', 'fs.list', 'fs.move', 'shell.run', 'http.fetch', 'http.post',
    'dom.query', 'dom.extract', 'archive.pack', 'archive.unpack', 'process.status']
    .map((name) => `tool:computer:${name}`),
]);

/** The computer-use primitive names (`COMPUTER_USE_PRIMITIVES`, `name()`). */
export const COMPUTER_USE_PRIMITIVES = [
  'fs.read', 'fs.write', 'fs.list', 'fs.move', 'shell.run', 'http.fetch', 'http.post',
  'dom.query', 'dom.extract', 'archive.pack', 'archive.unpack', 'process.status',
];

/** Mirrors `ComputerUsePrimitive::from_tool_name`: the dotted primitive name or null. */
export function computerPrimitiveFromToolName(name) {
  const lower = name.trim().replace(/[A-Z]/g, (letter) => letter.toLowerCase());
  return COMPUTER_USE_PRIMITIVES.find((dotted) => {
    const underscored = dotted.replaceAll('.', '_');
    return lower === dotted || lower === underscored || lower.endsWith(`__${dotted}`)
      || lower.endsWith(`__${underscored}`) || lower.endsWith(`_${underscored}`);
  }) ?? null;
}

/**
 * Mirrors `fn agentic_tool_permission_denial`: the first denial
 * `{capability, reason}` for an advertised tool whose capability class no
 * default package grants, or null.
 * @param {Array<string>} names
 * @param {(name: string) => string|null} classify `tool_capability`, as the permission suffix
 */
export function agenticToolPermissionDenial(names, classify) {
  const deny = (capability) => ({
    capability,
    reason: agenticMessage('package_permission_missing', { capability }),
  });
  if (!names.length) return DEFAULT_GRANTS.has('tool:*') ? null : deny('tool:*');
  for (const name of names) {
    const primitive = computerPrimitiveFromToolName(name);
    const key = primitive !== null
      ? `tool:computer:${primitive}`
      : (() => {
        const capability = classify(name);
        return capability === null ? null : `tool:capability:${capability}`;
      })();
    if (key !== null && !DEFAULT_GRANTS.has(key)) return deny(key);
  }
  return null;
}
