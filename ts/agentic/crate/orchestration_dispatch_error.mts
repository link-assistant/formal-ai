// `enum DispatchError` of `crate::orchestration::dispatch`
// (rust/src/orchestration/dispatch.rs), split out of the dispatch module so the
// attribution and incremental modules can raise it without importing dispatch
// back (Rust resolves that cycle at link time; ES modules prefer a leaf).

import { AgentRunError } from './orchestration_runner.mjs';
import { ReplayError } from './orchestration_replay.mjs';
import { IoError } from './orchestration_workspace.mjs';

/** Mirrors `derive(Debug) of DispatchError` in rust/src/orchestration/dispatch.rs. `Debug` rendering of a variant payload. */
function debugPayload(detail) {
  if (detail instanceof IoError) return detail.debug;
  if (detail instanceof AgentRunError) return detail.debug;
  if (detail instanceof ReplayError) return `${detail.code}`;
  return JSON.stringify(detail);
}

/** Mirrors `impl fmt::Display for DispatchError` in rust/src/orchestration/dispatch.rs. */
function display(variant, detail) {
  switch (variant) {
    case 'duplicate_cli': return `duplicate_cli:${detail}`;
    case 'run': return `run:${detail.message}`;
    case 'replay': return `replay:${detail.message}`;
    case 'io': return `io:${detail.message}`;
    case 'composition_conflict': return `composition_conflict:${detail}`;
    case 'attribution': return `attribution:${detail}`;
    default: return variant;
  }
}

/** Mirrors `enum DispatchError` in rust/src/orchestration/dispatch.rs; `message` is its `Display`, `debug` its `Debug`. */
export class DispatchError extends Error {
  /** @param {string} variant the snake_case variant @param {*} [detail] */
  constructor(variant, detail = null) {
    super(display(variant, detail));
    this.variant = variant;
    this.detail = detail;
    const name = variant.split('_').map((part) => part[0].toUpperCase() + part.slice(1)).join('');
    this.debug = detail === null ? name : `${name}(${debugPayload(detail)})`;
  }
}

/** Mirrors `fn attribution_error` in rust/src/orchestration/attribution.rs. */
export function attributionError(message) {
  return new DispatchError('attribution', message);
}
