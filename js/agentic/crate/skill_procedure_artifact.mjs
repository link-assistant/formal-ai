// Durable, source-grounded procedure artifacts: a port of
// rust/src/skill_procedure/artifact.rs (`artifact_links_notation`,
// `from_artifact_links_notation`, `validate_artifact`,
// `extract_compiled_procedure_artifact`). Errors are
// `ProcedureArtifactError`s; the planner only reads `.ok()`, so the reader
// returns null for any of them.

import { jsonText } from '../plan.mjs';
import { impulseIdFor } from './intent_formalization.mjs';
import { pushLinoNode } from './links_format.mjs';
import { byteToUtf16 } from './rust_str.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';
import { stableId } from './engine_stable_id.mjs';
import {
  KNOWLEDGE_SCHEMA_VERSION, MINIMUM_STEPS, ROLE_SKILL_PROCEDURE_STEP_OBJECT, ROLE_SKILL_PROCEDURE_STEP_VERB,
  ROLE_TRANSLATION_LANGUAGE, canonicalProgram, meaningHasRole, requirementId, stepId,
} from './skill_procedure.mjs';

/** A `ProcedureArtifactError`; `reason` is a slug, not the Rust prose. */
class ProcedureArtifactError extends Error {}

/**
 * Mirrors `CompiledProcedure::artifact_links_notation`: the complete
 * executable artifact with its source formalization and provenance.
 * @param {object} procedure a `CompiledProcedure`
 */
export function artifactLinksNotation(procedure) {
  let out = '';
  const push = (indent, name, value) => {
    out = pushLinoNode(out, indent, name, value);
  };
  push(0, 'compiled_procedure_artifact', procedure.id);
  push(2, 'schema_version', KNOWLEDGE_SCHEMA_VERSION);
  push(2, 'impulse_id', procedure.impulse_id);
  push(2, 'source_description', procedure.source_description);
  push(2, 'canonical_program', procedure.canonical_program);
  for (const requirement of procedure.requirements) {
    push(2, 'requirement', requirement.id);
    push(4, 'index', String(requirement.index));
    push(4, 'source_text', requirement.source_text);
    push(4, 'span_start', String(requirement.source_span[0]));
    push(4, 'span_end', String(requirement.source_span[1]));
  }
  push(2, 'trigger', null);
  push(4, 'requirement_id', procedure.trigger.requirement_id);
  push(4, 'source_text', procedure.trigger.source_text);
  push(4, 'span_start', String(procedure.trigger.source_span[0]));
  push(4, 'span_end', String(procedure.trigger.source_span[1]));
  for (const object of procedure.trigger.objects) push(4, 'object', object);
  for (const step of procedure.steps) {
    push(2, 'step', step.id);
    push(4, 'index', String(step.index));
    push(4, 'requirement_id', step.requirement_id);
    push(4, 'kind', step.kind);
    for (const object of step.objects) push(4, 'object', object);
    if (step.target_language !== null) push(4, 'target_language', step.target_language);
    push(4, 'source_text', step.source_text);
    push(4, 'span_start', String(step.source_span[0]));
    push(4, 'span_end', String(step.source_span[1]));
  }
  return out;
}

/** Mirrors `fn artifact_child` in rust/src/skill_procedure/artifact.rs. */
function artifactChild(node, name) {
  const value = findChildValue(node, name);
  if (value === '') throw new ProcedureArtifactError(`missing_field:${name}:${node.name}`);
  return value;
}

/** Mirrors `fn optional_artifact_child`. */
function optionalArtifactChild(node, name) {
  const value = findChildValue(node, name);
  return value === '' ? null : value;
}

/** Mirrors `fn artifact_usize`. */
function artifactUsize(node, name) {
  const value = artifactChild(node, name);
  if (!/^\+?[0-9]+$/.test(value)) throw new ProcedureArtifactError(`invalid_field:${name}:${node.name}`);
  return Number(value.replace(/^\+/, ''));
}

/** Mirrors `fn child_values`. */
const childValues = (node, name) => (node.children || []).filter((child) => child.name === name).map((child) => child.value);

/** Mirrors `fn span_matches`: `source.get(span) == Some(expected)` on UTF-8 byte offsets. */
function spanMatches(source, span, expected) {
  if (span[0] > span[1]) return false;
  const start = byteToUtf16(source, span[0]);
  const end = byteToUtf16(source, span[1]);
  return start !== null && end !== null && source.slice(start, end) === expected;
}

/**
 * Whether two `[start, end]` source spans are the same span.
 * @param {number[]} left
 * @param {number[]} right
 * @returns {boolean}
 */
const sameSpan = (left, right) => left[0] === right[0] && left[1] === right[1];

