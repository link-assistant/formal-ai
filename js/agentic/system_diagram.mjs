// The generated system diagrams (issue #538, R382): a high-level overview
// and one view per entry point, rendered as split mermaid parts from live
// data only. The solver handler registry comes from the seed's precedence
// and promotion tables, the CLI subcommands from the clap `Command` enum in
// rust/src/main.rs, and the HTTP routes from data/meta/server-routes.lino
// (the manifest both servers are held to). The wording and the overview's
// edges live in data/meta/system-diagrams.lino; every edge naming a
// subcommand or a route is checked against the live data.
//
// Rust twin: rust/src/agentic_coding/system_diagram.rs. Driver:
// scripts/generate-system-diagrams.mjs (--write, --check).

import { childValue, childrenNamed, parseLino } from './host.mjs';

/** Mirrors `const CONFIG_PATH`. */
export const CONFIG_PATH = 'data/meta/system-diagrams.lino';

/** Mirrors `const DIAGRAM_DIR`. */
export const DIAGRAM_DIR = 'docs/diagrams';

/** Mirrors `const BROWSER_CLASS`: the mermaid class of `browser_only` rows. */
const BROWSER_CLASS = 'browserOnly';

/** Mirrors `const BROWSER_STYLE`: a dashed outline. */
const BROWSER_STYLE = 'stroke-dasharray: 5 5';

const FENCE = '```';

/** Mirrors `fn fill`: `{name}` placeholders filled from `params`. */
function fill(template, params = {}) {
  return template.replace(/\{([a-z_]+)\}/g, (whole, name) =>
    Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : whole,
  );
}

/** Mirrors `fn config_error`. */
function configError(...parts) {
  return new Error(['system_diagrams', ...parts].join(':'));
}

/** Mirrors `fn cell`: a Markdown table cell, `|` escaped. */
function cell(text) {
  return text.replace(/\|/g, '\\|');
}

/** Mirrors `fn table`. */
function table(headers, rows) {
  const line = (cells) => `| ${cells.join(' | ')} |`;
  return [line(headers), line(headers.map(() => '---')), ...rows.map((row) => line(row.map(cell)))].join('\n');
}

/** Mirrors `fn mermaid`. */
function mermaid(lines) {
  return [`${FENCE}mermaid`, ...lines, FENCE].join('\n');
}

/**
 * Mirrors `fn precedence_rows`: the `handler` rows of a precedence document
 * in dispatch order (ascending rank, then name), as the seed loader orders them.
 * @param {string} text
 * @returns {Array<{name: string, rank: number, browserOnly: boolean}>}
 */
export function precedenceRows(text) {
  return childrenNamed(parseLino(text), 'handler')
    .map((node) => ({
      name: node.value,
      rank: Number.parseInt(childValue(node, 'rank'), 10),
      browserOnly: childValue(node, 'browser-only') === 'true',
    }))
    .sort((left, right) => left.rank - right.rank || (left.name < right.name ? -1 : left.name > right.name ? 1 : 0));
}

/**
 * Mirrors `fn promotion_rows`: the promotion rows in rank order (stable).
 * @param {string} text
 * @returns {Array<{handler: string, rank: number, because: string}>}
 */
export function promotionRows(text) {
  return childrenNamed(parseLino(text), 'promotion')
    .map((node) => ({
      handler: node.value,
      rank: Number.parseInt(childValue(node, 'rank'), 10),
      because: childValue(node, 'because').trim(),
    }))
    .sort((left, right) => left.rank - right.rank);
}

/** Mirrors `fn kebab_case`: a clap variant name as its subcommand name. */
function kebabCase(identifier) {
  let out = '';
  [...identifier].forEach((character, index) => {
    const lower = character.toLowerCase();
    if (index > 0 && lower !== character) out += '-';
    out += lower;
  });
  return out;
}

/** Mirrors `fn summary`: the first sentence of a doc comment's first paragraph. */
function summary(doc) {
  const blank = doc.indexOf('');
  const paragraph = (blank < 0 ? doc : doc.slice(0, blank)).join(' ');
  const stop = paragraph.indexOf('. ');
  return (stop < 0 ? paragraph : paragraph.slice(0, stop + 1)).trim();
}

