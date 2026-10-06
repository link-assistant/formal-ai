// MCP over JSON-RPC 2.0 (`POST /mcp`): rust/src/mcp.rs `handle_mcp_request`,
// with the computer-use primitives it advertises and executes
// (rust/src/computer_use/mod.rs `mcp_tool_definitions`, `input_schema`,
// rust/src/computer_use/seed.rs `tool_description`, and the isolated
// workspace executor of rust/src/computer_use/executor.rs).
//
// Every reply is a compact `json!` value (sorted keys) with status 200.

import { createHash } from 'node:crypto';
import { appendFileSync, existsSync, mkdirSync, readFileSync, readdirSync, renameSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { sortedKeys, toCompactJson } from './json.mjs';
import { childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';
import { serverMessage } from './messages.mjs';
import { JSON_TYPE, rawResponse } from './response.mjs';
import { packageVersion } from './seed.mjs';
import { solveSymbolic } from './solve.mjs';

const PROTOCOL_VERSION = '2025-06-18';
const CHAT_TOOL = 'formal_ai_chat';
const ARCHIVE_FORMAT = 'formal-ai-archive-v1';
const FIXTURE_DIR = 'data/fixtures/computer-use';
const FIXTURES = new Map([
  ['fixture://orders.json', 'orders.json'],
  ['fixture://status.html', 'status.html'],
  ['fixture://form.html', 'form.html'],
  ['fixture://inventory.csv', 'inventory.csv'],
]);
const SUBMIT_URL = 'fixture://submit';
const SUBMIT_TOKEN = 'token=fixture-token';
const REJECTED_SUBMISSION = '{"accepted":false,"error":"invalid token"}';

// `COMPUTER_USE_PRIMITIVES`: [name, required arguments, own properties].
const STRING = { type: 'string' };
const BOOLEAN = { type: 'boolean' };
const PRIMITIVES = [
  ['fs.read', ['path'], { path: STRING }],
  ['fs.write', ['path', 'content', 'confirmed'], { path: STRING, content: STRING, confirmed: BOOLEAN }],
  ['fs.list', ['path'], { path: STRING }],
  ['fs.move', ['from', 'to', 'confirmed'], { from: STRING, to: STRING, confirmed: BOOLEAN }],
  ['shell.run', ['operation', 'input', 'output', 'confirmed'], {
    operation: { type: 'string', enum: ['count_lines', 'filter_csv', 'unique_csv'] },
    input: STRING,
    output: STRING,
    column: STRING,
    equals: STRING,
    confirmed: BOOLEAN,
  }],
  ['http.fetch', ['url', 'save_as'], { url: STRING, save_as: STRING }],
  ['http.post', ['url', 'body', 'save_as', 'confirmed'], { url: STRING, body: STRING, save_as: STRING, confirmed: BOOLEAN }],
  ['dom.query', ['source', 'selector', 'save_as'], { source: STRING, selector: STRING, save_as: STRING }],
  ['dom.extract', ['source', 'pointer', 'save_as'], { source: STRING, pointer: STRING, save_as: STRING }],
  ['archive.pack', ['paths', 'archive', 'confirmed'], {
    paths: { type: 'array', items: STRING },
    archive: STRING,
    confirmed: BOOLEAN,
  }],
  ['archive.unpack', ['archive', 'destination', 'confirmed'], { archive: STRING, destination: STRING, confirmed: BOOLEAN }],
  ['process.status', ['save_as'], { save_as: STRING }],
];
const PLAN_CONTEXT = ['plan_id', 'step_id', 'precondition', 'postcondition'];
const STATE_CHANGING = new Set(['fs.write', 'fs.move', 'shell.run', 'http.post', 'archive.pack', 'archive.unpack']);

const isObject = (value) => Boolean(value) && typeof value === 'object' && !Array.isArray(value);
const field = (value, key) => (isObject(value) ? value[key] : undefined);
const byteLength = (text) => Buffer.byteLength(text, 'utf8');
const digest = (bytes) => createHash('sha256').update(bytes).digest('hex');

// ---------------------------------------------------------------- JSON-RPC

function jsonRpcResponse(value) {
  return rawResponse(200, JSON_TYPE, toCompactJson(sortedKeys(value)));
}

function jsonRpcResult(id, result) {
  return jsonRpcResponse({ jsonrpc: '2.0', id, result });
}

function jsonRpcError(id, code, message) {
  return jsonRpcResponse({ jsonrpc: '2.0', id, error: { code, message } });
}

// ---------------------------------------------------------------- seed text

let responseTexts = null;

/** `mcp_text`: the English `response_for` record of multilingual-responses.lino. */
function mcpText(intent) {
  if (!responseTexts) {
    responseTexts = new Map();
    for (const record of childrenNamed(parseLino(readRepoFile('data/seed/multilingual-responses.lino')), 'response')) {
      const key = `${childValue(record, 'intent')}\u0000${childValue(record, 'language')}`;
      if (!responseTexts.has(key)) responseTexts.set(key, childValue(record, 'text'));
    }
  }
  return responseTexts.get(`${intent}\u0000en`) ?? '';
}

let toolLines = null;

/** `computer_use::seed::tool_description`: the `note` after `    name <tool>`. */
function toolDescription(name) {
  if (!toolLines) toolLines = readRepoFile('data/seed/tools.lino').split('\n').map((line) => line.replace(/\r$/, ''));
  const index = toolLines.indexOf(`    name ${name}`);
  const next = index >= 0 ? toolLines[index + 1] : undefined;
  const quoted = next?.trim().startsWith('note ') ? next.trim().slice(5) : null;
  if (quoted !== null) {
    try {
      const parsed = JSON.parse(quoted);
      if (typeof parsed === 'string') return parsed;
    } catch {
      // Unparseable notes fall back to the tool name, as natively.
    }
  }
  return name;
}

function inputSchema([, required, properties]) {
  const merged = { ...properties };
  for (const name of PLAN_CONTEXT) merged[name] = STRING;
  return {
    type: 'object',
    properties: merged,
    required: [...required, ...PLAN_CONTEXT],
    additionalProperties: false,
  };
}

/** `listed_tools`. */
function listedTools() {
  return [
    {
      name: CHAT_TOOL,
      description: mcpText('mcp_tool_description'),
      inputSchema: {
        type: 'object',
        properties: { prompt: { type: 'string', description: mcpText('mcp_prompt_description') } },
        required: ['prompt'],
        additionalProperties: false,
      },
    },
    ...PRIMITIVES.map((primitive) => ({
      name: primitive[0],
      description: toolDescription(primitive[0]),
      inputSchema: inputSchema(primitive),
    })),
  ];
}

/** `ComputerUsePrimitive::from_tool_name`. */
function primitiveFromToolName(name) {
  const lower = name.trim().toLowerCase();
  return PRIMITIVES.find(([dotted]) => {
    const underscored = dotted.replace(/\./g, '_');
    return lower === dotted
      || lower === underscored
      || lower.endsWith(`__${dotted}`)
      || lower.endsWith(`__${underscored}`)
      || lower.endsWith(`_${underscored}`);
  }) || null;
}

// ---------------------------------------------------------------- executor

/** Rust's `io::Error` display: `<strerror> (os error <n>)`. */
function ioError(error) {
  const code = Math.abs(Number(error?.errno));
  const raw = String(error?.message || error);
  const colon = raw.indexOf(': ');
  const comma = raw.indexOf(', ', colon + 2);
  const text = colon >= 0 ? raw.slice(colon + 2, comma > colon ? comma : undefined) : raw;
  const sentence = text.charAt(0).toUpperCase() + text.slice(1);
  return Number.isFinite(code) && code > 0 ? `${sentence} (os error ${code})` : sentence;
}

/** `arg`: a non-blank string argument, or the named validation error. */
function arg(args, key) {
  const value = field(args, key);
  if (typeof value !== 'string' || !value.trim()) throw `invalid_argument:${key}:expected_non_empty_string`;
  return value;
}

function argOrEmpty(args, key) {
  const value = field(args, key);
  return typeof value === 'string' ? value : '';
}

function stringArray(args, key) {
  const values = field(args, key);
  if (!Array.isArray(values)) throw `invalid_argument:${key}:expected_array`;
  const parsed = values.map((value) => {
    if (typeof value !== 'string' || !value) throw `invalid_argument:${key}:expected_string_entries`;
    return value;
  });
  if (!parsed.length) throw `invalid_argument:${key}:expected_non_empty_array`;
  return parsed;
}

/** `str::lines`: split on `\n`, drop a trailing `\r`, no final empty line. */
function lines(text) {
  if (!text) return [];
  const parts = text.split('\n');
  if (parts[parts.length - 1] === '') parts.pop();
  return parts.map((line) => line.replace(/\r$/, ''));
}

function decodeUtf8(bytes) {
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  } catch {
    return null;
  }
}

function decodeBase64(text) {
  if (text.length % 4 !== 0 || !/^[A-Za-z0-9+/]*={0,2}$/.test(text)) throw 'invalid base64';
  return Buffer.from(text, 'base64');
}

function fixture(url) {
  const file = FIXTURES.get(url);
  return file ? readRepoFile(`${FIXTURE_DIR}/${file}`).trim() : null;
}

function contentType(url) {
  const extension = path.extname(url).slice(1).toLowerCase();
  if (extension === 'html') return 'text/html';
  if (extension === 'csv') return 'text/csv';
  return 'application/json';
}

function httpOutput(method, url, status, body, cachePath) {
  const sha256 = digest(body);
  return {
    method,
    url,
    status,
    body,
    headers: { 'content-type': contentType(url) },
    sha256,
    cache_path: cachePath,
    cached: true,
    provenance: { method, url, status, sha256, cache_path: cachePath },
  };
}

function csvColumn(content, column) {
  const rows = lines(content);
  if (!rows.length) throw 'csv_header_missing';
  const index = rows[0].split(',').indexOf(column);
  if (index < 0) throw `csv_column_missing:${column}`;
  return { header: rows[0], rows: rows.slice(1), index };
}

function filterCsv(content, column, expected) {
  const { header, rows, index } = csvColumn(content, column);
  const selected = [header, ...rows.filter((row) => row.split(',')[index] === expected)];
  return `${selected.join('\n')}\n`;
}

function uniqueCsv(content, column) {
  const { rows, index } = csvColumn(content, column);
  const values = rows.map((row) => row.split(',')[index]).filter((value) => value !== undefined).sort();
  return `${values.filter((value, at) => at === 0 || value !== values[at - 1]).join('\n')}\n`;
}

function extractTagContents(html, tag) {
  const start = html.indexOf(`<${tag}`);
  if (start < 0) throw `selector_not_found:${tag}`;
  const gt = html.indexOf('>', start);
  if (gt < 0) throw `selector_not_found:${tag}`;
  const end = html.indexOf(`</${tag}>`, gt + 1);
  if (end < 0) throw `selector_not_found:${tag}`;
  return html.slice(gt + 1, end);
}

function stripTags(value) {
  let output = '';
  let inside = false;
  for (const character of value) {
    if (character === '<') inside = true;
    else if (character === '>') inside = false;
    else if (!inside) output += character;
  }
  return output;
}

function queryHtml(html, selector) {
  if (selector.startsWith('#')) {
    const id = selector.slice(1);
    for (const quote of ['"', "'"]) {
      const attribute = html.indexOf(`id=${quote}${id}${quote}`);
      if (attribute < 0) continue;
      const open = html.lastIndexOf('<', attribute - 1);
      if (open < 0) throw `selector_not_found:${selector}`;
      const tail = html.slice(open + 1).search(/[\s>]/u);
      if (tail < 0) throw `selector_not_found:${selector}`;
      const tag = html.slice(open + 1, open + 1 + tail);
      return [stripTags(extractTagContents(html.slice(open), tag)).trim()];
    }
    throw `selector_not_found:${selector}`;
  }
  if (selector.startsWith('.')) throw `selector_unsupported_in_fixture_parser:${selector}`;
  return [stripTags(extractTagContents(html, selector)).trim()];
}

/** `Value::pointer` (RFC 6901, serde_json's index rules). */
function jsonPointer(document, pointer) {
  if (pointer === '') return document;
  if (!pointer.startsWith('/')) return undefined;
  let target = document;
  for (const raw of pointer.split('/').slice(1)) {
    const token = raw.replace(/~1/g, '/').replace(/~0/g, '~');
    if (Array.isArray(target)) {
      if (!/^(0|[1-9][0-9]*)$/.test(token)) return undefined;
      target = target[Number(token)];
    } else if (isObject(target) && Object.prototype.hasOwnProperty.call(target, token)) {
      target = target[token];
    } else {
      return undefined;
    }
    if (target === undefined) return undefined;
  }
  return target;
}

function parseArchive(bytes) {
  const archive = JSON.parse(bytes.toString('utf8'));
  if (!isObject(archive) || typeof archive.format !== 'string' || !Array.isArray(archive.entries)
    || !archive.entries.every((entry) => isObject(entry) && typeof entry.path === 'string' && typeof entry.content_base64 === 'string')) {
    throw new Error('invalid type');
  }
  return archive;
}

let sessionSequence = 0;
const sessions = new Map();

/** `ComputerUseSession`: one plan's isolated workspace under the temp dir. */
class ComputerUseSession {
  constructor(planId, agentMode) {
    const sequence = sessionSequence;
    sessionSequence += 1;
    const safeId = [...planId].map((character) => (/^[A-Za-z0-9_-]$/.test(character) ? character : '-')).join('');
    this.planId = planId;
    this.agentMode = agentMode;
    this.root = path.join(os.tmpdir(), 'formal-ai-computer-use', `${safeId}-${process.pid}-${sequence}`);
    mkdirSync(this.root, { recursive: true });
  }

  path(value) {
    const relative = value.trim();
    const parts = relative.split('/');
    if (!relative || relative.startsWith('/') || parts[0] === '.' || parts.includes('..')) {
      throw `path_escapes_workspace:${value}`;
    }
    return path.join(this.root, relative);
  }

  read(relative, label) {
    try {
      return readFileSync(this.path(relative));
    } catch (error) {
      if (typeof error === 'string') throw error;
      throw `${label}:${relative}:${ioError(error)}`;
    }
  }

  writeBytes(relative, bytes) {
    const target = this.path(relative);
    try {
      mkdirSync(path.dirname(target), { recursive: true });
    } catch (error) {
      throw `create_directory_failed:${relative}:${ioError(error)}`;
    }
    try {
      writeFileSync(target, bytes);
    } catch (error) {
      throw `write_failed:${relative}:${ioError(error)}`;
    }
  }

  fileDigest(relative) {
    try {
      return digest(readFileSync(this.path(relative)));
    } catch {
      return null;
    }
  }

  directoryEntries(relative) {
    try {
      return readdirSync(this.path(relative)).sort();
    } catch (error) {
      if (typeof error === 'string') throw error;
      throw `list_failed:${relative}:${ioError(error)}`;
    }
  }

  preconditionError(name, args) {
    if (!this.agentMode) return 'policy_refusal: agent_mode_required';
    if (STATE_CHANGING.has(name) && field(args, 'confirmed') !== true) {
      return `confirmation_required:destructive_or_effectful:${name}`;
    }
    try {
      this.validateArguments(name, args);
      return null;
    } catch (error) {
      return String(error);
    }
  }

  validateArguments(name, args) {
    let paths;
    if (name === 'fs.read' || name === 'fs.write' || name === 'fs.list') paths = [arg(args, 'path')];
    else if (name === 'fs.move') paths = [arg(args, 'from'), arg(args, 'to')];
    else if (name === 'shell.run') {
      const operation = arg(args, 'operation');
      if (!['count_lines', 'filter_csv', 'unique_csv'].includes(operation)) {
        throw `unsupported_allowlisted_operation:${operation}`;
      }
      paths = [arg(args, 'input'), arg(args, 'output')];
    } else if (name === 'http.fetch' || name === 'http.post') {
      const url = arg(args, 'url');
      if (!url.startsWith('fixture://')) throw `network_fixture_not_permitted:${url}`;
      paths = [arg(args, 'save_as')];
    } else if (name === 'dom.query' || name === 'dom.extract') paths = [arg(args, 'source'), arg(args, 'save_as')];
    else if (name === 'archive.pack') paths = [...stringArray(args, 'paths'), arg(args, 'archive')];
    else if (name === 'archive.unpack') paths = [arg(args, 'archive'), arg(args, 'destination')];
    else paths = [arg(args, 'save_as')];
    for (const item of paths) this.path(item);
    let inputs = [];
    if (name === 'fs.read' || name === 'fs.list') inputs = [arg(args, 'path')];
    else if (name === 'fs.move') inputs = [arg(args, 'from')];
    else if (name === 'shell.run') inputs = [arg(args, 'input')];
    else if (name === 'dom.query' || name === 'dom.extract') inputs = [arg(args, 'source')];
    else if (name === 'archive.pack') inputs = stringArray(args, 'paths');
    else if (name === 'archive.unpack') inputs = [arg(args, 'archive')];
    for (const input of inputs) {
      if (!existsSync(this.path(input))) throw `precondition_failed:input_not_found:${input}`;
    }
  }

  apply(name, args) {
    switch (name) {
      case 'fs.read': {
        const relative = arg(args, 'path');
        const bytes = this.read(relative, 'read_failed');
        const content = decodeUtf8(bytes);
        if (content === null) throw `read_failed:${relative}:content_not_utf8`;
        return { path: relative, content, bytes: bytes.length, sha256: digest(bytes) };
      }
      case 'fs.write': {
        const relative = arg(args, 'path');
        const content = arg(args, 'content');
        this.writeBytes(relative, content);
        return { path: relative, bytes: byteLength(content), sha256: digest(content) };
      }
      case 'fs.list': {
        const relative = arg(args, 'path');
        const entries = this.directoryEntries(relative);
        return { path: relative, entries, sha256: digest(toCompactJson(entries)) };
      }
      case 'fs.move': {
        const from = arg(args, 'from');
        const to = arg(args, 'to');
        const destination = this.path(to);
        const bytes = this.read(from, 'move_source_failed');
        try {
          mkdirSync(path.dirname(destination), { recursive: true });
        } catch (error) {
          throw `move_directory_failed:${to}:${ioError(error)}`;
        }
        try {
          renameSync(this.path(from), destination);
        } catch (error) {
          throw `move_failed:${from}->${to}:${ioError(error)}`;
        }
        return { from, to, moved: true, sha256: digest(bytes) };
      }
      case 'shell.run': return this.runAllowlisted(args);
      case 'http.fetch': {
        const url = arg(args, 'url');
        const saveAs = arg(args, 'save_as');
        const body = fixture(url);
        if (body === null) throw `fixture_not_found:${url}`;
        this.writeBytes(saveAs, body);
        return httpOutput('GET', url, 200, body, saveAs);
      }
      case 'http.post': {
        const url = arg(args, 'url');
        const saveAs = arg(args, 'save_as');
        const requestBody = arg(args, 'body');
        if (url !== SUBMIT_URL) throw `fixture_not_found:${url}`;
        const accepted = requestBody === SUBMIT_TOKEN;
        const body = accepted ? readRepoFile(`${FIXTURE_DIR}/submission.json`).trim() : REJECTED_SUBMISSION;
        this.writeBytes(saveAs, body);
        return httpOutput('POST', url, accepted ? 200 : 400, body, saveAs);
      }
      case 'dom.query': {
        const source = arg(args, 'source');
        const selector = arg(args, 'selector');
        const saveAs = arg(args, 'save_as');
        const html = this.readText(source, 'dom_source_failed');
        const matches = queryHtml(html, selector);
        const text = matches.join('\n');
        this.writeBytes(saveAs, text);
        return { selector, matches, text, sha256: digest(text) };
      }
      case 'dom.extract': {
        const source = arg(args, 'source');
        const pointer = arg(args, 'pointer');
        const saveAs = arg(args, 'save_as');
        const bytes = this.read(source, 'dom_source_failed');
        let document;
        try {
          document = JSON.parse(bytes.toString('utf8'));
        } catch (error) {
          throw `json_parse_failed:${source}:${error.message}`;
        }
        const value = jsonPointer(document, pointer);
        if (value === undefined) throw `json_pointer_not_found:${pointer}`;
        const text = typeof value === 'string' ? value : toCompactJson(sortedKeys(value));
        this.writeBytes(saveAs, text);
        return { pointer, value, text, sha256: digest(text) };
      }
      case 'archive.pack': return this.archivePack(args);
      case 'archive.unpack': return this.archiveUnpack(args);
      default: {
        const saveAs = arg(args, 'save_as');
        const bytes = toCompactJson(sortedKeys({ plan_id: this.planId, state: 'running', scope: 'isolated_workspace' }));
        this.writeBytes(saveAs, bytes);
        return { state: 'running', scope: 'isolated_workspace', save_as: saveAs, sha256: digest(bytes) };
      }
    }
  }

  readText(relative, label) {
    const content = decodeUtf8(this.read(relative, label));
    if (content === null) throw `${label}:${relative}:${serverMessage('mcp_invalid_utf8')}`;
    return content;
  }

  runAllowlisted(args) {
    const operation = arg(args, 'operation');
    const input = arg(args, 'input');
    const output = arg(args, 'output');
    const content = this.readText(input, 'shell_input_failed');
    let generated;
    if (operation === 'count_lines') generated = `${lines(content).length}\n`;
    else if (operation === 'filter_csv') generated = filterCsv(content, arg(args, 'column'), arg(args, 'equals'));
    else if (operation === 'unique_csv') generated = uniqueCsv(content, arg(args, 'column'));
    else throw `unsupported_allowlisted_operation:${operation}`;
    this.writeBytes(output, generated);
    return { operation, output, stdout: generated, sha256: digest(generated) };
  }

  archivePack(args) {
    const archivePath = arg(args, 'archive');
    const paths = [...stringArray(args, 'paths')].sort();
    const entries = paths.map((relative) => ({
      path: relative,
      content_base64: this.read(relative, 'archive_input_failed').toString('base64'),
    }));
    const encoded = toCompactJson({ format: ARCHIVE_FORMAT, entries });
    this.writeBytes(archivePath, encoded);
    return { archive: archivePath, entries: paths, sha256: digest(encoded) };
  }

  archiveUnpack(args) {
    const archivePath = arg(args, 'archive');
    const destination = arg(args, 'destination');
    const bytes = this.read(archivePath, 'archive_read_failed');
    let archive;
    try {
      archive = parseArchive(bytes);
    } catch (error) {
      throw `archive_invalid:${error.message}`;
    }
    if (archive.format !== ARCHIVE_FORMAT) throw `archive_format_unsupported:${archive.format}`;
    const restored = [];
    for (const entry of archive.entries) {
      let content;
      try {
        content = decodeBase64(entry.content_base64);
      } catch (error) {
        throw `archive_content_invalid:${error}`;
      }
      this.writeBytes(`${destination}/${entry.path}`, content);
      restored.push(entry.path);
    }
    restored.sort();
    return { destination, entries: restored, sha256: digest(bytes) };
  }

  verifyPostcondition(name, args, output) {
    const sha = field(output, 'sha256');
    const fileMatches = (key) => {
      const observed = this.fileDigest(argOrEmpty(args, key));
      return (observed ?? undefined) === (typeof sha === 'string' ? sha : undefined);
    };
    switch (name) {
      case 'fs.read':
      case 'fs.write': {
        let bytes;
        try {
          bytes = readFileSync(this.path(argOrEmpty(args, 'path')));
        } catch {
          return false;
        }
        if (name === 'fs.read') return sha === digest(bytes) && field(output, 'content') === decodeUtf8(bytes);
        return bytes.equals(Buffer.from(argOrEmpty(args, 'content'))) && sha === digest(bytes);
      }
      case 'fs.list': {
        let observed;
        try {
          observed = this.directoryEntries(argOrEmpty(args, 'path'));
        } catch {
          return false;
        }
        const reported = field(output, 'entries');
        return Array.isArray(reported)
          && toCompactJson(reported.filter((entry) => typeof entry === 'string')) === toCompactJson(observed);
      }
      case 'fs.move': {
        let fromMissing;
        try {
          fromMissing = !existsSync(this.path(argOrEmpty(args, 'from')));
        } catch {
          fromMissing = false;
        }
        return fromMissing && fileMatches('to');
      }
      case 'archive.unpack': return this.verifyArchiveUnpack(args, output);
      case 'process.status':
        return field(output, 'scope') === 'isolated_workspace' && field(output, 'state') === 'running' && fileMatches('save_as');
      case 'shell.run': return fileMatches('output');
      case 'archive.pack': return fileMatches('archive');
      default: return fileMatches('save_as');
    }
  }

  verifyArchiveUnpack(args, output) {
    const destination = argOrEmpty(args, 'destination');
    try {
      const bytes = readFileSync(this.path(argOrEmpty(args, 'archive')));
      const archive = parseArchive(bytes);
      if (archive.format !== ARCHIVE_FORMAT || field(output, 'sha256') !== digest(bytes)) return false;
      const observed = [];
      for (const entry of archive.entries) {
        const expected = decodeBase64(entry.content_base64);
        const actual = readFileSync(this.path(`${destination}/${entry.path}`));
        if (!actual.equals(expected)) return false;
        observed.push(entry.path);
      }
      observed.sort();
      const reported = field(output, 'entries');
      return Array.isArray(reported)
        && toCompactJson(reported.filter((entry) => typeof entry === 'string')) === toCompactJson(observed);
    } catch {
      return false;
    }
  }

  /** `execute_primitive`: precondition, effect, postcondition events. */
  executePrimitive(stepId, name, args, precondition, postcondition) {
    const event = (phase, passed, detail) => ({
      id: `${this.planId}:${stepId}:${phase}`,
      step_id: stepId,
      primitive: name,
      phase,
      passed,
      detail,
    });
    const preconditionError = this.preconditionError(name, args);
    const events = [event('precondition', preconditionError === null, preconditionError ?? precondition)];
    let output;
    let effectPassed;
    let effectDetail;
    try {
      if (preconditionError !== null) throw preconditionError;
      output = this.apply(name, args);
      effectPassed = true;
      const changed = STATE_CHANGING.has(name) || typeof field(args, 'save_as') === 'string';
      effectDetail = `executed=${name};changed=${changed}`;
    } catch (error) {
      if (typeof error !== 'string') throw error;
      output = { error };
      effectPassed = false;
      effectDetail = error;
    }
    events.push(event('effect', effectPassed, effectDetail));
    const postPassed = effectPassed && this.verifyPostcondition(name, args, output);
    events.push(event('postcondition', postPassed, postPassed ? postcondition : `postcondition_failed:${postcondition}`));
    return {
      plan_id: this.planId,
      step_id: stepId,
      primitive: name,
      arguments: args,
      output,
      events,
      verified: events.every((item) => item.passed),
    };
  }
}

/** A `ComputerStepRecord` as `serde_json::to_string` writes it. */
function recordJson(record) {
  return toCompactJson({
    ...record,
    arguments: sortedKeys(record.arguments),
    output: sortedKeys(record.output),
  });
}

function appendComputerUseAudit(record) {
  const file = process.env.FORMAL_AI_COMPUTER_USE_AUDIT_PATH;
  if (!file) return;
  appendFileSync(file, `${recordJson(record)}\n`);
}

function requiredComputerArgument(args, name) {
  const value = field(args, name);
  return typeof value === 'string' && value.trim() ? value : null;
}

function callComputerPrimitive(ctx, request, id, primitive) {
  const params = field(request, 'params');
  const args = field(params, 'arguments') !== undefined ? field(params, 'arguments') : {};
  const context = PLAN_CONTEXT.map((name) => requiredComputerArgument(args, name));
  if (context.includes(null)) return jsonRpcError(id, -32602, 'computer_use_verification_context_required');
  const [planId, stepId, precondition, postcondition] = context;
  const agentMode = Boolean(ctx.agentMode);
  const key = `${planId}:agent_mode=${agentMode}`;
  if (!sessions.has(key)) {
    try {
      sessions.set(key, new ComputerUseSession(planId, agentMode));
    } catch {
      return jsonRpcError(id, -32603, serverMessage('mcp_workspace_creation_failed'));
    }
  }
  const record = sessions.get(key).executePrimitive(stepId, primitive[0], args, precondition, postcondition);
  try {
    appendComputerUseAudit(record);
  } catch (error) {
    record.verified = false;
    const last = record.events[record.events.length - 1];
    if (last) {
      last.passed = false;
      last.detail = `audit_persistence_failed:${ioError(error)}`;
    }
  }
  return jsonRpcResult(id, {
    content: [{ type: 'text', text: recordJson(record) }],
    structuredContent: record,
    isError: !record.verified,
  });
}

async function callChat(ctx, request, id) {
  const prompt = field(field(field(request, 'params'), 'arguments'), 'prompt');
  if (typeof prompt !== 'string') return jsonRpcError(id, -32602, serverMessage('mcp_prompt_not_string'));
  const answer = await solveSymbolic(ctx, prompt, []);
  return jsonRpcResult(id, { content: [{ type: 'text', text: answer.answer }], isError: false });
}

async function callTool(ctx, request, id) {
  const name = field(field(request, 'params'), 'name');
  if (typeof name !== 'string') return jsonRpcError(id, -32602, serverMessage('mcp_tool_name_not_string'));
  if (name === CHAT_TOOL) return callChat(ctx, request, id);
  const primitive = primitiveFromToolName(name);
  if (!primitive) return jsonRpcError(id, -32601, serverMessage('mcp_tool_not_found'));
  return callComputerPrimitive(ctx, request, id, primitive);
}

/** `POST /mcp`: `handle_mcp_request`. */
export async function handleMcp(ctx, request) {
  let body;
  try {
    body = JSON.parse(request.body);
  } catch {
    return jsonRpcError(null, -32700, 'Parse error');
  }
  const rawId = field(body, 'id');
  const id = rawId === undefined ? null : rawId;
  const method = field(body, 'method');
  if (field(body, 'jsonrpc') !== '2.0' || typeof method !== 'string') {
    return jsonRpcError(id, -32600, 'Invalid Request');
  }
  switch (method) {
    case 'initialize': {
      const requested = field(field(body, 'params'), 'protocolVersion');
      return jsonRpcResult(id, {
        protocolVersion: typeof requested === 'string' ? requested : PROTOCOL_VERSION,
        capabilities: { tools: {} },
        serverInfo: { name: 'formal-ai', version: packageVersion() },
        instructions: mcpText('mcp_instructions'),
      });
    }
    case 'notifications/initialized':
    case 'ping':
      return jsonRpcResult(id, {});
    case 'tools/list':
      return jsonRpcResult(id, { tools: listedTools() });
    case 'tools/call':
      return callTool(ctx, body, id);
    default:
      return jsonRpcError(id, -32601, serverMessage('mcp_method_not_found'));
  }
}
