// The planner's result types and the argument builders every route shares
// (rust/src/agentic_coding/planner.rs `AgenticPlan`, `PlannedToolCall`,
// `plan_one`, `write_arguments`, `fetch_arguments`).
//
// An `AgenticPlan` is `{kind: 'tool_calls', calls: [{tool, arguments}]}` or
// `{kind: 'final', answer}`; `arguments` is the JSON text Rust's
// `serde_json::Value::to_string` prints (see `jsonText`).

/** Mirrors `enum AgenticPlan` in rust/src/agentic_coding/planner.rs (`ToolCalls`). @param {Array<{tool: string, arguments: string}>} calls */
export function toolCalls(calls) {
  return { kind: 'tool_calls', calls };
}

/** `AgenticPlan::Final`. @param {string} answer */
export function finalAnswer(answer) {
  return { kind: 'final', answer };
}

export { FinalDisposition, resolvedFinalAnswer, finalResult, canDeliverFinal, projectPlan } from './final_result.mjs';

/** Mirrors `struct PlannedToolCall` in rust/src/agentic_coding/planner.rs. @param {string} tool @param {string} args */
export function plannedCall(tool, args) {
  return { tool, arguments: args };
}

/** Whether `plan` is `AgenticPlan::ToolCalls`: Rust built-in `matches!`. */
export function isToolCalls(plan) {
  return plan?.kind === 'tool_calls';
}

/** Whether `plan` is `AgenticPlan::Final`: Rust built-in `matches!`. */
export function isFinal(plan) {
  return plan?.kind === 'final';
}

/**
 * `serde_json::Value::to_string` of a JSON object built with `json!`.
 *
 * serde_json without `preserve_order` stores objects in a `BTreeMap`, so its
 * text lists keys sorted by byte order at every depth; this prints the same.
 * @param {unknown} value
 * @returns {string}
 */
export function jsonText(value) {
  return JSON.stringify(sortKeys(value));
}

function sortKeys(value) {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value && typeof value === 'object') {
    const out = {};
    for (const key of Object.keys(value).sort(byteOrder)) out[key] = sortKeys(value[key]);
    return out;
  }
  return value;
}

function byteOrder(left, right) {
  const a = Array.from(left, (character) => character.codePointAt(0));
  const b = Array.from(right, (character) => character.codePointAt(0));
  for (let index = 0; index < Math.min(a.length, b.length); index += 1) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return a.length - b.length;
}

/** `plan_one`: a single planned call. @param {string} tool @param {string} args */
export function planOne(tool, args) {
  return toolCalls([plannedCall(tool, args)]);
}

/** `write_arguments`. @param {string} path @param {string} content */
export function writeArguments(path, content) {
  return jsonText({ path, filePath: path, file_path: path, content });
}

/** `fetch_arguments`. @param {string} url */
export function fetchArguments(url) {
  return jsonText({ url, format: 'text' });
}