/**
 * Mirrors `fn cli_definition`: the binary name and the subcommands of the clap
 * `Command` enum, in declaration order, each with its doc summary.
 * @param {string} source rust/src/main.rs
 * @returns {{binary: string, subcommands: Array<{name: string, summary: string}>}}
 */
export function cliDefinition(source) {
  const lines = source.split('\n');
  const binaryLine = lines.find((line) => line.trim().startsWith('name = "'));
  const binary = binaryLine ? binaryLine.trim().split('"')[1] : '';
  const start = lines.findIndex((line) => line.trim() === 'enum Command {');
  const subcommands = [];
  let doc = [];
  for (const line of start < 0 ? [] : lines.slice(start + 1)) {
    if (line === '}') break;
    if (line.startsWith('    ///')) {
      doc.push(line.slice(7).trim());
    } else if (line.startsWith('    ') && /^[A-Z]$/.test(line.charAt(4))) {
      const identifier = line.slice(4).match(/^[A-Za-z0-9]+/)[0];
      subcommands.push({ name: kebabCase(identifier), summary: summary(doc) });
      doc = [];
    }
  }
  return { binary, subcommands };
}

/**
 * Mirrors `fn route_rows`: the route manifest's rows in manifest order.
 * @param {string} text
 * @returns {Array<{id: string, method: string, paths: Array<string>, auth: string, deprecated: boolean}>}
 */
export function routeRows(text) {
  return childrenNamed(parseLino(text), 'route').map((node) => ({
    id: node.value,
    method: childValue(node, 'method'),
    paths: childrenNamed(node, 'path').map((path) => path.value),
    auth: childValue(node, 'auth') || 'bearer',
    deprecated: childValue(node, 'deprecated') === 'true',
  }));
}

/** Mirrors `struct Config`: the parsed data/meta/system-diagrams.lino. */
function readConfig(text) {
  const root = parseLino(text);
  const messages = new Map(childrenNamed(root, 'message').map((node) => [node.value, childValue(node, 'text')]));
  return {
    chunk: Number.parseInt(childValue(root, 'chunk'), 10),
    sources: childrenNamed(root, 'source').map((node) => ({ key: node.value, path: childValue(node, 'path') })),
    parts: childrenNamed(root, 'part').map((node) => ({
      key: node.value,
      file: childValue(node, 'file'),
      uses: childrenNamed(node, 'uses').map((use) => use.value),
    })),
    nodes: childrenNamed(root, 'node').map((node) => ({
      id: node.value,
      kind: childValue(node, 'kind'),
      label: childValue(node, 'label'),
      detail: childValue(node, 'detail'),
      count: childValue(node, 'count'),
    })),
    edges: childrenNamed(root, 'edge').map((node) => ({
      id: node.value,
      from: childValue(node, 'from'),
      to: childValue(node, 'to'),
      subcommand: childValue(node, 'subcommand'),
      route: childValue(node, 'route'),
    })),
    message(key, params) {
      if (!messages.has(key)) throw configError('missing_message', key);
      return fill(messages.get(key), params);
    },
  };
}

/** Mirrors `fn document`: the generated header, then the blocks. */
function document(config, part, titleKey, blocks) {
  const sources = part.uses.map((key) => {
    const source = config.sources.find((candidate) => candidate.key === key);
    if (!source) throw configError('part', part.key, 'unknown_source', key);
    return source.path;
  });
  const header = [config.message(titleKey), config.message('document_comment', { sources: sources.join(', ') })];
  return `${[...header, ...blocks].join('\n\n')}\n`;
}

/** Mirrors `fn count_of`. */
function countOf(data, kind) {
  switch (kind) {
    case 'cli_subcommands': return data.cli.subcommands.length;
    case 'routes': return data.routes.length;
    case 'browser_handlers': return data.handlers.filter((row) => row.browserOnly).length;
    case 'promotions': return data.promotions.length;
    case 'handlers': return data.handlers.length;
    default: throw configError('unknown_count', kind);
  }
}

