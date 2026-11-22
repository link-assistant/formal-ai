// Verified, non-visual computer-use plans (issue #707): a port of
// rust/src/computer_use/mod.rs (the primitive taxonomy, `plan_request`,
// `capability_gap_for_request`). The executor (executor.rs) is not ported
// here; the JavaScript MCP server has its own (js/server/mcp.mjs).
//
// A `ComputerUsePrimitive` is kept as its dotted name string (`fs.read`, ...).
// A `ComputerPlanStep` is `{id, primitive, arguments, precondition,
// postcondition}`; a `ComputerUsePlan` is `{id, locale, prompt, steps}`.

import { COMPUTER_USE_PRIMITIVES, computerPrimitiveFromToolName } from '../protocol_policy.mjs';
import { capabilityGapResponse, stepConditions } from './computer_use_seed.mjs';
import { capabilityGapForRequest as synthesisCapabilityGap } from './computer_use_synthesis.mjs';

import { detect } from './language.mjs';
import { sha256Hex } from './source_fetch.mjs';

export { COMPUTER_USE_PRIMITIVES };

/** Mirrors `ComputerUsePrimitive::from_tool_name`: the dotted primitive name or null. */
export const fromToolName = computerPrimitiveFromToolName;

/** Mirrors `ComputerUsePrimitive::permission_key`. @param {string} primitive */
export function permissionKey(primitive) {
  return `tool:computer:${primitive}`;
}

const STATE_CHANGING = new Set(['fs.write', 'fs.move', 'shell.run', 'http.post', 'archive.pack', 'archive.unpack']);

/** Mirrors `ComputerUsePrimitive::changes_state`. @param {string} primitive */
export function changesState(primitive) {
  return STATE_CHANGING.has(primitive);
}

const STRING = { type: 'string' };
const BOOLEAN = { type: 'boolean' };

/** The per-primitive `properties` of `ComputerUsePrimitive::input_schema`. */
function primitiveProperties(primitive) {
  switch (primitive) {
    case 'fs.read':
    case 'fs.list':
      return { path: STRING };
    case 'fs.write':
      return { path: STRING, content: STRING, confirmed: BOOLEAN };
    case 'fs.move':
      return { from: STRING, to: STRING, confirmed: BOOLEAN };
    case 'shell.run':
      return {
        operation: { type: 'string', enum: ['count_lines', 'filter_csv', 'unique_csv'] },
        input: STRING, output: STRING, column: STRING, equals: STRING, confirmed: BOOLEAN,
      };
    case 'http.fetch':
      return { url: STRING, save_as: STRING };
    case 'http.post':
      return { url: STRING, body: STRING, save_as: STRING, confirmed: BOOLEAN };
    case 'dom.query':
      return { source: STRING, selector: STRING, save_as: STRING };
    case 'dom.extract':
      return { source: STRING, pointer: STRING, save_as: STRING };
    case 'archive.pack':
      return { paths: { type: 'array', items: STRING }, archive: STRING, confirmed: BOOLEAN };
    case 'archive.unpack':
      return { archive: STRING, destination: STRING, confirmed: BOOLEAN };
    case 'process.status':
      return { save_as: STRING };
    default:
      return {};
  }
}

const REQUIRED = {
  'fs.read': ['path'],
  'fs.list': ['path'],
  'fs.write': ['path', 'content', 'confirmed'],
  'fs.move': ['from', 'to', 'confirmed'],
  'shell.run': ['operation', 'input', 'output', 'confirmed'],
  'http.fetch': ['url', 'save_as'],
  'http.post': ['url', 'body', 'save_as', 'confirmed'],
  'dom.query': ['source', 'selector', 'save_as'],
  'dom.extract': ['source', 'pointer', 'save_as'],
  'archive.pack': ['paths', 'archive', 'confirmed'],
  'archive.unpack': ['archive', 'destination', 'confirmed'],
  'process.status': ['save_as'],
};

const PLAN_FIELDS = ['plan_id', 'step_id', 'precondition', 'postcondition'];

/** Mirrors `ComputerUsePrimitive::input_schema` (with `merge_plan_properties`). */
export function inputSchema(primitive) {
  const properties = primitiveProperties(primitive);
  for (const key of PLAN_FIELDS) properties[key] = { type: 'string' };
  return {
    type: 'object',
    properties,
    required: [...(REQUIRED[primitive] || []), ...PLAN_FIELDS],
    additionalProperties: false,
  };
}

const PLAN_FENCE = '```computer-use';
const encoder = new TextEncoder();

function validPlanArguments(primitive, args) {
  if (!args || typeof args !== 'object' || Array.isArray(args)) return false;
  if (!REQUIRED[primitive].every((key) => Object.hasOwn(args, key))) return false;
  if (primitive === 'shell.run') {
    if (['filter_csv', 'unique_csv'].includes(args.operation) && !Object.hasOwn(args, 'column')) return false;
    if (args.operation === 'filter_csv' && !Object.hasOwn(args, 'equals')) return false;
  }
  const properties = primitiveProperties(primitive);
  return Object.entries(args).every(([key, value]) => {
    if (!Object.hasOwn(properties, key)) return false;
    const property = properties[key];
    if (property.type === 'array') {
      return Array.isArray(value) && value.length > 0 && value.every((entry) => typeof entry === 'string');
    }
    return typeof value === property.type && (property.enum === undefined || property.enum.includes(value));
  });
}

/** Collect and validate every step before handing any tool call to the client. */
export function planRequest(prompt) {
  const lines = prompt.split(/\r?\n/u);
  const start = lines.findIndex((line) => line.trimStart().startsWith('```'));
  const end = lines.findIndex((line, index) => index > start && line.trim() === '```');
  if (start < 0 || lines[start].trim() !== PLAN_FENCE || end < 0 || lines.slice(end + 1).some((line) => line.trim())) return null;
  const body = lines.slice(start + 1, end).join('\n');
  const bytes = encoder.encode(body);
  if (bytes.length > 256 * 1024) return null;
  let entries;
  try {
    entries = JSON.parse(body);
  } catch {
    return null;
  }
  if (!Array.isArray(entries) || entries.length < 1 || entries.length > 64) return null;
  const id = `computer-use-${sha256Hex(bytes).slice(0, 24)}`;
  const steps = [];
  for (const entry of entries) {
    if (entry === null || typeof entry !== 'object' || Object.keys(entry).length !== 2
        || typeof entry.primitive !== 'string' || !Object.hasOwn(entry, 'arguments')) return null;
    const primitive = fromToolName(entry.primitive);
    if (primitive !== entry.primitive || !validPlanArguments(primitive, entry.arguments)) return null;
    const number = String(steps.length + 1).padStart(2, '0');
    const [precondition, postcondition] = stepConditions(primitive);
    steps.push({ id: `${id}-${number}`, primitive, arguments: entry.arguments, precondition, postcondition });
  }
  return { id, locale: detect(lines.slice(0, start).join('\n')), prompt, steps };
}

/** A malformed explicit plan or a request requiring an unavailable capability. */
export function capabilityGapForRequest(prompt) {
  const lines = prompt.split(/\r?\n/u);
  const start = lines.findIndex((line) => line.trimStart().startsWith('```'));
  if (start >= 0 && lines[start].trim() === PLAN_FENCE) {
    return planRequest(prompt) === null
      ? capabilityGapResponse('plan-input', detect(lines.slice(0, start).join('\n')))
      : null;
  }
  return synthesisCapabilityGap(prompt);
}
