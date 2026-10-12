// The rebuild-and-reattach plan for an accepted change
// (rust/src/rebuild_plan.rs). Its grounded artifacts are content-addressed
// from the files the Rust crate embeds with `include_str!` (Cargo.toml, the
// worker glue, the UI entry) and from the owned manifest (src/main.rs); the
// port reads the same repository files through the host. Every step, role
// and the verification sentence are templates in agentic-messages.lino.

import { cached, readText } from '../host.mjs';
import { agenticMessage } from '../messages.mjs';
import { acceptedChange, canonicalChangeRequest, ownedContentId, quote } from './change_request.mjs';
import { stableId } from './engine_stable_identifier.mjs';
import { trimEnd } from './rust_str.mjs';

/** The `include_str!` paths of rust/src/rebuild_plan.rs `grounded_artifacts`. */
const EMBEDDED = Object.freeze({
  cargo_toml: 'rust/Cargo.toml',
  worker_glue: 'rust/embedded/js/worker/formal_ai_worker.js',
  ui_entry: 'rust/embedded/js/index.html',
});

function embeddedText(path) {
  try {
    return readText(path);
  } catch {
    return '';
  }
}

/** Mirrors `fn grounded_artifacts`. */
function groundedArtifacts() {
  return [
    { path: 'Cargo.toml', role: agenticMessage('rebuild_plan_role_cargo'), content_id: stableId('reattach_artifact', embeddedText(EMBEDDED.cargo_toml)) },
    { path: 'src/main.rs', role: agenticMessage('rebuild_plan_role_server'), content_id: ownedContentId('src/main.rs') ?? '' },
    { path: 'js/worker/formal_ai_worker.js', role: agenticMessage('rebuild_plan_role_worker'), content_id: stableId('reattach_artifact', embeddedText(EMBEDDED.worker_glue)) },
    { path: 'js/index.html', role: agenticMessage('rebuild_plan_role_ui'), content_id: stableId('reattach_artifact', embeddedText(EMBEDDED.ui_entry)) },
  ];
}

/** Mirrors `fn pipeline_steps`. */
function pipelineSteps() {
  return [1, 2, 3, 4, 5].map((ordinal) => ({
    ordinal,
    action: agenticMessage(`rebuild_plan_step_${ordinal}_action`),
    command: agenticMessage(`rebuild_plan_step_${ordinal}_command`),
    observable: agenticMessage(`rebuild_plan_step_${ordinal}_observable`),
    reversible: agenticMessage(`rebuild_plan_step_${ordinal}_reversible`),
  }));
}

/** Mirrors `RebuildPlan::for_accepted_change`. */
export function rebuildPlanFor(accepted) {
  const artifacts = groundedArtifacts();
  const artifactIds = artifacts.map((artifact) => artifact.content_id).join(',');
  return {
    id: stableId('rebuild_plan', `${accepted.change_id}:${artifactIds}`),
    change_id: accepted.change_id,
    requirement: accepted.requirement,
    reviewer: accepted.reviewer,
    artifacts,
    steps: pipelineSteps(),
    verification: agenticMessage('rebuild_plan_verification'),
  };
}

/** Mirrors `RebuildPlan::links_notation`. */
export function rebuildPlanLinksNotation(plan) {
  const field = (key, value) => `  ${key} "${quote(value)}"\n`;
  const sub = (key, value) => `      ${key} "${quote(value)}"\n`;
  let out = 'rebuild_plan\n';
  out += field('id', plan.id);
  out += field('change_id', plan.change_id);
  out += field('requirement', plan.requirement);
  out += field('reviewer', plan.reviewer);
  out += field('human_gated', 'true');
  out += '  reattached_artifacts\n';
  for (const artifact of plan.artifacts) {
    out += `    artifact "${quote(artifact.path)}"\n`;
    out += sub('role', artifact.role);
    out += sub('content_id', artifact.content_id);
  }
  out += '  rebuild_and_reattach_pipeline\n';
  for (const step of plan.steps) {
    out += `    step "${step.ordinal}"\n`;
    out += sub('action', step.action);
    out += sub('command', step.command);
    out += sub('observable', step.observable);
    out += sub('reversible', step.reversible);
  }
  out += field('verification', plan.verification);
  return trimEnd(out);
}

/**
 * Mirrors `fn canonical_rebuild_plan`: the canonical change, accepted by a
 * green issue-362 gate (4 passed) and the `maintainer`'s approval.
 */
export function canonicalRebuildPlan() {
  return cached('canonical-rebuild-plan', () => rebuildPlanFor(
    acceptedChange(canonicalChangeRequest(), 'maintainer', 'issue_362_multilingual_coding_modification', 4),
  ));
}
