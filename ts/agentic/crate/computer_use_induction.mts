// Auto-learning: induce generalized computer-use schemas from the recorded
// benchmark corpus (issue #707). A port of rust/src/computer_use/induction.rs.
//
// Rust `BTreeMap`/`BTreeSet` become `Map`/`Set` whose keys are iterated in
// byte order (`sortedKeys`), so `links_notation` prints the same record.
//
// Shapes: a `StepSignature` is `{primitive, operation}` (operation a string or
// null); a `StepSchema` is `{signature, constants: Map, varying: Set,
// support}`; an `OperationSchema` is `{operation, step, verification}`; a
// `ResourceBinding` is `{resource, steps, parameters: Map, support,
// alternatives}`; `LearnedSchemas` is `{operations: Map, resources: Map,
// rejected, unexplained}`.

import { cached } from '../host.mjs';
import { byteOrder, compactJson } from './rust_str.mjs';
import { normalize, operationCues, resourceCue } from './computer_use_lexicon.mjs';
import { benchmarkTasks, cloneStep } from './computer_use_seed.mjs';
import { COMPUTER_USE_PRIMITIVES } from '../protocol_policy.mjs';

/** Head of the record `links_notation` emits. */
const RECORD_TYPE = 'computer_use_learned_schemas';
/** The corpus the schemas are induced from. */
const CORPUS_PATH = 'data/seed/computer-use-tasks.lino';

/** `FETCH_OPERATION`: the operation realised by a materialisation prefix. */
export const FETCH_OPERATION = 'computer_use_fetch';

const sortedKeys = (map) => [...map.keys()].sort(byteOrder);
const sortedValues = (set) => [...set].sort(byteOrder);

/** Mirrors `StepSignature::of`. */
export function signatureOf(step) {
  const operation = step.arguments?.operation;
  return { primitive: step.primitive, operation: typeof operation === 'string' ? operation : null };
}

/** Mirrors `StepSignature::label`. */
export function signatureLabel(signature) {
  return signature.operation === null ? signature.primitive : `${signature.primitive}:${signature.operation}`;
}

/** `StepSignature`'s derived `Ord` (enum declaration order, then `Option<String>`). */
function compareSignatures(left, right) {
  const primitive = COMPUTER_USE_PRIMITIVES.indexOf(left.primitive) - COMPUTER_USE_PRIMITIVES.indexOf(right.primitive);
  if (primitive) return primitive;
  if (left.operation === null || right.operation === null) {
    return (left.operation === null ? 0 : 1) - (right.operation === null ? 0 : 1);
  }
  return byteOrder(left.operation, right.operation);
}

const isObjectValue = (value) => Boolean(value) && typeof value === 'object' && !Array.isArray(value);

/** Mirrors `fn learned`: the schemas induced from the committed corpus (cached). */
export function learned() {
  return cached('computer-use-learned', () => induce(benchmarkTasks()));
}

/** Mirrors `fn partition`: `[materialisation, body_end]`. */
export function partition(steps) {
  let materialisation = 0;
  while (materialisation < steps.length
    && (steps[materialisation].primitive === 'fs.write' || steps[materialisation].primitive === 'http.fetch')) {
    materialisation += 1;
  }
  let verification = 0;
  for (let index = steps.length - 1; index >= materialisation; index -= 1) {
    const primitive = steps[index].primitive;
    if (primitive !== 'fs.read' && primitive !== 'fs.list') break;
    verification += 1;
  }
  return [materialisation, steps.length - verification];
}

function pushInto(map, key, value) {
  if (!map.has(key)) map.set(key, []);
  map.get(key).push(value);
}

