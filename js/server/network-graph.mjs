// The links-network projection of the knowledge store: rust/src/engine.rs
// `knowledge_graph`, `knowledge_graph_dot`, `is_known_trace_id`,
// rust/src/associative_package.rs `default_package_graph_projection` /
// `AssociativePackage::links_notation`, and rust/src/links_query.rs
// `KnowledgeGraph::to_links_notation`.
//
// The node labels, package records and permission descriptions are data
// (data/meta/server-knowledge-graph.lino); rule answers come from the seed
// responses, the program catalog from the coding meanings.

import { readdirSync } from 'node:fs';
import path from 'node:path';

import { stableId } from './ids.mjs';
import { REPO_ROOT, childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';

export const GRAPH_FILE = 'data/meta/server-knowledge-graph.lino';
const ROOT_ID = 'formal_ai_knowledge';
const RESPONSES_MAIN = 'multilingual-responses.lino';
const RESPONSES_PREFIX = 'multilingual-responses-';

/** The seed response files in `RESPONSE_FILES` order (main file, then the rest sorted). */
function responseFiles() {
  const rest = readdirSync(path.join(REPO_ROOT, 'data/seed'))
    .filter((name) => name.startsWith(RESPONSES_PREFIX) && name.endsWith('.lino'))
    .sort((left, right) => {
      const a = left.slice(0, -'.lino'.length);
      const b = right.slice(0, -'.lino'.length);
      return a < b ? -1 : a > b ? 1 : 0;
    });
  return [RESPONSES_MAIN, ...rest].map((name) => `data/seed/${name}`);
}

let responseTable = null;

/** `seed::response_for(intent, language)`: the first matching record's text. */
export function responseFor(intent, language) {
  if (!responseTable) {
    responseTable = new Map();
    for (const file of responseFiles()) {
      for (const entry of childrenNamed(parseLino(readRepoFile(file)), 'response')) {
        const key = `${childValue(entry, 'intent')}\u0000${childValue(entry, 'language')}`;
        if (!childValue(entry, 'intent') || !childValue(entry, 'language')) continue;
        if (!responseTable.has(key)) responseTable.set(key, childValue(entry, 'text'));
      }
    }
  }
  const key = `${intent}\u0000${language}`;
  return responseTable.has(key) ? responseTable.get(key) : null;
}

/** `cached_response(intent, "en")`: the seed text, degrading to the intent slug. */
function englishAnswer(intent) {
  return responseFor(intent, 'en') ?? intent;
}

/** The program catalog slugs, in `PROGRAM_LANGUAGES` / `PROGRAM_TASKS` order. */
function programCatalog() {
  const root = parseLino(readRepoFile('data/seed/meanings-coding-catalog.lino'));
  const slugs = (prefix) => root.children
    .map((child) => child.name)
    .filter((name) => name.startsWith(prefix))
    .map((name) => name.slice(prefix.length));
  return {
    languages: slugs('program_language_').join(', '),
    tasks: slugs('program_task_').join(', '),
  };
}

let routing = null;

/** `seed::intent_routing()`: intent slugs and trace prefixes. */
function intentRouting() {
  if (routing) return routing;
  const root = parseLino(readRepoFile('data/seed/intent-routing.lino'));
  routing = {
    slugs: childrenNamed(root, 'intent').map((node) => childValue(node, 'slug')).filter(Boolean),
    tracePrefixes: childrenNamed(root, 'trace_prefix').map((node) => node.value),
  };
  return routing;
}

/** `is_known_trace_id`. @param {string} trace @returns {boolean} */
export function isKnownTraceId(trace) {
  const { slugs, tracePrefixes } = intentRouting();
  if (tracePrefixes.some((prefix) => trace.startsWith(prefix))) return true;
  const lower = asciiLower(trace);
  return slugs.some((slug) => lower === asciiLower(slug) || trace.includes(slug));
}

function asciiLower(text) {
  return text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());
}

/** `engine::normalize_prompt`. @param {string} prompt @returns {string} */
export function normalizePrompt(prompt) {
  const canonical = String(prompt).toLowerCase().split('c++').join(' cpp ').split('c#').join(' csharp ');
  let normalized = '';
  for (const character of canonical) {
    normalized += /[\p{Alphabetic}\p{N}\p{M}]/u.test(character) ? character : ' ';
  }
  return normalized.split(/\s+/u).filter(Boolean).join(' ');
}

