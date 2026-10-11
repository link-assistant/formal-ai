// Plan synthesis for unseen computer-use requests (issue #707): a port of
// rust/src/computer_use/synthesis.rs.
//
// Given a prompt in any of the four supported languages, recognise the
// resource and the operations the speaker named, materialise the resource
// with the learned binding, chain the learned operation steps in prompt
// order, and append the learned verification. A `Synthesis` is `{plan,
// operations, resource}`.

import { detect } from './language.mjs';
import { FETCH_OPERATION, learned } from './computer_use_induction.mjs';
import { capabilityGapCue, instructionSurface, normalize, operationCues, resourceCue } from './computer_use_lexicon.mjs';
import { capabilityGapResponse, cloneStep, stepConditions } from './computer_use_seed.mjs';
import { changesState, inputSchema } from './computer_use.mjs';

/** `PATH_FIELDS`: argument fields bound per request from the data flow. */
const PATH_FIELDS = ['path', 'paths', 'input', 'output', 'source', 'save_as', 'archive', 'destination', 'from', 'to'];
/** `RESOURCE_FIELDS`: argument fields taken from the resource binding. */
const RESOURCE_FIELDS = ['selector', 'pointer', 'column', 'equals'];

/** Mirrors `fn capability_gap_for_request` in rust/src/computer_use/synthesis.rs. */
export function capabilityGapForRequest(prompt) {
  const locale = detect(prompt);
  const capability = capabilityGapCue(normalize(instructionSurface(prompt)));
  if (capability === null) return null;
  return capabilityGapResponse(capability, locale);
}

/** Mirrors `fn synthesize`: a plan from the learned schemas, or null. */
export function synthesize(prompt) {
  return synthesizeWith(prompt, learned());
}

/** Mirrors `fn synthesize_with`. */
export function synthesizeWith(prompt, schemas) {
  const normalized = normalize(instructionSurface(prompt));
  if (capabilityGapCue(normalized) !== null) return null;
  const locale = detect(prompt);
  const resource = resourceCue(normalized)?.slug;
  if (resource === undefined) return null;
  const binding = schemas.resources.get(resource);
  if (!binding) return null;

  const operations = [];
  for (const cue of operationCues(normalized)) {
    if (!operations.includes(cue.slug) && schemas.operations.has(cue.slug)) operations.push(cue.slug);
  }
  if (!operations.length) return null;

  let steps = binding.steps.map(cloneStep);
  let artifact = lastArtifact(steps);
  if (artifact === null) return null;
  const materialisedFetch = steps.some((step) => step.primitive === 'http.fetch');

  for (const operation of operations) {
    if (operation === FETCH_OPERATION && materialisedFetch) continue;
    const schema = schemas.operations.get(operation);
    if (!schema) return null;
    const realised = realise(schema.step.signature.primitive, schema.step.signature.operation,
      schema.step.constants, schema.step.varying, binding.parameters, resource, artifact);
    if (realised === null) return null;
    steps.push(realised[0]);
    artifact = realised[1];
  }

  steps.push(verificationStep(artifact));
  const id = `synthesized-${resource}-${operations.join('-')}`;
  steps = finalize(id, steps);
  return { plan: { id, locale, prompt, steps }, operations, resource };
}

/** Mirrors `fn last_artifact`. */
function lastArtifact(steps) {
  const step = steps[steps.length - 1];
  if (!step) return null;
  const args = step.arguments;
  const has = (key) => args && typeof args === 'object' && !Array.isArray(args)
    && Object.prototype.hasOwnProperty.call(args, key);
  const value = has('save_as') ? args.save_as : has('path') ? args.path : undefined;
  return typeof value === 'string' ? { path: value, directory: false } : null;
}

/** Mirrors `fn realise`: one operation step bound to the data flow, plus the artifact it produces. */
function realise(primitive, operation, constants, varying, parameters, resource, input) {
  const accepted = acceptedFields(primitive);
  const uses = (key) => accepted.includes(key) && (constants.has(key) || varying.has(key));
  const args = {};
  for (const key of RESOURCE_FIELDS) {
    if (!uses(key)) continue;
    if (!parameters.has(key)) return null;
    args[key] = structuredClone(parameters.get(key));
  }
  for (const [key, value] of constants) {
    if (!PATH_FIELDS.includes(key) && !RESOURCE_FIELDS.includes(key) && uses(key)) {
      args[key] = structuredClone(value);
    }
  }

  const stem = resource.split('_').pop();
  let produced;
  switch (primitive) {
    case 'shell.run':
    case 'dom.query':
    case 'dom.extract': {
      const output = `reports/${operation ?? 'extract'}-${stem}.txt`;
      const [from, to] = primitive === 'shell.run' ? ['input', 'output'] : ['source', 'save_as'];
      args[from] = input.path;
      args[to] = output;
      produced = { path: output, directory: false };
      break;
    }
    case 'fs.list': {
      args.path = parentOf(input.path);
      delete args.confirmed;
      return [step(primitive, args), { path: input.path, directory: false }];
    }
    case 'archive.pack': {
      const archive = `out/${stem}.fai`;
      args.paths = [input.path];
      args.archive = archive;
      produced = { path: archive, directory: false };
      break;
    }
    case 'archive.unpack':
      args.archive = input.path;
      args.destination = 'restored';
      produced = { path: 'restored', directory: true };
      break;
    case 'fs.move': {
      const destination = `processed/${basenameOf(input.path)}`;
      args.from = input.path;
      args.to = destination;
      produced = { path: destination, directory: false };
      break;
    }
    case 'process.status': {
      const output = `reports/process-${stem}.json`;
      args.save_as = output;
      produced = { path: output, directory: false };
      break;
    }
    case 'http.post': {
      const output = `reports/submission-${stem}.json`;
      args.save_as = output;
      produced = { path: output, directory: false };
      break;
    }
    default:
      return null;
  }
  return [step(primitive, args), produced];
}

/** Mirrors `fn accepted_fields`: the property names of the primitive's input schema. */
function acceptedFields(primitive) {
  return Object.keys(inputSchema(primitive).properties);
}

/** Mirrors `fn verification_step`. */
function verificationStep(artifact) {
  return step(artifact.directory ? 'fs.list' : 'fs.read', { path: artifact.path });
}

/** Mirrors `fn step`: confirmation per `changes_state`, seeded conditions. */
function step(primitive, args) {
  if (changesState(primitive)) args.confirmed = true;
  else delete args.confirmed;
  const [precondition, postcondition] = stepConditions(primitive);
  return { id: '', primitive, arguments: args, precondition, postcondition };
}

/** Mirrors `fn finalize`: number the steps `<plan>-NN`. */
function finalize(planId, steps) {
  return steps.map((entry, index) => ({ ...entry, id: `${planId}-${String(index + 1).padStart(2, '0')}` }));
}

/** Mirrors `fn parent_of`. @param {string} path */
function parentOf(path) {
  const at = path.lastIndexOf('/');
  return at < 0 ? '.' : path.slice(0, at);
}

/** Mirrors `fn basename_of`. @param {string} path */
function basenameOf(path) {
  const at = path.lastIndexOf('/');
  return at < 0 ? path : path.slice(at + 1);
}
