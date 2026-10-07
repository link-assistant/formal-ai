// The method registry as link data (R331, R1013 server parity):
// rust/src/method_registry.rs `MethodRegistry` (`from_store_with_learned_seed`,
// `method_for_route`, `explicit_learned_method_relevants`,
// `ordered_method_names_for_relevants`, `to_links_notation`) with the data it
// joins - rust/src/solver_dispatch.rs `PRELUDE_METHOD_NAMES` /
// `CONTEXTUAL_HANDLER_NAMES`, the specialized table
// (`specialized_handlers`: data/seed/handler-precedence.lino in rank order
// minus the `browser_only` rows), data/seed/method-execution.lino,
// data/seed/learned-methods.lino bound against
// data/meta/recursive-core-recipe.lino (rust/src/recipe_interpreter.rs),
// data/meta/selection-heuristics.lino (rust/src/selection_heuristics.rs
// `shipped_catalog`) and data/meta/route-method-aliases.lino
// (rust/src/route_method_alias.rs).
//
// A method is `{name, order, surface, execution}`; `execution` is
// `{runtime, response_language, project_lookup, definition_fusion}`.

import { cached, childValue, childrenNamed, readText } from '../host.mjs';
import { parseLinoRoot } from '../write_lino.mjs';
import { formatLinoRecord } from './links_format.mjs';

/** Mirrors `PRELUDE_METHOD_NAMES` in rust/src/solver_dispatch.rs. */
export const PRELUDE_METHOD_NAMES = Object.freeze(['diagnostic', 'nl_tool', 'behavior_rules', 'feature_capability', 'playwright_script']);

/** Mirrors `CONTEXTUAL_HANDLER_NAMES` in rust/src/solver_dispatch.rs. */
export const CONTEXTUAL_HANDLER_NAMES = Object.freeze([
  'http_fetch', 'proof_request', 'meta_explanation', 'numeric_list', 'program_synthesis',
  'shell_command_transform', 'text_manipulation', 'task_decomposition', 'response_language_followup',
  'fact_checking', 'world_state', 'web_search', 'procedural_how_to', 'fact_lookup',
]);

const HANDLER_PRECEDENCE_PATH = 'data/seed/handler-precedence.lino';
const METHOD_EXECUTION_PATH = 'data/seed/method-execution.lino';
const LEARNED_METHODS_PATH = 'data/seed/learned-methods.lino';
const RECIPE_PATH = 'data/meta/recursive-core-recipe.lino';
const HEURISTICS_PATH = 'data/meta/selection-heuristics.lino';
const ALIASES_PATH = 'data/meta/route-method-aliases.lino';
const HEURISTIC_RECORD_TYPE = 'selection_heuristic';
const HEURISTIC_STRUCTURAL_FIELDS = new Set(['record_type', 'role', 'order', 'applies_when']);

/** Mirrors `fn handler_precedence` / `fn browser_only_handlers` in rust/src/seed/handler_precedence.rs. */
function specializedHandlerNames() {
  const root = childrenNamed(parseLinoRoot(readText(HANDLER_PRECEDENCE_PATH)), 'handler_precedence')[0];
  const rows = childrenNamed(root, 'handler').map((row) => ({
    name: row.value,
    rank: Number(childValue(row, 'rank')),
    browserOnly: childValue(row, 'browser_only') === 'true',
  }));
  rows.sort((left, right) => (left.rank - right.rank) || (left.name < right.name ? -1 : left.name > right.name ? 1 : 0));
  return rows.filter((row) => !row.browserOnly).map((row) => row.name);
}

/** Mirrors `fn parse_method_execution` in rust/src/method_registry.rs. */
function parseMethodExecution() {
  const root = childrenNamed(parseLinoRoot(readText(METHOD_EXECUTION_PATH)), 'method_execution')[0];
  const attributes = new Map();
  for (const method of childrenNamed(root, 'method')) {
    const execution = { runtime: null, response_language: null, project_lookup: null, definition_fusion: false };
    for (const attribute of method.children || []) {
      if (attribute.name === 'definition_fusion') execution.definition_fusion = attribute.value === 'true';
      else if (attribute.name in execution) execution[attribute.name] = attribute.value;
    }
    attributes.set(method.value, execution);
  }
  return attributes;
}