/** `links_format::format_lino_value`: sanitize, then the codec's indented quoting. */
export function formatLinoValue(value) {
  const flat = String(value)
    .replace(/\\/g, '\\\\')
    .replace(/\r/g, '\\r')
    .replace(/\n/g, '\\n')
    .replace(/\t/g, '\\t');
  const hasSingle = flat.includes("'");
  const hasDouble = flat.includes('"');
  if (hasDouble && !hasSingle) return `'${flat}'`;
  if (hasSingle && !hasDouble) return `"${flat}"`;
  if (hasSingle && hasDouble) return `'${flat.replace(/'/g, "''")}'`;
  return `"${flat}"`;
}

function pushLinoNode(lines, indent, name, value) {
  lines.push(`${' '.repeat(indent)}${name}${value === null ? '' : ` ${formatLinoValue(value)}`}`);
}

function fillLabel(template, params) {
  return template.replace(/\{([a-z_]+)\}/g, (whole, name) => (name in params ? String(params[name]) : whole));
}

let graphData = null;

/** The parsed data/meta/server-knowledge-graph.lino. */
function knowledgeGraphData() {
  if (graphData) return graphData;
  const root = parseLino(readRepoFile(GRAPH_FILE));
  const schemaVersion = childValue(root, 'knowledge_schema_version');
  const packages = childrenNamed(root, 'package').map((node) => {
    const pkg = {
      id: node.value,
      name: childValue(node, 'name'),
      version: schemaVersion,
      handlers: childrenNamed(node, 'handler').map((handler) => ({
        id: handler.value,
        kind: childValue(handler, 'kind'),
        capability: childValue(handler, 'capability'),
        response: childValue(handler, 'response'),
      })),
      triggers: childrenNamed(node, 'trigger').map((trigger) => ({
        id: trigger.value,
        kind: childValue(trigger, 'kind'),
        match_prompt: childValue(trigger, 'match_prompt'),
        normalized_match: normalizePrompt(childValue(trigger, 'match_prompt')),
        handler_id: childValue(trigger, 'handler'),
      })),
      permissions: [],
    };
    pkg.permissions = childrenNamed(node, 'permission').map((permission) => {
      const capability = permission.value;
      const description = childValue(permission, 'description');
      return {
        id: stableId('package_permission', `${pkg.id}:${capability}:${description}`),
        capability,
        effect: 'allow',
        description,
      };
    });
    return pkg;
  });
  graphData = {
    rootLabel: childValue(childrenNamed(root, 'root')[0], 'label'),
    rules: childrenNamed(root, 'rule').map((node) => ({
      id: node.value,
      label: childValue(node, 'label'),
      answer: childValue(node, 'answer'),
      responseLink: childValue(node, 'response_link'),
    })),
    writeProgram: childrenNamed(root, 'write_program').map((node) => ({
      id: node.value,
      label: childValue(node, 'label'),
      responseLink: childValue(node, 'response_link'),
    }))[0],
    schemaVersion,
    packageSource: childValue(root, 'package_source'),
    handlerLabel: childValue(root, 'handler_label'),
    triggerLabel: childValue(root, 'trigger_label'),
    permissionLabel: childValue(root, 'permission_label'),
    packages,
  };
  return graphData;
}

/** `AssociativePackage::links_notation`. */
function packageLinksNotation(pkg, data) {
  const lines = [];
  pushLinoNode(lines, 0, pkg.id, null);
  pushLinoNode(lines, 2, 'type', 'associative_package');
  pushLinoNode(lines, 2, 'schema_version', data.schemaVersion);
  pushLinoNode(lines, 2, 'name', pkg.name);
  pushLinoNode(lines, 2, 'version', pkg.version);
  pushLinoNode(lines, 2, 'source', data.packageSource);
  for (const handler of pkg.handlers) {
    pushLinoNode(lines, 2, 'handler', handler.id);
    pushLinoNode(lines, 4, 'kind', handler.kind);
    pushLinoNode(lines, 4, 'capability', handler.capability);
    pushLinoNode(lines, 4, 'response', handler.response);
  }
  for (const trigger of pkg.triggers) {
    pushLinoNode(lines, 2, 'trigger', trigger.id);
    pushLinoNode(lines, 4, 'kind', trigger.kind);
    pushLinoNode(lines, 4, 'match_prompt', trigger.match_prompt);
    pushLinoNode(lines, 4, 'normalized_match', trigger.normalized_match);
    pushLinoNode(lines, 4, 'handler', trigger.handler_id);
  }
  for (const permission of pkg.permissions) {
    pushLinoNode(lines, 2, 'permission', permission.id);
    pushLinoNode(lines, 4, 'effect', permission.effect);
    pushLinoNode(lines, 4, 'capability', permission.capability);
    pushLinoNode(lines, 4, 'description', permission.description);
  }
  return `${lines.join('\n')}\n`.trimEnd();
}

