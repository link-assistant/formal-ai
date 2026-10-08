// Verified, non-visual computer-use plans (issue #707): a port of
// rust/src/computer_use/mod.rs (the primitive taxonomy, `plan_request`,
// `capability_gap_for_request`). The executor (executor.rs) is not ported
// here; the JavaScript MCP server has its own (js/server/mcp.mjs).
//
// A `ComputerUsePrimitive` is kept as its dotted name string (`fs.read`, ...).
// A `ComputerPlanStep` is `{id, primitive, arguments, precondition,
// postcondition}`; a `ComputerUsePlan` is `{id, locale, prompt, steps}`.

import { COMPUTER_USE_PRIMITIVES, computerPrimitiveFromToolName } from '../protocol_policy.mjs';
import { capabilityGapForPrompt, planForPrompt } from './computer_use_seed.mjs';
import { capabilityGapForRequest as synthesisCapabilityGap, synthesize } from './computer_use_synthesis.mjs';

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

/**
 * Mirrors `fn plan_request` in rust/src/computer_use/mod.rs: the recorded plan
 * when the prompt is a seeded benchmark task verbatim, else a synthesized one.
 * @param {string} prompt
 */
export function planRequest(prompt) {
  return planForPrompt(prompt) ?? synthesize(prompt)?.plan ?? null;
}

/**
 * Mirrors `fn capability_gap_for_request` in rust/src/computer_use/mod.rs.
 * @param {string} prompt
 * @returns {{capability: string, locale: string, response: string}|null}
 */
export function capabilityGapForRequest(prompt) {
  return capabilityGapForPrompt(prompt) ?? synthesisCapabilityGap(prompt);
}