/** Mirrors `fn edge_label`: the live subcommand or route an edge names, or ''. */
function edgeLabel(data, edge) {
  if (edge.subcommand) {
    if (!data.cli.subcommands.some((sub) => sub.name === edge.subcommand)) {
      throw configError('edge', edge.id, 'unknown_subcommand', edge.subcommand);
    }
    return `${data.cli.binary} ${edge.subcommand}`;
  }
  if (edge.route) {
    const route = data.routes.find((row) => row.id === edge.route);
    if (!route) throw configError('edge', edge.id, 'unknown_route', edge.route);
    return `${route.method} ${route.paths[0]}`;
  }
  return '';
}

/** Mirrors `fn render_overview`. */
function renderOverview(config, data, part) {
  const links = config.parts
    .filter((other) => other.key !== part.key)
    .map((other) => config.message('overview_part_link', {
      title: config.message(`${other.key}_title`).replace(/^#+/, '').trim(),
      file: other.file,
    }));
  const lines = ['flowchart LR'];
  for (const [kind, group] of [['entry', 'entries'], ['layer', 'layers']]) {
    lines.push(`    subgraph ${group}["${config.message(`overview_${group}`)}"]`);
    for (const node of config.nodes.filter((candidate) => candidate.kind === kind)) {
      const detail = node.detail ? `<br/>${fill(node.detail, { count: node.count ? countOf(data, node.count) : '' })}` : '';
      lines.push(`        ${node.id}["${node.label}${detail}"]`);
    }
    lines.push('    end');
  }
  for (const edge of config.edges) {
    for (const end of [edge.from, edge.to]) {
      if (!config.nodes.some((node) => node.id === end)) throw configError('edge', edge.id, 'unknown_node', end);
    }
    const label = edgeLabel(data, edge);
    lines.push(label ? `    ${edge.from} -->|"${label}"| ${edge.to}` : `    ${edge.from} --> ${edge.to}`);
  }
  return document(config, part, 'overview_title', [
    [config.message('overview_intro'), ...links, config.message('overview_recipes_link')].join('\n'),
    config.message('overview_heading'),
    mermaid(lines),
  ]);
}

/** Mirrors `fn handler_label`. */
const handlerLabel = (row) => `${row.rank}: ${row.name}`;

/** Mirrors `fn render_handlers`. */
function renderHandlers(config, data, part) {
  const promotionLines = ['flowchart LR', `    prompt(["${config.message('handlers_prompt')}"])`];
  let previous = 'prompt';
  data.promotions.forEach((row, index) => {
    const node = `p_${index + 1}`;
    promotionLines.push(`    ${previous} --> ${node}["${row.rank}: ${row.handler}"]`);
    previous = node;
  });
  promotionLines.push(`    ${previous} --> precedence[["${config.message('handlers_precedence_node')}"]]`);
  const blocks = [
    config.message('handlers_intro'),
    config.message('handlers_promotions_heading'),
    mermaid(promotionLines),
    table(
      [config.message('column_rank'), config.message('column_handler'), config.message('column_because')],
      data.promotions.map((row) => [String(row.rank), `\`${row.handler}\``, row.because]),
    ),
  ];
  for (let start = 0, partNumber = 2; start < data.handlers.length; start += config.chunk, partNumber += 1) {
    const chunk = data.handlers.slice(start, start + config.chunk);
    const first = chunk[0];
    const last = chunk[chunk.length - 1];
    const lines = ['flowchart TD'];
    let prior = '';
    if (start > 0) {
      prior = `before_${partNumber}`;
      lines.push(`    ${prior}(["${config.message('handlers_previous', { rank: first.rank })}"])`);
    }
    for (const row of chunk) {
      const node = `h_${row.name}`;
      lines.push(prior ? `    ${prior} --> ${node}["${handlerLabel(row)}"]` : `    ${node}["${handlerLabel(row)}"]`);
      prior = node;
    }
    if (start + config.chunk < data.handlers.length) {
      lines.push(`    ${prior} --> after_${partNumber}(["${config.message('handlers_next', { rank: last.rank })}"])`);
    }
    const browser = chunk.filter((row) => row.browserOnly).map((row) => `h_${row.name}`);
    if (browser.length) {
      lines.push(`    classDef ${BROWSER_CLASS} ${BROWSER_STYLE}`);
      lines.push(`    class ${browser.join(',')} ${BROWSER_CLASS}`);
    }
    blocks.push(
      config.message('handlers_precedence_heading', { part: partNumber, first: first.rank, last: last.rank }),
      mermaid(lines),
      table(
        [config.message('column_rank'), config.message('column_handler'), config.message('column_surface')],
        chunk.map((row) => [
          String(row.rank),
          `\`${row.name}\``,
          config.message(row.browserOnly ? 'surface_browser' : 'surface_native'),
        ]),
      ),
    );
  }
  return document(config, part, 'handlers_title', blocks);
}

/** Mirrors `fn cli_node`. */
const cliNode = (name) => `c_${name.replace(/-/g, '_')}`;

/** Mirrors `fn render_cli`. */
function renderCli(config, data, part) {
  const lines = ['flowchart LR', `    cli(["${data.cli.binary}"])`];
  for (const sub of data.cli.subcommands) lines.push(`    cli --> ${cliNode(sub.name)}["${sub.name}"]`);
  return document(config, part, 'cli_title', [
    config.message('cli_intro'),
    config.message('cli_heading'),
    mermaid(lines),
    table(
      [config.message('column_subcommand'), config.message('column_summary')],
      data.cli.subcommands.map((sub) => [`\`${data.cli.binary} ${sub.name}\``, sub.summary]),
    ),
  ]);
}

/** Mirrors `fn render_routes`. */
function renderRoutes(config, data, part) {
  const lines = ['flowchart LR', `    server(["${config.message('routes_server')}"])`];
  const methods = [...new Set(data.routes.map((row) => row.method))];
  for (const method of methods) {
    lines.push(`    subgraph m_${method}["${method}"]`);
    for (const row of data.routes.filter((candidate) => candidate.method === method)) {
      const name = row.deprecated ? config.message('routes_deprecated', { route: row.id }) : row.id;
      lines.push(`        r_${row.id}["${[name, ...row.paths].join('<br/>')}"]`);
    }
    lines.push('    end');
  }
  for (const row of data.routes) {
    lines.push(row.auth === 'none' ? `    server -.->|"${row.auth}"| r_${row.id}` : `    server -->|"${row.auth}"| r_${row.id}`);
  }
  return document(config, part, 'routes_title', [
    config.message('routes_intro'),
    config.message('routes_heading'),
    mermaid(lines),
    table(
      [config.message('column_route'), config.message('column_method'), config.message('column_paths'), config.message('column_auth')],
      data.routes.map((row) => [`\`${row.id}\``, row.method, row.paths.map((path) => `\`${path}\``).join(', '), row.auth]),
    ),
  ]);
}

const RENDERERS = { overview: renderOverview, handlers: renderHandlers, cli: renderCli, routes: renderRoutes };

/**
 * Mirrors `fn render_system_diagrams`: every generated part as
 * `[file name under docs/diagrams, document]`, in config order.
 * @param {(path: string) => string} read a repository file as text
 * @returns {Array<[string, string]>}
 */
export function renderSystemDiagrams(read) {
  const config = readConfig(read(CONFIG_PATH));
  const sourceText = (key) => {
    const source = config.sources.find((candidate) => candidate.key === key);
    if (!source) throw configError('missing_source', key);
    return read(source.path);
  };
  const data = {
    handlers: precedenceRows(sourceText('handler_precedence')),
    promotions: promotionRows(sourceText('handler_promotions')),
    cli: cliDefinition(sourceText('cli')),
    routes: routeRows(sourceText('routes')),
  };
  return config.parts.map((part) => {
    const render = RENDERERS[part.key];
    if (!render) throw configError('unknown_part', part.key);
    return [part.file, render(config, data, part)];
  });
}