const node = (id, label, linksNotation) => ({ id, label, links_notation: linksNotation });
const edge = (from, to, role) => ({ from, to, role });

/** `default_package_graph_projection`. */
function packageProjection(data) {
  const nodes = [];
  const edges = [];
  for (const pkg of data.packages) {
    nodes.push(node(pkg.id, pkg.name, packageLinksNotation(pkg, data)));
    edges.push(edge(ROOT_ID, pkg.id, 'package'));
    for (const handler of pkg.handlers) {
      nodes.push(node(handler.id, fillLabel(data.handlerLabel, handler), `${handler.id} kind=${handler.kind}`));
      edges.push(edge(pkg.id, handler.id, 'package_handler'));
    }
    for (const trigger of pkg.triggers) {
      nodes.push(node(trigger.id, fillLabel(data.triggerLabel, trigger), `${trigger.id} handler=${trigger.handler_id}`));
      edges.push(edge(pkg.id, trigger.id, 'package_trigger'));
      edges.push(edge(trigger.id, trigger.handler_id, 'trigger_handler'));
    }
    for (const permission of pkg.permissions) {
      nodes.push(node(permission.id, fillLabel(data.permissionLabel, permission), `${permission.id} effect=${permission.effect}`));
      edges.push(edge(pkg.id, permission.id, 'package_permission'));
    }
  }
  return { nodes, edges };
}

let graphCache = null;

/**
 * `knowledge_graph()`: `{nodes: [{id, label, links_notation}], edges: [{from, to, role}]}`.
 * @returns {{nodes: Array<object>, edges: Array<object>}}
 */
export function knowledgeGraph() {
  if (graphCache) return graphCache;
  const data = knowledgeGraphData();
  const nodes = [node(ROOT_ID, data.rootLabel, ROOT_ID)];
  for (const rule of data.rules) {
    nodes.push(node(rule.id, rule.label, `${rule.id} answer=${englishAnswer(rule.answer)}`));
  }
  const edges = data.rules.map((rule) => edge(ROOT_ID, rule.id, 'contains'));
  for (const rule of data.rules) {
    if (rule.responseLink) edges.push(edge(rule.id, rule.responseLink, 'response_link'));
  }
  const program = data.writeProgram;
  const catalog = programCatalog();
  nodes.push(node(
    program.id,
    program.label,
    `${program.id} parameters=language,task languages=${catalog.languages} tasks=${catalog.tasks}`,
  ));
  edges.push(edge(ROOT_ID, program.id, 'contains'));
  edges.push(edge(program.id, program.responseLink, 'response_link'));
  const projection = packageProjection(data);
  graphCache = { nodes: [...nodes, ...projection.nodes], edges: [...edges, ...projection.edges] };
  return graphCache;
}

/** `knowledge_graph_dot`. @returns {string} */
export function knowledgeGraphDot() {
  const graph = knowledgeGraph();
  let dot = `digraph ${ROOT_ID} {\n`;
  for (const item of graph.nodes) dot += `  "${item.id}" [label="${item.label}"];\n`;
  for (const item of graph.edges) dot += `  "${item.from}" -> "${item.to}" [label="${item.role}"];\n`;
  return `${dot}}\n`;
}

/** `links_query::escape`. */
export function escapeQuoted(value) {
  return String(value).replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

/** The `node` / `edge` blocks shared by `/v1/links` and LinksQL results. */
export function graphBlocks(nodes, edges) {
  let out = '';
  for (const item of nodes) {
    out += `  node "${escapeQuoted(item.id)}"\n    label "${escapeQuoted(item.label)}"\n`
      + `    links_notation "${escapeQuoted(item.links_notation)}"\n`;
  }
  for (const item of edges) {
    out += `  edge\n    from "${escapeQuoted(item.from)}"\n    to "${escapeQuoted(item.to)}"\n`
      + `    role "${escapeQuoted(item.role)}"\n`;
  }
  return out;
}

/** `KnowledgeGraph::to_links_notation` (`GET /v1/links`). @returns {string} */
export function knowledgeGraphLinksNotation() {
  const graph = knowledgeGraph();
  return `knowledge_graph\n${graphBlocks(graph.nodes, graph.edges)}`;
}