/** Mirrors `CompiledProcedure::validate_artifact`. */
function validateArtifact(procedure) {
  if (procedure.requirements.length === 0 || procedure.steps.length < MINIMUM_STEPS) {
    throw new ProcedureArtifactError('incomplete_procedure');
  }
  if (procedure.impulse_id !== impulseIdFor(procedure.source_description)) {
    throw new ProcedureArtifactError('impulse_identity');
  }
  procedure.requirements.forEach((requirement, index) => {
    if (requirement.index !== index + 1
      || requirement.id !== requirementId(procedure.impulse_id, index + 1, requirement)
      || !spanMatches(procedure.source_description, requirement.source_span, requirement.source_text)) {
      throw new ProcedureArtifactError(`invalid_requirement_provenance:${requirement.id}`);
    }
  });
  const triggerIndex = procedure.requirements.findIndex((requirement) => requirement.id === procedure.trigger.requirement_id);
  if (triggerIndex < 0) throw new ProcedureArtifactError('invalid_trigger_requirement');
  const triggerRequirement = procedure.requirements[triggerIndex];
  if (procedure.requirements.length !== triggerIndex + procedure.steps.length + 1
    || procedure.trigger.source_text !== triggerRequirement.source_text
    || !sameSpan(procedure.trigger.source_span, triggerRequirement.source_span)
    || !spanMatches(procedure.source_description, procedure.trigger.source_span, procedure.trigger.source_text)) {
    throw new ProcedureArtifactError('invalid_trigger_provenance');
  }
  procedure.steps.forEach((step, index) => {
    const requirement = procedure.requirements[triggerIndex + index + 1];
    if (step.index !== index + 1 || step.requirement_id !== requirement.id
      || step.source_text !== requirement.source_text || !sameSpan(step.source_span, requirement.source_span)
      || !spanMatches(procedure.source_description, step.source_span, step.source_text)) {
      throw new ProcedureArtifactError(`invalid_step_provenance:${step.id}`);
    }
  });
  const objects = [...procedure.trigger.objects, ...procedure.steps.flatMap((step) => step.objects)];
  if (objects.some((slug) => !meaningHasRole(slug, ROLE_SKILL_PROCEDURE_STEP_OBJECT))
    || procedure.steps.some((step) => !meaningHasRole(step.kind, ROLE_SKILL_PROCEDURE_STEP_VERB))
    || procedure.steps.some((step) => step.target_language !== null && !meaningHasRole(step.target_language, ROLE_TRANSLATION_LANGUAGE))) {
    throw new ProcedureArtifactError('untyped_operation');
  }
  const canonical = canonicalProgram(procedure.trigger, procedure.steps);
  if (canonical !== procedure.canonical_program || stableId('compiled_procedure', canonical) !== procedure.id) {
    throw new ProcedureArtifactError('canonical_integrity');
  }
  for (const step of procedure.steps) {
    if (step.id !== stepId(procedure.id, step)) throw new ProcedureArtifactError(`step_id_integrity_failure:${step.id}`);
  }
}

/** Mirrors `CompiledProcedure::from_artifact_links_notation` (throws `ProcedureArtifactError`). */
function fromArtifactLinksNotation(text) {
  const root = (parseRoot(text).children || []).find((node) => node.name === 'compiled_procedure_artifact');
  if (!root) throw new ProcedureArtifactError('missing_artifact');
  if (root.value === '') throw new ProcedureArtifactError('artifact_id_empty');
  if (artifactChild(root, 'schema_version') !== KNOWLEDGE_SCHEMA_VERSION) throw new ProcedureArtifactError('unsupported_schema');
  const sourceDescription = artifactChild(root, 'source_description');
  const impulseId = artifactChild(root, 'impulse_id');
  const canonical = artifactChild(root, 'canonical_program');
  const requirements = (root.children || []).filter((node) => node.name === 'requirement').map((node) => ({
    id: node.value,
    index: artifactUsize(node, 'index'),
    source_text: artifactChild(node, 'source_text'),
    source_span: [artifactUsize(node, 'span_start'), artifactUsize(node, 'span_end')],
  }));
  const triggerNode = (root.children || []).find((node) => node.name === 'trigger');
  if (!triggerNode) throw new ProcedureArtifactError('missing_trigger');
  const trigger = {
    requirement_id: artifactChild(triggerNode, 'requirement_id'),
    objects: childValues(triggerNode, 'object'),
    source_text: artifactChild(triggerNode, 'source_text'),
    source_span: [artifactUsize(triggerNode, 'span_start'), artifactUsize(triggerNode, 'span_end')],
  };
  const steps = (root.children || []).filter((node) => node.name === 'step').map((node) => ({
    id: node.value,
    index: artifactUsize(node, 'index'),
    requirement_id: artifactChild(node, 'requirement_id'),
    kind: artifactChild(node, 'kind'),
    objects: childValues(node, 'object'),
    target_language: optionalArtifactChild(node, 'target_language'),
    source_text: artifactChild(node, 'source_text'),
    source_span: [artifactUsize(node, 'span_start'), artifactUsize(node, 'span_end')],
  }));
  const procedure = {
    id: root.value,
    source_description: sourceDescription,
    impulse_id: impulseId,
    requirements,
    trigger,
    steps,
    canonical_program: canonical,
  };
  validateArtifact(procedure);
  return procedure;
}

/**
 * Mirrors `fn extract_compiled_procedure_artifact(text).ok()`: the first
 * complete, integrity-checked artifact embedded in a response, or null.
 * @param {string} text
 * @returns {object|null}
 */
export function extractCompiledProcedureArtifact(text) {
  const start = text.indexOf('compiled_procedure_artifact ');
  if (start < 0) return null;
  const tail = text.slice(start);
  const fence = tail.indexOf('\n```');
  try {
    return fromArtifactLinksNotation(fence < 0 ? tail : tail.slice(0, fence));
  } catch (error) {
    if (error instanceof ProcedureArtifactError) return null;
    throw error;
  }
}

/** Rust built-in `#[derive(PartialEq)]` on `CompiledProcedure`. */
export function proceduresEqual(left, right) {
  return jsonText(left) === jsonText(right);
}