/** Mirrors `fn induce`. */
export function induce(tasks) {
  const observations = new Map();
  const verifications = new Map();
  const resourceSteps = new Map();
  const operationsPerTask = new Map();
  const schemas = { operations: new Map(), resources: new Map(), rejected: [], unexplained: [] };

  for (const task of tasks) {
    const operations = recognizedOperations(task);
    const resource = recognizedResource(task);
    operationsPerTask.set(task.id, operations.slice());
    const [materialised, bodyEnd] = partition(task.steps);
    const body = task.steps.slice(materialised, bodyEnd);
    const verification = task.steps.slice(bodyEnd);

    if (resource !== null) {
      const steps = task.steps.slice(0, materialised).map(cloneStep)
        .filter((step) => step.arguments !== null);
      pushInto(resourceSteps, resource, [task.id, steps]);
    }

    const bodyOperations = operations.filter((slug) => slug !== FETCH_OPERATION || materialised === 0);
    const attributed = body.map(() => false);
    bodyOperations.forEach((operation, index) => {
      if (index < body.length) {
        pushInto(observations, operation, [task.id, cloneStep(body[index])]);
        attributed[index] = true;
      }
    });
    if (bodyOperations.length && verification.length && bodyOperations.length >= body.length) {
      pushInto(verifications, bodyOperations[bodyOperations.length - 1], verification.map(cloneStep));
    }
    if (operations.includes(FETCH_OPERATION) && materialised > 0) {
      const fetch = task.steps.slice(0, materialised).find((step) => step.primitive === 'http.fetch');
      if (fetch) pushInto(observations, FETCH_OPERATION, [task.id, cloneStep(fetch)]);
    }
    body.forEach((step, index) => {
      if (!attributed[index]) schemas.unexplained.push(`${task.id}:${step.primitive}`);
    });
  }

  for (const operation of sortedKeys(observations)) {
    const examples = observations.get(operation);
    const signatures = uniqueSignatures(examples.map(([, step]) => signatureOf(step)));
    if (signatures.length !== 1) {
      schemas.rejected.push(`${operation}:ambiguous_signature`);
      continue;
    }
    const namedIn = [...operationsPerTask].filter(([, names]) => names.includes(operation)).map(([task]) => task);
    const observedIn = new Set(examples.map(([task]) => task));
    if (!namedIn.every((task) => observedIn.has(task))) {
      schemas.rejected.push(`${operation}:missing_in_supporting_task`);
      continue;
    }
    const step = stepSchema(signatures[0], examples);
    const observed = verifications.get(operation);
    schemas.operations.set(operation, {
      operation,
      step,
      verification: observed ? verificationSchema(observed) : [],
    });
  }

  for (const resource of sortedKeys(resourceSteps)) {
    const observed = resourceSteps.get(resource);
    if (!observed.length) continue;
    const [task, steps] = observed[0];
    schemas.resources.set(resource, {
      resource,
      steps: steps.map(cloneStep),
      parameters: resourceParameters(resource, tasks),
      support: [task],
      alternatives: observed.slice(1).map(([id]) => id),
    });
  }
  schemas.rejected = [...new Set(schemas.rejected)].sort(byteOrder);
  schemas.unexplained = [...new Set(schemas.unexplained)].sort(byteOrder);
  return schemas;
}

/** The distinct signatures, sorted (`BTreeSet<StepSignature>`). */
function uniqueSignatures(signatures) {
  const out = [];
  for (const signature of signatures) {
    if (!out.some((seen) => compareSignatures(seen, signature) === 0)) out.push(signature);
  }
  return out.sort(compareSignatures);
}

/** Mirrors `fn recognized_operations`: operations every localized prompt evidences. */
function recognizedOperations(task) {
  const perLocale = task.prompts.map(([, prompt]) => operationCues(normalize(prompt)).map((cue) => cue.slug));
  if (!perLocale.length) return [];
  const [first, ...rest] = perLocale;
  const shared = rest.reduce((set, locale) => new Set(locale.filter((slug) => set.has(slug))), new Set(first));
  return first.filter((slug) => shared.has(slug));
}

