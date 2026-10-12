// Named repository-protocol templates (rust/src/repository_workspace/mod.rs).
// The protocol document parser, the step applicability rule and the
// `repository-protocol.lino` trace renderer are ported; the protocol runners
// live callers stay native; injected JavaScript runners live in the sibling
// repository_workspace_runner.mjs and repository_workspace_authoring.mjs modules.

import { readText } from '../host.mjs';
import { findChildValue, parseLinoRoot } from '../write_lino.mjs';

/** Mirrors `fn render_protocol_template_from`. */
export function renderProtocolTemplateFrom(document, id, values) {
  const node = parseLinoRoot(document).children.find((candidate) => candidate.name === 'repository_template' && candidate.id === id);
  if (!node) return null;
  return values.reduce((rendered, [name, value]) => rendered.split(`{${name}}`).join(value), findChildValue(node, 'text'));
}

/**
 * Mirrors `fn render_protocol_template`.
 * @param {string} id
 * @param {Array<[string, string]>} values
 */
export function renderProtocolTemplate(id, values) {
  return renderProtocolTemplateFrom(readText('data/meta/repository-workspace-protocol.lino'), id, values);
}

const RECORD_STEP = 'meta_step';
const STEP_EDITOR_ANY = 'any';
const TRACE_RECORD = 'repository_protocol_trace';
const STAGE_FIELD = 'stage';

/** Mirrors `const EDITOR_STRUCTURAL` (rust/src/repository_workspace/trace.rs). */
export const EDITOR_STRUCTURAL = 'structural';
/** Mirrors `const EDITOR_AGENT_SESSION`. */
export const EDITOR_AGENT_SESSION = 'agent_session';

/**
 * Mirrors `fn parse` (`WorkspaceProtocol::parse`): the declared steps, in order.
 * @param {string} document
 */
export function parseProtocol(document) {
  return parseLinoRoot(document).children
    .filter((node) => findChildValue(node, 'record_type') === RECORD_STEP)
    .map((node) => ({
      order: Number.parseInt(findChildValue(node, 'order'), 10) || 0,
      id: findChildValue(node, 'id'),
      editor: findChildValue(node, 'editor'),
      precondition: [findChildValue(node, 'precondition')],
      postcondition: [findChildValue(node, 'postcondition')],
    }))
    .sort((left, right) => left.order - right.order);
}

/** Mirrors `fn load` (`WorkspaceProtocol::load`). */
export function loadProtocol() {
  return parseProtocol(readText('data/meta/repository-workspace-protocol.lino'));
}

/**
 * Mirrors `fn applies_to` (`ProtocolStep::applies_to`).
 * @param {{ editor: string }} step
 * @param {string} editor
 */
export function stepAppliesTo(step, editor) {
  return step.editor === '' || step.editor === STEP_EDITOR_ANY || step.editor === editor;
}

/**
 * Mirrors `fn new` (`ProtocolTrace::new`): every declared stage not reached
 * yet, except the stages another editor owns.
 * @param {Array<{ id: string, editor: string }>} steps
 * @param {string} caller
 * @param {string} editor
 */
export function newProtocolTrace(steps, caller, editor) {
  return {
    caller,
    editor,
    fields: [],
    stages: steps.map((step) => ({ id: step.id, status: stepAppliesTo(step, editor) ? 'not_reached' : 'not_applicable' })),
    open: [],
  };
}

/** Mirrors `fn record` (`ProtocolTrace::record`). */
export function recordStage(trace, id, status) {
  const stage = trace.stages.find((candidate) => candidate.id === id);
  if (stage) stage.status = status;
}

/** Mirrors `fn set_field` (`ProtocolTrace::set_field`). */
export function setTraceField(trace, name, value) {
  const field = trace.fields.find(([key]) => key === name);
  if (field) field[1] = value;
  else trace.fields.push([name, value]);
}

/** @param {string} value */
function quote(value) {
  return value.split('"').join('""');
}

/** Mirrors `fn render` (`ProtocolTrace::render`): the `repository-protocol.lino` document. */
export function renderProtocolTrace(trace) {
  const lines = [TRACE_RECORD, `  caller "${quote(trace.caller)}"`, `  editor "${quote(trace.editor)}"`];
  for (const [name, value] of trace.fields) lines.push(`  ${name} "${quote(value)}"`);
  for (const stage of trace.stages) lines.push(`  ${STAGE_FIELD} ${stage.id} "${stage.status}"`);
  for (const open of trace.open) lines.push(`  open "${quote(open)}"`);
  return `${lines.join('\n')}\n`;
}

/** Mirrors `fn stage_statuses`: every `[stage, status]` pair a rendered trace records. */
export function traceStageStatuses(document) {
  const pairs = [];
  for (const line of document.split('\n')) {
    const rest = line.trimStart();
    if (!rest.startsWith(`${STAGE_FIELD} `)) continue;
    const body = rest.slice(STAGE_FIELD.length + 1);
    const space = body.indexOf(' ');
    if (space < 0) continue;
    pairs.push([body.slice(0, space), body.slice(space + 1).replace(/^"+|"+$/gu, '')]);
  }
  return pairs;
}

/** Mirrors `fn stage_ids`: the stage ids a rendered trace names, in order. */
export function traceStageIds(document) {
  return traceStageStatuses(document).map(([id]) => id);
}