/** Mirrors `fn parse_steps` in rust/src/recipe_interpreter.rs: `{order, records}` per `meta_step`, by order. */
function recipeSteps() {
  const steps = [];
  let block = [];
  const field = (key) => {
    for (const raw of block) {
      const line = raw.trim();
      if (!line.startsWith(key)) continue;
      const rest = line.slice(key.length);
      if (!/^\s/u.test(rest)) continue;
      const value = rest.trim();
      return value.length >= 2 && value.startsWith('"') && value.endsWith('"') ? value.slice(1, -1) : value;
    }
    return null;
  };
  const flush = () => {
    if (field('record_type') === 'meta_step') {
      const order = Number.parseInt(field('order') ?? '', 10);
      if (Number.isInteger(order) && field('id') !== null && field('title') !== null && field('source_file') !== null) {
        steps.push({ order, records: field('records') });
      }
    }
    block = [];
  };
  for (const line of readText(RECIPE_PATH).split('\n')) {
    if (!line.trim()) continue;
    if (!/^\s/u.test(line)) flush();
    block.push(line);
  }
  flush();
  return steps.sort((left, right) => left.order - right.order);
}

/** Mirrors `fn step_emits` in rust/src/method_registry.rs. */
function stepEmits(step, base) {
  if (step.records === null) return false;
  const kind = step.records.startsWith('record_') ? step.records.slice('record_'.length) : step.records;
  return kind === base || kind.startsWith(`${base}_`);
}

/** Mirrors `LearnedMethod::to_recipe_program`: `{steps}` or `{error}`. */
function learnedRecipeProgram(method) {
  const recipe = recipeSteps();
  const orders = [];
  for (const operation of method.operations) {
    const base = operation.split(':')[0];
    const step = recipe.find((candidate) => stepEmits(candidate, base));
    if (!step) return { error: `unbound_operation:${operation}` };
    orders.push(step.order);
  }
  const last = orders.length ? Math.max(...orders) : 0;
  return { steps: recipe.filter((step) => step.records !== null && step.order <= last) };
}

/** Mirrors `fn parse_learned_methods` in rust/src/method_registry.rs (the shipped seed is valid). */
function parseLearnedMethods() {
  const values = (node, name) => childrenNamed(node, name).map((child) => child.value);
  return childrenNamed(parseLinoRoot(readText(LEARNED_METHODS_PATH)), 'learned_method').map((node) => ({
    name: node.value,
    status: childValue(node, 'status') === 'adopted_not_effective' ? 'adopted_not_effective' : 'adopted',
    algorithm_id: childValue(node, 'algorithm_id'),
    evidence_id: childValue(node, 'evidence_id'),
    operations: values(node, 'operation'),
  }));
}

/** Mirrors `fn catalog_from` in rust/src/selection_heuristics.rs (the shipped catalog). */
function shippedHeuristics() {
  return (parseLinoRoot(readText(HEURISTICS_PATH)).children || [])
    .filter((record) => childValue(record, 'record_type') === HEURISTIC_RECORD_TYPE)
    .map((record) => ({
      name: record.name,
      role: childValue(record, 'role'),
      order: Number(childValue(record, 'order')),
      applies_when: childrenNamed(record, 'applies_when').map((field) => field.value),
      parameters: (record.children || []).filter((field) => !HEURISTIC_STRUCTURAL_FIELDS.has(field.name))
        .map((field) => [field.name, field.value]),
    }));
}

/** Mirrors `fn load_aliases` in rust/src/route_method_alias.rs. */
function routeMethodAliases() {
  return (parseLinoRoot(readText(ALIASES_PATH)).children || [])
    .filter((record) => childValue(record, 'record_type') === 'route_method_alias')
    .map((record) => ({ route: childValue(record, 'route'), method: childValue(record, 'method') }))
    .filter((alias) => alias.route && alias.method);
}

/** Mirrors `MethodRegistry::shared` (`from_store_with_learned_seed`), built once per host. */
export function methodRegistry() {
  return cached('method-registry', () => {
    const execution = parseMethodExecution();
    const executionFor = (name) => execution.get(name)
      ?? { runtime: null, response_language: null, project_lookup: null, definition_fusion: false };
    const surface = (names, slug) => names.map((name, order) => ({ name, order, surface: slug, execution: executionFor(name) }));
    return {
      methods: [
        ...surface(PRELUDE_METHOD_NAMES, 'prelude'),
        ...surface(specializedHandlerNames(), 'specialized'),
        ...surface(CONTEXTUAL_HANDLER_NAMES, 'contextual'),
      ],
      learned_methods: parseLearnedMethods(),
      heuristics: shippedHeuristics(),
      aliases: routeMethodAliases(),
    };
  });
}