/** Mirrors `fn recognized_resource`: the resource most locales evidence (ties: smallest slug). */
function recognizedResource(task) {
  const counts = new Map();
  for (const [, prompt] of task.prompts) {
    const cue = resourceCue(normalize(prompt));
    if (cue !== null) counts.set(cue.slug, (counts.get(cue.slug) ?? 0) + 1);
  }
  let best = null;
  for (const slug of sortedKeys(counts)) {
    const count = counts.get(slug);
    // `max_by(count, then reversed slug)`: a later entry replaces only on a strictly higher count.
    if (best === null || count > best[1]) best = [slug, count];
  }
  return best === null ? null : best[0];
}

/** Mirrors `fn step_schema`: unanimous fields become constants, differing ones slots. */
function stepSchema(signature, examples) {
  const constants = new Map();
  const varying = new Set();
  for (const [, step] of examples) {
    if (!isObjectValue(step.arguments)) continue;
    for (const key of Object.keys(step.arguments).sort(byteOrder)) {
      const value = step.arguments[key];
      if (!constants.has(key)) {
        if (!varying.has(key)) constants.set(key, structuredClone(value));
      } else if (compactJson(constants.get(key)) !== compactJson(value)) {
        constants.delete(key);
        varying.add(key);
      }
    }
  }
  return { signature, constants, varying, support: examples.map(([task]) => task) };
}

/** Mirrors `fn verification_schema`. */
function verificationSchema(observed) {
  const first = observed[0];
  if (!first) return [];
  const sameShape = (steps) => steps.every((step, index) =>
    compareSignatures(signatureOf(step), signatureOf(first[index])) === 0);
  if (observed.some((steps) => steps.length !== first.length || !sameShape(steps))) return [];
  return first.map((step) => ({ signature: signatureOf(step), constants: new Map(), varying: new Set(), support: [] }));
}

const PARAMETER_FIELDS = ['selector', 'pointer', 'column', 'equals', 'body'];

/** Mirrors `fn resource_parameters`: first-seen field values per resource. */
function resourceParameters(resource, tasks) {
  const parameters = new Map();
  for (const task of tasks) {
    if (recognizedResource(task) !== resource) continue;
    for (const step of task.steps) {
      if (!isObjectValue(step.arguments)) continue;
      for (const key of PARAMETER_FIELDS) {
        if (Object.prototype.hasOwnProperty.call(step.arguments, key) && !parameters.has(key)) {
          parameters.set(key, structuredClone(step.arguments[key]));
        }
      }
    }
  }
  return parameters;
}

/** Mirrors `LearnedSchemas::links_notation`: the reviewable record of what was learned. */
export function linksNotation(schemas) {
  const out = [RECORD_TYPE, `  source ${CORPUS_PATH}`];
  for (const key of sortedKeys(schemas.operations)) {
    const schema = schemas.operations.get(key);
    out.push(`  operation ${schema.operation}`, `    step ${signatureLabel(schema.step.signature)}`);
    for (const name of sortedKeys(schema.step.constants)) {
      out.push(`    constant ${name} ${compactJson(schema.step.constants.get(name))}`);
    }
    for (const name of sortedValues(schema.step.varying)) out.push(`    slot ${name}`);
    for (const task of schema.step.support) out.push(`    support ${task}`);
    for (const step of schema.verification) out.push(`    verification ${signatureLabel(step.signature)}`);
  }
  for (const key of sortedKeys(schemas.resources)) {
    const binding = schemas.resources.get(key);
    out.push(`  resource ${binding.resource}`);
    for (const step of binding.steps) out.push(`    materialize ${step.primitive} ${compactJson(step.arguments)}`);
    for (const name of sortedKeys(binding.parameters)) {
      out.push(`    parameter ${name} ${compactJson(binding.parameters.get(name))}`);
    }
    for (const task of binding.support) out.push(`    support ${task}`);
    for (const task of binding.alternatives) out.push(`    alternative ${task}`);
  }
  for (const rejected of schemas.rejected) out.push(`  rejected ${rejected}`);
  for (const unexplained of schemas.unexplained) out.push(`  unexplained ${unexplained}`);
  return `${out.join('\n')}\n`;
}