/** Mirrors `MethodRegistry::method_for_route`. @returns {object|null} */
export function methodForRoute(registry, route) {
  const direct = registry.methods.find((method) => method.name === route);
  if (direct) return direct;
  const alias = registry.aliases.find((entry) => entry.route === route);
  return alias ? registry.methods.find((method) => method.name === alias.method) ?? null : null;
}

/** Mirrors `MethodRegistry::explicit_learned_method_relevants`. */
export function explicitLearnedMethodRelevants(registry, prompt) {
  const tokens = String(prompt).split(/[^\p{Alphabetic}\p{N}:_-]/u).filter(Boolean);
  return registry.learned_methods.map((method) => `method:${method.name}`).filter((reference) => tokens.includes(reference));
}

/** Mirrors `MethodRegistry::heuristics_for`: the heuristics for `role` that apply in `situation`, by order. */
export function heuristicsFor(registry, role, situation) {
  return registry.heuristics
    .filter((heuristic) => heuristic.role === role && (!heuristic.applies_when.length || heuristic.applies_when.includes(situation)))
    .sort((left, right) => left.order - right.order);
}

/** Mirrors `Method::to_links_notation`. */
function methodLinksNotation(method) {
  const pairs = [['record_type', 'method'], ['name', method.name], ['order', String(method.order)], ['surface', method.surface]];
  if (method.execution.runtime) pairs.push(['runtime', method.execution.runtime]);
  if (method.execution.response_language) pairs.push(['response_language', method.execution.response_language]);
  if (method.execution.project_lookup) pairs.push(['project_lookup', method.execution.project_lookup]);
  if (method.execution.definition_fusion) pairs.push(['definition_fusion', 'true']);
  return formatLinoRecord(method.name, pairs);
}

/** Mirrors `LearnedMethod::to_links_notation`. */
function learnedMethodLinksNotation(method) {
  const pairs = [
    ['record_type', 'learned_method'], ['name', method.name], ['status', method.status],
    ['algorithm_id', method.algorithm_id], ['evidence_id', method.evidence_id],
  ];
  for (const operation of method.operations) pairs.push(['operation', operation]);
  pairs.push(['trace_event', `method:learned:${method.name}`]);
  const program = learnedRecipeProgram(method);
  if (program.error) pairs.push(['unbound_operation', program.error]);
  else pairs.push(['bound_steps', String(program.steps.length)]);
  return formatLinoRecord(method.name, pairs);
}

/** Mirrors `HeuristicMethod::to_links_notation` in rust/src/selection_heuristics.rs. */
function heuristicLinksNotation(heuristic) {
  const pairs = [['record_type', HEURISTIC_RECORD_TYPE], ['role', heuristic.role], ['order', String(heuristic.order)]];
  for (const slug of heuristic.applies_when) pairs.push(['applies_when', slug]);
  for (const [key, value] of heuristic.parameters) pairs.push([key, value]);
  return formatLinoRecord(heuristic.name, pairs);
}

/** Mirrors `MethodRegistry::to_links_notation`. */
export function methodRegistryLinksNotation(registry) {
  const countOn = (surface) => String(registry.methods.filter((method) => method.surface === surface).length);
  const pairs = [
    ['record_type', 'method_registry'],
    ['method_count', String(registry.methods.length)],
    ['prelude_count', countOn('prelude')],
    ['specialized_count', countOn('specialized')],
    ['contextual_count', countOn('contextual')],
    ['learned_count', String(registry.learned_methods.length)],
    ['heuristic_count', String(registry.heuristics.length)],
  ];
  for (const method of registry.methods) pairs.push(['method', method.name]);
  for (const heuristic of registry.heuristics) pairs.push(['heuristic', heuristic.name]);
  let out = formatLinoRecord('method_registry', pairs);
  for (const method of registry.methods) out += `\n${methodLinksNotation(method)}`;
  for (const method of registry.learned_methods) out += `\n${learnedMethodLinksNotation(method)}`;
  for (const heuristic of registry.heuristics) out += `\n${heuristicLinksNotation(heuristic)}`;
  return out;
}

/** Mirrors `fn record_method_registry` in rust/src/method_registry.rs. */
export function recordMethodRegistry(log) {
  const registry = methodRegistry();
  log.push({ kind: 'method_registry', payload: methodRegistryLinksNotation(registry) });
  log.push({ kind: 'method_registry:count', payload: String(registry.methods.length) });
  return registry;
}
